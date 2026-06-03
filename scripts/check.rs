#!/usr/bin/env -S nix develop . -c cargo -q -Zscript

use std::process::Command;
use std::process::ExitCode;

const CARGO_PROGRAM: &str = "cargo";
const CARGO_TARGET_DIR_KEY: &str = "CARGO_TARGET_DIR";
const REPO_LOCAL_TARGET_DIR: &str = "target";
const GENERIC_FAILURE_EXIT_CODE: u8 = 1;
const PROCESS_STATUS_CODE_OVERFLOW_EXIT_CODE: u8 = 255;

const FORMAT_CHECK_ARGS: &[&str] = &["fmt", "--check"];
const TEST_ARGS: &[&str] = &["test"];
const CLIPPY_ARGS: &[&str] = &["clippy", "--all-targets", "--", "-D", "warnings"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckStep {
    name: &'static str,
    program: &'static str,
    args: &'static [&'static str],
    use_repo_local_target: bool,
}

const CHECK_STEPS: &[CheckStep] = &[
    CheckStep {
        name: "format",
        program: CARGO_PROGRAM,
        args: FORMAT_CHECK_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "tests",
        program: CARGO_PROGRAM,
        args: TEST_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "clippy",
        program: CARGO_PROGRAM,
        args: CLIPPY_ARGS,
        use_repo_local_target: true,
    },
];

fn main() -> ExitCode {
    match run_checks(CHECK_STEPS) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            error.exit_code
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CheckError {
    step_name: &'static str,
    exit_code: ExitCode,
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "check step '{}' failed", self.step_name)
    }
}

fn run_checks(steps: &[CheckStep]) -> Result<(), CheckError> {
    for step in steps {
        run_step(*step)?;
    }
    Ok(())
}

fn run_step(step: CheckStep) -> Result<(), CheckError> {
    println!("==> {}", step.name);
    let mut command = Command::new(step.program);
    command.args(step.args);
    if step.use_repo_local_target {
        command.env(CARGO_TARGET_DIR_KEY, REPO_LOCAL_TARGET_DIR);
    }

    let status = command.status().map_err(|_| CheckError {
        step_name: step.name,
        exit_code: ExitCode::from(GENERIC_FAILURE_EXIT_CODE),
    })?;

    if status.success() {
        return Ok(());
    }

    Err(CheckError {
        step_name: step.name,
        exit_code: status_exit_code(status.code()),
    })
}

fn status_exit_code(code: Option<i32>) -> ExitCode {
    let Some(code) = code else {
        return ExitCode::from(GENERIC_FAILURE_EXIT_CODE);
    };
    let Ok(code) = u8::try_from(code) else {
        return ExitCode::from(PROCESS_STATUS_CODE_OVERFLOW_EXIT_CODE);
    };
    ExitCode::from(code)
}
