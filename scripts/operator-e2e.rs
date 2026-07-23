#!/usr/bin/env -S nix develop . -c cargo -q -Zscript

use std::process::Command;
use std::process::ExitCode;

const CARGO_PROGRAM: &str = "cargo";
const CARGO_TARGET_DIR_KEY: &str = "CARGO_TARGET_DIR";
const REPO_LOCAL_TARGET_DIR: &str = "target";
const GENERIC_FAILURE_EXIT_CODE: u8 = 1;
const PROCESS_STATUS_CODE_OVERFLOW_EXIT_CODE: u8 = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OperatorE2eCase {
    name: &'static str,
    test_target: &'static str,
    test_filter: &'static str,
    exact_filter: bool,
}

const OPERATOR_E2E_CASES: &[OperatorE2eCase] = &[
    OperatorE2eCase {
        name: "TUI PTY success, failure/rerun, diagnostics, cancellation, resize, and non-terminal boundaries",
        test_target: "test_tui_ux_completion",
        test_filter: "operator_e2e_tui_process",
        exact_filter: false,
    },
    OperatorE2eCase {
        name: "webhook listener loopback, duplicate-delivery, and post-effect crash boundaries",
        test_target: "webhooks",
        test_filter: "operator_listener_",
        exact_filter: false,
    },
    OperatorE2eCase {
        name: "scheduler daemon process, pre/post-effect restart, and concurrent-owner boundaries",
        test_target: "test_scheduler",
        test_filter: "operator_e2e_scheduler_",
        exact_filter: false,
    },
    OperatorE2eCase {
        name: "orchestration recovery, forced-death, and concurrent-contender boundaries",
        test_target: "test_orchestration_cli",
        test_filter: "operator_e2e_orchestration_recovery",
        exact_filter: false,
    },
];

fn main() -> ExitCode {
    match run_operator_e2e_cases(OPERATOR_E2E_CASES) {
        Ok(()) => ExitCode::SUCCESS,
        Err(exit_code) => exit_code,
    }
}

fn run_operator_e2e_cases(cases: &[OperatorE2eCase]) -> Result<(), ExitCode> {
    for case in cases {
        println!("==> operator E2E: {}", case.name);
        let status = Command::new(CARGO_PROGRAM)
            .args(cargo_test_args(*case))
            .env(CARGO_TARGET_DIR_KEY, REPO_LOCAL_TARGET_DIR)
            .status()
            .map_err(|_| ExitCode::from(GENERIC_FAILURE_EXIT_CODE))?;
        if !status.success() {
            return Err(status_exit_code(status.code()));
        }
    }
    Ok(())
}

fn cargo_test_args(case: OperatorE2eCase) -> Vec<&'static str> {
    let mut args = vec!["test", "--test", case.test_target, case.test_filter, "--"];
    if case.exact_filter {
        args.push("--exact");
    }
    args
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
