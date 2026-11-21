//! The agent definition a profile revision realizes, its Lattice #25
//! revision lifecycle slots, and what that lifecycle admits and qualifies for
//! one revision.
//!
//! A definition identity frames the profile owner, the basis and a bounded
//! name only, so a new revision keeps it. Revision slots are projected from
//! the definition's lifecycle (`agent_revision::projection`) when the shell
//! supplies one. Without one every slot is `unmanaged`; a managed slot
//! without a fact is `unknown` with the owner that can supply it.

// r[impl lattice.worker_inventory.definition]
// r[impl lattice.worker_inventory.definition.scenario.stable]
// r[impl lattice.worker_inventory.definition.scenario.basis]
// r[impl lattice.worker_inventory.revisions]
// r[impl lattice.worker_inventory.revisions.scenario.unmanaged]
// r[impl lattice.worker_inventory.revisions.scenario.managed]
// r[impl lattice.worker_inventory.revisions.scenario.pinned]

/// Whether the operator named the definition or the owner's default applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionBasis {
    Declared,
    OwnerDefault,
}

impl DefinitionBasis {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::OwnerDefault => "owner_default",
        }
    }
}

/// An agent definition, stable across the profile revisions that realize it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub identity: String,
    pub name: String,
    pub basis: DefinitionBasis,
}

/// A definition name that is not a bounded capability name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidName;

/// The declared name, or the owner's default, and its identity.
pub fn definition(
    owner: super::inventory::ProfileOwner,
    default_name: &str,
    declared_name: Option<&str>,
) -> Result<Definition, InvalidName> {
    let (name, basis) = match declared_name {
        Some(declared) => (declared, DefinitionBasis::Declared),
        None => (default_name, DefinitionBasis::OwnerDefault),
    };
    if !super::super::capability::is_name(name) {
        return Err(InvalidName);
    }
    let identity = crate::content_identity::worker_definition(crate::content_identity::WorkerDefinitionInput {
        owner: owner.as_str(),
        basis: basis.as_str(),
        name,
    })
    .map_err(|_| InvalidName)?;
    debug_assert!(super::super::is_reference(&identity));
    debug_assert!(name.len() <= super::super::capability::MAX_NAME_BYTES);
    Ok(Definition {
        identity,
        name: name.to_owned(),
        basis,
    })
}

/// One revision lifecycle slot. `unmanaged` (no store, or no lifecycle for
/// the definition) is distinct from `unknown` (managed, no fact recorded).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "support", rename_all = "snake_case", deny_unknown_fields)]
pub enum Slot<T> {
    Unmanaged {},
    Unknown { owner: String },
    Recorded { value: T },
}

impl<T> Slot<T> {
    fn unknown(owner: &str) -> Self {
        Self::Unknown {
            owner: owner.to_owned(),
        }
    }

    /// A recorded fact, or `unknown` with `owner` when there is none.
    fn recorded_or(value: Option<T>, owner: &str) -> Self {
        value.map_or_else(|| Self::unknown(owner), |value| Self::Recorded { value })
    }
}

/// The definition's revision lifecycle slots.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionSlots {
    pub desired: Slot<crate::agent_revision::state::Desired>,
    /// Sorted revisions prepared for their desired generation.
    pub prepared: Slot<Vec<String>>,
    /// Qualifications per revision and task family, with reference, class
    /// and provenance.
    pub qualified: Slot<Vec<crate::agent_revision::projection::Qualified>>,
    pub selected_for_new_sessions: Slot<crate::agent_revision::projection::Selected>,
}

impl RevisionSlots {
    /// Every slot of a definition without a lifecycle.
    #[must_use]
    pub const fn unmanaged() -> Self {
        Self {
            desired: Slot::Unmanaged {},
            prepared: Slot::Unmanaged {},
            qualified: Slot::Unmanaged {},
            selected_for_new_sessions: Slot::Unmanaged {},
        }
    }

    /// Project the slots of `lifecycle`, or `unmanaged` without one.
    #[must_use]
    pub fn project(lifecycle: Option<&crate::agent_revision::state::Lifecycle>) -> Self {
        let Some(lifecycle) = lifecycle else {
            return Self::unmanaged();
        };
        let slots = crate::agent_revision::projection::slots(lifecycle);
        fn listed<T>(values: Vec<T>) -> Option<Vec<T>> {
            (!values.is_empty()).then_some(values)
        }
        let projected = Self {
            desired: Slot::recorded_or(slots.desired, super::wait::REVISION_OWNER),
            prepared: Slot::recorded_or(listed(slots.prepared), crate::agent_revision::PREPARATION_OWNER),
            qualified: Slot::recorded_or(listed(slots.qualified), crate::agent_revision::QUALIFICATION_OWNER),
            selected_for_new_sessions: Slot::recorded_or(slots.selected_for_new_sessions, super::wait::REVISION_OWNER),
        };
        debug_assert!(projected.is_managed());
        projected
    }

    #[must_use]
    pub const fn is_managed(&self) -> bool {
        !matches!(self.desired, Slot::Unmanaged {})
    }
}

/// What an attempt recorded about its session's admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordedAdmission<'a> {
    /// No intent records the admission yet.
    Unrecorded,
    /// Admitted unmanaged, or recorded before admission modes existed.
    Unmanaged,
    Managed(&'a crate::agent_revision::projection::RecordedPin),
}

/// The pinned revision of one instance's session. An unmanaged admission
/// stays `unmanaged` even after its definition becomes managed.
#[must_use]
pub fn pinned(
    lifecycle: Option<&crate::agent_revision::state::Lifecycle>,
    session: &str,
    recorded: RecordedAdmission<'_>,
) -> Slot<crate::agent_revision::projection::SessionPin> {
    let pin = match (recorded, lifecycle) {
        (RecordedAdmission::Unmanaged, _) | (RecordedAdmission::Unrecorded, None) => return Slot::Unmanaged {},
        (RecordedAdmission::Managed(_), None) => None,
        (RecordedAdmission::Managed(pin), Some(lifecycle)) => {
            crate::agent_revision::projection::session_pin(lifecycle, session, Some(pin))
        }
        (RecordedAdmission::Unrecorded, Some(lifecycle)) => {
            crate::agent_revision::projection::session_pin(lifecycle, session, None)
        }
    };
    Slot::recorded_or(pin, super::wait::REVISION_OWNER)
}

/// Whether a managed definition admits new sessions of `revision`: it is
/// selected, and neither it nor the selection's qualification is revoked.
pub(super) fn admits(lifecycle: &crate::agent_revision::state::Lifecycle, revision: &str) -> bool {
    lifecycle.selected.as_ref().is_some_and(|selected| {
        selected.revision == revision
            && !lifecycle.revoked.revisions.contains(revision)
            && !lifecycle.revoked.qualifications.contains(&selected.qualification_ref)
    })
}

/// Current qualifications of `revision`, sorted by task family: recorded by
/// the lifecycle, with neither the revision nor the attachment revoked.
pub(super) fn capability_qualifications(
    lifecycle: Option<&crate::agent_revision::state::Lifecycle>,
    revision: &str,
) -> Vec<super::inventory::CapabilityQualification> {
    let Some(lifecycle) = lifecycle else {
        return Vec::new();
    };
    if lifecycle.revoked.revisions.contains(revision) {
        return Vec::new();
    }
    let mut qualifications = lifecycle
        .qualifications
        .iter()
        .filter(|admitted| admitted.revision == revision)
        .filter(|admitted| !lifecycle.revoked.qualifications.contains(&admitted.attachment.qualification_ref))
        .map(|admitted| super::inventory::CapabilityQualification {
            task_family: admitted.task_family.clone(),
            qualification_ref: admitted.attachment.qualification_ref.clone(),
            evidence_class: admitted.attachment.evidence_class,
            provenance: admitted.attachment.provenance,
        })
        .collect::<Vec<_>>();
    qualifications.sort_by(|left, right| left.task_family.cmp(&right.task_family));
    debug_assert!(qualifications.len() <= lifecycle.qualifications.len());
    qualifications
}
