use super::*;

const GLOBAL_PROVENANCE: &str = "global-hermes-posture";
const PROJECT_PROVENANCE: &str = "project-policy";
const VALIDATION_RULE: &str = "run-smallest-relevant-check";
const STOP_RULE: &str = "stop-on-destructive-uncertainty";
const SUCCESS_RULE: &str = "observable-evidence-recorded";
const BAD_GUIDANCE_CASE_COUNT: usize = 4;

#[test]
fn trusted_global_descriptor_builds_attributed_segment() {
    let profile = checked_descriptor(global_draft()).expect("global profile should validate");
    let composition = compose_context(alloc::slice::from_ref(&profile), ProjectTrustMode::DenyProjectPolicy);

    assert_eq!(profile.kind(), Kind::GlobalPosture);
    assert_eq!(profile.scope(), SourceScope::Global);
    assert_eq!(profile.trust(), TrustStatus::Trusted);
    assert_eq!(composition.segments().len(), 1);
    assert_eq!(composition.segments()[0].provenance(), GLOBAL_PROVENANCE);
    assert!(composition.segments()[0].text().starts_with("Goal:"));
    assert!(composition.segments()[0].text().contains("Validation:"));
    assert!(composition.diagnostics().is_empty());
}

#[test]
fn global_safety_defeats_project_override() {
    let global = checked_descriptor(Draft {
        constraints: &[REQUIRE_APPROVAL_CONSTRAINT],
        ..global_draft()
    })
    .expect("global profile should validate");
    let project = checked_descriptor(Draft {
        kind: Some("project-overlay"),
        scope: Some("project"),
        constraints: &[ALLOW_DESTRUCTIVE_CONSTRAINT],
        provenance: Some(PROJECT_PROVENANCE),
        ..global_draft()
    })
    .expect("project profile should validate");
    let composition = compose_context(&[global, project], ProjectTrustMode::AdmitTrustedProjectPolicy);

    assert_eq!(composition.segments().len(), 1);
    assert_eq!(composition.segments()[0].provenance(), GLOBAL_PROVENANCE);
    assert_eq!(composition.diagnostics().len(), 1);
    assert_eq!(composition.diagnostics()[0].kind(), DiagnosticKind::GlobalSafetyPrecedence);
    assert_eq!(composition.diagnostics()[0].provenance(), PROJECT_PROVENANCE);
    assert!(!alloc::format!("{:?}", composition.diagnostics()).contains(ALLOW_DESTRUCTIVE_CONSTRAINT));
}

#[test]
fn rejects_bad_guidance_text() {
    let cases = [
        (
            Draft {
                prompt_snippets: &["long step-by-step script for every action"],
                ..global_draft()
            },
            PromptHygieneIssue::ProcessHeavy,
        ),
        (
            Draft {
                stop_rules: &["continue forever without stopping"],
                ..global_draft()
            },
            PromptHygieneIssue::UnboundedInstruction,
        ),
        (
            Draft {
                constraints: &["use gpt-4 only for all tasks"],
                ..global_draft()
            },
            PromptHygieneIssue::StaleModelGuidance,
        ),
        (
            Draft {
                validation_rules: &[],
                ..global_draft()
            },
            PromptHygieneIssue::MissingValidationRule,
        ),
    ];
    assert!(!cases.is_empty());
    assert_eq!(cases.len(), BAD_GUIDANCE_CASE_COUNT);

    for (draft, expected_issue) in cases {
        let error = checked_descriptor(draft).expect_err("bad policy should fail hygiene");
        assert_eq!(error, Error::PromptHygiene { issue: expected_issue });
    }
}

#[test]
fn omits_untrusted_project_guidance() {
    let global = checked_descriptor(global_draft()).expect("global profile should validate");
    let project = checked_descriptor(Draft {
        kind: Some("project-overlay"),
        scope: Some("project"),
        trust: Some("untrusted"),
        provenance: Some(PROJECT_PROVENANCE),
        ..global_draft()
    })
    .expect("untrusted project profile should still parse as data");
    let composition = compose_context(&[project, global], ProjectTrustMode::DenyProjectPolicy);

    assert_eq!(composition.segments().len(), 1);
    assert_eq!(composition.segments()[0].provenance(), GLOBAL_PROVENANCE);
    assert_eq!(composition.diagnostics().len(), 1);
    assert_eq!(composition.diagnostics()[0].kind(), DiagnosticKind::ProjectPolicyOmitted);
}

#[test]
fn read_only_guidance_keeps_runtime_guard() {
    let profile = checked_descriptor(Draft {
        constraints: &[READ_ONLY_CONSTRAINT],
        ..global_draft()
    })
    .expect("read-only guidance should validate");

    assert_eq!(profile.enforcement_boundary(), EnforcementBoundary::GuidanceOnly);
    assert_eq!(mutation_requires_runtime_guard(&profile), RuntimeGuardRequirement::Required);
}

fn checked_descriptor(draft: Draft<'_>) -> Result<Descriptor, Error> {
    Descriptor::validate(&draft)
}

fn global_draft() -> Draft<'static> {
    Draft {
        kind: Some("global-posture"),
        scope: Some("global"),
        trust: Some("trusted"),
        priority: MIN_POLICY_PRIORITY,
        goal: Some("Complete the requested outcome with concise evidence"),
        success_criteria: &[SUCCESS_RULE],
        constraints: &["preserve-user-changes"],
        validation_rules: &[VALIDATION_RULE],
        stop_rules: &[STOP_RULE],
        provenance: Some(GLOBAL_PROVENANCE),
        prompt_snippets: &["Run the smallest relevant check"],
    }
}
