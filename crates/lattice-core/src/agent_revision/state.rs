//! `lattice.agent-revision-lifecycle.v1`: the durable facts of one agent
//! definition. Each fact is kept separately; none is derived from another.

// r[impl lattice.agent_revisions.facts]

pub const LIFECYCLE_SCHEMA: &str = "lattice.agent-revision-lifecycle.v1";

type Map<T> = std::collections::BTreeMap<String, T>;
type Set = std::collections::BTreeSet<String>;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lifecycle {
    pub schema: String,
    /// `blake3:` definition identity; never changes across revisions.
    pub definition: String,
    /// Increments each time the desired revision changes.
    pub change_generation: u64,
    pub desired: Option<Desired>,
    /// Every revision ever desired, keyed by bare-hex revision.
    pub candidates: Map<Candidate>,
    /// Qualification admitted per `(revision, task family)` scope.
    pub qualifications: Vec<Qualification>,
    /// The most recent revision prepared for its then-current desired
    /// generation and not revoked.
    pub last_usable: Option<String>,
    /// Increments each time the selection for new sessions changes.
    pub rollout_generation: u64,
    pub selected: Option<Selection>,
    /// Revisions that have been selected; rollback targets only these.
    pub previously_selected: Set,
    /// Sessions keyed by session identifier.
    pub sessions: Map<Pin>,
    /// Applied promotion and rollback requests keyed by request identifier.
    pub receipts: Map<Selection>,
    pub revoked: Revoked,
}

impl Lifecycle {
    /// A definition with no desired, prepared or selected revision.
    #[must_use]
    pub fn new(definition: &str) -> Self {
        let lifecycle = Self {
            schema: LIFECYCLE_SCHEMA.to_owned(),
            definition: definition.to_owned(),
            change_generation: 0,
            desired: None,
            candidates: Map::new(),
            qualifications: Vec::new(),
            last_usable: None,
            rollout_generation: 0,
            selected: None,
            previously_selected: Set::new(),
            sessions: Map::new(),
            receipts: Map::new(),
            revoked: Revoked {
                revisions: Set::new(),
                qualifications: Set::new(),
                sessions: Set::new(),
            },
        };
        debug_assert!(lifecycle.candidates.is_empty());
        debug_assert_eq!(lifecycle.schema, LIFECYCLE_SCHEMA);
        lifecycle
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Desired {
    pub revision: String,
    pub generation: u64,
}

/// One revision: its resolved bundle, preparation and retained resources.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub bundle: super::facts::Bundle,
    pub bundle_identity: String,
    pub preparation: Preparation,
    /// Resources the preparation owner reported occupied; released only by
    /// an observed cleanup.
    pub artifacts: Vec<String>,
    pub cleanup: CleanupStatus,
    /// Checkpoints and recovery evidence that keep the artifacts.
    pub holds: Vec<Hold>,
}

/// Preparation of one revision, bound to the desired generation it served.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Preparation {
    Unprepared {},
    Pending {
        generation: u64,
    },
    Prepared {
        generation: u64,
    },
    /// Prepared for a generation that was no longer current; retained for
    /// cleanup only and never promotable.
    Superseded {
        generation: u64,
    },
    Failed {
        generation: u64,
    },
    Unknown {
        generation: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupStatus {
    NotRequested,
    /// The last cleanup failed or has no known outcome; resources stay occupied.
    Uncertain,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hold {
    pub kind: HoldKind,
    pub reference: String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum HoldKind {
    Checkpoint,
    RecoveryEvidence,
}

/// A qualification admitted for one revision and task-family scope.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Qualification {
    pub revision: String,
    pub task_family: String,
    pub attachment: super::ports::Attachment,
}

/// The promotion policy: the scope and the class it requires.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub task_family: String,
    pub required_class: super::ports::EvidenceClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionKind {
    Promotion,
    Rollback,
}

/// The revision selected for new sessions, and how it was selected.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub kind: SelectionKind,
    pub request_id: String,
    pub revision: String,
    pub qualification_ref: String,
    pub policy: Policy,
    pub policy_identity: String,
    pub rollout_generation: u64,
}

/// The revision one session was admitted under.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub revision: String,
    pub qualification_ref: String,
    pub rollout_generation: u64,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Live,
    Closed,
}

/// Current revocations. Revocation is monotonic.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revoked {
    pub revisions: Set,
    pub qualifications: Set,
    pub sessions: Set,
}
