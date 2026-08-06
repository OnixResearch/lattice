/// Checked, deterministically ordered simulation identity registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationIdentityRegistry {
    entries: Vec<SimulationIdentityEntry>,
    current: std::collections::BTreeSet<(SimulationIdentityKind, String)>,
    aliases: std::collections::BTreeMap<(SimulationIdentityKind, String), String>,
}

impl SimulationIdentityRegistry {
    /// Validates and constructs a registry.
    ///
    /// # Errors
    ///
    /// Returns every bounded duplicate, alias, shape, and SDK projection issue.
    pub fn new(
        mut entries: Vec<SimulationIdentityEntry>,
    ) -> Result<Self, Vec<SimulationIdentityDiagnostic>> {
        entries.sort_by(|left, right| (left.kind, &left.id).cmp(&(right.kind, &right.id)));
        if entries.len() > MAX_IDENTITIES {
            return Err(vec![diagnostic(
                SimulationIdentityDiagnosticCode::RegistryTooLarge,
                None,
                None,
            )]);
        }
        let mut issues = IdentityIssues::default();

        let mut current = std::collections::BTreeSet::new();
        let mut aliases = std::collections::BTreeMap::new();
        let mut sdk_ids =
            std::collections::BTreeMap::<u32, (SimulationIdentityKind, String)>::new();
        for entry in &entries {
            validate_entry(entry, &mut issues);
            let key = (entry.kind, entry.id.clone());
            if current.len() < MAX_IDENTITIES && !current.insert(key.clone()) {
                issues.push(diagnostic(
                    SimulationIdentityDiagnosticCode::DuplicateIdentity,
                    Some(entry.kind),
                    Some(&entry.id),
                ));
            }
            let projected = project_simulation_id(&entry.id);
            if let Some((other_kind, other_id)) = sdk_ids.insert(projected, key.clone())
                && (other_kind != entry.kind || other_id != entry.id)
            {
                issues.push(diagnostic(
                    SimulationIdentityDiagnosticCode::SdkProjectionCollision,
                    Some(entry.kind),
                    Some(&entry.id),
                ));
            }
            for alias in &entry.aliases {
                let alias_key = (entry.kind, alias.clone());
                if aliases.len() < MAX_IDENTITIES.saturating_mul(MAX_ALIASES_PER_IDENTITY)
                    && aliases.insert(alias_key, entry.id.clone()).is_some()
                {
                    issues.push(diagnostic(
                        SimulationIdentityDiagnosticCode::ConflictingAlias,
                        Some(entry.kind),
                        Some(alias),
                    ));
                }
            }
        }

        for (kind, alias) in aliases.keys() {
            if current.contains(&(*kind, alias.clone())) {
                issues.push(diagnostic(
                    SimulationIdentityDiagnosticCode::ConflictingAlias,
                    Some(*kind),
                    Some(alias),
                ));
            }
        }

        if issues.0.is_empty() {
            Ok(Self {
                entries,
                current,
                aliases,
            })
        } else {
            Err(issues.0)
        }
    }

    /// Returns entries in deterministic namespace/identity order.
    #[must_use]
    pub fn entries(&self) -> &[SimulationIdentityEntry] {
        &self.entries
    }

    /// Returns whether a current identity exists in the requested namespace.
    #[must_use]
    pub fn contains(&self, kind: SimulationIdentityKind, id: &str) -> bool {
        self.current.contains(&(kind, id.to_string()))
    }

    /// Resolves a current identity or reviewed alias to its current identity.
    #[must_use]
    pub fn resolve<'a>(&'a self, kind: SimulationIdentityKind, id: &'a str) -> Option<&'a str> {
        if self.contains(kind, id) {
            Some(id)
        } else {
            self.aliases
                .get(&(kind, id.to_string()))
                .map(String::as_str)
        }
    }

    /// Requires a current identity and returns a stable issue when it is absent.
    ///
    /// # Errors
    ///
    /// Returns `UnknownIdentity` for aliases and unregistered IDs because emitted
    /// evidence always uses the current canonical identity.
    pub fn require(
        &self,
        kind: SimulationIdentityKind,
        id: &str,
    ) -> Result<(), SimulationIdentityDiagnostic> {
        if self.contains(kind, id) {
            Ok(())
        } else {
            Err(diagnostic(
                SimulationIdentityDiagnosticCode::UnknownIdentity,
                Some(kind),
                Some(id),
            ))
        }
    }
}

/// Projects a semantic identity to the reviewed `ChaosControl` `u32` space.
///
/// The FNV-1a algorithm matches the public SDK identity convention, while the
/// registry rejects collisions before bridge evidence is emitted.
#[must_use]
pub const fn project_simulation_id(identity: &str) -> u32 {
    let bytes = identity.as_bytes();
    let mut hash = 0x811c_9dc5_u32;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u32;
        hash = hash.wrapping_mul(0x0100_0193);
        index += 1;
    }
    hash
}

#[derive(Default)]
struct IdentityIssues(Vec<SimulationIdentityDiagnostic>);

impl IdentityIssues {
    fn push(&mut self, issue: SimulationIdentityDiagnostic) {
        if self.0.len() < MAX_IDENTITY_ISSUES {
            self.0.push(issue);
        }
    }
}

fn validate_entry(entry: &SimulationIdentityEntry, issues: &mut IdentityIssues) {
    debug_assert!(entry.aliases.len() <= entry.aliases.capacity());
    debug_assert!(issues.0.len() <= MAX_IDENTITY_ISSUES);
    if !valid_identity(&entry.id) {
        issues.push(diagnostic(
            SimulationIdentityDiagnosticCode::InvalidIdentity,
            Some(entry.kind),
            Some(&entry.id),
        ));
    }
    if source_location_identity(&entry.id) {
        issues.push(diagnostic(
            SimulationIdentityDiagnosticCode::SourceLocationIdentity,
            Some(entry.kind),
            Some(&entry.id),
        ));
    }
    if entry.aliases.len() > MAX_ALIASES_PER_IDENTITY {
        issues.push(diagnostic(
            SimulationIdentityDiagnosticCode::TooManyAliases,
            Some(entry.kind),
            Some(&entry.id),
        ));
    }
    let mut local = std::collections::BTreeSet::new();
    for alias in entry.aliases.iter().take(MAX_ALIASES_PER_IDENTITY) {
        if !valid_identity(alias) || alias == &entry.id {
            issues.push(diagnostic(
                SimulationIdentityDiagnosticCode::InvalidAlias,
                Some(entry.kind),
                Some(alias),
            ));
        }
        if !local.insert(alias) {
            issues.push(diagnostic(
                SimulationIdentityDiagnosticCode::ConflictingAlias,
                Some(entry.kind),
                Some(alias),
            ));
        }
    }
}

fn valid_identity(identity: &str) -> bool {
    !identity.is_empty()
        && identity.len() <= MAX_ID_BYTES
        && identity.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn source_location_identity(identity: &str) -> bool {
    identity.contains(".rs") || identity.contains('/') || identity.contains('\\')
}

fn diagnostic(
    code: SimulationIdentityDiagnosticCode,
    kind: Option<SimulationIdentityKind>,
    identity: Option<&str>,
) -> SimulationIdentityDiagnostic {
    SimulationIdentityDiagnostic {
        code,
        kind,
        identity: identity.map(|value| value.chars().take(MAX_ID_BYTES).collect()),
    }
}
