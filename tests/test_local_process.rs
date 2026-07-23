#![cfg(unix)]

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use lattice::agent_node::AgentCommandPlan;
use lattice::agent_node::AgentOutput;
use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentRuntimeConfig;
use lattice::agent_node::AgentTarget;
use lattice::agent_node::ProcessError;
use lattice::agent_node::ProcessRequest;
use lattice::agent_node::ProcessRunner;
use lattice::agent_node::ProductionProcessRunner;
use lattice::agent_node::decode_agent_command_output;
use lattice::agent_node::run_agent_command_with_runner;
use lattice::credentials::CommandCredentialProvider;
use lattice::credentials::CommandCredentialProviderConfig;
use lattice::credentials::CredentialProvider;
use lattice::credentials::SecretRef;
use lattice::process::core::AdmissionInput;
use lattice::process::core::LimitConfig as ProcessLimitConfig;
use lattice::process::core::RuntimeConfig as LocalProcessConfig;
use lattice::process::core::admit;

const SHELL: &str = "/bin/sh";
const SHELL_COMMAND_FLAG: &str = "-c";
const SUCCESS_EXIT_CODE: i32 = 0;
const TEST_TIMEOUT_SECS: u64 = 2;
const LONG_TIMEOUT_SECS: u64 = 5;
const TIMEOUT_TEST_SECS: u64 = 1;
const POLL_INTERVAL_MILLIS: u64 = 10;
const TEARDOWN_MILLIS: u64 = 500;
const TEST_LIMIT_BYTES: usize = 4_096;
const SMALL_LIMIT_BYTES: usize = 32;
const TINY_LIMIT_BYTES: usize = 8;
const OVERFLOW_OUTPUT_BYTES: usize = 512;
const CANCELLATION_DELAY_MILLIS: u64 = 100;
const DESCENDANT_DELAY_SECS: u64 = 1;
const DESCENDANT_OBSERVATION_MILLIS: u64 = 1_500;
const TIMEOUT_DESCENDANT_DELAY_SECS: u64 = 2;
const TIMEOUT_DESCENDANT_OBSERVATION_MILLIS: u64 = 2_500;
const DIAGNOSTIC_MAX_CHARS: usize = 256;
const SECRET_ENV_VALUE: &str = "environment-secret-must-not-leak";
const SECRET_STDIN_VALUE: &str = "stdin-secret-must-not-leak";
const COMMAND_PROVIDER_ID: &str = "command";
const COMMAND_PROVIDER_KEY: &str = "service-token";
const COMMAND_AGENT_LABEL: &str = "pi";
const OWNER_EXECUTE_PERMISSION: u32 = 0o100;
const BOUNDED_AGENT_TEXT: &str = "bounded-agent-output";
const RESOLVED_CREDENTIAL: &str = "resolved-credential";

fn limits() -> ProcessLimitConfig {
    ProcessLimitConfig {
        stdin_max_bytes: TEST_LIMIT_BYTES,
        stdout_max_bytes: TEST_LIMIT_BYTES,
        stderr_max_bytes: TEST_LIMIT_BYTES,
        command_agent_timeout_secs: TEST_TIMEOUT_SECS,
        poll_interval_millis: POLL_INTERVAL_MILLIS,
        teardown_grace_millis: TEARDOWN_MILLIS,
    }
}

fn runner_with_limits(limits: ProcessLimitConfig) -> ProductionProcessRunner {
    ProductionProcessRunner::new(LocalProcessConfig {
        shell_program: SHELL.to_owned(),
        limits,
    })
}

fn shell_request(script: impl Into<String>) -> ProcessRequest {
    ProcessRequest {
        program: SHELL.to_owned(),
        args: vec![SHELL_COMMAND_FLAG.to_owned(), script.into()],
        cwd: Some("/".to_owned()),
        env: BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: TEST_TIMEOUT_SECS,
        output_limit_bytes: TEST_LIMIT_BYTES,
    }
}

fn repeated_output_script(stream: &str, byte_count: usize) -> String {
    format!("count=0; while [ \"$count\" -lt {byte_count} ]; do printf x {stream}; count=$((count + 1)); done")
}

// r[verify lattice.workflow_execution.bounded_process_compatibility]
// r[verify lattice.workflow_execution.bounded_process_compatibility.scenario.shell]
// r[verify lattice.workflow_execution.bounded_process_compatibility.scenario.non-claim]
#[test]
fn bounded_process_preserves_stdin_cwd_environment_and_exit_status() {
    let temp_dir = tempfile::tempdir().expect("temporary process cwd");
    let mut request = shell_request("input=$(cat); printf '%s:%s:%s' \"$input\" \"$TEST_KEY\" \"$PWD\"");
    request.cwd = Some(temp_dir.path().display().to_string());
    request.env.insert("TEST_KEY".to_owned(), "environment".to_owned());
    request.stdin = b"standard-input".to_vec();

    let output = runner_with_limits(limits()).run(&request).expect("bounded process must complete");

    assert_eq!(output.status_code, Some(SUCCESS_EXIT_CODE));
    assert_eq!(output.stderr, Vec::<u8>::new());
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 process output"),
        format!("standard-input:environment:{}", temp_dir.path().display())
    );
}

// r[verify lattice.workflow_execution.bounded_process_policy]
// r[verify lattice.workflow_execution.bounded_process_policy.scenario.valid]
#[test]
fn bounded_process_resolves_relative_executable_from_effective_path_and_cwd() {
    let temp_dir = tempfile::tempdir().expect("temporary PATH cwd");
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).expect("create relative PATH directory");
    let executable = bin_dir.join("relative-helper");
    fs::write(&executable, "#!/bin/sh\nprintf resolved").expect("write relative executable");
    let mut permissions = fs::metadata(&executable).expect("relative executable metadata").permissions();
    permissions.set_mode(permissions.mode() | OWNER_EXECUTE_PERMISSION);
    fs::set_permissions(&executable, permissions).expect("mark relative executable executable");

    let mut request = shell_request("unused");
    request.program = "relative-helper".to_owned();
    request.args.clear();
    request.cwd = Some(temp_dir.path().display().to_string());
    request.env.insert("PATH".to_owned(), "bin".to_owned());

    let output = runner_with_limits(limits()).run(&request).expect("relative executable must resolve");
    assert_eq!(output.status_code, Some(SUCCESS_EXIT_CODE));
    assert_eq!(output.stdout, b"resolved");
}

// r[verify lattice.workflow_execution.bounded_process_policy.scenario.invalid]
#[test]
fn process_resolution_rejects_missing_relative_executables_and_directories() {
    let temp_dir = tempfile::tempdir().expect("temporary negative resolution directory");
    let mut missing_program = shell_request("unused");
    missing_program.program = "missing/relative-helper".to_owned();
    missing_program.cwd = Some(temp_dir.path().display().to_string());
    assert!(matches!(runner_with_limits(limits()).run(&missing_program), Err(ProcessError::InvalidRequest(_))));

    let mut missing_directory = shell_request("unused");
    missing_directory.cwd = Some(temp_dir.path().join("missing-directory").display().to_string());
    assert!(matches!(runner_with_limits(limits()).run(&missing_directory), Err(ProcessError::InvalidRequest(_))));
}

// r[verify lattice.workflow_execution.bounded_process_mechanics]
// r[verify lattice.workflow_execution.bounded_process_mechanics.scenario.teardown]
#[test]
fn bounded_process_enforces_mandatory_timeout() {
    let mut request = shell_request(format!("sleep {LONG_TIMEOUT_SECS}"));
    request.timeout_secs = TIMEOUT_TEST_SECS;

    let error = runner_with_limits(limits()).run(&request).expect_err("long process must time out");
    assert_eq!(error, ProcessError::Timeout);
}

// r[verify lattice.workflow_execution.bounded_process_mechanics]
// r[verify lattice.workflow_execution.bounded_process_mechanics.scenario.output-flood]
#[test]
fn bounded_process_rejects_each_truncated_stream_without_returning_prefixes() {
    let mut stdout_limits = limits();
    stdout_limits.stdout_max_bytes = TINY_LIMIT_BYTES;
    let stdout_error = runner_with_limits(stdout_limits)
        .run(&shell_request(repeated_output_script("", OVERFLOW_OUTPUT_BYTES)))
        .expect_err("stdout truncation must fail closed");
    assert!(matches!(stdout_error, ProcessError::OutputLimitExceeded {
        limit: TINY_LIMIT_BYTES,
        actual: OVERFLOW_OUTPUT_BYTES,
    }));

    let mut stderr_limits = limits();
    stderr_limits.stderr_max_bytes = SMALL_LIMIT_BYTES;
    let stderr_error = runner_with_limits(stderr_limits)
        .run(&shell_request(repeated_output_script(">&2", OVERFLOW_OUTPUT_BYTES)))
        .expect_err("stderr truncation must fail closed");
    assert!(matches!(stderr_error, ProcessError::OutputLimitExceeded {
        limit: SMALL_LIMIT_BYTES,
        actual: OVERFLOW_OUTPUT_BYTES,
    }));
}

// r[verify lattice.workflow_execution.bounded_process_mechanics]
// r[verify lattice.workflow_execution.bounded_process_mechanics.scenario.success]
#[test]
fn bounded_process_drains_mixed_stdout_and_stderr_without_deadlock() {
    let script = format!(
        "count=0; while [ \"$count\" -lt {SMALL_LIMIT_BYTES} ]; do printf o; printf e >&2; count=$((count + 1)); done"
    );
    let output = runner_with_limits(limits())
        .run(&shell_request(script))
        .expect("mixed bounded output must complete");

    assert_eq!(output.stdout.len(), SMALL_LIMIT_BYTES);
    assert_eq!(output.stderr.len(), SMALL_LIMIT_BYTES);
}

// r[verify lattice.workflow_execution.bounded_process_policy]
// r[verify lattice.workflow_execution.bounded_process_policy.scenario.invalid]
#[test]
fn stdin_overflow_is_rejected_before_the_child_can_run() {
    let temp_dir = tempfile::tempdir().expect("temporary stdin overflow directory");
    let marker = temp_dir.path().join("spawned");
    let mut constrained = limits();
    constrained.stdin_max_bytes = TINY_LIMIT_BYTES;
    let mut request = shell_request(format!("touch '{}'", marker.display()));
    request.stdin = vec![b's'; SMALL_LIMIT_BYTES];

    let error = runner_with_limits(constrained).run(&request).expect_err("stdin overflow must be rejected");
    assert!(matches!(error, ProcessError::InvalidRequest(_)));
    assert!(!marker.exists(), "rejected request must not spawn a child");
}

// r[verify lattice.workflow_execution.bounded_process_mechanics]
// r[verify lattice.workflow_execution.bounded_process_mechanics.scenario.teardown]
#[test]
fn timeout_terminates_the_owned_process_group() {
    let temp_dir = tempfile::tempdir().expect("temporary timeout directory");
    let marker = temp_dir.path().join("timeout-descendant-survived");
    let script = format!("(sleep {TIMEOUT_DESCENDANT_DELAY_SECS}; printf leaked > '{}') & wait", marker.display());
    let mut request = shell_request(script);
    request.timeout_secs = TIMEOUT_TEST_SECS;

    let error = runner_with_limits(limits()).run(&request).expect_err("timed-out process group must fail");
    assert_eq!(error, ProcessError::Timeout);

    thread::sleep(Duration::from_millis(TIMEOUT_DESCENDANT_OBSERVATION_MILLIS));
    assert!(!marker.exists(), "timed-out descendant must not survive process-group teardown");
}

// r[verify lattice.workflow_execution.bounded_process_mechanics]
// r[verify lattice.workflow_execution.bounded_process_mechanics.scenario.teardown]
#[test]
fn explicit_cancellation_terminates_the_owned_process_group() {
    let temp_dir = tempfile::tempdir().expect("temporary cancellation directory");
    let marker = temp_dir.path().join("descendant-survived");
    let script = format!("(sleep {DESCENDANT_DELAY_SECS}; printf leaked > '{}') & wait", marker.display());
    let mut request = shell_request(script);
    request.timeout_secs = LONG_TIMEOUT_SECS;
    let runner = runner_with_limits(limits());
    let cancellation = Arc::new(AtomicBool::new(false));
    let worker_cancellation = Arc::clone(&cancellation);
    let worker = thread::spawn(move || runner.run_with_cancellation(&request, &worker_cancellation));

    thread::sleep(Duration::from_millis(CANCELLATION_DELAY_MILLIS));
    cancellation.store(true, Ordering::Release);
    let error = worker
        .join()
        .expect("bounded runner thread must not panic")
        .expect_err("cancelled process must not produce output");
    assert_eq!(error, ProcessError::Cancelled);

    thread::sleep(Duration::from_millis(DESCENDANT_OBSERVATION_MILLIS));
    assert!(!marker.exists(), "cancelled descendant must not survive process-group teardown");
}

// r[verify lattice.workflow_execution.bounded_process_mechanics]
// r[verify lattice.workflow_execution.bounded_process_mechanics.scenario.infrastructure]
#[test]
fn process_failures_use_fixed_bounded_diagnostics_without_secret_material() {
    let mut request = shell_request("unused");
    request.program = "/definitely/missing/lattice-process".to_owned();
    request.env.insert("SECRET".to_owned(), SECRET_ENV_VALUE.to_owned());
    request.stdin = SECRET_STDIN_VALUE.as_bytes().to_vec();

    let error = runner_with_limits(limits()).run(&request).expect_err("missing executable must fail");
    let diagnostic = error.to_string();
    assert!(diagnostic.len() <= DIAGNOSTIC_MAX_CHARS);
    assert!(!diagnostic.contains(SECRET_ENV_VALUE));
    assert!(!diagnostic.contains(SECRET_STDIN_VALUE));
    assert!(!diagnostic.contains("definitely/missing"));
}

// r[verify lattice.agent_runtime.bounded_command_process]
// r[verify lattice.agent_runtime.bounded_command_process.scenario.success]
#[test]
fn bounded_agent_output_reaches_typed_decoding() {
    let plan = AgentCommandPlan {
        target: AgentTarget::Pi,
        adapter_label: COMMAND_AGENT_LABEL,
        program: SHELL.to_owned(),
        args: vec![SHELL_COMMAND_FLAG.to_owned(), format!("printf {BOUNDED_AGENT_TEXT}")],
        cwd: Some("/".to_owned()),
        output_mode: AgentOutputMode::Text,
        timeout_secs: TEST_TIMEOUT_SECS,
        max_output_bytes: TEST_LIMIT_BYTES,
    };
    let output = run_agent_command_with_runner(&plan, &runner_with_limits(limits()))
        .expect("bounded agent command must complete");
    let response = decode_agent_command_output(&plan, &output).expect("complete bounded agent output must decode");
    assert_eq!(response.output, AgentOutput::Text(BOUNDED_AGENT_TEXT.to_owned()));
}

// r[verify lattice.agent_runtime.bounded_command_process]
// r[verify lattice.agent_runtime.bounded_command_process.scenario.failure]
#[test]
fn command_agent_timeout_fails_before_output_decoding() {
    let plan = AgentCommandPlan {
        target: AgentTarget::Pi,
        adapter_label: COMMAND_AGENT_LABEL,
        program: SHELL.to_owned(),
        args: vec![
            SHELL_COMMAND_FLAG.to_owned(),
            format!("sleep {LONG_TIMEOUT_SECS}; printf late"),
        ],
        cwd: Some("/".to_owned()),
        output_mode: AgentOutputMode::Text,
        timeout_secs: TIMEOUT_TEST_SECS,
        max_output_bytes: TEST_LIMIT_BYTES,
    };

    let error = run_agent_command_with_runner(&plan, &runner_with_limits(limits()))
        .expect_err("timed-out agent command must fail before decoding");
    assert!(error.to_string().contains("process timed out"));
}

// r[verify lattice.agent_runtime.bounded_command_process]
// r[verify lattice.agent_runtime.bounded_command_process.scenario.failure]
#[test]
fn truncated_agent_output_is_rejected_before_agent_output_decoding() {
    let runner = runner_with_limits(ProcessLimitConfig {
        stdout_max_bytes: SMALL_LIMIT_BYTES,
        ..limits()
    });
    let plan = AgentCommandPlan {
        target: AgentTarget::Pi,
        adapter_label: COMMAND_AGENT_LABEL,
        program: SHELL.to_owned(),
        args: vec![
            SHELL_COMMAND_FLAG.to_owned(),
            format!("printf '{{\"accepted\":true}}'; {}", repeated_output_script("", OVERFLOW_OUTPUT_BYTES)),
        ],
        cwd: Some("/".to_owned()),
        output_mode: AgentOutputMode::Json,
        timeout_secs: TEST_TIMEOUT_SECS,
        max_output_bytes: TEST_LIMIT_BYTES,
    };

    let error =
        run_agent_command_with_runner(&plan, &runner).expect_err("truncated agent output must not reach JSON decoding");
    assert!(error.to_string().contains("process output exceeded limit"));
}

// r[verify lattice.credentials.bounded_command_provider]
// r[verify lattice.credentials.bounded_command_provider.scenario.success]
#[test]
fn bounded_credential_output_resolves_only_at_the_execution_boundary() {
    let mut config = CommandCredentialProviderConfig::new(SHELL);
    config.args = vec![
        SHELL_COMMAND_FLAG.to_owned(),
        format!("cat >/dev/null; printf '{{\"value\":\"{RESOLVED_CREDENTIAL}\"}}'"),
    ];
    config.cwd = Some("/".to_owned());
    config.timeout_secs = TEST_TIMEOUT_SECS;
    config.output_limit_bytes = TEST_LIMIT_BYTES;
    let provider = CommandCredentialProvider::new(config, runner_with_limits(limits()));

    let value = provider
        .resolve(&SecretRef::new(COMMAND_PROVIDER_ID, COMMAND_PROVIDER_KEY, None))
        .expect("bounded credential response must resolve");
    assert_eq!(value.expose_for_execution(), RESOLVED_CREDENTIAL);
}

// r[verify lattice.credentials.bounded_command_provider]
// r[verify lattice.credentials.bounded_command_provider.scenario.failure]
#[test]
fn command_credential_timeout_fails_without_output_or_secret_leakage() {
    let mut config = CommandCredentialProviderConfig::new(SHELL);
    config.args = vec![
        SHELL_COMMAND_FLAG.to_owned(),
        format!("sleep {LONG_TIMEOUT_SECS}; printf '{{\"value\":\"late-secret\"}}'"),
    ];
    config.cwd = Some("/".to_owned());
    config.timeout_secs = TIMEOUT_TEST_SECS;
    config.output_limit_bytes = TEST_LIMIT_BYTES;
    let provider = CommandCredentialProvider::new(config, runner_with_limits(limits()));

    let error = provider
        .resolve(&SecretRef::new(COMMAND_PROVIDER_ID, COMMAND_PROVIDER_KEY, None))
        .expect_err("timed-out credential command must fail");
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("process timed out"));
    assert!(!diagnostic.contains("late-secret"));
}

// r[verify lattice.credentials.bounded_command_provider]
// r[verify lattice.credentials.bounded_command_provider.scenario.failure]
#[test]
fn truncated_credential_output_is_rejected_before_secret_parsing() {
    let script = format!(
        "printf '{{\"value\":\"must-not-be-returned\"}}'; {}",
        repeated_output_script("", OVERFLOW_OUTPUT_BYTES)
    );
    let mut config = CommandCredentialProviderConfig::new(SHELL);
    config.args = vec![SHELL_COMMAND_FLAG.to_owned(), script];
    config.cwd = Some("/".to_owned());
    config.timeout_secs = TEST_TIMEOUT_SECS;
    config.output_limit_bytes = SMALL_LIMIT_BYTES;
    let provider = CommandCredentialProvider::new(config, runner_with_limits(limits()));

    let error = provider
        .resolve(&SecretRef::new(COMMAND_PROVIDER_ID, COMMAND_PROVIDER_KEY, None))
        .expect_err("truncated credential output must not be parsed");
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("process output exceeded limit"));
    assert!(!diagnostic.contains("must-not-be-returned"));
}

// r[verify lattice.agent_runtime.bounded_command_process]
#[test]
fn command_agent_plans_always_receive_a_positive_configured_deadline() {
    let config = AgentRuntimeConfig::default();
    assert!(config.local_process.limits.command_agent_timeout_secs > 0);
}

// r[verify lattice.workflow_execution.bounded_process_policy]
// r[verify lattice.workflow_execution.bounded_process_policy.scenario.valid]
#[test]
fn application_config_propagates_typed_process_limits_and_shell_identity() {
    let temp_dir = tempfile::tempdir().expect("temporary typed process config directory");
    let config_path = temp_dir.path().join("config.ncl");
    fs::write(&config_path, lattice::config::default_config_ncl()).expect("write typed default process config");
    let loaded = lattice::config::load_app_config(&config_path);
    assert!(loaded.process_limits.stdin_max_bytes > 0);
    assert!(loaded.process_limits.stdout_max_bytes > 0);
    assert!(loaded.process_limits.stderr_max_bytes > 0);
    assert!(loaded.process_limits.command_agent_timeout_secs > 0);
    assert!(loaded.process_limits.poll_interval_millis > 0);
    assert!(loaded.process_limits.teardown_grace_millis > 0);

    let process_limits = ProcessLimitConfig {
        stdout_max_bytes: SMALL_LIMIT_BYTES,
        ..ProcessLimitConfig::default()
    };
    let config = lattice::config::AppConfig {
        shell: SHELL.to_owned(),
        process_limits,
        ..lattice::config::AppConfig::default()
    };

    let runtime = config.effective_agent_runtime();
    assert_eq!(runtime.local_process.shell_program, SHELL);
    assert_eq!(runtime.local_process.limits.stdout_max_bytes, SMALL_LIMIT_BYTES);
    assert!(runtime.local_process.limits.stdin_max_bytes > 0);
    assert!(runtime.local_process.limits.stderr_max_bytes > 0);
    assert!(runtime.local_process.limits.poll_interval_millis > 0);
    assert!(runtime.local_process.limits.teardown_grace_millis > 0);
}

fn valid_admission_input() -> AdmissionInput {
    AdmissionInput {
        program: PathBuf::from(SHELL),
        args: vec![SHELL_COMMAND_FLAG.to_owned(), "printf ok".to_owned()],
        current_dir: PathBuf::from("/tmp"),
        environment_mode: bounded_exec::EnvironmentMode::Inherit,
        environment: vec![("KEY".to_owned(), "value".to_owned())],
        stdin: Vec::new(),
        timeout: Duration::from_secs(TEST_TIMEOUT_SECS),
        stdout_max_bytes: TEST_LIMIT_BYTES,
        stderr_max_bytes: TEST_LIMIT_BYTES,
        stdin_max_bytes: TEST_LIMIT_BYTES,
        poll_interval: Duration::from_millis(POLL_INTERVAL_MILLIS),
        teardown_grace: Duration::from_millis(TEARDOWN_MILLIS),
        accepted_exit_codes: vec![SUCCESS_EXIT_CODE],
        reject_stdout_truncation: true,
        reject_stderr_truncation: true,
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
    }
}

// r[verify lattice.workflow_execution.bounded_process_policy.scenario.valid]
#[test]
fn pure_admission_accepts_a_complete_absolute_request() {
    let admitted = admit(valid_admission_input()).expect("valid request must be admitted");
    assert_eq!(admitted.program, PathBuf::from(SHELL));
    assert_eq!(admitted.environment.len(), 1);
}

// r[verify lattice.workflow_execution.bounded_process_policy.scenario.invalid]
#[test]
fn pure_admission_rejects_invalid_paths_environment_limits_and_policy() {
    let mut relative_program = valid_admission_input();
    relative_program.program = PathBuf::from("sh");
    assert!(matches!(admit(relative_program), Err(ProcessError::InvalidRequest(_))));

    let mut relative_current_dir = valid_admission_input();
    relative_current_dir.current_dir = PathBuf::from("tmp");
    assert!(matches!(admit(relative_current_dir), Err(ProcessError::InvalidRequest(_))));

    let mut duplicate_environment = valid_admission_input();
    duplicate_environment.environment.push(("KEY".to_owned(), "other".to_owned()));
    assert!(matches!(admit(duplicate_environment), Err(ProcessError::InvalidRequest(_))));

    let mut zero_limit = valid_admission_input();
    zero_limit.stdout_max_bytes = 0;
    assert!(matches!(admit(zero_limit), Err(ProcessError::InvalidRequest(_))));

    let mut unsupported_poll = valid_admission_input();
    unsupported_poll.poll_interval = unsupported_poll.timeout + Duration::from_millis(POLL_INTERVAL_MILLIS);
    assert!(matches!(admit(unsupported_poll), Err(ProcessError::InvalidRequest(_))));

    let mut empty_exit_policy = valid_admission_input();
    empty_exit_policy.accepted_exit_codes.clear();
    assert!(matches!(admit(empty_exit_policy), Err(ProcessError::InvalidRequest(_))));

    let mut unsupported_argument = valid_admission_input();
    unsupported_argument.args.push("bad\0argument".to_owned());
    assert!(matches!(admit(unsupported_argument), Err(ProcessError::InvalidRequest(_))));

    let mut unsupported_environment = valid_admission_input();
    unsupported_environment.environment.push(("BAD=KEY".to_owned(), "value".to_owned()));
    assert!(matches!(admit(unsupported_environment), Err(ProcessError::InvalidRequest(_))));
}

// r[verify lattice.workflow_execution.bounded_process_policy.scenario.invalid]
#[test]
fn pure_admission_rejects_stdin_overflow() {
    let mut overflow = valid_admission_input();
    overflow.stdin = vec![b'x'; TEST_LIMIT_BYTES + 1];
    assert!(matches!(admit(overflow), Err(ProcessError::InvalidRequest(_))));
}
