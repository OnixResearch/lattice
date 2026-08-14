//! Print deterministic identities used by the built-in agent-evaluation catalog.

use std::fs;

use lattice::agent_evaluation::format::load_catalog;
use lattice::agent_evaluation::identity::hash_bytes;
use lattice::agent_evaluation::identity::hash_value;

const ASSET_DOMAIN: &str = "lattice.agent-evaluation.case-asset.v1";
const LABEL_DOMAIN: &str = "lattice.agent-evaluation.catalog-label.v1";
const ASSETS: &[&str] = &[
    "builtins/agent-evaluations/cases/unit-local.txt",
    "builtins/agent-evaluations/cases/skill-local.txt",
    "builtins/agent-evaluations/cases/hard-local.txt",
    "builtins/agent-evaluations/cases/live-sandbox.txt",
];
const LABELS: &[&str] = &[
    "repository",
    "workload-unit",
    "workload-skill",
    "workload-hard",
    "workload-live",
    "fixture-unit",
    "fixture-skill",
    "fixture-hard",
    "fixture-live",
    "scaffold-unit",
    "scaffold-skill",
    "scaffold-hard",
    "scaffold-live",
    "initial-unit",
    "initial-skill",
    "initial-hard",
    "initial-live",
    "agent-local",
    "agent-hard",
    "agent-live",
    "environment-local",
    "environment-isolated",
    "environment-live",
    "gate-format",
    "gate-test",
    "gate-policy",
    "gate-deploy",
    "gate-input-unit",
    "gate-input-skill",
    "gate-input-hard",
    "gate-input-live",
    "rubric-skill",
    "judge-reviewer",
    "reference-unit",
    "reference-skill",
    "reference-hard",
    "reference-live",
    "reference-source-unit",
    "reference-source-skill",
    "reference-source-hard",
    "reference-source-live",
    "artifact-local",
    "artifact-hard",
    "artifact-live",
    "policy",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in ASSETS {
        let bytes = fs::read(path)?;
        println!("asset {path} {}", hash_bytes(ASSET_DOMAIN, &bytes));
    }
    for label in LABELS {
        println!("label {label} {}", hash_bytes(LABEL_DOMAIN, label.as_bytes()));
    }
    let catalog = load_catalog(std::path::Path::new("builtins/agent-evaluations"))?;
    for case in catalog.cases {
        let subject_identity = hash_value("lattice.agent-evaluation.subject.v1", &case.definition.subject)?;
        println!("subject {} {subject_identity}", case.definition.id);
    }
    Ok(())
}
