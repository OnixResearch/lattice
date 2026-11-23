//! Explicit single-shot OMP JSONL profile; generic command agents do not use it.
//!
//! Wire definitions are taken from OMP e325ffc73ab86e03bdd20a91b77d8e2b3964c906,
//! docs/cli-reference.md, modes/print-mode.ts and session/session-entries.ts.

pub const OMP_REVISION: &str = "e325ffc73ab86e03bdd20a91b77d8e2b3964c906";
pub const OMP_RESULT_SCHEMA: &str = "lattice.omp-jsonl-observation.v1";

pub(crate) mod profile;
pub mod stop;
mod stream;

/// Operator-admitted executable bytes, not an executable-name compatibility claim.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub revision: String,
    pub executable_blake3: String,
    pub base_identity: String,
    pub model: String,
    pub prompt: String,
    pub model_broker: super::broker::Profile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terminal {
    Completed,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Observation {
    pub schema: String,
    pub session_id: String,
    pub event_count: u64,
    pub terminal: Terminal,
    pub raw_events_blake3: String,
    /// The event stream is not a complete invocation/billing ledger.
    pub usage_complete: bool,
}

#[derive(Debug)]
pub struct RunResult {
    pub outcome: super::Outcome,
    pub observation: Result<Observation, String>,
    pub revision: String,
    pub executable_blake3: String,
    pub argv_blake3: String,
    pub compatibility: Option<super::Compatibility>,
}

/// Runtime ports for launching one admitted OMP worker attempt.
pub struct LaunchOptions<'a, 'b> {
    pub config: &'a crate::process::core::RuntimeConfig,
    pub cancellation: &'a std::sync::atomic::AtomicBool,
    pub services: super::broker::Services<'b>,
    pub launcher: &'a dyn super::sandbox::launcher::SandboxLauncher,
}

// r[impl lattice.candidate_worker.omp_jsonl]
pub fn run(
    request: &super::Request,
    profile: &Profile,
    config: &crate::process::core::RuntimeConfig,
    cancellation: &std::sync::atomic::AtomicBool,
    services: super::broker::Services<'_>,
) -> Result<RunResult, super::Error> {
    let options = LaunchOptions {
        config,
        cancellation,
        services,
        launcher: &super::sandbox::launcher::Direct,
    };
    run_with_launcher(request, profile, options)
}

/// Run the reviewed OMP profile through an explicit `SandboxLauncher`.
pub fn run_with_launcher(
    request: &super::Request,
    profile: &Profile,
    options: LaunchOptions<'_, '_>,
) -> Result<RunResult, super::Error> {
    if let Err(reason) = profile::admit(request, profile) {
        return Ok(blocked(profile, reason));
    }
    debug_assert!(!request.boundary.network && request.require_isolation);
    let mut selected = request.clone();
    selected.args = profile::arguments(request, profile);
    let argv_blake3 = argument_identity(&selected.args)?;
    let (config, cancellation, launcher) = (options.config, options.cancellation, options.launcher);
    let result =
        super::broker::with_route(selected, &profile.model_broker, options.services, cancellation, |selected| {
            let compatibility = match compatible(&selected, profile) {
                Ok(value) => Some(value),
                Err(reason) => return Ok(blocked(profile, reason)),
            };
            let outcome = dispatch(&selected, &profile.base_identity, config, cancellation, launcher)?;
            Ok(finish_profile(&selected, profile, outcome, argv_blake3, compatibility))
        });
    match result {
        Err(reason) => Ok(blocked(profile, &reason)),
        Ok((result, true)) => result,
        Ok((result, false)) => result.map(broker_failure),
    }
}

fn broker_failure(mut result: RunResult) -> RunResult {
    result.outcome = match result.outcome {
        super::Outcome::Produced { stdout, stderr, .. } | super::Outcome::Unchanged { stdout, stderr, .. } => {
            super::Outcome::Unknown {
                blocker: "model broker stopped unexpectedly after worker dispatch; retain workspace".into(),
                stdout,
                stderr,
            }
        }
        other => other,
    };
    result
}

impl RunResult {
    /// The bounded raw artifact is process stdout, never normalized completion text.
    pub fn raw_events(&self) -> &[u8] {
        raw_events(&self.outcome)
    }
}

fn raw_events(outcome: &super::Outcome) -> &[u8] {
    match outcome {
        super::Outcome::Produced { stdout, .. }
        | super::Outcome::Failed { stdout, .. }
        | super::Outcome::Unchanged { stdout, .. }
        | super::Outcome::Unknown { stdout, .. }
        | super::Outcome::Interrupted { stdout, .. }
        | super::Outcome::Cancelled { stdout, .. } => stdout,
        super::Outcome::Blocked { .. } => &[],
    }
}

/// Normalize bounded JSONL without trusting model text as a candidate receipt.
/// Unknown event types are retained in raw_events and counted, not interpreted.
pub(crate) fn normalize_jsonl(raw: &[u8], max_bytes: usize, expected_cwd: &str) -> Result<Observation, String> {
    if raw.is_empty() || raw.len() > max_bytes || raw.last() != Some(&b'\n') {
        return Err(stream::reject());
    }
    let text = std::str::from_utf8(raw).map_err(|_| stream::reject())?;
    let mut records = text.lines();
    let session_id =
        session_identity(records.next().ok_or_else(stream::reject)?, SessionBinding { cwd: expected_cwd })?;
    let mut progress = stream::Progress::new();
    for line in records {
        let event: serde_json::Value = serde_json::from_str(line).map_err(|_| stream::reject())?;
        progress.observe(&event)?;
    }
    let (event_count, terminal) = progress.finish()?;
    Ok(observation(raw, session_id, event_count, terminal))
}

fn stop_reason(message: &serde_json::Value) -> Result<Terminal, String> {
    match message["stopReason"].as_str() {
        Some("stop") => Ok(Terminal::Completed),
        Some("error") => Ok(Terminal::Failed),
        Some("aborted") => Ok(Terminal::Aborted),
        // A tool-use or token-limit stop is not a completed implementation turn.
        _ => Err("OMP assistant terminal stop reason is not a completed turn".into()),
    }
}

struct SessionBinding<'a> {
    cwd: &'a str,
}

fn session_identity(line: &str, binding: SessionBinding<'_>) -> Result<String, String> {
    let reject = || "OMP JSONL stream is malformed, truncated or incomplete".to_string();
    let header: serde_json::Value = serde_json::from_str(line).map_err(|_| reject())?;
    let is_session = header["type"] == "session" && header["version"] == 3;
    let has_extra_directories = header
        .get("additionalDirectories")
        .is_some_and(|dirs| dirs.as_array().is_none_or(|dirs| !dirs.is_empty()));
    if !is_session || header["cwd"] != binding.cwd || has_extra_directories {
        return Err(reject());
    }
    header["id"].as_str().filter(|id| !id.is_empty()).map(str::to_string).ok_or_else(reject)
}

fn observation(raw: &[u8], session_id: String, event_count: u64, terminal: Terminal) -> Observation {
    Observation {
        schema: OMP_RESULT_SCHEMA.into(),
        session_id,
        event_count,
        terminal,
        raw_events_blake3: crate::pure_inet_profile::blake3_identity(raw),
        usage_complete: false,
    }
}

#[cfg(test)]
mod tests;

/// The reviewed argv, the bounded process, the host capture, the JSONL
/// normalization and the completion admission of [`run`], over a scripted
/// executable without the sandbox, broker or byte pins. Tests only: it
/// lets shell tests drive a fake OMP through the same observation path.
#[cfg(test)]
pub(crate) fn run_scripted(
    request: &super::Request,
    profile: &Profile,
    cancellation: &std::sync::atomic::AtomicBool,
) -> Result<RunResult, super::Error> {
    let mut selected = request.clone();
    selected.args = profile::arguments(request, profile);
    selected.require_isolation = false;
    selected.sandbox_program = None;
    let argv_blake3 = argument_identity(&selected.args)?;
    let config = crate::process::core::RuntimeConfig::default();
    let outcome =
        dispatch(&selected, &profile.base_identity, &config, cancellation, &super::sandbox::launcher::Direct)?;
    Ok(finish_profile(&selected, profile, outcome, argv_blake3, None))
}

fn blocked(profile: &Profile, reason: &str) -> RunResult {
    RunResult {
        outcome: super::Outcome::Blocked {
            blocker: reason.to_string(),
        },
        observation: Err(reason.to_string()),
        revision: profile.revision.clone(),
        executable_blake3: profile.executable_blake3.clone(),
        argv_blake3: crate::pure_inet_profile::blake3_identity(b""),
        compatibility: None,
    }
}

fn argument_identity(args: &[String]) -> Result<String, super::Error> {
    let encoded = serde_json::to_vec(args).map_err(|_| super::Error::Result("OMP argument encoding failed".into()))?;
    Ok(crate::pure_inet_profile::blake3_identity(&encoded))
}

fn compatible(request: &super::Request, profile: &Profile) -> Result<super::Compatibility, &'static str> {
    super::compatibility::observe(request)
        .filter(|value| value.worker_program_identity == profile.executable_blake3)
        .ok_or("OMP executable/provider identities are unavailable or differ from the admitted profile")
}

fn dispatch(
    request: &super::Request,
    base_identity: &str,
    config: &crate::process::core::RuntimeConfig,
    cancellation: &std::sync::atomic::AtomicBool,
    launcher: &dyn super::sandbox::launcher::SandboxLauncher,
) -> Result<super::Outcome, super::Error> {
    super::execution::run(request, config, cancellation, Some(base_identity), |process, config, cancellation| {
        launcher.execute(process, config, cancellation)
    })
}

fn finish_profile(
    request: &super::Request,
    profile: &Profile,
    outcome: super::Outcome,
    argv_blake3: String,
    compatibility: Option<super::Compatibility>,
) -> RunResult {
    let observation = normalize_jsonl(raw_events(&outcome), request.output_limit_bytes, "/workspace");
    let outcome = admit_completion(outcome, &observation, &profile.base_identity);
    RunResult {
        outcome,
        observation,
        revision: OMP_REVISION.into(),
        executable_blake3: profile.executable_blake3.clone(),
        argv_blake3,
        compatibility,
    }
}

fn admit_completion(
    outcome: super::Outcome,
    observation: &Result<Observation, String>,
    expected_base: &str,
) -> super::Outcome {
    // Exit zero and a changed checkout do not establish a valid terminal protocol.
    let has_completed = matches!(observation, Ok(value) if value.terminal == Terminal::Completed);
    let has_matching_base =
        matches!(&outcome, super::Outcome::Produced { receipt, .. } if receipt.base_identity == expected_base);
    if has_completed && has_matching_base {
        return outcome;
    }
    match outcome {
        super::Outcome::Produced { stdout, stderr, .. } => super::Outcome::Unknown {
            blocker: "OMP terminal JSONL observation is incomplete or failed; retain candidate workspace".into(),
            stdout,
            stderr,
        },
        other => other,
    }
}
