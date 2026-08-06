pub(super) fn validate(draft: &super::Draft<'_>) -> Result<super::Descriptor, super::Error> {
    let name = required_name(super::NAME_FIELD, draft.name)?;
    let description = required_description(draft.description)?;
    let version = required_version(draft.version)?;
    let category = required_name(super::CATEGORY_FIELD, draft.category)?;
    let tags = collect_unique_names(super::TAG_FIELD, draft.tags, DuplicatePolicy::RejectTags)?;
    let related_skills =
        collect_unique_names(super::RELATED_SKILL_FIELD, draft.related_skills, DuplicatePolicy::Dedup)?;
    let provenance = required_text(super::PROVENANCE_FIELD, draft.provenance)?;
    let trust_status = super::TrustStatus::parse(draft.trust_status)?;
    let content_identity = required_content_identity(draft.content_identity)?;

    assert!(!name.is_empty(), "validated descriptor has a name");
    assert!(
        description.chars().count() <= super::MAX_DESCRIPTION_CHARS,
        "validated descriptor summary stays bounded"
    );
    assert!(tags.len() <= super::MAX_ITEMS, "validated tags stay bounded");
    assert!(related_skills.len() <= super::MAX_ITEMS, "validated related skills stay bounded");
    assert_eq!(
        content_identity.as_str().len(),
        super::CONTENT_HASH_HEX_CHARS,
        "validated content identity is a full BLAKE3 hex string"
    );

    Ok(super::Descriptor {
        name,
        description,
        version,
        category,
        tags,
        related_skills,
        provenance,
        trust_status,
        content_identity,
    })
}

fn required_description(value: Option<&str>) -> Result<super::Text, super::Error> {
    let description = required_text(super::DESCRIPTION_FIELD, value)?;
    let actual_chars = description.chars().count();
    if actual_chars > super::MAX_DESCRIPTION_CHARS {
        return Err(super::Error::DescriptionTooLong {
            max_chars: super::MAX_DESCRIPTION_CHARS,
            actual_chars,
        });
    }
    Ok(description)
}

fn required_version(value: Option<&str>) -> Result<super::Text, super::Error> {
    let version = required_text(super::VERSION_FIELD, value)?;
    if !valid_version(version.as_str()) {
        return Err(super::Error::UnsupportedVersion { value: version });
    }
    Ok(version)
}

fn required_name(field: crate::validation::FieldName, value: Option<&str>) -> Result<super::Text, super::Error> {
    let name = required_text(field, value)?;
    validate_name(field, name.as_str())?;
    Ok(name)
}

pub(super) fn required_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<super::Text, super::Error> {
    match value {
        Some(raw) => crate::validation::normalized_adapter_field(field, raw).map_err(Into::into),
        None => Err(super::Error::MissingField { field: field.as_str() }),
    }
}

fn required_content_identity(value: Option<&str>) -> Result<crate::artifact::Hash, super::Error> {
    let identity = required_text(super::CONTENT_IDENTITY_FIELD, value)?;
    crate::artifact::Hash::new(identity.as_str()).map_err(|error| content_identity_error(&error))
}

fn content_identity_error(error: &crate::artifact::Error) -> super::Error {
    match error {
        crate::artifact::Error::InvalidHash { reason, .. } => super::Error::InvalidContentIdentity {
            field: super::CONTENT_IDENTITY_FIELD.as_str(),
            reason,
        },
        crate::artifact::Error::PayloadTooLong { .. } | crate::artifact::Error::PayloadContainsNul { .. } => {
            super::Error::InvalidContentIdentity {
                field: super::CONTENT_IDENTITY_FIELD.as_str(),
                reason: "skill content identity must be lowercase BLAKE3 hex",
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DuplicatePolicy {
    RejectTags,
    Dedup,
}

fn collect_unique_names(
    field: crate::validation::FieldName,
    values: &[&str],
    duplicate_policy: DuplicatePolicy,
) -> Result<super::Collection<super::Text>, super::Error> {
    validate_item_count(field, values.len())?;
    let mut names = super::Collection::with_capacity(values.len());
    for value in values {
        let name = required_text(field, Some(*value))?;
        validate_name(field, name.as_str())?;
        if names.iter().any(|item| item == &name) {
            if duplicate_policy == DuplicatePolicy::RejectTags {
                return Err(super::Error::DuplicateTag { tag: name });
            }
            continue;
        }
        names.push(name);
    }
    names.sort();
    Ok(names)
}

fn validate_item_count(field: crate::validation::FieldName, item_count: usize) -> Result<(), super::Error> {
    if item_count > super::MAX_ITEMS {
        return Err(super::Error::TooManyItems {
            field: field.as_str(),
            max_items: super::MAX_ITEMS,
            actual_items: item_count,
        });
    }
    Ok(())
}

fn validate_name(field: crate::validation::FieldName, value: &str) -> Result<(), super::Error> {
    if valid_name(value) {
        Ok(())
    } else {
        Err(super::Error::MalformedName {
            field: field.as_str(),
            value: super::Text::from(value),
        })
    }
}

fn valid_name(value: &str) -> bool {
    assert!(!value.is_empty(), "name candidate is non-empty");
    assert!(
        value.chars().count() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "name candidate fits the adapter field bound"
    );
    let mut is_previous_separator = false;
    let mut has_visible_character = false;
    for character in value.chars() {
        let is_name_separator =
            character == super::NAME_SEPARATOR_DASH || character == super::NAME_SEPARATOR_UNDERSCORE;
        if is_name_separator {
            if !has_visible_character || is_previous_separator {
                return false;
            }
            is_previous_separator = true;
            continue;
        }
        if !(character.is_ascii_lowercase() || character.is_ascii_digit()) {
            return false;
        }
        is_previous_separator = false;
        has_visible_character = true;
    }
    has_visible_character && !is_previous_separator
}

fn valid_version(value: &str) -> bool {
    let mut segments_seen: usize = 0;
    for segment in value.split(super::VERSION_SEPARATOR) {
        segments_seen = segments_seen.saturating_add(1);
        if segment.is_empty() || !segment.chars().all(|character| character.is_ascii_digit()) {
            return false;
        }
    }
    segments_seen == super::SEMVER_SEGMENT_COUNT
}
