//! Typed collection projection inputs and outputs.

/// Maximum identities accepted by one collection snapshot.
pub const MAX_COLLECTION_ITEMS: usize = 1_000_000;
/// Maximum explicit measurements accepted by one request.
pub const MAX_MEASUREMENTS: usize = 100_000;
/// Maximum overscan in the request coordinate space.
pub const MAX_OVERSCAN: f64 = 1_000_000.0;

/// Monotonic identity of an immutable ordered collection snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CollectionGeneration(u64);

impl CollectionGeneration {
    /// Creates a non-zero generation.
    pub fn new(value: u64) -> Result<Self, CollectionDiagnostic> {
        (value > 0)
            .then_some(Self(value))
            .ok_or_else(|| CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidGeneration, "snapshot"))
    }

    /// Returns the underlying generation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

/// Positive finite extent in renderer-normalized logical units.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LogicalExtent(f64);

impl LogicalExtent {
    /// Validates a positive finite extent.
    pub fn new(value: f64) -> Result<Self, CollectionDiagnostic> {
        (value.is_finite() && value > 0.0)
            .then_some(Self(value))
            .ok_or_else(|| CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidExtent, "extent"))
    }

    /// Returns the checked value.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// Immutable ordered stable identities owned by one domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSnapshot<Id> {
    /// Monotonic collection generation.
    pub generation: CollectionGeneration,
    /// Presentation order supplied by the domain owner.
    pub items: Vec<Id>,
}

/// Generation-checked snapshot mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotChange<Id> {
    /// Replace the complete order.
    Reset(Vec<Id>),
    /// Insert an identity at an index.
    Insert {
        /// Resulting insertion index.
        index: usize,
        /// Stable identity to insert.
        id: Id,
    },
    /// Remove a known identity.
    Remove {
        /// Stable identity to remove.
        id: Id,
    },
    /// Move a known identity to the resulting index.
    Move {
        /// Stable identity to move.
        id: Id,
        /// Resulting presentation index.
        to: usize,
    },
}

/// One accepted renderer measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemMeasurement<Id> {
    /// Stable domain identity.
    pub id: Id,
    /// Measured item extent.
    pub extent: LogicalExtent,
}

/// Anchor evidence supplied from the previous accepted plan.
#[derive(Debug, Clone, PartialEq)]
pub struct AnchorInput<Id> {
    /// Identity whose visual reference should survive remeasurement.
    pub id: Id,
    /// Previous logical item offset.
    pub previous_item_offset: f64,
    /// Renderer-owned reference position inside the viewport.
    pub viewport_reference: f64,
}

/// Scroll-to-item alignment policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAlignment {
    /// Place the item at the viewport start.
    Start,
    /// Center the item.
    Center,
    /// Place the item at the viewport end.
    End,
    /// Move only when the item is outside the viewport.
    Nearest,
}

/// Pure materialization request assembled by a renderer adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterializationRequest<Id> {
    /// Collection generation expected by measurements and state.
    pub collection_generation: CollectionGeneration,
    /// Monotonic renderer-owned measurement generation.
    pub measurement_generation: u64,
    /// Logical viewport start.
    pub viewport_start: f64,
    /// Positive viewport extent.
    pub viewport_extent: LogicalExtent,
    /// Optional extent shared by every item.
    pub uniform_extent: Option<LogicalExtent>,
    /// Explicit measured extents, ignored only when uniform extent exists.
    pub measurements: Vec<ItemMeasurement<Id>>,
    /// Positive estimate for unmeasured variable items.
    pub estimated_extent: LogicalExtent,
    /// Non-negative bounded materialization overscan.
    pub overscan: f64,
    /// Optional prior anchor evidence.
    pub anchor: Option<AnchorInput<Id>>,
    /// Optional scroll target and alignment.
    pub scroll_to: Option<(Id, ScrollAlignment)>,
}

/// One item requested for renderer materialization.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterializedItem<Id> {
    /// Stable identity.
    pub id: Id,
    /// Current presentation index.
    pub index: usize,
    /// Logical offset from collection start.
    pub offset: f64,
    /// Measured or estimated extent.
    pub extent: f64,
    /// Whether the extent came from accepted measurement evidence.
    pub measured: bool,
    /// Whether this item intersects the non-overscanned viewport.
    pub visible: bool,
}

/// Anchor result and renderer-applicable correction.
#[derive(Debug, Clone, PartialEq)]
pub struct AnchorProjection<Id> {
    /// Preserved anchor identity.
    pub id: Id,
    /// Current logical item offset.
    pub item_offset: f64,
    /// Preserved viewport reference.
    pub viewport_reference: f64,
    /// Delta from the previous item offset.
    pub scroll_correction: f64,
}

/// Pure request to scroll a renderer-owned viewport.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollRequest<Id> {
    /// Requested identity.
    pub id: Id,
    /// Target logical viewport start.
    pub viewport_start: f64,
    /// Requested alignment.
    pub alignment: ScrollAlignment,
}

/// Successful pure materialization projection.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterializationPlan<Id> {
    /// Collection generation consumed.
    pub collection_generation: CollectionGeneration,
    /// Measurement generation consumed.
    pub measurement_generation: u64,
    /// Ordered visible identities.
    pub visible: Vec<Id>,
    /// Ordered items in the bounded overscan window.
    pub materialized: Vec<MaterializedItem<Id>>,
    /// Extent preceding the materialized window.
    pub before_extent: f64,
    /// Extent following the materialized window.
    pub after_extent: f64,
    /// Estimated complete collection extent.
    pub total_extent: f64,
    /// Optional accepted anchor correction.
    pub anchor: Option<AnchorProjection<Id>>,
    /// Optional renderer-applicable scroll request.
    pub scroll_request: Option<ScrollRequest<Id>>,
}

/// Stable validation and repair diagnostic classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CollectionDiagnosticCode {
    /// Generation is zero, stale, skipped, or overflowing.
    InvalidGeneration,
    /// Ordered identities contain a duplicate.
    DuplicateIdentity,
    /// A mutation or measurement references an unknown identity.
    UnknownIdentity,
    /// A mutation index is outside accepted bounds.
    InvalidIndex,
    /// A numeric extent is zero, negative, non-finite, or overflowing.
    InvalidExtent,
    /// Overscan is negative, non-finite, or above policy.
    InvalidOverscan,
    /// A request exceeds item or measurement limits.
    LimitExceeded,
    /// Measurements duplicate an identity.
    DuplicateMeasurement,
    /// Semantic collection metadata or support reason is malformed.
    InvalidSemantics,
    /// Active/anchor identity was repaired or removed.
    IdentityRepaired,
    /// Requested scroll target is absent.
    MissingScrollTarget,
    /// Projection evidence does not match current accepted inputs.
    StaleEvidence,
    /// Exact-window parity was requested across unlike coordinate families.
    IncompatibleCoordinateParity,
}

/// Deterministic collection diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionDiagnostic {
    /// Stable diagnostic class.
    pub code: CollectionDiagnosticCode,
    /// Stable identity/index subject.
    pub subject: String,
}

impl CollectionDiagnostic {
    pub(crate) fn new(code: CollectionDiagnosticCode, subject: impl Into<String>) -> Self {
        Self {
            code,
            subject: subject.into(),
        }
    }
}
