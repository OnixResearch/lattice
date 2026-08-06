use super::*;

#[test]
fn missing_description_fails_validation() {
    let missing_description = Descriptor::validate(&Draft {
        description: None,
        ..valid_draft()
    })
    .expect_err("missing description should fail");

    assert_eq!(missing_description, Error::MissingField {
        field: DESCRIPTION_FIELD.as_str(),
    });
}

#[test]
fn overlong_description_fails_validation() {
    let overlong_chars = MAX_DESCRIPTION_CHARS.saturating_add(DESCRIPTION_OVERFLOW_CHARS);
    let long_description = alloc::string::String::from("x").repeat(overlong_chars);
    let overlong_description = Descriptor::validate(&Draft {
        description: Some(long_description.as_str()),
        ..valid_draft()
    })
    .expect_err("overlong description should fail");

    assert_eq!(overlong_description, Error::DescriptionTooLong {
        max_chars: MAX_DESCRIPTION_CHARS,
        actual_chars: overlong_chars,
    });
}

#[test]
fn unsupported_semver_shape_fails_validation() {
    let bad_shape_error = Descriptor::validate(&Draft {
        version: Some(BAD_VERSION),
        ..valid_draft()
    })
    .expect_err("unsupported version shape should fail");

    assert_eq!(bad_shape_error, Error::UnsupportedVersion {
        value: Text::from(BAD_VERSION),
    });
}

#[test]
fn malformed_name_fails_validation() {
    let invalid_name = Descriptor::validate(&Draft {
        name: Some(INVALID_NAME),
        ..valid_draft()
    })
    .expect_err("invalid skill name should fail");

    assert_eq!(invalid_name, Error::MalformedName {
        field: NAME_FIELD.as_str(),
        value: Text::from(INVALID_NAME),
    });
}

#[test]
fn missing_content_identity_fails_validation() {
    let missing_identity = Descriptor::validate(&Draft {
        content_identity: None,
        ..valid_draft()
    })
    .expect_err("missing content identity should fail");

    assert_eq!(missing_identity, Error::MissingField {
        field: CONTENT_IDENTITY_FIELD.as_str(),
    });
}

#[test]
fn unsafe_content_identities_fail_validation() {
    let unsafe_identity = Descriptor::validate(&Draft {
        content_identity: Some(RAW_PATH_HANDLE),
        ..valid_draft()
    })
    .expect_err("path-like content handles should fail");
    assert!(matches!(unsafe_identity, Error::InvalidContentIdentity { .. }));

    let uppercase_identity = Descriptor::validate(&Draft {
        content_identity: Some(UPPERCASE_HASH),
        ..valid_draft()
    })
    .expect_err("uppercase hash should fail");
    assert!(matches!(uppercase_identity, Error::InvalidContentIdentity { .. }));
}

#[test]
fn duplicate_tag_fails_validation() {
    let duplicate_tag = Descriptor::validate(&Draft {
        tags: &[TAG, TAG],
        ..valid_draft()
    })
    .expect_err("duplicate tags should fail");

    assert_eq!(duplicate_tag, Error::DuplicateTag { tag: Text::from(TAG) });
}

#[test]
fn malformed_related_name_fails_validation() {
    let malformed_related = Descriptor::validate(&Draft {
        related_skills: &[INVALID_RELATED_NAME],
        ..valid_draft()
    })
    .expect_err("malformed related skill names should fail");

    assert_eq!(malformed_related, Error::MalformedName {
        field: RELATED_SKILL_FIELD.as_str(),
        value: Text::from(INVALID_RELATED_NAME),
    });
}

#[test]
fn catalog_diagnostics_fail_closed_for_duplicates_and_unresolved_related() {
    let first = valid_manifest();
    let duplicate = Descriptor::validate(&Draft {
        content_identity: Some(SECOND_HASH),
        ..valid_draft()
    })
    .expect("duplicate descriptor should validate before cataloging");
    let unresolved = Descriptor::validate(&Draft {
        name: Some(SECOND_NAME),
        content_identity: Some(SECOND_HASH),
        related_skills: &[MISSING_RELATED_NAME],
        ..valid_draft()
    })
    .expect("descriptor with unresolved relation should validate before cataloging");

    let catalog = advertise(&[first, duplicate, unresolved]);

    assert_eq!(catalog.summaries().len(), EXPECTED_SINGLE_ITEM);
    assert_eq!(catalog.summaries()[0].name(), SECOND_NAME);
    assert_eq!(catalog.diagnostics().len(), EXPECTED_TWO_ITEMS);
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.issue() == Issue::DuplicateSkillName
            && diagnostic.name() == NAME
            && diagnostic.related_skill().is_none()
    }));
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.issue() == Issue::UnresolvedRelatedSkill
            && diagnostic.name() == SECOND_NAME
            && diagnostic.related_skill() == Some(MISSING_RELATED_NAME)
    }));
}

#[test]
fn untrusted_selected_body_is_omitted() {
    let trusted = valid_manifest();
    let untrusted = untrusted_manifest();
    let manifests = [trusted, untrusted];
    let untrusted_body = [BodyDraft {
        name: UNTRUSTED_NAME,
        content_identity: VALID_HASH,
        body: SELECTED_BODY,
    }];
    let untrusted_context = build_context(Request {
        manifests: &manifests,
        loaded_bodies: &untrusted_body,
        selected_name: Some(UNTRUSTED_NAME),
    });

    assert!(untrusted_context.selected_body().is_none());
}

#[test]
fn mismatched_body_identity_is_omitted() {
    let manifest = valid_manifest();
    let manifests = [manifest];
    let wrong_identity_body = [BodyDraft {
        name: NAME,
        content_identity: SECOND_HASH,
        body: SELECTED_BODY,
    }];
    let wrong_identity_context = build_context(Request {
        manifests: &manifests,
        loaded_bodies: &wrong_identity_body,
        selected_name: Some(NAME),
    });

    assert!(wrong_identity_context.selected_body().is_none());
}

#[test]
fn empty_selected_body_is_omitted() {
    let manifest = valid_manifest();
    let manifests = [manifest];
    let empty_body = [BodyDraft {
        name: NAME,
        content_identity: VALID_HASH,
        body: EMPTY_BODY,
    }];
    let empty_body_context = build_context(Request {
        manifests: &manifests,
        loaded_bodies: &empty_body,
        selected_name: Some(NAME),
    });

    assert!(empty_body_context.selected_body().is_none());
}
