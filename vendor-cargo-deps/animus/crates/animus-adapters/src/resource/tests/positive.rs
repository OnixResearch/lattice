use super::super::*;
use super::support::*;

#[test]
fn descriptors_cover_all_supported_kinds_without_full_content() {
    let skill = valid_descriptor();
    let template = descriptor_with(DescriptorFixture {
        kind: Kind::PromptTemplate,
        name: SECOND_NAME,
        description: SECOND_DESCRIPTION,
    });
    let context = descriptor_with(DescriptorFixture {
        kind: Kind::ContextGuidance,
        name: "context-notes",
        description: "Context note summary",
    });
    let host = descriptor_with(DescriptorFixture {
        kind: Kind::HostResource,
        name: "host-policy",
        description: "Host policy summary",
    });
    let catalog = advertise(&[host, context, skill, template]);

    assert!(catalog.diagnostics().is_empty());
    assert_eq!(catalog.summaries().len(), EXPECTED_FOUR_ITEMS);
    assert_eq!(catalog.summaries()[0].name(), "context-notes");
    assert_eq!(catalog.summaries()[0].kind(), Kind::ContextGuidance);
    assert_eq!(catalog.summaries()[1].name(), "host-policy");
    assert_eq!(catalog.summaries()[1].kind(), Kind::HostResource);
    assert_eq!(catalog.summaries()[2].name(), SECOND_NAME);
    assert_eq!(catalog.summaries()[2].kind(), Kind::PromptTemplate);
    assert_eq!(catalog.summaries()[3].name(), NAME);
    assert_eq!(catalog.summaries()[3].kind(), Kind::Skill);
    assert_eq!(catalog.summaries()[3].description(), DESCRIPTION);
    assert_eq!(catalog.summaries()[3].scope(), Scope::Global);
    assert_eq!(catalog.summaries()[3].provenance(), PROVENANCE);
    assert_ne!(catalog.summaries()[3].description(), SELECTED_BODY);
}

#[test]
fn explicit_load_includes_only_selected_trusted_content() {
    let first = valid_descriptor();
    let second = descriptor_with(DescriptorFixture {
        kind: Kind::PromptTemplate,
        name: SECOND_NAME,
        description: SECOND_DESCRIPTION,
    });
    let descriptors = [first, second];
    let bodies = [
        ContentDraft {
            name: NAME,
            content_identity: VALID_HASH,
            body: UNRELATED_BODY,
        },
        ContentDraft {
            name: SECOND_NAME,
            content_identity: VALID_HASH,
            body: SELECTED_BODY,
        },
    ];
    let context = build_context(Request {
        descriptors: &descriptors,
        loaded_contents: &bodies,
        selected_name: Some(SECOND_NAME),
    });

    assert_eq!(context.catalog().summaries().len(), EXPECTED_TWO_ITEMS);
    let selected = context
        .selected_content()
        .expect("selected trusted content should load");
    assert_eq!(selected.name(), SECOND_NAME);
    assert_eq!(selected.kind(), Kind::PromptTemplate);
    assert_eq!(selected.content_identity().as_str(), VALID_HASH);
    assert_eq!(selected.body(), SELECTED_BODY);
    assert_ne!(selected.body(), UNRELATED_BODY);
}

#[test]
fn host_owned_boundary_lists_prohibited_effects() {
    let descriptor = Descriptor::validate(&Draft {
        kind: Some(HOST_RESOURCE_KIND_LABEL),
        scope: Some(HOST_SCOPE_LABEL),
        provenance: Some("host:embedded"),
        ..valid_draft()
    })
    .expect("host resource descriptor should validate");
    let context = build_context(Request {
        descriptors: &[descriptor],
        loaded_contents: &[],
        selected_name: None,
    });

    assert_eq!(context.catalog().summaries().len(), EXPECTED_SINGLE_ITEM);
    assert!(context.selected_content().is_none());
    assert!(PROHIBITED_DISCOVERY_EFFECTS.contains(&"filesystem traversal"));
    assert!(PROHIBITED_DISCOVERY_EFFECTS.contains(&"package installation"));
    assert!(PROHIBITED_DISCOVERY_EFFECTS.contains(&"extension code loading"));
}
