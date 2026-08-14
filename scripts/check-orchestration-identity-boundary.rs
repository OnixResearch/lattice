#!/usr/bin/env -S nix shell nixpkgs#rustc nixpkgs#cargo --command rust-script

//! Static guard for typed orchestration identity boundaries.
//!
//! r[impl lattice.agent_orchestration.nominal_identities.octet]
//! r[impl lattice.agent_orchestration.nominal_identities.octet.guard]
//! r[verify lattice.agent_orchestration.nominal_identities.inventory]
//! r[verify lattice.agent_orchestration.nominal_identities.inventory.display]
//! r[verify lattice.agent_orchestration.nominal_identities.octet]
//! r[verify lattice.agent_orchestration.nominal_identities.octet.guard]
//! r[verify lattice.agent_orchestration.nominal_identities.docs]
//! r[verify lattice.agent_orchestration.nominal_identities.final_checks]

use std::path::Path;

const TYPE_ALIASES: &[&str] = &[
    "DefinitionId",
    "TriggerId",
    "TriggerIdempotencyKey",
    "ReservationId",
    "DefinitionRevision",
    "WorkflowRevision",
    "InstanceId",
    "DispatchId",
    "ProposalId",
    "ApprovalId",
    "FactId",
    "AuthorityPolicyIdentity",
    "EventId",
    "PreviousEventId",
    "LedgerHeadId",
    "StateDigest",
    "TriggerScopedKey",
];
const CORE_FIELDS: &[(&str, &str)] = &[
    ("pub struct WorkflowRevisionRef", "pub content_hash: super::identity::WorkflowRevision"),
    ("pub struct OrchestrationDefinition", "pub id: super::identity::DefinitionId"),
    ("pub struct OrchestrationTriggerBinding", "pub trigger_id: super::identity::TriggerId"),
    ("pub struct DispatchIntent", "pub id: super::identity::DispatchId"),
    ("pub struct RevisionProposal", "pub id: super::identity::ProposalId"),
    ("pub struct ApprovalDecision", "pub id: super::identity::ApprovalId"),
    ("pub struct OrchestrationInstance", "pub id: super::identity::InstanceId"),
    ("pub struct OrchestrationFact", "pub id: super::identity::FactId"),
    ("pub struct OrchestrationEvent", "pub event_hash: super::identity::EventId"),
    ("pub struct OrchestrationEvent", "pub prior_hash: Option<super::identity::PreviousEventId>"),
    ("pub struct OrchestrationInstance", "pub event_head: Option<super::identity::LedgerHeadId>"),
];
const WIRE_MARKERS: &[&str] = &[
    "struct DefinitionRecord",
    "struct ProposalRecord",
    "id: String",
    "content_hash: String",
];
const FORMAT_MARKERS: &[&str] = &["wire.try_into()", "wire::DefinitionRecord", "wire::ProposalRecord"];
const STORE_MARKERS: &[&str] = &[
    "InstanceId::try_from_hex",
    "TriggerScopedKey::try_from_hex",
    ".as_str()",
];
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "provider truth",
    "remote cancellation",
    "external exactly-once effects",
    "production readiness",
];

fn main() {
    let argument = std::env::args().nth(1);
    if argument.as_deref() == Some("--self-test") {
        self_test();
        return;
    }
    let root = argument.as_deref().unwrap_or(".");
    check_repository(Path::new(root)).unwrap_or_else(|error| panic!("{error}"));
    println!("orchestration identity boundary check: PASS");
}

fn check_repository(root: &Path) -> Result<(), String> {
    let identity = read(root, "src/orchestration/identity.rs")?;
    let core = read(root, "src/orchestration/core.rs")?;
    let format = read(root, "src/orchestration/format.rs")?;
    let wire = read(root, "src/orchestration/format/wire.rs")?;
    let store = read(root, "src/store/mod.rs")?;
    let docs = read(root, "docs/orchestration-identities.md")?;
    check_identity_source(&identity)?;
    check_core_source(&core)?;
    require_markers("orchestration wire record", &wire, WIRE_MARKERS)?;
    require_markers("orchestration wire adapter", &format, FORMAT_MARKERS)?;
    require_markers("redb identity adapter", &store, STORE_MARKERS)?;
    require_markers("orchestration identity non-claims", &docs, REQUIRED_NON_CLAIMS)?;
    Ok(())
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    std::fs::read_to_string(root.join(relative)).map_err(|error| format!("read {relative}: {error}"))
}

fn check_identity_source(source: &str) -> Result<(), String> {
    for identity in TYPE_ALIASES {
        let multiline = format!("    {identity},");
        let inline = format!("digest_identity!({identity},");
        if !source.contains(&multiline) && !source.contains(&inline) {
            return Err(format!("identity module is missing checked wrapper {identity}"));
        }
        let raw = format!("pub type {identity} = String");
        if source.contains(&raw) {
            return Err(format!("identity alias {identity} regressed to String"));
        }
    }
    require_markers("checked orchestration identity constructor", source, &[
        "pub fn try_from_text",
        "pub fn try_from_hex",
        "pub fn as_str",
    ])
}

fn check_core_source(source: &str) -> Result<(), String> {
    for (owner, field) in CORE_FIELDS {
        let block = struct_block(source, owner)?;
        if !block.contains(field) {
            return Err(format!("{owner} is missing nominal field `{field}`"));
        }
    }
    Ok(())
}

fn struct_block<'a>(source: &'a str, owner: &str) -> Result<&'a str, String> {
    let tail = source.split(owner).nth(1).ok_or_else(|| format!("missing {owner}"))?;
    tail.split("}\n").next().ok_or_else(|| format!("unterminated {owner}"))
}

fn require_markers(label: &str, source: &str, markers: &[&str]) -> Result<(), String> {
    for marker in markers {
        if !source.contains(marker) {
            return Err(format!("{label} is missing `{marker}`"));
        }
    }
    Ok(())
}

fn self_test() {
    let valid_identity = TYPE_ALIASES
        .iter()
        .map(|identity| format!("digest_identity!({identity}, \"fixture\");"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\npub fn try_from_text() {}\npub fn try_from_hex() {}\npub fn as_str() {}";
    assert!(check_identity_source(&valid_identity).is_ok());

    let invalid_identity =
        valid_identity.replace("digest_identity!(InstanceId, \"fixture\");", "pub type InstanceId = String;");
    let error = check_identity_source(&invalid_identity).expect_err("raw instance identity must fail");
    assert!(error.contains("InstanceId"));

    let valid_core = r#"
pub struct WorkflowRevisionRef { pub content_hash: super::identity::WorkflowRevision, }
pub struct OrchestrationDefinition { pub id: super::identity::DefinitionId, }
pub struct OrchestrationTriggerBinding { pub trigger_id: super::identity::TriggerId, }
pub struct DispatchIntent { pub id: super::identity::DispatchId, }
pub struct RevisionProposal { pub id: super::identity::ProposalId, }
pub struct ApprovalDecision { pub id: super::identity::ApprovalId, }
pub struct OrchestrationInstance {
    pub id: super::identity::InstanceId,
    pub event_head: Option<super::identity::LedgerHeadId>,
}
pub struct OrchestrationFact { pub id: super::identity::FactId, }
pub struct OrchestrationEvent {
    pub event_hash: super::identity::EventId,
    pub prior_hash: Option<super::identity::PreviousEventId>,
}
"#;
    assert!(check_core_source(valid_core).is_ok());
    let invalid_core = valid_core.replace("pub id: super::identity::DispatchId", "pub id: String");
    let error = check_core_source(&invalid_core).expect_err("raw dispatch identity must fail");
    assert!(error.contains("DispatchIntent"));
    println!("orchestration identity boundary self-test: PASS");
}
