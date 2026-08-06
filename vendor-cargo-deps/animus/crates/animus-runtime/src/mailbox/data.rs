/// Safety class assigned by host policy before queue admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageSafetyClass {
    /// The message body was admitted by host policy.
    Safe,
    /// The message body was denied by host policy.
    Unsafe,
}

impl MessageSafetyClass {
    /// Returns the stable safety label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Unsafe => "unsafe",
        }
    }
}

/// Budget class assigned by host policy before queue admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageBudgetClass {
    /// The message is within the configured queue budget.
    WithinBudget,
    /// The message exceeds the configured queue budget.
    OverBudget,
}

impl MessageBudgetClass {
    /// Returns the stable budget label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WithinBudget => "within_budget",
            Self::OverBudget => "over_budget",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, super::MessageQueueError> {
        match value {
            "within_budget" => Ok(Self::WithinBudget),
            "over_budget" => Ok(Self::OverBudget),
            other => Err(super::MessageQueueError::new(
                super::MessageQueueErrorKind::UnsupportedValue,
                super::FIELD_BUDGET_CLASS,
                format!("unsupported budget class: {other}"),
            )),
        }
    }
}

/// Redaction state for safe queue receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageRedactionState {
    /// Raw text was absent and only safe hashes/classes were retained.
    Redacted,
    /// Receipt candidate was rejected due to unsafe material.
    RejectedLeak,
}

impl MessageRedactionState {
    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Redacted => "redacted",
            Self::RejectedLeak => "rejected_leak",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, super::MessageQueueError> {
        match value {
            "redacted" => Ok(Self::Redacted),
            "rejected_leak" => Ok(Self::RejectedLeak),
            other => Err(super::MessageQueueError::new(
                super::MessageQueueErrorKind::UnsupportedValue,
                super::FIELD_REDACTION_STATE,
                format!("unsupported redaction state: {other}"),
            )),
        }
    }
}

/// Active host cancellation capability class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancellationHandleClass {
    /// The host supplied a usable cancellation handle.
    Available,
    /// The host supplied no cancellation handle.
    Missing,
    /// The active work does not support cancellation.
    Unsupported,
}

impl CancellationHandleClass {
    /// Returns the stable cancellation-handle label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::Unsupported => "unsupported",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, super::MessageQueueError> {
        match value {
            "available" => Ok(Self::Available),
            "missing" => Ok(Self::Missing),
            "unsupported" => Ok(Self::Unsupported),
            other => Err(super::MessageQueueError::new(
                super::MessageQueueErrorKind::UnsupportedValue,
                super::FIELD_CANCELLATION_HANDLE_CLASS,
                format!("unsupported cancellation handle class: {other}"),
            )),
        }
    }
}

/// Active work state supplied by the host shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveWorkStatus {
    /// Work is running and can accept control events.
    Running,
    /// Work is paused by the host.
    Paused,
    /// Work is complete and cannot accept live control events.
    Completed,
}

impl ActiveWorkStatus {
    /// Returns whether live control actions can target this state.
    #[must_use]
    pub const fn accepts_live_controls(self) -> bool {
        matches!(self, Self::Running)
    }
}

/// Explicit send mode for interrupt classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptSendMode {
    /// Let policy-owned keyword classes choose a control intent.
    Heuristic,
    /// Queue the message without making it model-visible immediately.
    Queue,
    /// Dispatch the message immediately through a host-owned prompt path.
    ImmediateDispatch,
    /// Request a pause control event.
    Pause,
    /// Request a cancellation control event.
    Cancel,
    /// Request a redirection control event.
    Redirect,
}

/// Policy-owned keyword class used by heuristic interrupt mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptKeyword {
    /// Stop-like keyword.
    Stop,
    /// Wait-like keyword.
    Wait,
    /// Pause-like keyword.
    Pause,
    /// Redirect-like keyword.
    Redirect,
    /// Continue-like keyword.
    Continue,
}

/// Dispatch acknowledgement result returned by a host shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageDispatchAcknowledgementOutcome {
    /// Host accepted the queued message for dispatch.
    Accepted,
    /// Host rejected dispatch and left the item queued.
    Rejected,
}

/// Queue operation class used by reducers and receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageQueueOperationKind {
    /// Append a queued message.
    Append,
    /// Edit a queued message.
    Edit,
    /// Reorder a queued message.
    Reorder,
    /// Delete a queued message.
    Delete,
    /// Clear messages for a source.
    Clear,
    /// Begin dispatch of the next queued message.
    DispatchNext,
    /// Acknowledge a pending dispatch.
    Acknowledge,
    /// Cancel one pending queued message.
    CancelPending,
    /// Classify an interrupt signal.
    Interrupt,
}

impl MessageQueueOperationKind {
    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Append => "append",
            Self::Edit => "edit",
            Self::Reorder => "reorder",
            Self::Delete => "delete",
            Self::Clear => "clear",
            Self::DispatchNext => "dispatch_next",
            Self::Acknowledge => "acknowledge",
            Self::CancelPending => "cancel_pending",
            Self::Interrupt => "interrupt",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, super::MessageQueueError> {
        match value {
            "append" => Ok(Self::Append),
            "edit" => Ok(Self::Edit),
            "reorder" => Ok(Self::Reorder),
            "delete" => Ok(Self::Delete),
            "clear" => Ok(Self::Clear),
            "dispatch_next" => Ok(Self::DispatchNext),
            "acknowledge" => Ok(Self::Acknowledge),
            "cancel_pending" => Ok(Self::CancelPending),
            "interrupt" => Ok(Self::Interrupt),
            other => Err(super::MessageQueueError::new(
                super::MessageQueueErrorKind::UnsupportedValue,
                super::FIELD_OPERATION_KIND,
                format!("unsupported operation kind: {other}"),
            )),
        }
    }
}

/// Borrowed queue policy input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageQueuePolicyDraft<'a> {
    /// Maximum queued items admitted at once.
    pub max_items: u32,
    /// Source labels allowed to create or mutate queue state.
    pub allowed_sources: &'a [&'a str],
    /// Safe policy label.
    pub policy_label: &'a str,
    /// BLAKE3 hash of the policy document or exported policy data.
    pub policy_hash: &'a str,
    /// Whether queue mutation operations are enabled.
    pub allow_queue_mutations: bool,
    /// Whether interrupt classification is enabled.
    pub allow_interrupts: bool,
}

/// Runtime queue policy over already-loaded host policy data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageQueuePolicy {
    max_items: u32,
    allowed_sources: Vec<String>,
    policy_label: String,
    policy_hash: String,
    allow_queue_mutations: bool,
    allow_interrupts: bool,
}

impl MessageQueuePolicy {
    /// Validates a queue-control policy.
    ///
    /// # Errors
    ///
    /// Returns [`super::MessageQueueError`] when bounds, source labels, policy
    /// labels, or policy hash values are unsafe.
    pub fn new(draft: MessageQueuePolicyDraft<'_>) -> Result<Self, super::MessageQueueError> {
        validate_policy_bounds(draft.max_items, draft.allowed_sources)?;
        debug_assert!(draft.max_items >= super::MIN_MAX_QUEUE_ITEMS);
        debug_assert!(!draft.allowed_sources.is_empty());
        let allowed_sources = draft
            .allowed_sources
            .iter()
            .map(|source| normalize_safe_metadata(super::FIELD_SOURCE, source))
            .collect::<Result<Vec<_>, _>>()?;
        let policy_label = normalize_safe_metadata(super::FIELD_POLICY_LABEL, draft.policy_label)?;
        let policy_hash = validate_hash_metadata(super::FIELD_POLICY_HASH, draft.policy_hash)?;
        Ok(Self {
            max_items: draft.max_items,
            allowed_sources,
            policy_label,
            policy_hash,
            allow_queue_mutations: draft.allow_queue_mutations,
            allow_interrupts: draft.allow_interrupts,
        })
    }

    /// Returns the maximum queued item count.
    #[must_use]
    pub const fn max_items(&self) -> u32 {
        self.max_items
    }

    /// Returns the policy label.
    #[must_use]
    pub fn policy_label(&self) -> &str {
        self.policy_label.as_str()
    }

    /// Returns the policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns whether queue mutations are enabled.
    #[must_use]
    pub const fn allow_queue_mutations(&self) -> bool {
        self.allow_queue_mutations
    }

    /// Returns whether interrupts are enabled.
    #[must_use]
    pub const fn allow_interrupts(&self) -> bool {
        self.allow_interrupts
    }

    /// Returns whether the source is admitted by this policy.
    #[must_use]
    pub fn admits_source(&self, source: &str) -> bool {
        self.allowed_sources.iter().any(|allowed| allowed.as_str() == source)
    }
}

/// Borrowed queued-message input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueuedMessageDraft<'a> {
    /// Stable item identity.
    pub item_id: &'a str,
    /// Active work this queued message belongs to.
    pub active_work_id: &'a str,
    /// Source identity for authorization and receipts.
    pub source: &'a str,
    /// BLAKE3 hash of the queued body, not raw body text.
    pub body_hash: &'a str,
    /// Host-provided safety class.
    pub safety: MessageSafetyClass,
    /// Host-provided budget class.
    pub budget: MessageBudgetClass,
    /// Policy hash used during admission.
    pub policy_hash: &'a str,
}

/// Admitted queued message metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedMessage {
    item_id: String,
    active_work_id: String,
    source: String,
    body_hash: String,
    safety: MessageSafetyClass,
    budget: MessageBudgetClass,
    policy_hash: String,
}

impl QueuedMessage {
    /// Validates a queued-message draft.
    ///
    /// # Errors
    ///
    /// Returns [`super::MessageQueueError`] when identities or hashes are unsafe,
    /// or when the safety/budget classes deny admission.
    pub fn new(draft: QueuedMessageDraft<'_>) -> Result<Self, super::MessageQueueError> {
        validate_message_admission(draft.safety, draft.budget)?;
        Ok(Self {
            item_id: normalize_safe_metadata(super::FIELD_ITEM_ID, draft.item_id)?,
            active_work_id: normalize_safe_metadata(super::FIELD_ACTIVE_WORK_ID, draft.active_work_id)?,
            source: normalize_safe_metadata(super::FIELD_SOURCE, draft.source)?,
            body_hash: validate_hash_metadata(super::FIELD_BODY_HASH, draft.body_hash)?,
            safety: draft.safety,
            budget: draft.budget,
            policy_hash: validate_hash_metadata(super::FIELD_POLICY_HASH, draft.policy_hash)?,
        })
    }

    /// Returns the queue item identity.
    #[must_use]
    pub fn item_id(&self) -> &str {
        self.item_id.as_str()
    }

    /// Returns the active-work identity.
    #[must_use]
    pub fn active_work_id(&self) -> &str {
        self.active_work_id.as_str()
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the queued body hash.
    #[must_use]
    pub fn body_hash(&self) -> &str {
        self.body_hash.as_str()
    }

    /// Returns the safety class.
    #[must_use]
    pub const fn safety(&self) -> MessageSafetyClass {
        self.safety
    }

    /// Returns the budget class.
    #[must_use]
    pub const fn budget(&self) -> MessageBudgetClass {
        self.budget
    }

    /// Returns the policy hash used for admission.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    pub(crate) fn with_replacement(&self, draft: MessageEditDraft<'_>) -> Result<Self, super::MessageQueueError> {
        validate_message_admission(draft.safety, draft.budget)?;
        Ok(Self {
            item_id: self.item_id.clone(),
            active_work_id: self.active_work_id.clone(),
            source: self.source.clone(),
            body_hash: validate_hash_metadata(super::FIELD_BODY_HASH, draft.replacement_body_hash)?,
            safety: draft.safety,
            budget: draft.budget,
            policy_hash: self.policy_hash.clone(),
        })
    }
}

/// Pure queue state for one active-work identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageQueueState {
    active_work_id: String,
    items: Vec<QueuedMessage>,
    pending_dispatch: Option<MessageDispatch>,
}

impl MessageQueueState {
    /// Creates an empty queue for one active-work identity.
    ///
    /// # Errors
    ///
    /// Returns [`super::MessageQueueError`] when the active-work identity is unsafe.
    pub fn new(active_work_id: &str) -> Result<Self, super::MessageQueueError> {
        Ok(Self {
            active_work_id: normalize_safe_metadata(super::FIELD_ACTIVE_WORK_ID, active_work_id)?,
            items: Vec::new(),
            pending_dispatch: None,
        })
    }

    /// Returns the active-work identity.
    #[must_use]
    pub fn active_work_id(&self) -> &str {
        self.active_work_id.as_str()
    }

    /// Returns queued messages in dispatch order.
    #[must_use]
    pub fn items(&self) -> &[QueuedMessage] {
        self.items.as_slice()
    }

    /// Returns pending dispatch metadata, if a dispatch is awaiting acknowledgement.
    #[must_use]
    pub const fn pending_dispatch(&self) -> Option<&MessageDispatch> {
        self.pending_dispatch.as_ref()
    }

    pub(crate) fn replace_items(&mut self, items: Vec<QueuedMessage>) {
        self.items = items;
    }

    pub(crate) fn set_pending_dispatch(&mut self, pending_dispatch: Option<MessageDispatch>) {
        self.pending_dispatch = pending_dispatch;
    }
}

/// Borrowed edit operation input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageEditDraft<'a> {
    /// Queue item identity to edit.
    pub item_id: &'a str,
    /// Active-work identity expected by the caller.
    pub active_work_id: &'a str,
    /// Source identity attempting the mutation.
    pub source: &'a str,
    /// Replacement body hash, not raw body text.
    pub replacement_body_hash: &'a str,
    /// Replacement safety class.
    pub safety: MessageSafetyClass,
    /// Replacement budget class.
    pub budget: MessageBudgetClass,
}

/// Borrowed item-scoped operation input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageItemOperationDraft<'a> {
    /// Queue item identity to mutate.
    pub item_id: &'a str,
    /// Active-work identity expected by the caller.
    pub active_work_id: &'a str,
    /// Source identity attempting the mutation.
    pub source: &'a str,
}

/// Borrowed queue-scope operation input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageScopeOperationDraft<'a> {
    /// Active-work identity expected by the caller.
    pub active_work_id: &'a str,
    /// Source identity attempting the operation.
    pub source: &'a str,
}

/// Borrowed reorder operation input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageReorderDraft<'a> {
    /// Queue item identity to move.
    pub item_id: &'a str,
    /// Identity to insert before; absent means move to the back.
    pub before_item_id: Option<&'a str>,
    /// Active-work identity expected by the caller.
    pub active_work_id: &'a str,
    /// Source identity attempting the mutation.
    pub source: &'a str,
}

/// Borrowed dispatch acknowledgement input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchAcknowledgementDraft<'a> {
    /// Acknowledged queue item identity.
    pub item_id: &'a str,
    /// Active-work identity expected by the caller.
    pub active_work_id: &'a str,
    /// Dispatch result.
    pub outcome: MessageDispatchAcknowledgementOutcome,
}

/// Queue reducer operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageQueueOperation<'a> {
    /// Append a queued message.
    Append(QueuedMessageDraft<'a>),
    /// Edit a queued message.
    Edit(MessageEditDraft<'a>),
    /// Reorder a queued message.
    Reorder(MessageReorderDraft<'a>),
    /// Delete a queued message.
    Delete(MessageItemOperationDraft<'a>),
    /// Clear messages from one source.
    Clear(MessageScopeOperationDraft<'a>),
    /// Start dispatch for the next queued message.
    DispatchNext(MessageScopeOperationDraft<'a>),
    /// Acknowledge pending dispatch.
    Acknowledge(DispatchAcknowledgementDraft<'a>),
    /// Cancel one pending queued message.
    CancelPending(MessageItemOperationDraft<'a>),
}

impl MessageQueueOperation<'_> {
    /// Returns the operation kind.
    #[must_use]
    pub const fn kind(&self) -> MessageQueueOperationKind {
        match self {
            Self::Append(_) => MessageQueueOperationKind::Append,
            Self::Edit(_) => MessageQueueOperationKind::Edit,
            Self::Reorder(_) => MessageQueueOperationKind::Reorder,
            Self::Delete(_) => MessageQueueOperationKind::Delete,
            Self::Clear(_) => MessageQueueOperationKind::Clear,
            Self::DispatchNext(_) => MessageQueueOperationKind::DispatchNext,
            Self::Acknowledge(_) => MessageQueueOperationKind::Acknowledge,
            Self::CancelPending(_) => MessageQueueOperationKind::CancelPending,
        }
    }
}

/// Metadata for one dispatch attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageDispatch {
    item_id: String,
    item_hash: String,
    source: String,
}

impl MessageDispatch {
    pub(crate) fn from_message(message: &QueuedMessage) -> Self {
        Self {
            item_id: String::from(message.item_id()),
            item_hash: super::digest::blake3_message_queue_hash(&[
                super::MESSAGE_QUEUE_SCHEMA,
                message.item_id(),
                message.body_hash(),
            ]),
            source: String::from(message.source()),
        }
    }

    /// Returns the queue item identity.
    #[must_use]
    pub fn item_id(&self) -> &str {
        self.item_id.as_str()
    }

    /// Returns the queue item hash.
    #[must_use]
    pub fn item_hash(&self) -> &str {
        self.item_hash.as_str()
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }
}

/// Safe event metadata for one queue item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueItemEvent {
    item_id: String,
    item_hash: String,
    source: String,
}

impl QueueItemEvent {
    pub(crate) fn from_message(message: &QueuedMessage) -> Self {
        Self {
            item_id: String::from(message.item_id()),
            item_hash: super::digest::blake3_message_queue_hash(&[
                super::MESSAGE_QUEUE_SCHEMA,
                message.item_id(),
                message.body_hash(),
            ]),
            source: String::from(message.source()),
        }
    }

    /// Returns the item identity.
    #[must_use]
    pub fn item_id(&self) -> &str {
        self.item_id.as_str()
    }

    /// Returns the item hash.
    #[must_use]
    pub fn item_hash(&self) -> &str {
        self.item_hash.as_str()
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }
}

/// User-control or queue event emitted by pure reducers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserControlEvent {
    /// A message was admitted to the queue.
    QueueAdmitted(QueueItemEvent),
    /// A queued message was edited.
    QueueEdited(QueueItemEvent),
    /// A queued message was reordered.
    QueueReordered(QueueItemEvent),
    /// A queued message was deleted.
    QueueDeleted(QueueItemEvent),
    /// Messages owned by one source were cleared.
    QueueCleared {
        /// Source whose queued messages were cleared.
        source: String,
        /// Number of queued messages removed.
        removed_count: u32,
    },
    /// A queued message is ready for host-owned dispatch.
    DispatchReady(MessageDispatch),
    /// A dispatch acknowledgement was applied.
    DispatchAcknowledged {
        /// Acknowledged item identity.
        item_id: String,
        /// Dispatch outcome.
        outcome: MessageDispatchAcknowledgementOutcome,
    },
    /// A pending message was cancelled before dispatch.
    PendingCancelled(QueueItemEvent),
    /// User requested pause.
    Pause(PauseRequest),
    /// User requested cancellation.
    Cancellation(CancellationRequest),
    /// User requested redirection.
    Redirection(RedirectionPayload),
    /// User requested queueing through interrupt controls.
    QueueInterrupt(InterruptQueueAdmission),
    /// User requested immediate host-owned prompt dispatch.
    ImmediateDispatch(ImmediateDispatchRequest),
}

impl UserControlEvent {
    /// Returns the stable decision class used by receipts.
    #[must_use]
    pub const fn decision_class(&self) -> &'static str {
        match self {
            Self::QueueAdmitted(_) => "queue_admitted",
            Self::QueueEdited(_) => "queue_edited",
            Self::QueueReordered(_) => "queue_reordered",
            Self::QueueDeleted(_) => "queue_deleted",
            Self::QueueCleared { .. } => "queue_cleared",
            Self::DispatchReady(_) => "dispatch_ready",
            Self::DispatchAcknowledged { .. } => "dispatch_acknowledged",
            Self::PendingCancelled(_) => "pending_cancelled",
            Self::Pause(_) => "pause_requested",
            Self::Cancellation(_) => "cancellation_requested",
            Self::Redirection(_) => "redirection_requested",
            Self::QueueInterrupt(_) => "interrupt_queued",
            Self::ImmediateDispatch(_) => "immediate_dispatch_requested",
        }
    }
}

/// Queue transition output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageQueueTransition {
    /// Updated queue state.
    pub state: MessageQueueState,
    /// Safe event describing the transition.
    pub event: UserControlEvent,
}

/// Alias retained for consumers that name queue-only events.
pub type MessageQueueEvent = UserControlEvent;

/// Active work metadata supplied by a host shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveWorkReference {
    work_id: String,
    status: ActiveWorkStatus,
    cancellation_handle: CancellationHandleClass,
}

impl ActiveWorkReference {
    /// Validates active work metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::MessageQueueError`] when the work identity is unsafe.
    pub fn new(
        work_id: &str,
        status: ActiveWorkStatus,
        cancellation_handle: CancellationHandleClass,
    ) -> Result<Self, super::MessageQueueError> {
        Ok(Self {
            work_id: normalize_safe_metadata(super::FIELD_ACTIVE_WORK_ID, work_id)?,
            status,
            cancellation_handle,
        })
    }

    /// Returns the active-work identity.
    #[must_use]
    pub fn work_id(&self) -> &str {
        self.work_id.as_str()
    }

    /// Returns the active-work status.
    #[must_use]
    pub const fn status(&self) -> ActiveWorkStatus {
        self.status
    }

    /// Returns cancellation-handle availability.
    #[must_use]
    pub const fn cancellation_handle(&self) -> CancellationHandleClass {
        self.cancellation_handle
    }
}

/// Borrowed redirection payload input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedirectionPayloadDraft<'a> {
    /// Active-work identity.
    pub active_work_id: &'a str,
    /// Source identity requesting redirection.
    pub source: &'a str,
    /// Hash of the redirection target.
    pub target_hash: &'a str,
    /// Hash of the redirection instruction.
    pub instruction_hash: &'a str,
}

/// Safe redirection payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedirectionPayload {
    active_work_id: String,
    source: String,
    target_hash: String,
    instruction_hash: String,
}

impl RedirectionPayload {
    /// Validates redirection metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::MessageQueueError`] when identities or hashes are unsafe.
    pub fn new(draft: RedirectionPayloadDraft<'_>) -> Result<Self, super::MessageQueueError> {
        Ok(Self {
            active_work_id: normalize_safe_metadata(super::FIELD_ACTIVE_WORK_ID, draft.active_work_id)?,
            source: normalize_safe_metadata(super::FIELD_SOURCE, draft.source)?,
            target_hash: validate_hash_metadata(super::FIELD_REDIRECTION_HASH, draft.target_hash)?,
            instruction_hash: validate_hash_metadata(super::FIELD_REDIRECTION_HASH, draft.instruction_hash)?,
        })
    }

    /// Returns the active-work identity.
    #[must_use]
    pub fn active_work_id(&self) -> &str {
        self.active_work_id.as_str()
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the target hash.
    #[must_use]
    pub fn target_hash(&self) -> &str {
        self.target_hash.as_str()
    }

    /// Returns the instruction hash.
    #[must_use]
    pub fn instruction_hash(&self) -> &str {
        self.instruction_hash.as_str()
    }
}

/// Safe cancellation request event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancellationRequest {
    active_work_id: String,
    source: String,
    cancellation_handle: CancellationHandleClass,
}

impl CancellationRequest {
    pub(crate) fn new(active_work_id: &str, source: &str) -> Result<Self, super::MessageQueueError> {
        Ok(Self {
            active_work_id: normalize_safe_metadata(super::FIELD_ACTIVE_WORK_ID, active_work_id)?,
            source: normalize_safe_metadata(super::FIELD_SOURCE, source)?,
            cancellation_handle: CancellationHandleClass::Available,
        })
    }

    /// Returns the active-work identity.
    #[must_use]
    pub fn active_work_id(&self) -> &str {
        self.active_work_id.as_str()
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the cancellation handle class.
    #[must_use]
    pub const fn cancellation_handle(&self) -> CancellationHandleClass {
        self.cancellation_handle
    }
}

/// Safe pause request event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PauseRequest {
    active_work_id: String,
    source: String,
}

impl PauseRequest {
    pub(crate) fn new(active_work_id: &str, source: &str) -> Result<Self, super::MessageQueueError> {
        Ok(Self {
            active_work_id: normalize_safe_metadata(super::FIELD_ACTIVE_WORK_ID, active_work_id)?,
            source: normalize_safe_metadata(super::FIELD_SOURCE, source)?,
        })
    }

    /// Returns the active-work identity.
    #[must_use]
    pub fn active_work_id(&self) -> &str {
        self.active_work_id.as_str()
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }
}

/// Interrupt queue-admission event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptQueueAdmission {
    active_work_id: String,
    source: String,
    message_hash: String,
}

impl InterruptQueueAdmission {
    pub(crate) fn new(
        active_work_id: &str,
        source: &str,
        message_hash: &str,
    ) -> Result<Self, super::MessageQueueError> {
        Ok(Self {
            active_work_id: normalize_safe_metadata(super::FIELD_ACTIVE_WORK_ID, active_work_id)?,
            source: normalize_safe_metadata(super::FIELD_SOURCE, source)?,
            message_hash: validate_hash_metadata(super::FIELD_BODY_HASH, message_hash)?,
        })
    }

    /// Returns the active-work identity.
    #[must_use]
    pub fn active_work_id(&self) -> &str {
        self.active_work_id.as_str()
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the message hash.
    #[must_use]
    pub fn message_hash(&self) -> &str {
        self.message_hash.as_str()
    }
}

/// Immediate dispatch request event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImmediateDispatchRequest {
    active_work_id: String,
    source: String,
    message_hash: String,
}

impl ImmediateDispatchRequest {
    pub(crate) fn new(
        active_work_id: &str,
        source: &str,
        message_hash: &str,
    ) -> Result<Self, super::MessageQueueError> {
        Ok(Self {
            active_work_id: normalize_safe_metadata(super::FIELD_ACTIVE_WORK_ID, active_work_id)?,
            source: normalize_safe_metadata(super::FIELD_SOURCE, source)?,
            message_hash: validate_hash_metadata(super::FIELD_BODY_HASH, message_hash)?,
        })
    }

    /// Returns the active-work identity.
    #[must_use]
    pub fn active_work_id(&self) -> &str {
        self.active_work_id.as_str()
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the message hash.
    #[must_use]
    pub fn message_hash(&self) -> &str {
        self.message_hash.as_str()
    }
}

/// Borrowed interrupt signal input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterruptSignalDraft<'a> {
    /// Active-work identity expected by the caller.
    pub active_work_id: &'a str,
    /// Source identity requesting control.
    pub source: &'a str,
    /// Hash of the submitted text, not raw text.
    pub message_hash: &'a str,
    /// Explicit send mode selected by host UI or SDK.
    pub send_mode: InterruptSendMode,
    /// Optional policy-owned keyword class for heuristic mode.
    pub keyword: Option<InterruptKeyword>,
    /// Optional redirection payload for redirect mode.
    pub redirection: Option<RedirectionPayloadDraft<'a>>,
}

pub(crate) fn normalize_safe_metadata(field: &'static str, value: &str) -> Result<String, super::MessageQueueError> {
    debug_assert!(!field.is_empty());
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::EmptyField,
            field,
            "field must not be empty",
        ));
    }
    if trimmed.len() > super::MAX_SAFE_METADATA_CHARS {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::FieldTooLong,
            field,
            "field exceeded safe metadata bound",
        ));
    }
    if let Some(marker) = leak_marker(trimmed) {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::UnsafeField,
            field,
            format!("unsafe metadata marker: {marker}"),
        ));
    }
    Ok(String::from(trimmed))
}

pub(crate) fn validate_hash_metadata(field: &'static str, value: &str) -> Result<String, super::MessageQueueError> {
    let trimmed = value.trim();
    if trimmed == super::EMPTY_HASH_MARKER {
        return Ok(String::from(trimmed));
    }
    if trimmed.len() != super::HASH_HEX_LENGTH || !trimmed.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::InvalidHash,
            field,
            "hash must be lowercase or uppercase hex with BLAKE3 length",
        ));
    }
    Ok(String::from(trimmed))
}

pub(crate) fn leak_marker(value: &str) -> Option<&'static str> {
    let lowered = value.to_ascii_lowercase();
    super::LEAK_MARKERS
        .iter()
        .find(|marker| lowered.contains(marker.needle))
        .map(|marker| marker.marker)
}

fn validate_policy_bounds(max_items: u32, allowed_sources: &[&str]) -> Result<(), super::MessageQueueError> {
    if max_items < super::MIN_MAX_QUEUE_ITEMS {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::InvalidPolicy,
            "max_items",
            "max queue items must be positive",
        ));
    }
    if allowed_sources.len() < super::MIN_POLICY_SOURCE_COUNT {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::InvalidPolicy,
            super::FIELD_SOURCE,
            "at least one source must be admitted",
        ));
    }
    if allowed_sources.len() > super::MAX_POLICY_SOURCE_COUNT {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::InvalidPolicy,
            super::FIELD_SOURCE,
            "too many source labels in policy",
        ));
    }
    assert!(max_items >= super::MIN_MAX_QUEUE_ITEMS, "validated queue policy must admit a positive item bound");
    assert!(
        (super::MIN_POLICY_SOURCE_COUNT..=super::MAX_POLICY_SOURCE_COUNT).contains(&allowed_sources.len()),
        "validated queue policy source count must stay bounded"
    );
    Ok(())
}

fn validate_message_admission(
    safety: MessageSafetyClass,
    budget: MessageBudgetClass,
) -> Result<(), super::MessageQueueError> {
    if safety == MessageSafetyClass::Unsafe {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::UnsafeQueuedBody,
            super::FIELD_BODY_HASH,
            "queued body was denied by safety policy",
        ));
    }
    if budget == MessageBudgetClass::OverBudget {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::BudgetExceeded,
            super::FIELD_BUDGET_CLASS,
            "queued body exceeded budget policy",
        ));
    }
    Ok(())
}
