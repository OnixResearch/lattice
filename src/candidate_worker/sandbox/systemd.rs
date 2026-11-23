//! Pure planning and validation for launching the candidate worker sandbox
//! inside a transient systemd service unit.

const MAX_UNIT_PREFIX_BYTES: usize = 48;
const MAX_SLICE_BYTES: usize = 128;
const CRASH_BACKSTOP_EXTRA_SECS: u64 = 5;
const MILLIS_PER_SEC: u64 = 1_000;
const ROUND_UP_OFFSET_MILLIS: u64 = 999;
const UNIT_DIGEST_HEX_BYTES: usize = 16;
const SLICE_SUFFIX: &str = ".slice";
const SERVICE_SUFFIX: &str = ".service";
const WORKER_PROTECT_KERNEL_TUNABLES: &str = "--property=ProtectKernelTunables=no";
const WORKER_ADDRESS_FAMILIES: &str = "--property=RestrictAddressFamilies=AF_UNIX AF_NETLINK";
const WORKER_KILL_MODE: &str = "--property=KillMode=control-group";
const WORKER_KILL_SIGNAL: &str = "--property=KillSignal=SIGKILL";

pub const DEFAULT_UNIT_PREFIX: &str = "lattice-worker";

const fn absent<T>() -> Option<T> {
    None
}

/// Systemd manager bus used by [`Config`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bus {
    /// System manager (`--system`), used by hardened host services.
    System,
    /// User manager (`--user`), passing only the minimal bus locator variables
    /// to `systemd-run` and `systemctl`; `bwrap --clearenv` strips them before
    /// the worker starts.
    User {
        #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
        runtime_dir: Option<String>,
        #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
        bus_address: Option<String>,
    },
}

/// Configuration for launching the candidate worker's `bwrap` sandbox inside a
/// transient systemd service unit.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Absolute path to the reviewed `systemd-run` executable.
    pub systemd_run_program: String,
    /// Absolute path to the reviewed `systemctl` executable for teardown.
    pub systemctl_program: String,
    /// Target systemd manager bus.
    pub bus: Bus,
    /// Safe prefix for transient unit names (for example [`DEFAULT_UNIT_PREFIX`]).
    pub unit_prefix: String,
    /// Optional systemd slice placement (for example `"lattice-workers.slice"`).
    #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
    pub slice: Option<String>,
}

/// Pure launch and teardown commands derived for one candidate worker invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub unit_name: String,
    pub launch_args: Vec<String>,
    pub stop_args: Vec<String>,
    pub client_env: std::collections::BTreeMap<String, String>,
}

/// Pure inputs required to build a [`LaunchPlan`].
pub struct PlanInput<'a> {
    pub config: &'a Config,
    pub process: &'a crate::agent_node::ProcessRequest,
    pub teardown_grace_millis: u64,
    pub unit_token: &'a str,
}

struct TimingBounds {
    timeout_secs: u64,
    teardown_grace_millis: u64,
}

struct ProgramCheck<'a> {
    path: &'a str,
    diagnostic: &'static str,
}

/// Validate a [`Config`] without touching the host filesystem.
pub fn validate_config(config: &Config) -> Result<(), &'static str> {
    validate_absolute_program(ProgramCheck {
        path: &config.systemd_run_program,
        diagnostic: "systemd-run path must be absolute",
    })?;
    validate_absolute_program(ProgramCheck {
        path: &config.systemctl_program,
        diagnostic: "systemctl path must be absolute",
    })?;
    validate_unit_prefix(&config.unit_prefix)?;
    if let Some(slice) = config.slice.as_deref() {
        validate_slice(slice)?;
    }
    validate_bus(&config.bus)?;
    Ok(())
}

/// Build the transient service `systemd-run` argv, matching `systemctl stop`
/// argv, and minimal client environment while preserving `process.program` and
/// `process.args` byte-for-byte after `--`.
pub fn plan_launch(input: PlanInput<'_>) -> Result<LaunchPlan, &'static str> {
    validate_config(input.config)?;
    validate_unit_token(input.unit_token)?;
    if input.process.program.is_empty() || input.process.program.contains('\0') {
        return Err("sandbox process program is invalid");
    }
    let unit_name = format!("{}-{}{SERVICE_SUFFIX}", input.config.unit_prefix, input.unit_token);
    debug_assert!(unit_name.ends_with(SERVICE_SUFFIX));
    let bus_flag = bus_flag(&input.config.bus);
    let mut launch_args = vec![
        bus_flag.to_string(),
        "--wait".to_string(),
        "--pipe".to_string(),
        "--collect".to_string(),
        "--quiet".to_string(),
        "--service-type=exec".to_string(),
        format!("--unit={unit_name}"),
    ];
    if let Some(slice) = input.config.slice.as_deref() {
        launch_args.push(format!("--slice={slice}"));
    }
    if let Some(cwd) = input.process.cwd.as_deref() {
        launch_args.push(format!("--working-directory={cwd}"));
    }
    let backstop_secs = crash_backstop_secs(TimingBounds {
        timeout_secs: input.process.timeout_secs,
        teardown_grace_millis: input.teardown_grace_millis,
    });
    launch_args.extend([
        WORKER_PROTECT_KERNEL_TUNABLES.to_string(),
        WORKER_ADDRESS_FAMILIES.to_string(),
        WORKER_KILL_MODE.to_string(),
        WORKER_KILL_SIGNAL.to_string(),
        format!("--property=TimeoutStopSec={}ms", input.teardown_grace_millis),
        format!("--property=RuntimeMaxSec={backstop_secs}s"),
        "--".to_string(),
        input.process.program.clone(),
    ]);
    launch_args.extend(input.process.args.iter().cloned());
    let stop_args = vec![bus_flag.to_string(), "stop".to_string(), unit_name.clone()];
    let client_env = client_environment(&input.config.bus);
    Ok(LaunchPlan {
        unit_name,
        launch_args,
        stop_args,
        client_env,
    })
}

/// Format the leading 16 lowercase hex characters of a BLAKE3 hash as a unit token.
pub fn token_from_hash(hash: blake3::Hash) -> String {
    let hex = hash.to_hex().to_string();
    debug_assert!(hex.len() >= UNIT_DIGEST_HEX_BYTES);
    hex.chars().take(UNIT_DIGEST_HEX_BYTES).collect()
}

fn crash_backstop_secs(bounds: TimingBounds) -> u64 {
    let grace_secs = bounds.teardown_grace_millis.saturating_add(ROUND_UP_OFFSET_MILLIS) / MILLIS_PER_SEC;
    bounds.timeout_secs.saturating_add(grace_secs).saturating_add(CRASH_BACKSTOP_EXTRA_SECS)
}

fn bus_flag(bus: &Bus) -> &'static str {
    match bus {
        Bus::System => "--system",
        Bus::User { .. } => "--user",
    }
}

fn client_environment(bus: &Bus) -> std::collections::BTreeMap<String, String> {
    let mut env = std::collections::BTreeMap::new();
    if let Bus::User {
        runtime_dir,
        bus_address,
    } = bus
    {
        if let Some(dir) = runtime_dir {
            env.insert("XDG_RUNTIME_DIR".to_string(), dir.clone());
        }
        if let Some(address) = bus_address {
            env.insert("DBUS_SESSION_BUS_ADDRESS".to_string(), address.clone());
        }
    }
    env
}

fn validate_absolute_program(check: ProgramCheck<'_>) -> Result<(), &'static str> {
    let has_control = check.path.chars().any(char::is_control);
    if check.path.is_empty() || has_control || !std::path::Path::new(check.path).is_absolute() {
        return Err(check.diagnostic);
    }
    Ok(())
}

fn validate_unit_prefix(prefix: &str) -> Result<(), &'static str> {
    let is_length_valid = !prefix.is_empty() && prefix.len() <= MAX_UNIT_PREFIX_BYTES;
    let is_ascii_safe = prefix.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if !is_length_valid || !is_ascii_safe {
        return Err("systemd unit prefix must be bounded ASCII alphanumeric, '-' or '_'");
    }
    Ok(())
}

fn validate_slice(slice: &str) -> Result<(), &'static str> {
    let is_length_valid = slice.len() > SLICE_SUFFIX.len() && slice.len() <= MAX_SLICE_BYTES;
    let is_ascii_safe = slice
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || byte == b'.');
    if !is_length_valid || !is_ascii_safe || !slice.ends_with(SLICE_SUFFIX) {
        return Err("systemd slice must be a bounded ASCII .slice unit name");
    }
    Ok(())
}

fn validate_unit_token(token: &str) -> Result<(), &'static str> {
    if token.len() != UNIT_DIGEST_HEX_BYTES || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("systemd unit token must be 16 ASCII hex characters");
    }
    Ok(())
}

fn validate_bus(bus: &Bus) -> Result<(), &'static str> {
    match bus {
        Bus::System => Ok(()),
        Bus::User {
            runtime_dir,
            bus_address,
        } => {
            let has_runtime = runtime_dir.as_deref().is_some_and(is_safe_env_value);
            let has_address = bus_address.as_deref().is_some_and(is_safe_env_value);
            let has_invalid_runtime = runtime_dir.as_deref().is_some_and(|value| !is_safe_env_value(value));
            let has_invalid_address = bus_address.as_deref().is_some_and(|value| !is_safe_env_value(value));
            if has_invalid_runtime || has_invalid_address || (!has_runtime && !has_address) {
                return Err("systemd user bus requires a valid runtime_dir or bus_address");
            }
            Ok(())
        }
    }
}

fn is_safe_env_value(value: &str) -> bool {
    !value.is_empty() && !value.chars().any(char::is_control)
}
