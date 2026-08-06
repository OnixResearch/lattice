#[derive(Clone, Copy)]
struct LineBounds {
    line_count: u32,
    line_number: u32,
}

/// Borrowed tagged line descriptor produced by a host-owned snapshot reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaggedLineDraft<'a> {
    /// One-based line number represented by this tag.
    pub line_number: u32,
    /// Opaque line tag shown to the model.
    pub tag: &'a str,
    /// BLAKE3 hash of normalized line content or tag-generation input.
    pub line_hash: &'a str,
}

/// Validated tagged line descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaggedLine {
    line_number: u32,
    tag: super::Text,
    line_hash: super::Text,
}

impl TaggedLine {
    /// Returns the one-based line number.
    #[must_use]
    pub const fn line_number(&self) -> u32 {
        self.line_number
    }

    /// Returns the opaque line tag.
    #[must_use]
    pub fn tag(&self) -> &str {
        self.tag.as_str()
    }

    /// Returns the BLAKE3 line/tag input hash.
    #[must_use]
    pub fn line_hash(&self) -> &str {
        self.line_hash.as_str()
    }
}

/// Borrowed hashed edit snapshot input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotDraft<'a> {
    /// Safe resource class label, such as `file` or `buffer`.
    pub resource_class: &'a str,
    /// Safe resource identity label or hash chosen by the host.
    pub resource_identity: &'a str,
    /// Full-content BLAKE3 snapshot hash.
    pub snapshot_hash: &'a str,
    /// Text normalization label.
    pub normalization: &'a str,
    /// Total line count in the normalized snapshot.
    pub line_count: u32,
    /// Tag-generation metadata label.
    pub tag_generation: &'a str,
    /// Tagged lines available for span selection.
    pub tagged_lines: &'a [TaggedLineDraft<'a>],
}

/// Validated hashed edit snapshot over in-memory metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    resource_class: super::Text,
    resource_identity: super::Text,
    content_hash: super::Text,
    normalization: super::TextNormalization,
    line_count: u32,
    tag_generation: super::Text,
    tagged_lines: super::Collection<TaggedLine>,
    identity_hash: super::Text,
}

impl Snapshot {
    /// Validates a hashed edit snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the snapshot has malformed fields,
    /// duplicate/ambiguous tags, invalid BLAKE3 hashes, or impossible line
    /// positions.
    pub fn new(draft: &SnapshotDraft<'_>) -> Result<Self, super::Error> {
        let resource_class = super::required_label(super::RESOURCE_CLASS_FIELD, draft.resource_class)?;
        let resource_identity = super::required_label(super::RESOURCE_IDENTITY_FIELD, draft.resource_identity)?;
        let snapshot_hash = super::validate_hash(super::SNAPSHOT_HASH_FIELD, draft.snapshot_hash)?;
        let normalization = super::TextNormalization::parse(draft.normalization)?;
        let tag_generation = super::required_label(super::TAG_GENERATION_FIELD, draft.tag_generation)?;
        let tagged_lines = collect_tagged_lines(draft.line_count, draft.tagged_lines)?;
        let identity_hash = super::hash::snapshot_identity(super::hash::SnapshotIdentity {
            resource_class: resource_class.as_str(),
            resource_identity: resource_identity.as_str(),
            snapshot_hash: snapshot_hash.as_str(),
            normalization,
            line_count: draft.line_count,
            tag_generation: tag_generation.as_str(),
            tagged_lines: tagged_lines.as_slice(),
        });
        Ok(Self {
            resource_class,
            resource_identity,
            content_hash: snapshot_hash,
            normalization,
            line_count: draft.line_count,
            tag_generation,
            tagged_lines,
            identity_hash,
        })
    }

    /// Returns the safe resource class.
    #[must_use]
    pub fn resource_class(&self) -> &str {
        self.resource_class.as_str()
    }

    /// Returns the safe resource identity label or hash.
    #[must_use]
    pub fn resource_identity(&self) -> &str {
        self.resource_identity.as_str()
    }

    /// Returns the full-content BLAKE3 snapshot hash.
    #[must_use]
    pub fn snapshot_hash(&self) -> &str {
        self.content_hash.as_str()
    }

    /// Returns the snapshot text normalization.
    #[must_use]
    pub const fn normalization(&self) -> super::TextNormalization {
        self.normalization
    }

    /// Returns the total normalized line count.
    #[must_use]
    pub const fn line_count(&self) -> u32 {
        self.line_count
    }

    /// Returns the tag-generation metadata label.
    #[must_use]
    pub fn tag_generation(&self) -> &str {
        self.tag_generation.as_str()
    }

    /// Returns validated tagged line descriptors.
    #[must_use]
    pub fn tagged_lines(&self) -> &[TaggedLine] {
        self.tagged_lines.as_slice()
    }

    /// Returns the BLAKE3 hash of safe snapshot identity fields.
    #[must_use]
    pub fn identity_hash(&self) -> &str {
        self.identity_hash.as_str()
    }

    pub(super) fn line_for_tag(&self, tag: &str) -> Result<&TaggedLine, super::Error> {
        let mut matches = self.tagged_lines.iter().filter(|line| line.tag() == tag);
        let Some(first_match) = matches.next() else {
            return Err(super::Error::UnknownTag {
                tag: super::Text::from(tag),
            });
        };
        if matches.next().is_some() {
            return Err(super::Error::AmbiguousTag {
                tag: super::Text::from(tag),
            });
        }
        Ok(first_match)
    }
}

fn collect_tagged_lines(
    line_count: u32,
    drafts: &[TaggedLineDraft<'_>],
) -> Result<super::Collection<TaggedLine>, super::Error> {
    super::validate_non_empty_collection(super::TAGGED_LINES_FIELD, drafts.len())?;
    super::validate_collection_count(super::CountLimit {
        field: super::TAGGED_LINES_FIELD,
        actual_items: drafts.len(),
        max_items: super::MAX_LINES,
    })?;
    assert!(!drafts.is_empty());
    assert!(drafts.len() <= super::MAX_LINES);
    let mut tagged_lines = super::Collection::with_capacity(drafts.len());
    for draft in drafts {
        validate_line_number(LineBounds {
            line_count,
            line_number: draft.line_number,
        })?;
        let tag = super::required_label(super::LINE_TAG_FIELD, draft.tag)?;
        let line_hash = super::validate_hash(super::LINE_HASH_FIELD, draft.line_hash)?;
        validate_unique_tag(tagged_lines.as_slice(), tag.as_str())?;
        validate_unique_line(tagged_lines.as_slice(), draft.line_number)?;
        tagged_lines.push(TaggedLine {
            line_number: draft.line_number,
            tag,
            line_hash,
        });
    }
    tagged_lines.sort_by_key(TaggedLine::line_number);
    Ok(tagged_lines)
}

fn validate_line_number(bounds: LineBounds) -> Result<(), super::Error> {
    if bounds.line_number == 0 || bounds.line_number > bounds.line_count {
        return Err(super::Error::LineNumberOutOfRange {
            line_number: bounds.line_number,
            line_count: bounds.line_count,
        });
    }
    Ok(())
}

fn validate_unique_tag(lines: &[TaggedLine], tag: &str) -> Result<(), super::Error> {
    if lines.iter().any(|line| line.tag() == tag) {
        return Err(super::Error::DuplicateTag {
            tag: super::Text::from(tag),
        });
    }
    Ok(())
}

fn validate_unique_line(lines: &[TaggedLine], line_number: u32) -> Result<(), super::Error> {
    if lines.iter().any(|line| line.line_number() == line_number) {
        return Err(super::Error::AmbiguousLine { line_number });
    }
    Ok(())
}
