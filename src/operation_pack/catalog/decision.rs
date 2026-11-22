//! Per-bundle admission decision over pack declarations and loader observations.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "each bundle outcome is a typed report entry instead of a process-terminating assertion"
)]

pub const INSTALL_REPORT_SCHEMA: &str = "lattice.operation-pack-catalog-install.v1";
const BLAKE3_PREFIX: &str = "b3:";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Verified and absent from the catalog; preview only.
    Admitted,
    /// Verified, staged, re-verified and published into the catalog.
    Installed,
    /// Rejected; other bundles are still decided independently.
    Rejected,
    /// The catalog already holds this bundle id; installation never replaces it.
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BundleOutcome {
    pub id: String,
    pub declared_identity: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// What the workflow-template loader validated for one bundle directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observed {
    pub id: String,
    pub identity: String,
    pub manifest_identity: String,
    pub files: std::collections::BTreeMap<String, String>,
}

/// One loader diagnostic, keyed by the loader's safe bundle label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderDiagnostic {
    pub bundle: String,
    pub code: &'static str,
    pub message: &'static str,
}

/// Shell-collected facts for one decision: loader results, the catalog's existing ids and how
/// many more bundle directories the catalog can hold before the loader's limit.
#[derive(Debug, Clone, Copy)]
pub struct Facts<'facts> {
    pub observed: &'facts [Observed],
    pub diagnostics: &'facts [LoaderDiagnostic],
    pub existing: &'facts std::collections::BTreeSet<String>,
    pub free_slots: usize,
}

/// Remaining bundle directories before `workflow_template::MAX_CATALOG_BUNDLES` is reached.
pub(crate) fn free_slots(directory_count: usize) -> usize {
    crate::workflow_template::MAX_CATALOG_BUNDLES.saturating_sub(directory_count)
}

/// Reject one bundle because publishing it would exceed the loader's catalog limit.
pub fn reject_for_capacity(outcome: &mut BundleOutcome) {
    settle(outcome, Status::Rejected, Finding {
        code: "catalog_capacity",
        reason: "the catalog already holds the loader's maximum number of bundle directories",
    });
}

/// Project the workflow-template loader's catalog into decision facts.
pub fn observe(catalog: &crate::workflow_template::Catalog) -> (Vec<Observed>, Vec<LoaderDiagnostic>) {
    let observed = catalog
        .bundles
        .iter()
        .map(|bundle| Observed {
            id: bundle.manifest.id.clone(),
            identity: prefixed(bundle.identity.as_str()),
            manifest_identity: prefixed(bundle.manifest_identity.as_str()),
            files: bundle
                .file_identities
                .iter()
                .map(|(path, identity)| (path.clone(), prefixed(identity.as_str())))
                .collect(),
        })
        .collect();
    let diagnostics = catalog
        .diagnostics
        .iter()
        .map(|diagnostic| LoaderDiagnostic {
            bundle: diagnostic.bundle.clone(),
            code: diagnostic.code.as_str(),
            message: diagnostic.message,
        })
        .collect();
    (observed, diagnostics)
}

/// Decide every declared bundle independently, so one bad bundle never hides another.
///
/// A bundle is admitted only when the loader validated it, its loader-measured manifest and
/// file identities equal the pack's declared content identities, its loader identity equals the
/// declared `bundle_identity`, the catalog does not already hold its id, and the catalog has a
/// free slot left after the bundles admitted before it.
pub fn decide(manifest: &super::super::model::Manifest, facts: &Facts<'_>) -> Vec<BundleOutcome> {
    let mut outcomes = Vec::with_capacity(manifest.workflow_bundles.len());
    let mut free_slots = facts.free_slots;
    for declaration in &manifest.workflow_bundles {
        let mut outcome = BundleOutcome {
            id: declaration.id.clone(),
            declared_identity: declaration.bundle_identity.clone(),
            status: Status::Rejected,
            observed_identity: None,
            code: None,
            reason: None,
        };
        decide_one(manifest, declaration, facts, &mut outcome);
        if outcome.status == Status::Admitted {
            match free_slots.checked_sub(1) {
                Some(remaining) => free_slots = remaining,
                None => reject_for_capacity(&mut outcome),
            }
        }
        outcomes.push(outcome);
    }
    debug_assert_eq!(outcomes.len(), manifest.workflow_bundles.len());
    outcomes
}

fn decide_one(
    manifest: &super::super::model::Manifest,
    declaration: &super::WorkflowBundleDeclaration,
    facts: &Facts<'_>,
    outcome: &mut BundleOutcome,
) {
    if let Some(diagnostic) = facts.diagnostics.iter().find(|diagnostic| diagnostic.bundle == declaration.id) {
        settle(outcome, Status::Rejected, Finding {
            code: diagnostic.code,
            reason: diagnostic.message,
        });
        return;
    }
    let Some(bundle) = facts.observed.iter().find(|bundle| bundle.id == declaration.id) else {
        settle(outcome, Status::Rejected, Finding {
            code: "missing",
            reason: "the workflow-template loader did not validate this bundle",
        });
        return;
    };
    outcome.observed_identity = Some(bundle.identity.clone());
    let declared = super::bundle_contents(manifest, &declaration.id);
    let declared_manifest = declared.get(super::BUNDLE_MANIFEST).map(|content| content.content_identity.as_str());
    let declared_files = declared
        .iter()
        .filter(|(path, _)| **path != super::BUNDLE_MANIFEST)
        .map(|(path, content)| ((*path).to_string(), content.content_identity.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    if declared_manifest != Some(bundle.manifest_identity.as_str()) || declared_files != bundle.files {
        settle(outcome, Status::Rejected, Finding {
            code: "pack_content_mismatch",
            reason: "loader-validated bytes differ from the pack declaration",
        });
        return;
    }
    if bundle.identity != declaration.bundle_identity {
        settle(outcome, Status::Rejected, Finding {
            code: "identity_mismatch",
            reason: "bundle identity differs from the pack declaration",
        });
        return;
    }
    if facts.existing.contains(&declaration.id) {
        settle(outcome, Status::Conflict, Finding {
            code: "catalog_conflict",
            reason: "the catalog already holds this bundle id; installation never replaces it",
        });
        return;
    }
    outcome.status = Status::Admitted;
}

/// One closed rejection or conflict: a stable code and its fixed operator text.
struct Finding {
    code: &'static str,
    reason: &'static str,
}

fn settle(outcome: &mut BundleOutcome, status: Status, finding: Finding) {
    outcome.status = status;
    outcome.code = Some(finding.code.to_string());
    outcome.reason = Some(finding.reason.to_string());
}

fn prefixed(hex: &str) -> String {
    format!("{BLAKE3_PREFIX}{hex}")
}
