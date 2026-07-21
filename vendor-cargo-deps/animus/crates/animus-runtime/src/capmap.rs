const MIN_FAMILY_RECORDS: usize = 1;

const FAMILY_BATCH: &str = "batch";
const FAMILY_COMPATIBILITY: &str = "compatibility";
const FAMILY_COMPACTION: &str = "compaction";
const FAMILY_DATASET_EXPORT: &str = "dataset-export";
const FAMILY_HEADLESS: &str = "headless";
const FAMILY_LOOP: &str = "loop";
const FAMILY_POLICY: &str = "policy";
const FAMILY_PROVENANCE: &str = "provenance";
const FAMILY_QUALITY: &str = "quality";
const FAMILY_RESEARCH: &str = "research";
const FAMILY_SCAFFOLD: &str = "scaffold";
const FAMILY_STATUS: &str = "status";
const FAMILY_WORKER_TEAM: &str = "worker-team";
const FEATURE_COMPAT: &str = "compat";
const FEATURE_BATCH: &str = "batch";
const FEATURE_COMPACTION: &str = "compaction";
const FEATURE_DATASET_EXPORT: &str = "dataset-export";
const FEATURE_HEADLESS: &str = "headless";
const FEATURE_LOOP: &str = "loop";
const FEATURE_POLICY: &str = "policy";
const FEATURE_PROVENANCE: &str = "provenance";
const FEATURE_QUALITY: &str = "quality";
const FEATURE_RESEARCH: &str = "research";
const FEATURE_SCAFFOLD: &str = "scaffold";
const FEATURE_STATUS: &str = "status";
const FEATURE_WORKER_TEAM: &str = "worker-team";
const AUTHORITY_NONE: &str = "none";
const AUTHORITY_INJECTED: &str = "injected-service";

/// Runtime capability family used to select focused composition surfaces.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeCapabilityFamily {
    /// Core loop contract and transition family.
    Loop,
    /// Headless command/event protocol family.
    Headless,
    /// Provenance ledger, receipt, and projection family.
    Provenance,
    /// Batch trajectory/job family.
    Batch,
    /// Worker-team delegation family.
    WorkerTeam,
    /// Policy, middleware, hook, and guard family.
    Policy,
    /// Quality-record family.
    Quality,
    /// `ResearchSpec` lifecycle family.
    Research,
    /// Scaffold/evaluation strategy family.
    Scaffold,
    /// Prompt-pressure and compaction family.
    Compaction,
    /// Dataset export family.
    DatasetExport,
    /// Semantic status-selection family.
    Status,
    /// Compatibility bundle family.
    Compatibility,
}

impl RuntimeCapabilityFamily {
    /// Returns the stable family label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Loop => FAMILY_LOOP,
            Self::Headless => FAMILY_HEADLESS,
            Self::Provenance => FAMILY_PROVENANCE,
            Self::Batch => FAMILY_BATCH,
            Self::WorkerTeam => FAMILY_WORKER_TEAM,
            Self::Policy => FAMILY_POLICY,
            Self::Quality => FAMILY_QUALITY,
            Self::Research => FAMILY_RESEARCH,
            Self::Scaffold => FAMILY_SCAFFOLD,
            Self::Compaction => FAMILY_COMPACTION,
            Self::DatasetExport => FAMILY_DATASET_EXPORT,
            Self::Status => FAMILY_STATUS,
            Self::Compatibility => FAMILY_COMPATIBILITY,
        }
    }
}

/// Inventory record for one runtime capability family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeCapabilityFamilyRecord {
    family: RuntimeCapabilityFamily,
    feature_gate: &'static str,
    required_authority: &'static str,
    description: &'static str,
}

impl RuntimeCapabilityFamilyRecord {
    /// Builds a capability-family map record.
    #[must_use]
    pub const fn new(
        family: RuntimeCapabilityFamily,
        feature_gate: &'static str,
        required_authority: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            family,
            feature_gate,
            required_authority,
            description,
        }
    }

    /// Returns the family.
    #[must_use]
    pub const fn family(self) -> RuntimeCapabilityFamily {
        self.family
    }

    /// Returns the Cargo feature gate label.
    #[must_use]
    pub const fn feature_gate(self) -> &'static str {
        self.feature_gate
    }

    /// Returns the required authority class.
    #[must_use]
    pub const fn required_authority(self) -> &'static str {
        self.required_authority
    }

    /// Returns the safe family description.
    #[must_use]
    pub const fn description(self) -> &'static str {
        self.description
    }
}

const FAMILY_MAP: &[RuntimeCapabilityFamilyRecord] = &[
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Loop,
        FEATURE_LOOP,
        AUTHORITY_INJECTED,
        "neutral loop DTOs, transitions, fabrics, and adapter dispatch",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Headless,
        FEATURE_HEADLESS,
        AUTHORITY_NONE,
        "headless command, event, response, and thread contracts",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Provenance,
        FEATURE_PROVENANCE,
        AUTHORITY_NONE,
        "safe provenance ledger, receipt, replay, and projection contracts",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Batch,
        FEATURE_BATCH,
        AUTHORITY_INJECTED,
        "batch trajectory and job planning over host-injected execution",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::WorkerTeam,
        FEATURE_WORKER_TEAM,
        AUTHORITY_INJECTED,
        "worker delegation DTOs and host-supplied worker services",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Policy,
        FEATURE_POLICY,
        AUTHORITY_INJECTED,
        "middleware, hook, guard, and policy binding contracts",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Quality,
        FEATURE_QUALITY,
        AUTHORITY_NONE,
        "quality records, freshness, and sidecar validation contracts",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Research,
        FEATURE_RESEARCH,
        AUTHORITY_NONE,
        "ResearchSpec study, hypothesis, run, keep, and promotion contracts",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Scaffold,
        FEATURE_SCAFFOLD,
        AUTHORITY_NONE,
        "reviewed scaffold profile selection and admission contracts",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Compaction,
        FEATURE_COMPACTION,
        AUTHORITY_INJECTED,
        "prompt-pressure and compaction planning over injected summarizers",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::DatasetExport,
        FEATURE_DATASET_EXPORT,
        AUTHORITY_NONE,
        "safe chat, preference, and RL trajectory dataset export contracts",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Status,
        FEATURE_STATUS,
        AUTHORITY_NONE,
        "semantic status selection, evidence, and presentation contracts",
    ),
    RuntimeCapabilityFamilyRecord::new(
        RuntimeCapabilityFamily::Compatibility,
        FEATURE_COMPAT,
        AUTHORITY_INJECTED,
        "default broad compatibility bundle for existing embedders",
    ),
];

/// Returns the maintained runtime capability-family map.
#[must_use]
pub const fn runtime_capability_family_map() -> &'static [RuntimeCapabilityFamilyRecord] {
    FAMILY_MAP
}

/// Validates a runtime capability-family map.
///
/// # Panics
///
/// Panics when `records` contains fewer than `MIN_FAMILY_RECORDS` entries or the
/// canonical family map is empty.
///
/// # Errors
///
/// Returns [`RuntimeCapabilityFamilyMapError`] when a family is duplicated or a
/// feature/authority field is empty or unsupported.
pub fn validate_runtime_capability_family_map(
    records: &[RuntimeCapabilityFamilyRecord],
) -> Result<(), RuntimeCapabilityFamilyMapError> {
    assert!(
        records.len() >= MIN_FAMILY_RECORDS,
        "family map must contain at least one record"
    );
    assert!(
        FAMILY_MAP.len() >= MIN_FAMILY_RECORDS,
        "canonical family map must contain records"
    );
    let mut families = std::collections::BTreeSet::new();
    for record in records {
        if !families.insert(record.family()) {
            return Err(RuntimeCapabilityFamilyMapError::DuplicateFamily {
                family: record.family(),
            });
        }
        if !is_known_feature(record.feature_gate()) {
            return Err(RuntimeCapabilityFamilyMapError::UnknownFeature {
                feature: record.feature_gate(),
            });
        }
        if !is_known_authority(record.required_authority()) {
            return Err(RuntimeCapabilityFamilyMapError::UnknownAuthority {
                authority: record.required_authority(),
            });
        }
    }
    Ok(())
}

/// Runtime capability-family map validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeCapabilityFamilyMapError {
    /// A family appeared more than once.
    DuplicateFamily {
        /// Duplicated family.
        family: RuntimeCapabilityFamily,
    },
    /// A feature gate was empty or unsupported.
    UnknownFeature {
        /// Rejected feature gate.
        feature: &'static str,
    },
    /// An authority class was empty or unsupported.
    UnknownAuthority {
        /// Rejected authority class.
        authority: &'static str,
    },
}

impl core::fmt::Display for RuntimeCapabilityFamilyMapError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DuplicateFamily { family } => {
                write!(formatter, "duplicate capability family {}", family.as_str())
            }
            Self::UnknownFeature { feature } => {
                write!(formatter, "unknown capability-family feature {feature}")
            }
            Self::UnknownAuthority { authority } => {
                write!(formatter, "unknown capability-family authority {authority}")
            }
        }
    }
}

impl std::error::Error for RuntimeCapabilityFamilyMapError {}

fn is_known_feature(feature: &str) -> bool {
    matches!(
        feature,
        FEATURE_COMPAT
            | FEATURE_BATCH
            | FEATURE_COMPACTION
            | FEATURE_DATASET_EXPORT
            | FEATURE_HEADLESS
            | FEATURE_LOOP
            | FEATURE_POLICY
            | FEATURE_PROVENANCE
            | FEATURE_QUALITY
            | FEATURE_RESEARCH
            | FEATURE_SCAFFOLD
            | FEATURE_STATUS
            | FEATURE_WORKER_TEAM
    )
}

fn is_known_authority(authority: &str) -> bool {
    matches!(authority, AUTHORITY_NONE | AUTHORITY_INJECTED)
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNKNOWN_FEATURE: &str = "ambient";
    const UNKNOWN_AUTHORITY: &str = "filesystem";
    const PRELUDE_SOURCE: &str = include_str!("prelude.rs");
    const HEADLESS_PRELUDE_GUARD: &str =
        "#[cfg(any(feature = \"compat\", feature = \"headless\"))]\npub mod headless_prelude";
    const PROVENANCE_PRELUDE_GUARD: &str =
        "#[cfg(any(feature = \"compat\", feature = \"provenance\"))]\npub mod provenance_prelude";
    const POLICY_PRELUDE_GUARD: &str =
        "#[cfg(any(feature = \"compat\", feature = \"policy\"))]\npub mod policy_prelude";

    #[test]
    fn capability_family_map_covers_expected_families_once() {
        validate_runtime_capability_family_map(runtime_capability_family_map())
            .expect("family map should validate");

        assert!(
            runtime_capability_family_map()
                .iter()
                .any(|record| record.family() == RuntimeCapabilityFamily::Loop)
        );
        assert!(
            runtime_capability_family_map()
                .iter()
                .any(|record| record.family() == RuntimeCapabilityFamily::Compatibility)
        );
    }

    #[test]
    fn family_preludes_are_feature_guarded_against_disabled_family_leaks() {
        assert!(PRELUDE_SOURCE.contains(HEADLESS_PRELUDE_GUARD));
        assert!(PRELUDE_SOURCE.contains(PROVENANCE_PRELUDE_GUARD));
        assert!(PRELUDE_SOURCE.contains(POLICY_PRELUDE_GUARD));
    }

    #[test]
    fn capability_family_map_rejects_duplicates_and_bad_metadata() {
        let duplicate = [
            RuntimeCapabilityFamilyRecord::new(
                RuntimeCapabilityFamily::Loop,
                FEATURE_LOOP,
                AUTHORITY_INJECTED,
                "loop",
            ),
            RuntimeCapabilityFamilyRecord::new(
                RuntimeCapabilityFamily::Loop,
                FEATURE_LOOP,
                AUTHORITY_INJECTED,
                "loop duplicate",
            ),
        ];
        let bad_feature = [RuntimeCapabilityFamilyRecord::new(
            RuntimeCapabilityFamily::Loop,
            UNKNOWN_FEATURE,
            AUTHORITY_INJECTED,
            "loop",
        )];
        let bad_authority = [RuntimeCapabilityFamilyRecord::new(
            RuntimeCapabilityFamily::Loop,
            FEATURE_LOOP,
            UNKNOWN_AUTHORITY,
            "loop",
        )];

        assert!(matches!(
            validate_runtime_capability_family_map(&duplicate),
            Err(RuntimeCapabilityFamilyMapError::DuplicateFamily { .. })
        ));
        assert!(matches!(
            validate_runtime_capability_family_map(&bad_feature),
            Err(RuntimeCapabilityFamilyMapError::UnknownFeature { .. })
        ));
        assert!(matches!(
            validate_runtime_capability_family_map(&bad_authority),
            Err(RuntimeCapabilityFamilyMapError::UnknownAuthority { .. })
        ));
    }
}
