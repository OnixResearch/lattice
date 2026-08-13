use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

const BASALT_REPO_DIR: &str = "basalt";
const CLANKERS_NAME: &str = "clankers";
const ASPEN_NAME: &str = "aspen";
const ONIXOS_NAME: &str = "onixos";
const ONIX_MODULES_COMPAT_NAME: &str = "onix-modules";
const UCAN_REPO_DIR: &str = "ucan";
const TARGET_ROOT: &str = "target/downstream-compat";
const WORKTREE_ROOT_DIR: &str = "worktrees";
const CARGO_TARGET_ENV: &str = "CARGO_TARGET_DIR";
const RUSTC_WRAPPER_ENV: &str = "RUSTC_WRAPPER";
const CARGO_GIT_FETCH_ENV: &str = "CARGO_NET_GIT_FETCH_WITH_CLI";
const OUTPUT_TAIL_LIMIT: usize = 4_096;
const PATCHED_BASALT_SOURCE: &str = "ssh://git@github.com/OnixResearch/basalt.git";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NixShellRoot {
    Basalt,
    Consumer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConsumerSpec {
    name: &'static str,
    repo_dir: &'static str,
    shell_root: NixShellRoot,
}

const CONSUMERS: &[ConsumerSpec] = &[
    ConsumerSpec {
        name: CLANKERS_NAME,
        repo_dir: CLANKERS_NAME,
        shell_root: NixShellRoot::Basalt,
    },
    ConsumerSpec {
        name: ASPEN_NAME,
        repo_dir: ASPEN_NAME,
        shell_root: NixShellRoot::Consumer,
    },
    ConsumerSpec {
        name: ONIXOS_NAME,
        repo_dir: ONIXOS_NAME,
        shell_root: NixShellRoot::Basalt,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunnerMode {
    Nix,
    Direct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    workspace_root: PathBuf,
    basalt_root: PathBuf,
    selected_consumers: Vec<String>,
    require_present: bool,
    runner_mode: RunnerMode,
    rustup_toolchain: Option<String>,
    list_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandSpec {
    program: String,
    args: Vec<String>,
    current_dir: PathBuf,
    envs: Vec<(String, String)>,
}

#[derive(Debug)]
struct PreparedConsumer {
    spec: ConsumerSpec,
    original_repo: PathBuf,
    compat_root: PathBuf,
    worktree_path: PathBuf,
}

impl Drop for PreparedConsumer {
    fn drop(&mut self) {
        if self.original_repo.exists() && self.worktree_path.exists() {
            let _ignored = Command::new("git")
                .arg("-C")
                .arg(&self.original_repo)
                .arg("worktree")
                .arg("remove")
                .arg("--force")
                .arg(&self.worktree_path)
                .status();
        }
        let _ignored = fs::remove_dir_all(&self.compat_root);
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let current_dir =
        env::current_dir().map_err(|error| format!("failed to read current directory: {error}"))?;
    let config = normalize_config(parse_args(env::args().skip(1), &current_dir)?)?;
    let selected = select_consumers(&config.selected_consumers)?;

    if config.list_only {
        for consumer in selected {
            println!("{}", consumer.name);
        }
        return Ok(());
    }

    let mut ran = Vec::new();
    let mut skipped = Vec::new();
    for consumer in selected {
        let Some(prepared) = prepare_consumer(&config, *consumer)? else {
            skipped.push(consumer.name);
            continue;
        };
        let command = build_command(&config, &prepared);
        println!("==> {}", prepared.spec.name);
        println!("{}", format_command(&command));
        run_command(&command)?;
        ran.push(prepared.spec.name);
    }

    if ran.is_empty() && config.require_present {
        return Err("no downstream consumers were available to check".to_owned());
    }

    println!("downstream compatibility passed for: {}", ran.join(", "));
    if !skipped.is_empty() {
        println!("skipped missing consumers: {}", skipped.join(", "));
    }
    Ok(())
}

fn parse_args<I>(args: I, current_dir: &Path) -> Result<Config, String>
where
    I: IntoIterator<Item = String>,
{
    let basalt_root = current_dir.to_path_buf();
    let workspace_root = current_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut config = Config {
        workspace_root,
        basalt_root,
        selected_consumers: Vec::new(),
        require_present: false,
        runner_mode: RunnerMode::Nix,
        rustup_toolchain: None,
        list_only: false,
    };

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--workspace-root" => {
                config.workspace_root =
                    PathBuf::from(required_value(&mut iter, "--workspace-root")?)
            }
            "--basalt-root" => {
                config.basalt_root = PathBuf::from(required_value(&mut iter, "--basalt-root")?)
            }
            "--consumer" => config
                .selected_consumers
                .push(required_value(&mut iter, "--consumer")?),
            "--require-present" => config.require_present = true,
            "--direct" => config.runner_mode = RunnerMode::Direct,
            "--nix" => config.runner_mode = RunnerMode::Nix,
            "--rustup-toolchain" => {
                config.rustup_toolchain = Some(required_value(&mut iter, "--rustup-toolchain")?)
            }
            "--list" => config.list_only = true,
            "--help" | "-h" => return Err(help_text()),
            unknown => return Err(format!("unknown argument {unknown}\n{}", help_text())),
        }
    }

    Ok(config)
}

fn normalize_config(mut config: Config) -> Result<Config, String> {
    config.workspace_root = canonicalize_path(&config.workspace_root, "workspace root")?;
    config.basalt_root = canonicalize_path(&config.basalt_root, "Basalt root")?;
    Ok(config)
}

fn canonicalize_path(path: &Path, label: &str) -> Result<PathBuf, String> {
    fs::canonicalize(path)
        .map_err(|error| format!("failed to canonicalize {label} {}: {error}", path.display()))
}

fn required_value<I>(iter: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    iter.next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn help_text() -> String {
    format!(
        "usage: downstream-compat [--workspace-root PATH] [--basalt-root PATH] [--consumer NAME] [--require-present] [--direct] [--rustup-toolchain TOOLCHAIN] [--list]\n\nConsumers: {}",
        CONSUMERS
            .iter()
            .map(|consumer| consumer.name)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn select_consumers(requested: &[String]) -> Result<Vec<&'static ConsumerSpec>, String> {
    if requested.is_empty() {
        return Ok(CONSUMERS.iter().collect());
    }

    let mut selected = Vec::new();
    for name in requested {
        let canonical_name = match name.as_str() {
            ONIX_MODULES_COMPAT_NAME => ONIXOS_NAME,
            other => other,
        };
        let consumer = CONSUMERS
            .iter()
            .find(|consumer| consumer.name == canonical_name)
            .ok_or_else(|| format!("unknown downstream consumer {name}"))?;
        selected.push(consumer);
    }
    Ok(selected)
}

fn prepare_consumer(
    config: &Config,
    spec: ConsumerSpec,
) -> Result<Option<PreparedConsumer>, String> {
    let original_repo = config.workspace_root.join(spec.repo_dir);
    if !original_repo.exists() {
        if config.require_present {
            return Err(format!(
                "required downstream consumer {} is missing at {}",
                spec.name,
                original_repo.display()
            ));
        }
        return Ok(None);
    }

    let compat_root = config
        .basalt_root
        .join(TARGET_ROOT)
        .join(WORKTREE_ROOT_DIR)
        .join(unique_worktree_name(spec.name)?);
    fs::create_dir_all(&compat_root).map_err(|error| {
        format!(
            "failed to create compatibility root {}: {error}",
            compat_root.display()
        )
    })?;

    let basalt_link = compat_root.join(BASALT_REPO_DIR);
    create_symlink(&config.basalt_root, &basalt_link)?;

    let worktree_path = compat_root.join(spec.repo_dir);
    let status = Command::new("git")
        .arg("-C")
        .arg(&original_repo)
        .arg("worktree")
        .arg("add")
        .arg("--detach")
        .arg(&worktree_path)
        .arg("HEAD")
        .status()
        .map_err(|error| format!("failed to start git worktree for {}: {error}", spec.name))?;
    if !status.success() {
        return Err(format!("git worktree add failed for {}", spec.name));
    }

    if spec.name == ONIXOS_NAME {
        create_symlink(&basalt_link, &worktree_path.join(BASALT_REPO_DIR))?;
        let ucan_repo = config.workspace_root.join(UCAN_REPO_DIR);
        if ucan_repo.exists() {
            create_symlink(&ucan_repo, &worktree_path.join(UCAN_REPO_DIR))?;
        }
    }

    Ok(Some(PreparedConsumer {
        spec,
        original_repo,
        compat_root,
        worktree_path,
    }))
}

fn unique_worktree_name(consumer: &str) -> Result<String, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before UNIX_EPOCH: {error}"))?
        .as_nanos();
    Ok(format!("{consumer}-{}-{timestamp}", std::process::id()))
}

#[cfg(unix)]
fn create_symlink(source: &Path, link: &Path) -> Result<(), String> {
    std::os::unix::fs::symlink(source, link).map_err(|error| {
        format!(
            "failed to link {} -> {}: {error}",
            link.display(),
            source.display()
        )
    })
}

#[cfg(not(unix))]
fn create_symlink(_source: &Path, _link: &Path) -> Result<(), String> {
    Err("downstream compatibility runner currently requires Unix symlinks".to_owned())
}

fn build_command(config: &Config, prepared: &PreparedConsumer) -> CommandSpec {
    let cargo_target_dir = config
        .basalt_root
        .join(TARGET_ROOT)
        .join(prepared.spec.name);
    let cargo_args = cargo_args(
        prepared.spec,
        &prepared.compat_root,
        &prepared.worktree_path,
    );
    let envs = vec![
        (
            CARGO_TARGET_ENV.to_owned(),
            cargo_target_dir.display().to_string(),
        ),
        (RUSTC_WRAPPER_ENV.to_owned(), String::new()),
        (CARGO_GIT_FETCH_ENV.to_owned(), "true".to_owned()),
    ];

    match config.runner_mode {
        RunnerMode::Direct => {
            let mut args = Vec::new();
            if let Some(toolchain) = config.rustup_toolchain.as_deref() {
                args.push(format!("+{toolchain}"));
            }
            args.extend(cargo_args);
            CommandSpec {
                program: "cargo".to_owned(),
                args,
                current_dir: config.basalt_root.clone(),
                envs,
            }
        }
        RunnerMode::Nix => {
            let shell_root = match prepared.spec.shell_root {
                NixShellRoot::Basalt => config.basalt_root.clone(),
                NixShellRoot::Consumer => prepared.worktree_path.clone(),
            };
            let mut args = vec![
                "develop".to_owned(),
                shell_root.display().to_string(),
                "-c".to_owned(),
                "env".to_owned(),
            ];
            for (key, value) in &envs {
                args.push(format!("{key}={value}"));
            }
            args.push("cargo".to_owned());
            args.extend(cargo_args);
            CommandSpec {
                program: "nix".to_owned(),
                args,
                current_dir: config.basalt_root.clone(),
                envs: Vec::new(),
            }
        }
    }
}

fn cargo_args(spec: ConsumerSpec, compat_root: &Path, worktree_path: &Path) -> Vec<String> {
    match spec.name {
        CLANKERS_NAME => vec![
            "test".to_owned(),
            "--manifest-path".to_owned(),
            worktree_path
                .join("examples/basalt-consumer-fixture/Cargo.toml")
                .display()
                .to_string(),
            "--no-default-features".to_owned(),
        ],
        ASPEN_NAME => vec![
            "check".to_owned(),
            "--manifest-path".to_owned(),
            worktree_path.join("Cargo.toml").display().to_string(),
            "-p".to_owned(),
            "molten".to_owned(),
            "--lib".to_owned(),
            "--config".to_owned(),
            format!(
                "patch.\"{PATCHED_BASALT_SOURCE}\".basalt.path=\"{}\"",
                compat_root.join(BASALT_REPO_DIR).display()
            ),
        ],
        ONIXOS_NAME => vec![
            "check".to_owned(),
            "--manifest-path".to_owned(),
            worktree_path.join("cli/Cargo.toml").display().to_string(),
        ],
        _ => unreachable!("unknown consumer spec must be rejected before command planning"),
    }
}

fn run_command(command: &CommandSpec) -> Result<(), String> {
    let mut process = Command::new(&command.program);
    process
        .current_dir(&command.current_dir)
        .args(&command.args);
    for (key, value) in &command.envs {
        process.env(key, value);
    }
    let output = process
        .output()
        .map_err(|error| format!("failed to start {}: {error}", command.program))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    print!("{stdout}");
    eprint!("{stderr}");
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "command failed with {}\nstdout tail:\n{}\nstderr tail:\n{}",
            output.status,
            tail(&stdout),
            tail(&stderr)
        ))
    }
}

fn format_command(command: &CommandSpec) -> String {
    let mut rendered = command.program.clone();
    for arg in &command.args {
        rendered.push(' ');
        rendered.push_str(arg);
    }
    rendered
}

fn tail(input: &str) -> String {
    if input.len() <= OUTPUT_TAIL_LIMIT {
        return input.to_owned();
    }
    let mut start = input.len() - OUTPUT_TAIL_LIMIT;
    while !input.is_char_boundary(start) {
        start += 1;
    }
    input[start..].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(args: &[&str]) -> Config {
        parse_args(
            args.iter().map(|arg| (*arg).to_owned()),
            Path::new("/workspace/basalt"),
        )
        .expect("args parse")
    }

    #[test]
    fn parses_direct_consumer_selection() {
        let config = test_config(&[
            "--workspace-root",
            "/workspace",
            "--consumer",
            ASPEN_NAME,
            "--direct",
            "--rustup-toolchain",
            "nightly",
            "--require-present",
        ]);

        assert_eq!(config.workspace_root, PathBuf::from("/workspace"));
        assert_eq!(config.selected_consumers, vec![ASPEN_NAME.to_owned()]);
        assert_eq!(config.runner_mode, RunnerMode::Direct);
        assert_eq!(config.rustup_toolchain.as_deref(), Some("nightly"));
        assert!(config.require_present);
    }

    #[test]
    fn accepts_onix_modules_compatibility_alias() {
        let requested = vec![ONIX_MODULES_COMPAT_NAME.to_owned()];

        let selected = select_consumers(&requested).expect("compatibility alias should resolve");

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, ONIXOS_NAME);
        assert_eq!(selected[0].repo_dir, ONIXOS_NAME);
    }

    #[test]
    fn rejects_unknown_consumer_selection() {
        let requested = vec!["unknown".to_owned()];

        let error = select_consumers(&requested).expect_err("unknown consumer should fail");

        assert!(
            error.contains("unknown downstream consumer"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn direct_aspen_command_uses_nightly_and_local_basalt_patch() {
        let config = test_config(&["--direct", "--rustup-toolchain", "nightly"]);
        let prepared = PreparedConsumer {
            spec: CONSUMERS
                .iter()
                .find(|consumer| consumer.name == ASPEN_NAME)
                .copied()
                .expect("aspen spec"),
            original_repo: PathBuf::from("/workspace/aspen"),
            compat_root: PathBuf::from("/workspace/.compat/aspen"),
            worktree_path: PathBuf::from("/workspace/.compat/aspen/aspen"),
        };

        let command = build_command(&config, &prepared);

        assert_eq!(command.program, "cargo");
        assert_eq!(command.args.first().map(String::as_str), Some("+nightly"));
        assert!(
            command
                .args
                .iter()
                .any(|arg| arg.contains(PATCHED_BASALT_SOURCE))
        );
        assert!(
            command
                .args
                .iter()
                .any(|arg| arg.contains("/workspace/.compat/aspen/basalt"))
        );
    }

    #[test]
    fn nix_onixos_command_uses_basalt_shell_and_cli_manifest() {
        let config = test_config(&[]);
        let prepared = PreparedConsumer {
            spec: CONSUMERS
                .iter()
                .find(|consumer| consumer.name == ONIXOS_NAME)
                .copied()
                .expect("onixos spec"),
            original_repo: PathBuf::from("/workspace/onixos"),
            compat_root: PathBuf::from("/workspace/.compat/onixos"),
            worktree_path: PathBuf::from("/workspace/.compat/onixos/onixos"),
        };

        let command = build_command(&config, &prepared);

        assert_eq!(command.program, "nix");
        assert!(
            command
                .args
                .windows(2)
                .any(|window| window == ["develop", "/workspace/basalt"])
        );
        assert!(
            command
                .args
                .iter()
                .any(|arg| arg.ends_with("onixos/cli/Cargo.toml"))
        );
    }

    #[test]
    fn clankers_command_runs_consumer_fixture_tests() {
        let spec = CONSUMERS
            .iter()
            .find(|consumer| consumer.name == CLANKERS_NAME)
            .copied()
            .expect("clankers spec");

        let args = cargo_args(spec, Path::new("/compat"), Path::new("/compat/clankers"));

        assert_eq!(args.first().map(String::as_str), Some("test"));
        assert!(
            args.iter()
                .any(|arg| arg.ends_with("examples/basalt-consumer-fixture/Cargo.toml"))
        );
        assert!(args.iter().any(|arg| arg == "--no-default-features"));
    }
}
