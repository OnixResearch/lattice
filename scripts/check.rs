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
const CAIRN_SPECS_DIR: &str = ".cairn/specs";
const SPEC_FILE_NAME: &str = "spec.md";
const GENERIC_FAILURE_EXIT_CODE: u8 = 1;
const PROCESS_STATUS_CODE_OVERFLOW_EXIT_CODE: u8 = 255;
const LINE_NUMBER_BASE: usize = 1;

const SCRIPT_FORMAT_ARGS: &[&str] = &["--check", "--edition", "2024", "scripts/check.rs"];
const OPERATOR_E2E_SCRIPT_FORMAT_ARGS: &[&str] = &["--check", "--edition", "2024", "scripts/operator-e2e.rs"];
const INVENTORY_SCRIPT_FORMAT_ARGS: &[&str] = &[
    "--check",
    "--edition",
    "2024",
    "scripts/traceability-inventory.rs",
    "scripts/traceability_inventory/core.rs",
];
const RELEASE_INPUTS_SCRIPT_FORMAT_ARGS: &[&str] = &[
    "--check",
    "--edition",
    "2024",
    "scripts/release-readiness-inputs.rs",
    "scripts/release_readiness/core.rs",
];
const OCTET_MIGRATION_FORMAT_ARGS: &[&str] = &[
    "--check",
    "--edition",
    "2024",
    "scripts/octet-baseline-migrate.rs",
    "scripts/octet_baseline_migration.rs",
];
const TOON_SYNC_SCRIPT_FORMAT_ARGS: &[&str] = &["--check", "--edition", "2024", "scripts/sync_toon_format.rs"];
const WORKFLOW_BOUNDARY_SCRIPT_FORMAT_ARGS: &[&str] = &[
    "--check",
    "--edition",
    "2024",
    "scripts/check-workflow-domain-boundaries.rs",
];
const EFFECT_AUTHORITY_SCRIPT_FORMAT_ARGS: &[&str] = &[
    "--check",
    "--edition",
    "2024",
    "scripts/check-choregraph-effect-authority.rs",
];
const ROLE_PROTOCOL_SCRIPT_FORMAT_ARGS: &[&str] = &[
    "--check",
    "--edition",
    "2024",
    "scripts/check-role-protocol-boundary.rs",
];
const ORCHESTRATION_IDENTITY_SCRIPT_FORMAT_ARGS: &[&str] = &[
    "--check",
    "--edition",
    "2024",
    "scripts/check-orchestration-identity-boundary.rs",
];
// Cargo scripts generate their own lockfile under the target directory; a fresh
// target has none, so these invocations cannot use --locked. Each script pins
// its direct dependencies exactly.
const INVENTORY_SCRIPT_SMOKE_ARGS: &[&str] = &[
    "-q",
    "-Zscript",
    "scripts/traceability-inventory.rs",
    "hash",
    "tests/fixtures/traceability-inventory-coverage.json",
];
const TOON_SYNC_SCRIPT_TEST_ARGS: &[&str] = &[
    "-q",
    "-Zscript",
    "test",
    "--manifest-path",
    "scripts/sync_toon_format.rs",
];
const WORKFLOW_BOUNDARY_SELF_TEST_ARGS: &[&str] = &[
    "-q",
    "-Zscript",
    "scripts/check-workflow-domain-boundaries.rs",
    "--self-test",
];
const WORKFLOW_BOUNDARY_CHECK_ARGS: &[&str] = &["-q", "-Zscript", "scripts/check-workflow-domain-boundaries.rs"];
const EFFECT_AUTHORITY_CHECK_ARGS: &[&str] = &["-q", "-Zscript", "scripts/check-choregraph-effect-authority.rs"];
const ROLE_PROTOCOL_CHECK_ARGS: &[&str] = &["-q", "-Zscript", "scripts/check-role-protocol-boundary.rs"];
const ORCHESTRATION_IDENTITY_SELF_TEST_ARGS: &[&str] = &[
    "-q",
    "-Zscript",
    "scripts/check-orchestration-identity-boundary.rs",
    "--self-test",
];
const ORCHESTRATION_IDENTITY_CHECK_ARGS: &[&str] =
    &["-q", "-Zscript", "scripts/check-orchestration-identity-boundary.rs"];
const OPERATOR_E2E_SCRIPT_ARGS: &[&str] = &["-q", "-Zscript", "scripts/operator-e2e.rs"];
const FORMAT_CHECK_ARGS: &[&str] = &[
    "fmt",
    "--check",
    "--package",
    "lattice",
    "--package",
    "budget-ledger-core",
    "--package",
    "lattice-core",
    "--package",
    "lattice-compose-binding",
    "--package",
    "lattice-workflow-contract",
    "--package",
    "lattice-workflow-transport",
];
const POLICY_TYPECHECK_ARGS: &[&str] = &[
    "typecheck",
    "cairn-policy/default.ncl",
    "cairn-policy/traceability-inventory.ncl",
];
const POLICY_FRESHNESS_ARGS: &[&str] = &[
    "-c",
    "\"${LATTICE_CAIRN_BIN:-cairn}\" policy export --source cairn-policy/default.ncl --output cairn-policy/generated/cairn-policy.json --check",
];
const RUNTIME_NICKEL_TYPECHECK_ARGS: &[&str] = &[
    "-c",
    "nickel typecheck cairn-policy/*.ncl policy/*.ncl profiles/*.ncl contracts/*.ncl",
];
const EXAMPLE_NICKEL_TYPECHECK_ARGS: &[&str] = &[
    "-c",
    "nickel typecheck examples/config/*.ncl examples/orchestrations/*.ncl examples/templates/*.ncl examples/workflows/*.ncl",
];
const FOCUSED_CONSUMER_CHECK_ARGS: &[&str] = &["check", "--package", "lattice", "--all-targets", "--locked"];
const ALL_FEATURES_CHECK_ARGS: &[&str] = &[
    "check",
    "--package",
    "lattice",
    "--all-features",
    "--all-targets",
    "--locked",
];
const ALL_FEATURES_TEST_ARGS: &[&str] = &["test", "--workspace", "--all-features", "--locked"];
// r[impl lattice.agent_evaluation.verification]
const AGENT_EVALUATION_TEST_ARGS: &[&str] = &[
    "test",
    "--test",
    "test_agent_evaluation_core",
    "--test",
    "test_agent_evaluation_shell",
    "--locked",
];
const OPERATION_CONTRACT_TEST_ARGS: &[&str] = &[
    "test",
    "--test",
    "test_operation_contracts",
    "--test",
    "test_operation_cli",
    "--locked",
];
const TEST_ARGS: &[&str] = &["test", "--workspace", "--locked"];
const CLIPPY_ARGS: &[&str] = &[
    "clippy",
    "--workspace",
    "--exclude",
    "lattice-repository-validation-fixture",
    "--all-features",
    "--all-targets",
    "--no-deps",
    "--locked",
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
        name: "operator E2E script format",
        program: RUSTFMT_PROGRAM,
        args: OPERATOR_E2E_SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "inventory script format",
        program: RUSTFMT_PROGRAM,
        args: INVENTORY_SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "Octet migration script format",
        program: RUSTFMT_PROGRAM,
        args: OCTET_MIGRATION_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "TOON sync script format",
        program: RUSTFMT_PROGRAM,
        args: TOON_SYNC_SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "workflow boundary script format",
        program: RUSTFMT_PROGRAM,
        args: WORKFLOW_BOUNDARY_SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "effect authority script format",
        program: RUSTFMT_PROGRAM,
        args: EFFECT_AUTHORITY_SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "role protocol script format",
        program: RUSTFMT_PROGRAM,
        args: ROLE_PROTOCOL_SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "orchestration identity script format",
        program: RUSTFMT_PROGRAM,
        args: ORCHESTRATION_IDENTITY_SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "release inputs script format",
        program: RUSTFMT_PROGRAM,
        args: RELEASE_INPUTS_SCRIPT_FORMAT_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "inventory script smoke",
        program: CARGO_PROGRAM,
        args: INVENTORY_SCRIPT_SMOKE_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "TOON sync script tests",
        program: CARGO_PROGRAM,
        args: TOON_SYNC_SCRIPT_TEST_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "workflow boundary guard self-test",
        program: CARGO_PROGRAM,
        args: WORKFLOW_BOUNDARY_SELF_TEST_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "workflow boundary guard",
        program: CARGO_PROGRAM,
        args: WORKFLOW_BOUNDARY_CHECK_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "effect authority guard",
        program: CARGO_PROGRAM,
        args: EFFECT_AUTHORITY_CHECK_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "role protocol boundary guard",
        program: CARGO_PROGRAM,
        args: ROLE_PROTOCOL_CHECK_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "orchestration identity boundary self-test",
        program: CARGO_PROGRAM,
        args: ORCHESTRATION_IDENTITY_SELF_TEST_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "orchestration identity boundary guard",
        program: CARGO_PROGRAM,
        args: ORCHESTRATION_IDENTITY_CHECK_ARGS,
        use_repo_local_target: true,
    },
    // r[impl lattice.choregraph_effect_authority.validation.scenario.crash]
    CheckStep {
        name: "operator E2E",
        program: CARGO_PROGRAM,
        args: OPERATOR_E2E_SCRIPT_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "format",
        program: CARGO_PROGRAM,
        args: FORMAT_CHECK_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "policy typecheck",
        program: "nickel",
        args: POLICY_TYPECHECK_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "policy freshness",
        program: "sh",
        args: POLICY_FRESHNESS_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "runtime Nickel typecheck",
        program: "sh",
        args: RUNTIME_NICKEL_TYPECHECK_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "example Nickel typecheck",
        program: "sh",
        args: EXAMPLE_NICKEL_TYPECHECK_ARGS,
        use_repo_local_target: false,
    },
    CheckStep {
        name: "Rats focused consumer compile",
        program: CARGO_PROGRAM,
        args: FOCUSED_CONSUMER_CHECK_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "agent evaluation",
        program: CARGO_PROGRAM,
        args: AGENT_EVALUATION_TEST_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "operation contracts",
        program: CARGO_PROGRAM,
        args: OPERATION_CONTRACT_TEST_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "all features",
        program: CARGO_PROGRAM,
        args: ALL_FEATURES_CHECK_ARGS,
        use_repo_local_target: true,
    },
    CheckStep {
        name: "all features tests",
        program: CARGO_PROGRAM,
        args: ALL_FEATURES_TEST_ARGS,
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
    if !Path::new("Cargo.toml").is_file() {
        eprintln!("run scripts/check.rs from the Lattice repository root");
        return ExitCode::FAILURE;
    }
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

// r[impl lattice.nickel_toolchain.validation]
// r[impl lattice.nickel_toolchain.validation.scenario.passed]
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

    let status = command.status().map_err(|error| CheckError {
        step_name: step.name.to_string(),
        detail: Some(format!("failed to run '{}': {error}", rendered_command(step))),
        exit_code: ExitCode::from(GENERIC_FAILURE_EXIT_CODE),
    })?;

    if status.success() {
        return Ok(());
    }

    Err(CheckError {
        step_name: step.name.to_string(),
        detail: Some(format!("command failed: {}", rendered_command(step))),
        exit_code: status_exit_code(status.code()),
    })
}

fn rendered_command(step: CheckStep) -> String {
    if step.args.is_empty() {
        step.program.to_string()
    } else {
        format!("{} {}", step.program, step.args.join(" "))
    }
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
        if let Some(title) = trimmed.strip_prefix("### Requirement:") {
            if title.trim().is_empty() {
                diagnostics.push(diagnostic(path, line_index, "requirement heading must include a title"));
            }
            validate_requirement(path, &lines, line_index, &mut diagnostics);
        }
        if let Some(title) = trimmed.strip_prefix("#### Scenario:") {
            if title.trim().is_empty() {
                diagnostics.push(diagnostic(path, line_index, "scenario heading must include a title"));
            }
            validate_scenario(path, &lines, line_index, &mut diagnostics);
        }
    }

    diagnostics
}

fn is_marker(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("r[") else {
        return false;
    };
    let Some((id, _)) = rest.split_once(']') else {
        return false;
    };
    !id.is_empty() && !id.contains('[') && !id.chars().any(char::is_whitespace)
}

fn validate_requirement(path: &str, lines: &[&str], heading_index: usize, diagnostics: &mut Vec<SpecDiagnostic>) {
    let Some(marker_index) = next_nonempty_line_index(lines, heading_index) else {
        diagnostics.push(diagnostic(
            path,
            heading_index,
            "requirement heading must be followed by an r[...] marker line",
        ));
        return;
    };
    let marker_line = lines[marker_index].trim();

    if !is_marker(marker_line) {
        diagnostics.push(diagnostic(
            path,
            marker_index,
            "requirement must start with a complete non-empty r[...] marker",
        ));
    }
    if !NORMATIVE_WORDS.iter().any(|word| marker_line.contains(word)) {
        diagnostics.push(diagnostic(
            path,
            marker_index,
            "requirement marker line must include MUST, SHOULD, SHALL, or MAY",
        ));
    }
    let has_scenario = lines
        .iter()
        .skip(marker_index + LINE_NUMBER_BASE)
        .take_while(|line| !line.trim_start().starts_with("### Requirement:"))
        .any(|line| line.trim_start().starts_with("#### Scenario:"));
    if !has_scenario {
        diagnostics.push(diagnostic(path, heading_index, "requirement must include at least one scenario"));
    }
}

fn is_clause(line: &str, keyword: &str) -> bool {
    line.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix(' '))
        .is_some_and(|text| !text.trim().is_empty())
}

fn validate_scenario(path: &str, lines: &[&str], heading_index: usize, diagnostics: &mut Vec<SpecDiagnostic>) {
    let Some(marker_index) = next_nonempty_line_index(lines, heading_index) else {
        diagnostics.push(diagnostic(path, heading_index, "scenario heading must be followed by an r[...] marker line"));
        return;
    };
    let marker_line = lines[marker_index].trim();

    if !is_marker(marker_line) {
        diagnostics.push(diagnostic(path, marker_index, "scenario must start with a complete non-empty r[...] marker"));
    }

    let scenario_lines = lines
        .iter()
        .skip(marker_index + LINE_NUMBER_BASE)
        .take_while(|line| !line.trim_start().starts_with("###") && !line.trim_start().starts_with("####"));
    let scenario_text = scenario_lines.map(|line| line.trim()).collect::<Vec<_>>();

    for required_clause in ["- GIVEN", "- WHEN", "- THEN"] {
        if !scenario_text.iter().any(|line| is_clause(line, required_clause)) {
            diagnostics.push(diagnostic(
                path,
                heading_index,
                &format!("scenario must include a non-empty {required_clause} clause"),
            ));
        }
    }
}

fn next_nonempty_line_index(lines: &[&str], heading_index: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(heading_index + LINE_NUMBER_BASE)
        .take_while(|(_, line)| !line.trim_start().starts_with("###"))
        .find_map(|(index, line)| (!line.trim().is_empty()).then_some(index))
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

#[cfg(test)]
mod tests {
    use super::is_clause;
    use super::is_marker;
    use super::validate_spec_text;

    #[test]
    fn markers_require_a_closed_nonempty_identifier() {
        assert!(is_marker("r[lattice.example]"));
        assert!(is_marker("r[lattice.example] MUST remain"));
        assert!(!is_marker("r["));
        assert!(!is_marker("r[]"));
        assert!(!is_marker("r[lattice.example"));
        assert!(!is_marker("r[[nested]]"));
        assert!(!is_marker("r[two words]"));
        assert!(!is_marker("not-a-marker"));
    }

    #[test]
    fn clauses_require_separated_non_empty_text() {
        assert!(is_clause("- GIVEN a state", "- GIVEN"));
        assert!(is_clause("- WHEN  an action", "- WHEN"));
        assert!(is_clause("- THEN a result", "- THEN"));
        assert!(!is_clause("- GIVEN", "- GIVEN"));
        assert!(!is_clause("- GIVEN   ", "- GIVEN"));
        assert!(!is_clause("- WHENEVER a condition", "- WHEN"));
        assert!(!is_clause("GIVEN a state", "- GIVEN"));
    }

    #[test]
    fn headings_require_titles() {
        let text = "### Requirement:\nr[lattice.example] MUST hold\n\n#### Scenario:\nr[lattice.example.scenario]\n- GIVEN a state\n- WHEN an action\n- THEN a result\n";
        let diagnostics = validate_spec_text("spec.md", text);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "requirement heading must include a title")
        );
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message == "scenario heading must include a title"));
    }

    #[test]
    fn requirements_require_a_scenario() {
        let text = "### Requirement: Title\nr[lattice.example] MUST hold\n";
        let diagnostics = validate_spec_text("spec.md", text);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "requirement must include at least one scenario")
        );
        let complete = "### Requirement: Title\nr[lattice.example] MUST hold\n\n#### Scenario: Case\nr[lattice.example.scenario]\n- GIVEN a state\n- WHEN an action\n- THEN a result\n";
        assert!(validate_spec_text("spec.md", complete).is_empty());
    }
}
