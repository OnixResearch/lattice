use std::path::Path;

use nickel_lang_core::eval::cache::CacheImpl;
use nickel_lang_core::program::ProgramBuilder;
use nickel_lang_core::serialize::ExportFormat;
use nickel_lang_core::serialize::{self};

const PRODUCT_FIXTURES: [&str; 4] = [
    "examples/config/animus-agent.ncl",
    "examples/templates/complex-sort-lines.ncl",
    "examples/workflows/complex-text-normalization.ncl",
    "examples/orchestrations/llm-review-loop.ncl",
];
const PROPOSAL_FIXTURE: &str = r#"{ proposal_id = "review-1", revision = 1, accepted = false }"#;

fn direct_cohort_file(path: &Path) -> Result<serde_json::Value, String> {
    let mut program =
        ProgramBuilder::new().add_path(path).build::<CacheImpl>().map_err(|error| format!("{error:?}"))?;
    let runtime = program.eval_full_for_export().map_err(|error| format!("{error:?}"))?;
    serialize::validate(ExportFormat::Json, &runtime).map_err(|error| format!("{error:?}"))?;
    let mut json = Vec::new();
    serde_json::to_writer(&mut json, &runtime).map_err(|error| error.to_string())?;
    serde_json::from_slice(&json).map_err(|error| error.to_string())
}

// r[verify lattice.nickel_eval_adoption.compatibility]
// r[verify lattice.nickel_eval_adoption.compatibility.scenario.agree]
// r[verify lattice.nickel_eval_adoption.cutover.scenario.accepted]
// r[verify lattice.nickel_toolchain.fixture_cohort.scenario.aligned]
#[test]
fn direct_and_shared_cohort_evaluators_agree_for_lattice_fixtures() {
    for fixture in PRODUCT_FIXTURES {
        let path = Path::new(fixture);
        let direct = direct_cohort_file(path).expect("direct cohort product fixture evaluates");
        let shared = nickel_eval::evaluate_to_json(path).expect("shared product fixture evaluates");
        assert_eq!(direct, shared, "product fixture differs: {fixture}");
    }

    let directory = tempfile::tempdir().expect("temporary directory");
    let proposal = directory.path().join("proposal.ncl");
    std::fs::write(&proposal, PROPOSAL_FIXTURE).expect("proposal fixture writes");
    let direct = direct_cohort_file(&proposal).expect("direct cohort proposal evaluates");
    let shared = nickel_eval::evaluate_to_json(&proposal).expect("shared proposal evaluates");
    assert_eq!(direct, shared);
}

// r[verify lattice.nickel_eval_adoption.compatibility.scenario.differ]
// r[verify lattice.nickel_toolchain.compatibility.scenario.matrix]
#[test]
fn direct_and_shared_cohort_evaluators_reject_invalid_fixtures() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let malformed = directory.path().join("malformed.ncl");
    let missing_import = directory.path().join("missing-import.ncl");
    std::fs::write(&malformed, "let x = in").expect("malformed fixture writes");
    std::fs::write(&missing_import, r#"import "missing.ncl""#).expect("import fixture writes");

    for fixture in [&malformed, &missing_import] {
        assert!(direct_cohort_file(fixture).is_err());
        assert!(nickel_eval::evaluate_to_json(fixture).is_err());
    }
}
