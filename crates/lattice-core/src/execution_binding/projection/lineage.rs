//! The agent definition a profile revision realizes, and Lattice #25's
//! revision lifecycle slots.
//!
//! A definition identity frames the profile owner, the basis and a bounded
//! name only, so a new revision keeps it. The revision slots are owned
//! unknowns until Lattice #25 publishes desired, prepared, qualified, selected
//! and pinned revisions.

// r[impl lattice.worker_inventory.definition]
// r[impl lattice.worker_inventory.definition.scenario.stable]
// r[impl lattice.worker_inventory.definition.scenario.basis]
// r[impl lattice.worker_inventory.revisions]
// r[impl lattice.worker_inventory.revisions.scenario.unpublished]

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

/// Lattice #25 revision lifecycle slots; none is published yet.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionSlots {
    pub desired: super::OwnedUnknown,
    pub prepared: super::OwnedUnknown,
    pub qualified: super::OwnedUnknown,
    pub selected_for_new_sessions: super::OwnedUnknown,
    pub pinned: super::OwnedUnknown,
}

impl RevisionSlots {
    #[must_use]
    pub fn unpublished() -> Self {
        let slot = || super::OwnedUnknown::unknown(super::wait::REVISION_OWNER);
        Self {
            desired: slot(),
            prepared: slot(),
            qualified: slot(),
            selected_for_new_sessions: slot(),
            pinned: slot(),
        }
    }
}
