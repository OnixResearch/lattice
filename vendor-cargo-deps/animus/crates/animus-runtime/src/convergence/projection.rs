/// Exact omission facts for one bounded rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Omission {
    /// Number of original characters outside the visible prefix.
    pub omitted_chars: u64,
    /// Store-typed handle for further retrieval.
    pub handle: super::Handle,
}

/// Model-facing projection of one typed datum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelView {
    /// Shared typed-datum identity.
    pub datum_identity: alloc::string::String,
    /// Bounded text plus an exact omission statement when required.
    pub text: alloc::string::String,
    /// Structured omission facts.
    pub omission: Option<Omission>,
}

/// Human-facing projection of one typed datum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanView {
    /// Shared typed-datum identity.
    pub datum_identity: alloc::string::String,
    /// Stable call identity.
    pub call_id: alloc::string::String,
    /// Adapter result kind.
    pub result_kind: alloc::string::String,
    /// Bounded visible text.
    pub visible_text: alloc::string::String,
    /// Structured omission facts.
    pub omission: Option<Omission>,
    /// Optional bounded telemetry.
    pub telemetry: Option<super::ResourceTelemetry>,
}

/// Structured append-only projection for persistence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedRecord {
    /// Shared typed-datum identity.
    pub datum_identity: alloc::string::String,
    /// Stable call identity.
    pub call_id: alloc::string::String,
    /// Adapter result kind.
    pub result_kind: alloc::string::String,
    /// Complete or truncated status.
    pub status: animus_adapters::tool::OutputStatus,
    /// Original byte count.
    pub byte_count: u64,
    /// Original character count.
    pub char_count: u64,
    /// Original line count.
    pub line_count: u64,
    /// Optional BLAKE3 full-content identity.
    pub content_hash: Option<alloc::string::String>,
    /// Optional store-typed handle.
    pub handle: Option<super::Handle>,
    /// Optional bounded telemetry.
    pub telemetry: Option<super::ResourceTelemetry>,
}

/// Projects a model-facing view from one canonical datum.
///
/// # Errors
///
/// Returns [`super::Error::SilentTruncation`] if omission facts are incomplete.
pub fn project_model(datum: &super::Datum) -> Result<ModelView, super::Error> {
    let omission = project_omission(datum)?;
    debug_assert!(!datum.identity().is_empty());
    debug_assert!(
        u64::try_from(datum.visible_text().chars().count())
            .is_ok_and(|visible_chars| visible_chars <= datum.char_count())
    );
    let mut text = alloc::string::String::from(datum.visible_text());
    if let Some(facts) = &omission {
        use core::fmt::Write as _;
        write!(
            text,
            "\n[omitted {} characters; follow {}:{}]",
            facts.omitted_chars,
            facts.handle.kind().label(),
            facts.handle.token()
        )
        .map_err(|_format| super::Error::InvalidIdentity)?;
    }
    Ok(ModelView {
        datum_identity: alloc::string::String::from(datum.identity()),
        text,
        omission,
    })
}

/// Projects a human-facing view from one canonical datum.
///
/// # Errors
///
/// Returns [`super::Error::SilentTruncation`] if omission facts are incomplete.
pub fn project_human(datum: &super::Datum) -> Result<HumanView, super::Error> {
    Ok(HumanView {
        datum_identity: alloc::string::String::from(datum.identity()),
        call_id: alloc::string::String::from(datum.call_id()),
        result_kind: alloc::string::String::from(datum.result_kind()),
        visible_text: alloc::string::String::from(datum.visible_text()),
        omission: project_omission(datum)?,
        telemetry: datum.telemetry(),
    })
}

/// Projects a structured persisted record from one canonical datum.
///
/// # Errors
///
/// Returns [`super::Error::SilentTruncation`] if omission facts are incomplete.
pub fn project_record(datum: &super::Datum) -> Result<PersistedRecord, super::Error> {
    let _omission = project_omission(datum)?;
    Ok(PersistedRecord {
        datum_identity: alloc::string::String::from(datum.identity()),
        call_id: alloc::string::String::from(datum.call_id()),
        result_kind: alloc::string::String::from(datum.result_kind()),
        status: datum.status(),
        byte_count: datum.byte_count(),
        char_count: datum.char_count(),
        line_count: datum.line_count(),
        content_hash: datum.content_hash().map(alloc::string::String::from),
        handle: datum.handle().cloned(),
        telemetry: datum.telemetry(),
    })
}

/// Inspects that a persisted projection still derives from its typed datum.
///
/// # Errors
///
/// Returns [`super::Error::InvalidIdentity`] for an ad-hoc or drifted record.
pub fn inspect_record(datum: &super::Datum, record: &PersistedRecord) -> Result<(), super::Error> {
    let expected = project_record(datum)?;
    if expected != *record {
        return Err(super::Error::InvalidIdentity);
    }
    Ok(())
}

fn project_omission(datum: &super::Datum) -> Result<Option<Omission>, super::Error> {
    if datum.status() == animus_adapters::tool::OutputStatus::Complete {
        return Ok(None);
    }
    let visible_chars = u64::try_from(datum.visible_text().chars().count())
        .map_err(|_| super::Error::SilentTruncation)?;
    let omitted_chars = datum
        .char_count()
        .checked_sub(visible_chars)
        .ok_or(super::Error::SilentTruncation)?;
    let handle = datum
        .handle()
        .cloned()
        .ok_or(super::Error::SilentTruncation)?;
    Ok(Some(Omission {
        omitted_chars,
        handle,
    }))
}
