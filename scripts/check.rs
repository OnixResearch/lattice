#!/usr/bin/env -S nix develop . -c cargo -q -Zscript

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

const CARGO_PROGRAM: &str = "cargo";
const RUSTFMT_PROGRAM: &str = "rustfmt";
const CARGO_TARGET_DIR_KEY: &str = "CARGO_TARGET_DIR";
const REPO_LOCAL_TARGET_DIR: &str = "target";
const CAIRN_SPECS_DIR: &str = "cairn/specs";
const SPEC_FILE_NAME: &str = "spec.md";
const GENERIC_FAILURE_EXIT_CODE: u8 = 1;
const PROCESS_STATUS_CODE_OVERFLOW_EXIT_CODE: u8 = 255;
const LINE_NUMBER_BASE: usize = 1;

const SCRIPT_FORMAT_ARGS: &[&str] = &["--check", "scripts/check.rs"];
const INVENTORY_SCRIPT_FORMAT_ARGS: &[&str] = &["--check", "--edition", "2024", "scripts/traceability-inventory.rs"];
const INVENTORY_SCRIPT_SMOKE_ARGS: &[&str] = &[
    "-q",
    "-Zscript",
    "scripts/traceability-inventory.rs",
    "hash",
    "tests/fixtures/traceability-inventory-coverage.json",
];
const FORMAT_CHECK_ARGS: &[&str] = &["fmt", "--check", "--package", "lattice"];
const FOCUSED_CONSUMER_CHECK_ARGS: &[&str] = &["check", "--package", "lattice", "--all-targets"];
const TEST_ARGS: &[&str] = &["test"];
const CLIPPY_ARGS: &[&str] = &[
    "clippy",
    "--package",
    "lattice",
    "--all-targets",
    "--no-deps",
    "--",
    "-D",
    "warnings",
];
const NORMATIVE_WORDS: &[&str] = &["MUST", "SHOULD", "SHALL", "MAY"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckStep {
    name: &'static str,
    program: &'static str,
    args: &'static [&'static str],
    use_repo_local_target: bool,
}

const COMMAND_STEPS: &[CheckStep] = &[
    CheckStep {
        name: "script format",
        program: RUSTFMT_PROGRAM,
        args: SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "inventory script format",
        program: RUSTFMT_PROGRAM,
        args: INVENTORY_SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "inventory script smoke",
        program: CARGO_PROGRAM,
        args: INVENTORY_SCRIPT_SMOKE_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "format",
        program: CARGO_PROGRAM,
        args: FORMAT_CHECK_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "Rats focused consumer compile",
        program: CARGO_PROGRAM,
        args: FOCUSED_CONSUMER_CHECK_ARGS,
        use_repo_local_target: true,
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
    match run_checks(COMMAND_STEPS) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            error.exit_code
        }
    }
}

#[derive(Debug, Clone)]
struct CheckError {
    step_name: String,
    detail: Option<String>,
    exit_code: ExitCode,
}

impl CheckError {
    fn failed_step(step_name: &str, exit_code: ExitCode) -> Self {
        Self {
            step_name: step_name.to_string(),
            detail: None,
            exit_code,
        }
    }

    fn with_detail(step_name: &str, detail: String) -> Self {
        Self {
            step_name: step_name.to_string(),
            detail: Some(detail),
            exit_code: ExitCode::from(GENERIC_FAILURE_EXIT_CODE),
        }
    }
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            Some(detail) => write!(formatter, "check step '{}' failed\n{detail}", self.step_name),
            None => write!(formatter, "check step '{}' failed", self.step_name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpecDiagnostic {
    path: String,
    line_number: usize,
    message: String,
}

impl std::fmt::Display for SpecDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}:{}: {}", self.path, self.line_number, self.message)
    }
}

fn run_checks(command_steps: &[CheckStep]) -> Result<(), CheckError> {
    for step in command_steps {
        run_step(*step)?;
    }
    run_cairn_spec_validation()?;
    Ok(())
}

fn run_step(step: CheckStep) -> Result<(), CheckError> {
    println!("==> {}", step.name);
    let mut command = Command::new(step.program);
    command.args(step.args);
    if step.use_repo_local_target {
        command.env(CARGO_TARGET_DIR_KEY, REPO_LOCAL_TARGET_DIR);
    }

    let status = command
        .status()
        .map_err(|_| CheckError::failed_step(step.name, ExitCode::from(GENERIC_FAILURE_EXIT_CODE)))?;

    if status.success() {
        return Ok(());
    }

    Err(CheckError::failed_step(step.name, status_exit_code(status.code())))
}

fn run_cairn_spec_validation() -> Result<(), CheckError> {
    println!("==> cairn specs");
    let spec_files = collect_spec_files(Path::new(CAIRN_SPECS_DIR))
        .map_err(|error| CheckError::with_detail("cairn specs", error.to_string()))?;
    let mut diagnostics = Vec::new();

    for path in spec_files {
        let text = fs::read_to_string(&path)
            .map_err(|error| CheckError::with_detail("cairn specs", format!("{}: {error}", path.display())))?;
        diagnostics.extend(validate_spec_text(&path.display().to_string(), &text));
    }

    if diagnostics.is_empty() {
        return Ok(());
    }

    let detail = diagnostics.into_iter().map(|diagnostic| diagnostic.to_string()).collect::<Vec<_>>().join("\n");
    Err(CheckError::with_detail("cairn specs", detail))
}

fn collect_spec_files(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    collect_spec_files_into(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_spec_files_into(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if path.is_file() {
        if path.file_name().and_then(|name| name.to_str()) == Some(SPEC_FILE_NAME) {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        collect_spec_files_into(&entry.path(), files)?;
    }
    Ok(())
}

fn validate_spec_text(path: &str, text: &str) -> Vec<SpecDiagnostic> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut diagnostics = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("### Requirement:") {
            validate_requirement(path, &lines, line_index, &mut diagnostics);
        }
        if trimmed.starts_with("#### Scenario:") {
            validate_scenario(path, &lines, line_index, &mut diagnostics);
        }
    }

    diagnostics
}

fn validate_requirement(path: &str, lines: &[&str], heading_index: usize, diagnostics: &mut Vec<SpecDiagnostic>) {
    let marker_index = heading_index + LINE_NUMBER_BASE;
    let Some(marker_line) = lines.get(marker_index).map(|line| line.trim()) else {
        diagnostics.push(diagnostic(
            path,
            heading_index,
            "requirement heading must be followed by an r[...] marker line",
        ));
        return;
    };

    if !marker_line.starts_with("r[") {
        diagnostics.push(diagnostic(path, marker_index, "requirement must start with an r[...] marker"));
    }
    if !NORMATIVE_WORDS.iter().any(|word| marker_line.contains(word)) {
        diagnostics.push(diagnostic(
            path,
            marker_index,
            "requirement marker line must include MUST, SHOULD, SHALL, or MAY",
        ));
    }
}

fn validate_scenario(path: &str, lines: &[&str], heading_index: usize, diagnostics: &mut Vec<SpecDiagnostic>) {
    let marker_index = heading_index + LINE_NUMBER_BASE;
    let Some(marker_line) = lines.get(marker_index).map(|line| line.trim()) else {
        diagnostics.push(diagnostic(path, heading_index, "scenario heading must be followed by an r[...] marker line"));
        return;
    };

    if !marker_line.starts_with("r[") {
        diagnostics.push(diagnostic(path, marker_index, "scenario must start with an r[...] marker"));
    }

    let scenario_lines = lines
        .iter()
        .skip(marker_index + LINE_NUMBER_BASE)
        .take_while(|line| !line.trim_start().starts_with("###") && !line.trim_start().starts_with("####"));
    let scenario_text = scenario_lines.map(|line| line.trim()).collect::<Vec<_>>();

    for required_clause in ["- GIVEN", "- WHEN", "- THEN"] {
        if !scenario_text.iter().any(|line| line.starts_with(required_clause)) {
            diagnostics.push(diagnostic(
                path,
                heading_index,
                &format!("scenario must include a {required_clause} clause"),
            ));
        }
    }
}

fn diagnostic(path: &str, zero_based_line_index: usize, message: &str) -> SpecDiagnostic {
    SpecDiagnostic {
        path: path.to_string(),
        line_number: zero_based_line_index + LINE_NUMBER_BASE,
        message: message.to_string(),
    }
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
