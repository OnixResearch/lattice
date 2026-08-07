use std::path::Path;

use nickel_lang_core::error::NullReporter;
use nickel_lang_core::eval::cache::CacheImpl;
use nickel_lang_core::program::Program;
use nickel_lang_core::serialize::ExportFormat;
use nickel_lang_core::serialize::{self};

const RID: &str = "z21miuQLqLwe1HkaMu2mEZy5fdG9F";
const REVIEWED_OBJECT: &str = "bf16cf223ed3dfe75a28f02ac21ed7891a60bac1";
const ADAPTER: &str = "https://seed.radicle.garden/";
const FORBIDDEN_SOURCES: [&str; 4] = ["github.com", "path = \"../nickel", "branch =", "tag ="];
const PRODUCT_FIXTURES: [&str; 4] = [
    "examples/config/animus-agent.ncl",
    "examples/templates/complex-sort-lines.ncl",
    "examples/workflows/complex-text-normalization.ncl",
    "examples/orchestrations/llm-review-loop.ncl",
];
const PROPOSAL_FIXTURE: &str = r#"{ proposal_id = "review-1", revision = 1, accepted = false }"#;

fn legacy_file(path: &Path) -> Result<serde_json::Value, String> {
    let mut program = Program::<CacheImpl>::new_from_file(path, std::io::sink(), NullReporter {})
        .map_err(|error| format!("{error:?}"))?;
    let runtime = program.eval_full_for_export().map_err(|error| format!("{error:?}"))?;
    serialize::validate(ExportFormat::Json, &runtime).map_err(|error| format!("{error:?}"))?;
    let mut json = Vec::new();
    serde_json::to_writer(&mut json, &runtime).map_err(|error| error.to_string())?;
    serde_json::from_slice(&json).map_err(|error| error.to_string())
}

fn nickel_source_lines(input: &str) -> String {
    input
        .lines()
        .filter(|line| {
            line.contains("nickel-eval")
                || line.contains("nickelEval")
                || line.contains(RID)
                || line.contains(REVIEWED_OBJECT)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn source_is_admitted(cargo: &str, flake: &str) -> bool {
    let combined = nickel_source_lines(&format!("{cargo}\n{flake}"));
    combined.contains(RID)
        && combined.contains(REVIEWED_OBJECT)
        && combined.contains(ADAPTER)
        && !FORBIDDEN_SOURCES.iter().any(|forbidden| combined.contains(forbidden))
}

#[test]
fn legacy_and_shared_evaluators_agree_for_lattice_fixtures() {
    for fixture in PRODUCT_FIXTURES {
        let path = Path::new(fixture);
        let legacy = legacy_file(path).expect("legacy product fixture evaluates");
        let shared = nickel_eval::evaluate_to_json(path).expect("shared product fixture evaluates");
        assert_eq!(legacy, shared, "product fixture differs: {fixture}");
    }

    let directory = tempfile::tempdir().expect("temporary directory");
    let proposal = directory.path().join("proposal.ncl");
    std::fs::write(&proposal, PROPOSAL_FIXTURE).expect("proposal fixture writes");
    let legacy = legacy_file(&proposal).expect("legacy proposal evaluates");
    let shared = nickel_eval::evaluate_to_json(&proposal).expect("shared proposal evaluates");
    assert_eq!(legacy, shared);
}

#[test]
fn legacy_and_shared_evaluators_reject_invalid_fixtures() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let malformed = directory.path().join("malformed.ncl");
    let missing_import = directory.path().join("missing-import.ncl");
    std::fs::write(&malformed, "let x = in").expect("malformed fixture writes");
    std::fs::write(&missing_import, r#"import "missing.ncl""#).expect("import fixture writes");

    for fixture in [&malformed, &missing_import] {
        assert!(legacy_file(fixture).is_err());
        assert!(nickel_eval::evaluate_to_json(fixture).is_err());
    }
}

#[test]
fn cargo_and_nix_pin_one_radicle_source_without_fallback() {
    let cargo = include_str!("../Cargo.toml");
    let flake = include_str!("../flake.nix");

    assert!(source_is_admitted(cargo, flake));
}

#[test]
fn source_guard_rejects_floating_and_github_sources() {
    let floating = format!("nickel-eval = {{ git = \"{ADAPTER}{RID}.git\", branch = \"main\" }}");
    let github =
        format!("nickel-eval = {{ git = \"https://github.com/example/nickel-eval\", rev = \"{REVIEWED_OBJECT}\" }}");

    assert!(!source_is_admitted(&floating, ""));
    assert!(!source_is_admitted(&github, ""));
}
