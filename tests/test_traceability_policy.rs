use serde_json::Value;
use serde_json::json;

#[path = "support/cairn.rs"]
mod cairn;
use cairn::Fixture;
use cairn::TestResult;

const GENERATED_POLICY: &str = include_str!("../cairn-policy/generated/cairn-policy.json");

fn marker(verb: &str, id: &str) -> String {
    format!("r[{verb} {id}]\n")
}

fn requirement(id: &str) -> String {
    format!("r[{id}] The fixture MUST retain exact linkage.\n")
}

// r[verify lattice.repository_validation.traceability_sources.scenario.discovery]
// r[verify lattice.architecture.core_crate.traceability]
#[test]
fn default_traceability_profile_discovers_each_owned_evidence_root() -> TestResult {
    let fixture = Fixture::new()?;
    let policy: Value = serde_json::from_str(GENERATED_POLICY)?;
    let profile = policy["traceability_policy"]["profiles"]
        .as_array()
        .ok_or("profiles")?
        .iter()
        .find(|profile| profile["id"] == "cairn-default")
        .ok_or("default profile")?;
    let sources = profile["evidence_sources"].as_array().ok_or("evidence sources")?;
    let mut spec = String::new();
    for (index, source) in sources.iter().enumerate() {
        let id = format!("fixture.discovery.source_{index}");
        spec.push_str(&requirement(&id));
        let root = source["root"].as_str().ok_or("source root")?;
        let path = if std::path::Path::new(root).extension().is_some() {
            root.to_string()
        } else {
            let extension = source["extensions"][0].as_str().ok_or("source extension")?;
            format!("{root}/probe.{extension}")
        };
        fixture.write(&path, marker("impl", &id))?;
    }
    fixture.write(".cairn/specs/probe/spec.md", spec)?;
    fixture.write("tools/stray.rs", marker("impl", "fixture.unowned.tool"))?;

    let (success, receipt) = fixture.run(&["traceability", "coverage", "--json"])?;
    assert!(success, "{receipt}");
    assert_eq!(receipt["requirements"], sources.len());
    assert_eq!(receipt["referenced"], sources.len());
    assert_eq!(receipt["missing"], json!([]));
    assert_eq!(receipt["dangling"], json!([]));
    assert_eq!(receipt["valid"], true);
    Ok(())
}

// r[verify lattice.repository_validation.traceability_sources.scenario.fail-closed]
// r[verify lattice.repository_validation.traceability_sources.scenario.marker-reconciliation]
#[test]
fn missing_and_dangling_linkage_fails_closed_without_hiding_unrelated_debt() -> TestResult {
    let fixture = Fixture::new()?;
    let covered = "fixture.linkage.covered";
    let repaired = "fixture.linkage.repaired";
    let unrelated = "fixture.linkage.unrelated";
    fixture.write(
        ".cairn/specs/probe/spec.md",
        [requirement(covered), requirement(repaired), requirement(unrelated)].concat(),
    )?;
    fixture.write("src/lib.rs", [marker("impl", covered), marker("related", "fixture.linkage.stale")].concat())?;
    let (success, receipt) = fixture.run(&["traceability", "coverage", "--json"])?;
    assert!(!success, "{receipt}");
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["missing"], json!([repaired, unrelated]));
    assert_eq!(receipt["dangling"], json!(["fixture.linkage.stale"]));

    fixture.write("src/lib.rs", [marker("impl", covered), marker("impl", repaired)].concat())?;
    let (success, receipt) = fixture.run(&["traceability", "coverage", "--json"])?;
    assert!(!success, "{receipt}");
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["missing"], json!([unrelated]));
    assert_eq!(receipt["dangling"], json!([]));
    Ok(())
}

// r[verify lattice.repository_validation.traceability_sources.scenario.unsafe-root]
#[test]
fn nickel_export_rejects_unsafe_roots_before_evidence_discovery() -> TestResult {
    let fixture = Fixture::new()?;
    let contracts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("cairn-policy/contracts.ncl");
    let local = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("cairn-policy/traceability-contracts.ncl");
    fixture.write(
        "policy.ncl",
        format!(
            "let Contracts = import {} in let Local = import {} in (import \"policy.json\") | (Contracts.Policy & {{traceability_policy | Local.Policy}})\n",
            serde_json::to_string(&contracts)?,
            serde_json::to_string(&local)?
        ),
    )?;
    let valid: Value = serde_json::from_str(GENERATED_POLICY)?;
    for unsafe_root in [
        None,
        Some("../src"),
        Some("/src"),
        Some("src/../outside"),
        Some("src/./inside"),
        Some("src//inside"),
    ] {
        let mut policy = valid.clone();
        if let Some(root) = unsafe_root {
            policy["traceability_policy"]["profiles"][0]["evidence_sources"][0]["root"] = json!(root);
        }
        fixture.write("policy.json", serde_json::to_vec(&policy)?)?;
        let output = std::process::Command::new("nickel")
            .current_dir(fixture.root.path())
            .args(["export", "--format", "json", "policy.ncl"])
            .output()?;
        assert_eq!(
            output.status.success(),
            unsafe_root.is_none(),
            "root={unsafe_root:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
