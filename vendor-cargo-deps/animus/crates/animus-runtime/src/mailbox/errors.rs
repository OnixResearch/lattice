/// Error classes for queued-message and interrupt control validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageQueueErrorKind {
    /// A required field was empty.
    EmptyField,
    /// A field exceeded the bounded metadata size.
    FieldTooLong,
    /// A field contained unsafe evidence or leak markers.
    UnsafeField,
    /// A BLAKE3 hash field was malformed.
    InvalidHash,
    /// A value was not supported for the requested operation.
    UnsupportedValue,
    /// Queue policy was internally invalid.
    InvalidPolicy,
    /// The operation targeted a stale active-work identity.
    StaleActiveWork,
    /// The queue already contains the requested item identity.
    DuplicateQueueItem,
    /// The queue does not contain the requested item identity.
    UnknownQueueItem,
    /// The caller is not authorized to mutate that queue item or scope.
    UnauthorizedSource,
    /// Queue admission would exceed the configured queue bound.
    MaxQueueExceeded,
    /// Queued material was marked unsafe.
    UnsafeQueuedBody,
    /// Queued material exceeded the configured prompt budget class.
    BudgetExceeded,
    /// Policy denied the operation.
    PolicyDenied,
    /// Cancellation was requested without a usable host cancellation handle.
    MissingCancellationHandle,
    /// The active-work state cannot support the requested control action.
    UnsupportedActiveWorkState,
    /// Dispatch acknowledgement was missing, stale, or mismatched.
    MalformedAcknowledgement,
    /// A dispatch acknowledgement arrived without a pending dispatch.
    MissingPendingDispatch,
    /// A second dispatch was requested while one was pending acknowledgement.
    PendingDispatchAlreadyExists,
    /// An interrupt signal was missing required data for its send mode.
    MalformedInterrupt,
    /// Durable receipt text contained sensitive or raw material.
    ReceiptLeak,
}

/// Validation failure for queued-message and interrupt control contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageQueueError {
    kind: MessageQueueErrorKind,
    field: &'static str,
    detail: String,
}

impl MessageQueueError {
    pub(crate) fn new(
        kind: MessageQueueErrorKind,
        field: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            field,
            detail: detail.into(),
        }
    }

    /// Returns the stable error kind.
    #[must_use]
    pub const fn kind(&self) -> MessageQueueErrorKind {
        self.kind
    }

    /// Returns the field that failed validation.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns a safe diagnostic detail.
    #[must_use]
    pub fn detail(&self) -> &str {
        self.detail.as_str()
    }
}
