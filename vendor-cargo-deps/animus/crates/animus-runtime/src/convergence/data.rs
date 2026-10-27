const HASH_DOMAIN: &str = "animus.harness-convergence.datum.v1";
const HASH_SEPARATOR: &str = "|";

/// Bounded resource facts attached to one result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceTelemetry {
    /// Declared CPU-time ceiling.
    pub cpu_bound_ms: u64,
    /// Observed CPU time.
    pub cpu_observed_ms: u64,
    /// Declared output ceiling.
    pub output_bound_bytes: u64,
    /// Observed output bytes.
    pub output_observed_bytes: u64,
}

impl ResourceTelemetry {
    fn validate(self) -> Result<(), Error> {
        if self.cpu_bound_ms == 0 || self.output_bound_bytes == 0 {
            return Err(Error::TelemetryOutOfBounds);
        }
        if self.cpu_observed_ms > self.cpu_bound_ms
            || self.output_observed_bytes > self.output_bound_bytes
        {
            return Err(Error::TelemetryOutOfBounds);
        }
        Ok(())
    }
}

/// Borrowed facts used to admit one typed result datum.
#[derive(Clone, Copy)]
pub struct DatumDraft<'a> {
    /// Stable call identity.
    pub call_id: &'a str,
    /// Adapter-declared result kind.
    pub result_kind: &'a str,
    /// Existing bounded adapter output.
    pub output: &'a animus_adapters::tool::BoundedOutput,
    /// Store handle required for truncated output.
    pub handle: Option<&'a super::Handle>,
    /// Optional bounded resource telemetry.
    pub telemetry: Option<ResourceTelemetry>,
}

/// Canonical typed tool-result datum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Datum {
    call_id: alloc::string::String,
    result_kind: alloc::string::String,
    visible_text: alloc::string::String,
    status: animus_adapters::tool::OutputStatus,
    byte_count: u64,
    char_count: u64,
    line_count: u64,
    content_hash: Option<alloc::string::String>,
    handle: Option<super::Handle>,
    telemetry: Option<ResourceTelemetry>,
    identity: alloc::string::String,
}

impl Datum {
    /// Returns the stable typed-datum identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the stable call identity.
    #[must_use]
    pub fn call_id(&self) -> &str {
        self.call_id.as_str()
    }

    /// Returns the adapter-declared result kind.
    #[must_use]
    pub fn result_kind(&self) -> &str {
        self.result_kind.as_str()
    }

    /// Returns the bounded visible text projection source.
    #[must_use]
    pub fn visible_text(&self) -> &str {
        self.visible_text.as_str()
    }

    /// Returns complete or truncated status.
    #[must_use]
    pub const fn status(&self) -> animus_adapters::tool::OutputStatus {
        self.status
    }

    /// Returns original output byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    /// Returns original output character count.
    #[must_use]
    pub const fn char_count(&self) -> u64 {
        self.char_count
    }

    /// Returns original output line count.
    #[must_use]
    pub const fn line_count(&self) -> u64 {
        self.line_count
    }

    /// Returns optional full-content identity.
    #[must_use]
    pub fn content_hash(&self) -> Option<&str> {
        self.content_hash.as_deref()
    }

    /// Returns the typed handle when present.
    #[must_use]
    pub const fn handle(&self) -> Option<&super::Handle> {
        self.handle.as_ref()
    }

    /// Returns optional bounded telemetry.
    #[must_use]
    pub const fn telemetry(&self) -> Option<ResourceTelemetry> {
        self.telemetry
    }
}

/// Store resolution capability. Implementations own persistence and authority.
pub trait StorePort {
    /// Resolves one exact store-typed handle.
    ///
    /// # Errors
    ///
    /// Returns a typed store error without guessing another namespace.
    fn resolve(&self, handle: &super::Handle) -> Result<Option<Datum>, Error>;
}

/// Contract failure for convergence rails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Required identity or kind was blank.
    InvalidIdentity,
    /// Handle syntax or length was invalid.
    InvalidHandle,
    /// Truncation omitted a required handle.
    SilentTruncation,
    /// Telemetry exceeded its named bound.
    TelemetryOutOfBounds,
    /// Store did not know the exact handle.
    UnknownHandle {
        /// Exact opaque token supplied by the caller.
        token: alloc::string::String,
    },
    /// Store returned a datum bound to another handle.
    StoreMismatch,
    /// History input was malformed or exceeded its bound.
    InvalidHistory,
    /// Supervision facts were malformed or out of sequence.
    InvalidSupervision,
}

/// Admits one canonical typed datum.
///
/// # Errors
///
/// Returns [`Error`] for blank identities, silent truncation, or unbounded telemetry.
pub fn admit(draft: DatumDraft<'_>) -> Result<Datum, Error> {
    validate_label(draft.call_id)?;
    validate_label(draft.result_kind)?;
    debug_assert!(!draft.call_id.trim().is_empty());
    debug_assert!(!draft.result_kind.trim().is_empty());
    if draft.output.status() == animus_adapters::tool::OutputStatus::Truncated
        && draft.handle.is_none()
    {
        return Err(Error::SilentTruncation);
    }
    if let Some(telemetry) = draft.telemetry {
        telemetry.validate()?;
    }
    let handle = draft.handle.cloned();
    let content_hash = draft.output.content_hash().map(alloc::string::String::from);
    let identity = hash_parts(&[
        draft.call_id,
        draft.result_kind,
        draft.output.visible_text(),
        output_status_label(draft.output.status()),
        draft.output.content_hash().unwrap_or("none"),
        handle.as_ref().map_or("none", super::Handle::identity),
    ]);
    Ok(Datum {
        call_id: alloc::string::String::from(draft.call_id),
        result_kind: alloc::string::String::from(draft.result_kind),
        visible_text: alloc::string::String::from(draft.output.visible_text()),
        status: draft.output.status(),
        byte_count: draft.output.byte_count(),
        char_count: draft.output.char_count(),
        line_count: draft.output.line_count(),
        content_hash,
        handle,
        telemetry: draft.telemetry,
        identity,
    })
}

/// Resolves an exact handle by asking its owning store.
///
/// # Errors
///
/// Returns [`Error::UnknownHandle`] with the exact token or
/// [`Error::StoreMismatch`] for substituted data.
pub fn resolve<Store: StorePort>(store: &Store, handle: &super::Handle) -> Result<Datum, Error> {
    let datum = store.resolve(handle)?.ok_or_else(|| Error::UnknownHandle {
        token: alloc::string::String::from(handle.token()),
    })?;
    if datum.handle().map(super::Handle::identity) != Some(handle.identity()) {
        return Err(Error::StoreMismatch);
    }
    Ok(datum)
}

fn validate_label(value: &str) -> Result<(), Error> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(Error::InvalidIdentity);
    }
    Ok(())
}

fn hash_parts(parts: &[&str]) -> alloc::string::String {
    let mut framed = alloc::string::String::from(HASH_DOMAIN);
    for part in parts {
        framed.push_str(HASH_SEPARATOR);
        framed.push_str(part);
    }
    alloc::format!("{}", blake3::hash(framed.as_bytes()).to_hex())
}

const fn output_status_label(status: animus_adapters::tool::OutputStatus) -> &'static str {
    match status {
        animus_adapters::tool::OutputStatus::Complete => "complete",
        animus_adapters::tool::OutputStatus::Truncated => "truncated",
    }
}
