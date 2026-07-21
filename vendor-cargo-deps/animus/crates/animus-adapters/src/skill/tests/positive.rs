use super::*;

#[test]
fn valid_manifest_becomes_summary_without_body_content() {
    let manifest = valid_manifest();
    assert_eq!(manifest.name(), NAME);
    assert_eq!(manifest.description(), DESCRIPTION);
    assert_eq!(manifest.version(), VERSION);
    assert_eq!(manifest.category(), CATEGORY);
    assert_eq!(manifest.tags(), &[Text::from(SECOND_TAG), Text::from(TAG)]);
    assert!(manifest.related_skills().is_empty());
    assert_eq!(manifest.provenance(), PROVENANCE);
    assert_eq!(manifest.trust_status(), TrustStatus::Trusted);
    assert_eq!(manifest.trust_status().as_str(), TRUSTED_LABEL);
    assert_eq!(manifest.content_identity().as_str(), VALID_HASH);
    assert_eq!(CONTENT_HASH_HEX_CHARS, VALID_HASH.len());

    let catalog = advertise(&[manifest]);
    assert!(catalog.diagnostics().is_empty());
    assert_eq!(catalog.summaries().len(), EXPECTED_SINGLE_ITEM);
    let summary = &catalog.summaries()[0];
    assert_eq!(summary.name(), NAME);
    assert_eq!(summary.description(), DESCRIPTION);
    assert_eq!(summary.category(), CATEGORY);
    assert_eq!(summary.tags(), &[Text::from(SECOND_TAG), Text::from(TAG)]);
    assert_eq!(summary.provenance(), PROVENANCE);
    assert_ne!(summary.description(), SELECTED_BODY);
}

#[test]
fn explicit_load_includes_only_selected_trusted_body() {
    let first = valid_manifest();
    let second = second_manifest();
    let manifests = [first, second];
    let bodies = [
        BodyDraft {
            name: NAME,
            content_identity: VALID_HASH,
            body: UNRELATED_BODY,
        },
        BodyDraft {
            name: SECOND_NAME,
            content_identity: SECOND_HASH,
            body: SELECTED_BODY,
        },
    ];
    let context = build_context(Request {
        manifests: &manifests,
        loaded_bodies: &bodies,
        selected_name: Some(SECOND_NAME),
    });

    assert_eq!(context.catalog().summaries().len(), EXPECTED_TWO_ITEMS);
    let selected = context
        .selected_body()
        .expect("selected trusted body should load");
    assert_eq!(selected.name(), SECOND_NAME);
    assert_eq!(selected.content_identity().as_str(), SECOND_HASH);
    assert_eq!(selected.body(), SELECTED_BODY);
    assert_ne!(selected.body(), UNRELATED_BODY);
}

#[test]
fn non_hermes_store_uses_same_descriptor_boundary() {
    let manifest = Descriptor::validate(&database_draft())
        .expect("database-backed descriptor should validate as data");
    let context = build_context(Request {
        manifests: &[manifest],
        loaded_bodies: &[],
        selected_name: None,
    });

    assert_eq!(context.catalog().summaries().len(), EXPECTED_SINGLE_ITEM);
    assert_eq!(
        context.catalog().summaries()[0].category(),
        DATABASE_CATEGORY
    );
    assert_eq!(
        context.catalog().summaries()[0].provenance(),
        DATABASE_PROVENANCE
    );
    assert!(context.selected_body().is_none());
    assert!(PROHIBITED_MANIFEST_EFFECTS.contains(&"home-directory scan"));
    assert!(PROHIBITED_MANIFEST_EFFECTS.contains(&"skill body file read"));
}
