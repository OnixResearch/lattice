use super::super::*;
use super::support::*;

#[test]
fn missing_description_fails() {
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
fn invalid_names_and_kind_labels_fail() {
    let invalid_name = Descriptor::validate(&Draft {
        name: Some(INVALID_NAME),
        ..valid_draft()
    })
    .expect_err("invalid descriptor name should fail");
    assert_eq!(invalid_name, Error::MalformedName {
        field: NAME_FIELD.as_str(),
        value: Text::from(INVALID_NAME),
    });
    assert_eq!(
        Descriptor::validate(&Draft {
            kind: Some(UNSUPPORTED_KIND),
            ..valid_draft()
        })
        .expect_err("unsupported resource kind should fail"),
        Error::UnsupportedValue {
            field: KIND_FIELD.as_str(),
            value: Text::from(UNSUPPORTED_KIND),
        }
    );
}

#[test]
fn unsafe_identity_and_provenance_fail() {
    let unsafe_identity = Descriptor::validate(&Draft {
        content_identity: Some(RAW_PATH_HANDLE),
        ..valid_draft()
    })
    .expect_err("path-like content handles should fail");
    let uppercase_identity = Descriptor::validate(&Draft {
        content_identity: Some(UPPERCASE_HASH),
        ..valid_draft()
    })
    .expect_err("uppercase hash should fail");
    let unsafe_provenance = Descriptor::validate(&Draft {
        provenance: Some(UNSAFE_PROVENANCE),
        ..valid_draft()
    })
    .expect_err("path-like provenance should fail");

    assert!(matches!(unsafe_identity, Error::InvalidContentIdentity { .. }));
    assert!(matches!(uppercase_identity, Error::InvalidContentIdentity { .. }));
    assert_eq!(unsafe_provenance, Error::MalformedProvenance {
        value: Text::from(UNSAFE_PROVENANCE),
    });
}

#[test]
fn overlong_description_fails() {
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
