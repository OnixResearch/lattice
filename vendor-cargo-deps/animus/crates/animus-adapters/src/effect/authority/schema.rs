const CONTRACT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("capability_contract");
const RESOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("capability_resource");
const RESOURCE_PREFIX_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("capability_resource_prefix");
const ABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("capability_ability");
pub(super) const HASH_SEPARATOR: &str = "|";
pub(super) const HASH_SCHEMA: &str = "animus.clash-ucan-authority.v1";

/// Typed capability request that must be authorized before guarded dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityRequest {
    action: super::GuardedAction,
    contract: alloc::string::String,
    resource: alloc::string::String,
    ability: alloc::string::String,
}

impl CapabilityRequest {
    /// Builds a normalized guarded-action capability request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when contract, resource, or ability
    /// is empty, too long, or contains a NUL character.
    pub fn new(
        action: super::GuardedAction,
        contract: &str,
        resource: &str,
        ability: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            action,
            contract: crate::validation::normalized_adapter_field(CONTRACT_FIELD, contract)?,
            resource: crate::validation::normalized_adapter_field(RESOURCE_FIELD, resource)?,
            ability: crate::validation::normalized_adapter_field(ABILITY_FIELD, ability)?,
        })
    }

    /// Returns the guarded action class.
    #[must_use]
    pub const fn action(&self) -> super::GuardedAction {
        self.action
    }

    /// Returns the contract identifier.
    #[must_use]
    pub fn contract(&self) -> &str {
        self.contract.as_str()
    }

    /// Returns the requested resource identifier.
    #[must_use]
    pub fn resource(&self) -> &str {
        self.resource.as_str()
    }

    /// Returns the requested ability label.
    #[must_use]
    pub fn ability(&self) -> &str {
        self.ability.as_str()
    }
}

/// Nickel-reviewed capability contract fixture consumed by Rust without runtime Nickel evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityPolicyContract {
    action: super::GuardedAction,
    contract: alloc::string::String,
    resource_prefix: alloc::string::String,
    ability: alloc::string::String,
    policy_hash: alloc::string::String,
}

impl CapabilityPolicyContract {
    /// Builds a checked capability policy contract fixture.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when contract, resource prefix, or
    /// ability is empty, too long, or contains a NUL character.
    pub fn new(
        action: super::GuardedAction,
        contract: &str,
        resource_prefix: &str,
        ability: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        let contract = crate::validation::normalized_adapter_field(CONTRACT_FIELD, contract)?;
        let resource_prefix =
            crate::validation::normalized_adapter_field(RESOURCE_PREFIX_FIELD, resource_prefix)?;
        let ability = crate::validation::normalized_adapter_field(ABILITY_FIELD, ability)?;
        let policy_hash = hash_parts(&[
            HASH_SCHEMA,
            action.as_str(),
            contract.as_str(),
            resource_prefix.as_str(),
            ability.as_str(),
        ]);

        Ok(Self {
            action,
            contract,
            resource_prefix,
            ability,
            policy_hash,
        })
    }

    /// Returns true when this contract permits the request shape.
    #[must_use]
    pub fn permits(&self, request: &CapabilityRequest) -> bool {
        self.action == request.action
            && self.contract == request.contract
            && self.ability == request.ability
            && request.resource.starts_with(self.resource_prefix.as_str())
    }

    /// Returns the contract identifier.
    #[must_use]
    pub fn contract(&self) -> &str {
        self.contract.as_str()
    }

    /// Returns the allowed ability label.
    #[must_use]
    pub fn ability(&self) -> &str {
        self.ability.as_str()
    }

    /// Returns the BLAKE3 hash of checked contract fields.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }
}

/// Verified UCAN grant fixture for a contract, resource prefix, and ability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UcanGrant {
    contract: alloc::string::String,
    resource_prefix: alloc::string::String,
    ability: alloc::string::String,
}

impl UcanGrant {
    /// Builds a checked UCAN grant fixture.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when contract, resource prefix, or
    /// ability is empty, too long, or contains a NUL character.
    pub fn new(
        contract: &str,
        resource_prefix: &str,
        ability: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            contract: crate::validation::normalized_adapter_field(CONTRACT_FIELD, contract)?,
            resource_prefix: crate::validation::normalized_adapter_field(
                RESOURCE_PREFIX_FIELD,
                resource_prefix,
            )?,
            ability: crate::validation::normalized_adapter_field(ABILITY_FIELD, ability)?,
        })
    }

    /// Returns true when this grant authorizes the request resource and ability.
    #[must_use]
    pub fn authorizes(&self, request: &CapabilityRequest) -> bool {
        self.contract == request.contract
            && self.ability == request.ability
            && request.resource.starts_with(self.resource_prefix.as_str())
    }
}

pub(super) fn hash_parts(parts: &[&str]) -> alloc::string::String {
    let mut text = alloc::string::String::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            text.push_str(HASH_SEPARATOR);
        }
        text.push_str(part);
    }
    alloc::format!("{}", blake3::hash(text.as_bytes()).to_hex())
}
