use super::model::{
    AST_GREP_FRESH_ENV, AUTHORITY_FRESH_ENV, CARGO_STEPS_ENV, CLANKERS_CONSUMER_MANIFEST,
    DEFAULT_RECEIPT_DIR, GIT_FETCH_ENV, ISSUER_FRESH_ENV, LICENSE_FRESH_ENV, OUTPUT_TAIL_LIMIT,
    POLICY_FRESH_ENV, RECEIPT_DIR_ENV, SEMANTICS_FRESH_ENV, Step, StepKind, StepReceipt,
    TARGET_ENV, WRAPPER_ENV,
};
use super::plan::steps;
use super::receipts::write_receipts;
use super::scratch;
use super::validators::{
    validate_license_metadata, validate_nickel_identity, validate_package_contents,
    validate_private_source,
};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

pub(super) fn run() -> ExitCode {
    let root = env::current_dir().unwrap_or_else(|error| {
        eprintln!("failed to read current directory: {error}");
        std::process::exit(1);
    });
    let scratch = match scratch_directory() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };
    let receipt_dir = env::var_os(RECEIPT_DIR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_RECEIPT_DIR));
    if let Err(error) = fs::create_dir_all(&receipt_dir) {
        eprintln!(
            "failed to create receipt directory {}: {error}",
            receipt_dir.display()
        );
        return ExitCode::from(1);
    }

    let mut receipts = Vec::new();
    let mut failed = false;
    for step in steps() {
        let receipt = run_step(&root, &step, scratch.as_deref());
        if !receipt.success {
            eprintln!("release verification failed at {}", step.name);
            failed = true;
        }
        receipts.push(receipt);
        if failed {
            break;
        }
    }

    let all_passed = !failed;
    if let Err(error) = write_receipts(&receipt_dir, all_passed, &receipts) {
        eprintln!("failed to write release verification receipts: {error}");
        return ExitCode::from(1);
    }

    if all_passed {
        println!("release verification passed");
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn prevalidated_receipt(step: &Step, environment_name: &str) -> StepReceipt {
    println!("==> {}", step.name);
    StepReceipt {
        name: step.name,
        command: format!("{environment_name}=1 prevalidated by flake check input"),
        status: Some(0),
        success: true,
        elapsed: Duration::ZERO,
        stdout_tail: String::new(),
        stderr_tail: String::new(),
    }
}

fn scratch_directory() -> Result<Option<String>, String> {
    let Some(value) = env::var_os(scratch::ENV) else {
        return Ok(None);
    };
    let value = value
        .into_string()
        .map_err(|_| "release scratch directory must be UTF-8")?;
    scratch::validate(&value)?;
    fs::create_dir_all(&value)
        .map_err(|error| format!("cannot create release scratch directory: {error}"))?;
    Ok(Some(value))
}

fn run_step(root: &Path, step: &Step, scratch: Option<&str>) -> StepReceipt {
    if matches!(step.kind, StepKind::PolicyFreshness) && env_flag(POLICY_FRESH_ENV) {
        return prevalidated_receipt(step, POLICY_FRESH_ENV);
    }
    if matches!(step.kind, StepKind::AuthorityFixtureFreshness) && env_flag(AUTHORITY_FRESH_ENV) {
        return prevalidated_receipt(step, AUTHORITY_FRESH_ENV);
    }
    if matches!(step.kind, StepKind::SemanticsFixtureFreshness) && env_flag(SEMANTICS_FRESH_ENV) {
        return prevalidated_receipt(step, SEMANTICS_FRESH_ENV);
    }
    if matches!(step.kind, StepKind::LicenseFixtureFreshness) && env_flag(LICENSE_FRESH_ENV) {
        return prevalidated_receipt(step, LICENSE_FRESH_ENV);
    }
    if matches!(step.kind, StepKind::IssuerFixtureFreshness) && env_flag(ISSUER_FRESH_ENV) {
        return prevalidated_receipt(step, ISSUER_FRESH_ENV);
    }
    if matches!(step.kind, StepKind::AstGrepAuditFreshness) && env_flag(AST_GREP_FRESH_ENV) {
        return prevalidated_receipt(step, AST_GREP_FRESH_ENV);
    }
    if step.cargo_env && env::var_os(CARGO_STEPS_ENV).is_some_and(|value| value == "0") {
        println!("==> {}", step.name);
        return StepReceipt {
            name: step.name,
            command: format!("{}=0 skipped in sandboxed flake check", CARGO_STEPS_ENV),
            status: Some(0),
            success: true,
            elapsed: Duration::ZERO,
            stdout_tail: "skipped: run the direct release verifier for Cargo-dependent rails"
                .to_owned(),
            stderr_tail: String::new(),
        };
    }

    if matches!(step.kind, StepKind::ClankersConsumerFixture)
        && !root.join(CLANKERS_CONSUMER_MANIFEST).exists()
    {
        println!("==> {}", step.name);
        return StepReceipt {
            name: step.name,
            command: format!("cargo test --manifest-path {CLANKERS_CONSUMER_MANIFEST}"),
            status: Some(0),
            success: true,
            elapsed: Duration::ZERO,
            stdout_tail: format!(
                "deferred: {CLANKERS_CONSUMER_MANIFEST} is unavailable in this checkout"
            ),
            stderr_tail: String::new(),
        };
    }

    let program = scratch::relocate(scratch, step.program);
    let target = scratch::relocate(scratch, step.target_dir);
    let mut args: Vec<OsString> = step
        .args
        .iter()
        .map(|arg| OsString::from(scratch::relocate(scratch, arg)))
        .collect();
    let check_name = match step.kind {
        StepKind::PolicyFreshness => Some("policy-export"),
        StepKind::AuthorityFixtureFreshness => Some("authority-core-fixtures"),
        StepKind::SemanticsFixtureFreshness => Some("ucan-semantics-fixtures"),
        StepKind::LicenseFixtureFreshness => Some("license-consumption-fixtures"),
        StepKind::IssuerFixtureFreshness => Some("secretspec-issuer-fixtures"),
        StepKind::AstGrepAuditFreshness => Some("ast-grep-policy-audits"),
        _ => None,
    };
    if let Some(check_name) = check_name {
        match check_attr(check_name, scratch) {
            Ok(attr) => args.push(attr),
            Err(error) => {
                return StepReceipt {
                    name: step.name,
                    command: format_command(&program, &args),
                    status: None,
                    success: false,
                    elapsed: Duration::ZERO,
                    stdout_tail: String::new(),
                    stderr_tail: error,
                };
            }
        }
    }

    println!("==> {}", step.name);
    let start = Instant::now();
    let mut command = Command::new(&program);
    command.current_dir(root).args(&args);
    if let Some(scratch) = scratch {
        command
            .env("TMPDIR", scratch)
            .env("TMP", scratch)
            .env("TEMP", scratch);
    }
    let mut description = format_command(&program, &args);
    if step.cargo_env {
        description = format!("{TARGET_ENV}={target:?} {description}");
        command
            .env(TARGET_ENV, &target)
            .env(WRAPPER_ENV, "")
            .env(GIT_FETCH_ENV, "true");
    }

    match command.output() {
        Ok(output) => receipt_from_output(root, step, description, start, output),
        Err(error) => StepReceipt {
            name: step.name,
            command: description,
            status: None,
            success: false,
            elapsed: start.elapsed(),
            stdout_tail: String::new(),
            stderr_tail: format!("could not start {}: {error}", step.program),
        },
    }
}

fn receipt_from_output(
    root: &Path,
    step: &Step,
    description: String,
    start: Instant,
    output: std::process::Output,
) -> StepReceipt {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let validation = if matches!(step.kind, StepKind::PackageContents) && output.status.success() {
        validate_package_contents(&stdout).and_then(|()| validate_private_source(root))
    } else if matches!(step.kind, StepKind::LicenseMetadata) && output.status.success() {
        validate_license_metadata(&root.join("Cargo.toml"), &stdout)
    } else if matches!(step.kind, StepKind::NickelCohortIdentity) && output.status.success() {
        validate_nickel_identity(&stdout)
    } else if matches!(step.kind, StepKind::EnforcementDependencies) && output.status.success() {
        super::enforcement_profile::validate(&stdout)
    } else {
        Ok(())
    };
    if output.status.success() {
        print!("{stdout}");
        eprint!("{stderr}");
    }
    let mut success = output.status.success();
    let mut stderr_tail = tail(&stderr);
    if let Err(error) = validation {
        success = false;
        if !stderr_tail.is_empty() {
            stderr_tail.push('\n');
        }
        stderr_tail.push_str(&error);
        eprintln!("{error}");
    }
    StepReceipt {
        name: step.name,
        command: description,
        status: output.status.code(),
        success,
        elapsed: start.elapsed(),
        stdout_tail: tail(&stdout),
        stderr_tail,
    }
}

fn check_attr(check_name: &str, scratch: Option<&str>) -> Result<OsString, String> {
    let mut command = Command::new("nix");
    if let Some(scratch) = scratch {
        command.env("TMPDIR", scratch);
    }
    let output = command
        .args([
            "eval",
            "--raw",
            "--impure",
            "--expr",
            "builtins.currentSystem",
        ])
        .output()
        .map_err(|error| format!("could not evaluate current Nix system: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "nix eval exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let system = String::from_utf8(output.stdout)
        .map_err(|error| format!("Nix system was not UTF-8: {error}"))?;
    Ok(OsString::from(format!(
        "path:.#checks.{}.{check_name}",
        system.trim()
    )))
}

fn env_flag(name: &str) -> bool {
    env::var_os(name).is_some_and(|value| value == "1" || value == "true")
}

fn format_command(program: &str, args: &[OsString]) -> String {
    let mut command = program.to_owned();
    for arg in args {
        command.push(' ');
        command.push_str(&arg.to_string_lossy());
    }
    command
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
