use super::super::*;
use super::support::*;

#[test]
fn untrusted_project_descriptor_is_omitted() {
    let project = Descriptor::validate(&Draft {
        name: Some(PROJECT_NAME),
        scope: Some(PROJECT_SCOPE_LABEL),
        provenance: Some(PROJECT_PROVENANCE),
        trust_status: Some(UNTRUSTED_LABEL),
        ..valid_draft()
    })
    .expect("untrusted project descriptor still parses as data");
    let context = build_context(Request {
        descriptors: &[project],
        loaded_contents: &[ContentDraft {
            name: PROJECT_NAME,
            content_identity: VALID_HASH,
            body: SELECTED_BODY,
        }],
        selected_name: Some(PROJECT_NAME),
    });

    assert!(context.catalog().summaries().is_empty());
    assert_eq!(context.catalog().diagnostics().len(), EXPECTED_SINGLE_ITEM);
    assert_eq!(
        context.catalog().diagnostics()[0].issue(),
        Issue::UntrustedDescriptor
    );
    assert_eq!(context.catalog().diagnostics()[0].scope(), Scope::Project);
    assert!(context.selected_content().is_none());
}

#[test]
fn duplicate_names_disable_lookup_deterministically() {
    let first = valid_descriptor();
    let duplicate = Descriptor::validate(&Draft {
        kind: Some(PROMPT_TEMPLATE_KIND_LABEL),
        name: Some(NAME),
        content_identity: Some(SECOND_HASH),
        ..valid_draft()
    })
    .expect("duplicate descriptor validates before cataloging");
    let context = build_context(Request {
        descriptors: &[duplicate, first],
        loaded_contents: &[ContentDraft {
            name: NAME,
            content_identity: VALID_HASH,
            body: SELECTED_BODY,
        }],
        selected_name: Some(NAME),
    });

    assert!(context.catalog().summaries().is_empty());
    assert_eq!(context.catalog().diagnostics().len(), EXPECTED_TWO_ITEMS);
    assert!(context.catalog().diagnostics().iter().all(|diagnostic| {
        diagnostic.issue() == Issue::DuplicateName && diagnostic.name() == NAME
    }));
    assert_eq!(context.catalog().diagnostics()[0].kind(), Kind::Skill);
    assert_eq!(
        context.catalog().diagnostics()[1].kind(),
        Kind::PromptTemplate
    );
    assert!(context.selected_content().is_none());
}

#[test]
fn mismatched_or_empty_body_is_omitted() {
    let descriptor = valid_descriptor();
    let descriptors = [descriptor];
    let wrong_identity_context = build_context(Request {
        descriptors: &descriptors,
        loaded_contents: &[ContentDraft {
            name: NAME,
            content_identity: SECOND_HASH,
            body: SELECTED_BODY,
        }],
        selected_name: Some(NAME),
    });
    let empty_body_context = build_context(Request {
        descriptors: &descriptors,
        loaded_contents: &[ContentDraft {
            name: NAME,
            content_identity: VALID_HASH,
            body: EMPTY_BODY,
        }],
        selected_name: Some(NAME),
    });

    assert!(wrong_identity_context.selected_content().is_none());
    assert!(empty_body_context.selected_content().is_none());
}
