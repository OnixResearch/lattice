#!/usr/bin/env -S nix develop . -c cargo -q -Zscript

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const DEFAULT_ROOT: &str = ".";
const SELF_TEST_ARGUMENT: &str = "--self-test";
const GENERIC_FAILURE_EXIT_CODE: u8 = 1;
const SOURCE_ROOT: &str = "src";
const CORE_EXPRESSIONS: &str = "crates/lattice-core/src/expressions.rs";
const TUI_SCOPE_PREFIX: &str = "src/tui/";
const RAT_NODEGRAPH_TOKEN: &str = "rat_nodegraph";
const CLOCK_TOKENS: &[(&str, &str)] = &[
    ("Utc::now", "ambient clock"),
    ("SystemTime::now", "ambient clock"),
    ("Instant::now", "ambient monotonic clock"),
];
const CORE_SCOPES: &[&str] = &[
    "src/engine/workflow.rs",
    "src/engine/domain/model.rs",
    "src/engine/domain/mutation.rs",
    "src/workflow_validation.rs",
    "src/expressions.rs",
    "crates/lattice-core/src/expressions.rs",
    "src/replay.rs",
    "src/run/history.rs",
    "src/store/mod.rs",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    path: String,
    line: usize,
    effect_class: &'static str,
    token: &'static str,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}:{}: rejected {} `{}`", self.path, self.line, self.effect_class, self.token)
    }
}

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let result = if arguments.first().map(String::as_str) == Some(SELF_TEST_ARGUMENT) {
        run_self_test()
    } else {
        let root = arguments.first().map_or_else(|| Path::new(DEFAULT_ROOT), Path::new);
        check_repository(root)
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(GENERIC_FAILURE_EXIT_CODE)
        }
    }
}

// r[impl lattice.workflow_validation.domain_boundary.enforcement]
// r[impl lattice.workflow_validation.domain_boundary.subwayrat]
fn check_repository(root: &Path) -> Result<(), String> {
    let mut diagnostics = Vec::new();
    for relative_path in CORE_SCOPES {
        let path = root.join(relative_path);
        let source = read_source(&path)?;
        diagnostics.extend(find_clock_reads(relative_path, &source));
    }

    let mut rust_files = Vec::new();
    collect_rust_files(&root.join(SOURCE_ROOT), &mut rust_files)?;
    let core_expression_path = root.join(CORE_EXPRESSIONS);
    if core_expression_path.is_file() {
        rust_files.push(core_expression_path);
    }
    rust_files.sort();

    for path in rust_files {
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        if relative.starts_with(TUI_SCOPE_PREFIX) {
            continue;
        }
        let source = read_source(&path)?;
        diagnostics.extend(find_token(&relative, &source, RAT_NODEGRAPH_TOKEN, "Subwayrat type leakage"));
    }

    if diagnostics.is_empty() {
        println!("workflow domain boundary guard passed");
        return Ok(());
    }

    diagnostics.sort_by(|left, right| left.path.cmp(&right.path).then(left.line.cmp(&right.line)));
    Err(diagnostics.into_iter().map(|diagnostic| diagnostic.to_string()).collect::<Vec<_>>().join("\n"))
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn collect_rust_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    let entries = fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", path.display()))?;
        collect_rust_files(&entry.path(), files)?;
    }
    Ok(())
}

fn find_clock_reads(path: &str, source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for (token, effect_class) in CLOCK_TOKENS {
        diagnostics.extend(find_token(path, source, token, effect_class));
    }
    diagnostics
}

fn find_token(path: &str, source: &str, token: &'static str, effect_class: &'static str) -> Vec<Diagnostic> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(token))
        .map(|(line_index, _)| Diagnostic {
            path: path.to_string(),
            line: line_index + 1,
            effect_class,
            token,
        })
        .collect()
}

fn run_self_test() -> Result<(), String> {
    let accepted = "pub fn touch(timestamp: WorkflowTimestamp) { let _ = timestamp; }";
    if !find_clock_reads("accepted.rs", accepted).is_empty() {
        return Err("positive clock fixture was rejected".to_string());
    }
    if !find_token("accepted.rs", accepted, RAT_NODEGRAPH_TOKEN, "Subwayrat type leakage").is_empty() {
        return Err("positive dependency fixture was rejected".to_string());
    }

    let clock_diagnostics = find_clock_reads("negative-clock.rs", "let now = chrono::Utc::now();");
    // r[verify lattice.workflow_validation.domain_boundary.enforcement.clock]
    if clock_diagnostics.len() != 1 || clock_diagnostics[0].effect_class != "ambient clock" {
        return Err("negative clock fixture did not report one bounded diagnostic".to_string());
    }

    let dependency_diagnostics = find_token(
        "negative-dependency.rs",
        "pub fn graph() -> rat_nodegraph::model::Graph { todo!() }",
        RAT_NODEGRAPH_TOKEN,
        "Subwayrat type leakage",
    );
    // r[verify lattice.workflow_validation.domain_boundary.subwayrat.leak]
    if dependency_diagnostics.len() != 1 {
        return Err("negative dependency fixture did not report one bounded diagnostic".to_string());
    }

    println!("workflow domain boundary guard self-test passed");
    Ok(())
}
