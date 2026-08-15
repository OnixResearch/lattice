//! Tests for accepted Cairn spec shape: requirement markers, normative language, and scenario
//! clauses.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use pretty_assertions::assert_eq;

const CAIRN_SPECS_DIR: &str = ".cairn/specs";
const SPEC_FILE_NAME: &str = "spec.md";
const LINE_NUMBER_BASE: usize = 1;
const NORMATIVE_WORDS: &[&str] = &["MUST", "SHOULD", "SHALL", "MAY"];
const VALID_SPEC: &str = "# Example\n\n## Requirements\n\n### Requirement: Example\n\nr[example.requirement] The system MUST do the thing.\n\n#### Scenario: Happy path\n\nr[example.requirement.happy]\n- GIVEN valid input\n- WHEN the action runs\n- THEN the result is visible\n";
const INVALID_SPEC: &str = "# Example\n\n## Requirements\n\n### Requirement: Example\nr[example.requirement] The system does the thing.\n\n#### Scenario: Broken path\nr[example.requirement.broken]\n- GIVEN valid input\n- THEN the result is visible\n";

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpecDiagnostic {
    path: String,
    line_number: usize,
    message: String,
}

#[test]
fn accepted_cairn_specs_have_required_shape() {
    let spec_files = collect_spec_files(Path::new(CAIRN_SPECS_DIR)).expect("read cairn specs");
    let mut diagnostics = Vec::new();

    for path in spec_files {
        let text = fs::read_to_string(&path).expect("read spec file");
        diagnostics.extend(validate_spec_text(&path.display().to_string(), &text));
    }

    assert_eq!(diagnostics, Vec::<SpecDiagnostic>::new());
}

#[test]
fn validator_accepts_valid_requirement_and_scenario() {
    assert_eq!(validate_spec_text("valid.md", VALID_SPEC), Vec::<SpecDiagnostic>::new());
}

#[test]
fn validator_rejects_missing_normative_word_and_when_clause() {
    let diagnostics = validate_spec_text("invalid.md", INVALID_SPEC);

    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("MUST")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("- WHEN")));
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
    let Some(marker_index) = next_nonempty_line_index(lines, heading_index) else {
        diagnostics.push(diagnostic(
            path,
            heading_index,
            "requirement heading must be followed by an r[...] marker line",
        ));
        return;
    };
    let marker_line = lines[marker_index].trim();

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
    let Some(marker_index) = next_nonempty_line_index(lines, heading_index) else {
        diagnostics.push(diagnostic(path, heading_index, "scenario heading must be followed by an r[...] marker line"));
        return;
    };
    let marker_line = lines[marker_index].trim();

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
