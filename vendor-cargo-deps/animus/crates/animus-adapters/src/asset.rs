//! Host-supplied asset reconciliation contracts.
//!
//! Hosts discover installed skills, resources, plugin manifests, prompt profiles,
//! policy profiles, or other prompt-affecting assets and pass typed observations
//! into this module. This adapter code validates and classifies in-memory data
//! only; it never traverses filesystems, resolves symlinks, copies, links,
//! unlinks, installs plugins, writes manifests, or treats a global host store as
//! canonical.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::bool_naming,
        tigerstyle::function_length,
        tigerstyle::non_trait_imports,
        tigerstyle::numeric_units,
        tigerstyle::path_segment_repetition,
        tigerstyle::raw_arithmetic_overflow,
        tigerstyle::too_many_parameters,
        tigerstyle::unbounded_collection_growth
    )
)]

use alloc::format;

use crate::artifact::Hash;
use crate::validation::AdapterContractError;
use crate::validation::FieldName;
use crate::validation::{self};

const ASSET_ID_FIELD: FieldName = FieldName::new("asset_id");
const STORE_LABEL_FIELD: FieldName = FieldName::new("asset_store_label");
const CONTENT_IDENTITY_FIELD: FieldName = FieldName::new("asset_content_identity");
const PROVENANCE_FIELD: FieldName = FieldName::new("asset_provenance_label");
const OBSERVATION_ID_FIELD: FieldName = FieldName::new("asset_observation_id");
const ACTION_ID_FIELD: FieldName = FieldName::new("asset_action_id");
const CAPABILITY_LABEL_FIELD: FieldName = FieldName::new("asset_capability_label");
const RECEIPT_STATUS_FIELD: FieldName = FieldName::new("asset_receipt_status");
const ISSUE_CODE_FIELD: FieldName = FieldName::new("asset_issue_code");

const MAX_ASSET_ITEMS: usize = 64;
const MAX_PLAN_ACTIONS: usize = MAX_ASSET_ITEMS;
const MAX_RECEIPT_HASHES: usize = MAX_ASSET_ITEMS;
const STALE_OBSERVATION_AGE_EPOCHS: u32 = 3;
const SUPPORTED_STORE_LABELS: &[&str] = &[
    "skill-store",
    "resource-store",
    "plugin-store",
    "tool-catalog",
    "policy-profile",
    "prompt-profile",
    "host-bundle",
    "embedded-bundle",
];
const UNSAFE_LABEL_MARKERS: &[&str] = &[
    "../",
    "/home/",
    "/tmp/",
    "~/.hermes",
    "~/.pi",
    "c:\\",
    "\\users\\",
    "raw prompt",
    "provider payload",
    "secret",
    "token",
    "credential",
    "raw diff",
    "raw file body",
    "raw tool output",
    "unbounded transcript",
];

const _: () = assert!(MAX_ASSET_ITEMS > 0);
const _: () = assert!(MAX_PLAN_ACTIONS == MAX_ASSET_ITEMS);
const _: () = assert!(MAX_RECEIPT_HASHES == MAX_ASSET_ITEMS);
const _: () = assert!(STALE_OBSERVATION_AGE_EPOCHS > 0);

/// Descriptor-side effects that asset reconciliation explicitly does not perform.
pub const PROHIBITED_ASSET_RECONCILIATION_EFFECTS: &[&str] = &[
    "filesystem traversal",
    "path canonicalization",
    "symlink resolution",
    "copy/link/unlink",
    "enable/disable writes",
    "plugin installation",
    "manifest writes",
    "global store discovery",
];

/// Asset kind covered by reconciliation manifests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAssetKind {
    /// Host-provided skill package or skill descriptor.
    Skill,
    /// Host resource descriptor or bundled prompt resource.
    Resource,
    /// Plugin manifest or tool adapter package.
    Plugin,
    /// Tool catalog entry.
    ToolCatalog,
    /// Policy profile.
    PolicyProfile,
    /// Prompt profile.
    PromptProfile,
    /// Other host-owned optional asset.
    Other,
}

/// Trust state supplied by the host inventory boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAssetTrust {
    /// Trusted asset can be planned for install/update/enable actions.
    Trusted,
    /// Untrusted asset is reported but not planned for mutation.
    Untrusted,
}

/// Installed enablement state supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAssetEnablement {
    /// Asset is enabled in the host store.
    Enabled,
    /// Asset exists but is disabled.
    Disabled,
}

/// Host service availability for mutation admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAssetServiceStatus {
    /// Host injected a reconciliation service.
    Available,
    /// No reconciliation service is injected.
    Missing,
    /// Service exists but reported unavailable.
    Unavailable,
}

/// Store-neutral drift classes for manifest/installed comparisons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAssetDriftClass {
    /// Installed asset matches the manifest identity and is enabled.
    InstalledMatchesManifest,
    /// Installed asset matches the manifest identity but is disabled.
    InstalledDisabled,
    /// Installed asset differs from the manifest identity.
    IdentityMismatch,
    /// Same asset appears more than once in a manifest or observations.
    DuplicateAsset,
    /// Installed observation is older than the host freshness window.
    StaleObservation,
    /// Host supplied an unsupported store label.
    UnsupportedStore,
    /// Asset appears only in the manifest.
    ManifestOnly,
    /// Asset appears only in the installed host observation set.
    HostOnly,
    /// Asset is untrusted and therefore advisory-only.
    UntrustedAsset,
}

/// Advisory reconciliation action kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAssetActionKind {
    /// Install a manifest-only asset.
    Install,
    /// Update an installed asset to the manifest identity.
    Update,
    /// Enable a disabled matching asset.
    Enable,
    /// Disable an asset.
    Disable,
    /// Remove a host-only asset.
    Remove,
    /// Write or refresh a manifest entry.
    WriteManifest,
}

/// Plan status for dry-run reconciliation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAssetPlanStatus {
    /// Plan contains advisory actions only and no blocking diagnostics.
    DryRun,
    /// Blocking diagnostics prevent safe planning.
    Blocked,
    /// No actions are needed.
    Noop,
}

/// Mutation request admission status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAssetMutationStatus {
    /// Request is admitted for the host-owned service to execute.
    Admitted,
    /// No host service is available.
    ServiceUnavailable,
    /// Requested action was not present in a fresh dry-run plan.
    PlanMismatch,
    /// Requested action lacks a capability label.
    MissingCapability,
    /// Request contains unsafe metadata.
    UnsafeMetadata,
}

/// Asset reconciliation validation diagnostic code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAssetDiagnosticCode {
    /// Required field is empty.
    EmptyField,
    /// Field is too long.
    FieldTooLong,
    /// Field includes a NUL byte.
    NulCharacter,
    /// Hash is not a lowercase BLAKE3 hex identity.
    InvalidHash,
    /// Metadata includes unsafe path/payload/secret markers.
    UnsafeMetadata,
    /// Duplicate manifest or observation asset id.
    DuplicateAsset,
    /// Store label is not in the host-approved set.
    UnsupportedStore,
    /// Observation is too stale for mutation planning.
    StaleObservation,
    /// Collection size exceeds the bounded contract.
    TooManyItems,
    /// Required collection is empty.
    EmptyCollection,
    /// Mutation was requested without a host service.
    ServiceUnavailable,
    /// Mutation request does not match a dry-run plan action.
    PlanMismatch,
    /// Required capability label is missing.
    MissingCapability,
}

/// Typed diagnostic emitted by validation, classification, and planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetDiagnostic {
    code: HostAssetDiagnosticCode,
    asset_id: Option<Text>,
    message: Text,
}

/// Draft manifest entry supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostAssetManifestEntryDraft<'a> {
    /// Stable host asset id.
    pub asset_id: &'a str,
    /// Asset kind.
    pub kind: HostAssetKind,
    /// Host-approved store label.
    pub store_label: &'a str,
    /// Lowercase BLAKE3 content identity.
    pub content_identity: &'a str,
    /// Safe provenance label or hash.
    pub provenance_label: &'a str,
    /// Trust state.
    pub trust: HostAssetTrust,
}

/// Validated manifest entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetManifestEntry {
    asset_id: Text,
    kind: HostAssetKind,
    store_label: Text,
    content_identity: Text,
    provenance_label: Text,
    trust: HostAssetTrust,
}

/// Draft installed observation supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostAssetObservationDraft<'a> {
    /// Stable host asset id.
    pub asset_id: &'a str,
    /// Asset kind.
    pub kind: HostAssetKind,
    /// Host-approved store label.
    pub store_label: &'a str,
    /// Lowercase BLAKE3 content identity.
    pub content_identity: &'a str,
    /// Safe observation id/hash.
    pub observation_id: &'a str,
    /// Installed enablement state.
    pub enablement: HostAssetEnablement,
    /// Trust state.
    pub trust: HostAssetTrust,
    /// Host-supplied monotonically increasing observation epoch.
    pub observed_epoch: u32,
}

/// Validated installed observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetObservation {
    asset_id: Text,
    kind: HostAssetKind,
    store_label: Text,
    content_identity: Text,
    observation_id: Text,
    enablement: HostAssetEnablement,
    trust: HostAssetTrust,
    observed_epoch: u32,
}

/// Draft inventory comparing manifests and installed observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostAssetInventoryDraft<'a> {
    /// Manifest entries supplied by the host.
    pub manifests: &'a [HostAssetManifestEntryDraft<'a>],
    /// Installed observations supplied by the host.
    pub observations: &'a [HostAssetObservationDraft<'a>],
    /// Current host epoch used only for freshness arithmetic.
    pub current_epoch: u32,
}

/// Validated host asset inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetInventory {
    manifests: Collection<HostAssetManifestEntry>,
    observations: Collection<HostAssetObservation>,
    current_epoch: u32,
}

/// Drift report for one asset id.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetDriftReport {
    asset_id: Text,
    kind: HostAssetKind,
    class: HostAssetDriftClass,
    manifest_identity: Option<Text>,
    installed_identity: Option<Text>,
    store_label: Text,
    diagnostic: Option<HostAssetDiagnostic>,
}

/// Advisory dry-run action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetAction {
    action_id: Text,
    asset_id: Text,
    kind: HostAssetActionKind,
    required_capability: Text,
    content_identity: Option<Text>,
}

/// Dry-run reconciliation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetReconciliationPlan {
    status: HostAssetPlanStatus,
    reports: Collection<HostAssetDriftReport>,
    actions: Collection<HostAssetAction>,
    diagnostics: Collection<HostAssetDiagnostic>,
}

/// Host-owned mutation request draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostAssetMutationRequestDraft<'a> {
    /// Action id from a dry-run plan.
    pub action_id: &'a str,
    /// Asset id from a dry-run plan.
    pub asset_id: &'a str,
    /// Requested action kind.
    pub kind: HostAssetActionKind,
    /// Host capability admission label.
    pub capability_label: &'a str,
    /// Plan action identity/hash supplied by host.
    pub plan_identity: &'a str,
}

/// Validated mutation request admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetMutationAdmission {
    status: HostAssetMutationStatus,
    request: Option<HostAssetMutationRequest>,
    diagnostic: Option<HostAssetDiagnostic>,
}

/// Validated mutation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetMutationRequest {
    action_id: Text,
    asset_id: Text,
    kind: HostAssetActionKind,
    capability_label: Text,
    plan_identity: Text,
}

/// Durable receipt draft for host-owned asset actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostAssetReconciliationReceiptDraft<'a> {
    /// Plan status label supplied by host.
    pub status: &'a str,
    /// Action ids completed or skipped.
    pub action_hashes: &'a [&'a str],
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
}

/// Validated safe receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAssetReconciliationReceipt {
    status: Text,
    action_hashes: Collection<Text>,
    issue_code: Option<Text>,
}

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

type AssetResult<T> = Result<T, HostAssetDiagnostic>;

impl HostAssetDiagnostic {
    /// Diagnostic code.
    #[must_use]
    pub const fn code(&self) -> HostAssetDiagnosticCode {
        self.code
    }

    /// Asset id when available.
    #[must_use]
    pub fn asset_id(&self) -> Option<&str> {
        self.asset_id.as_deref()
    }

    /// Safe diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl HostAssetManifestEntry {
    /// Asset id.
    #[must_use]
    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    /// Asset kind.
    #[must_use]
    pub const fn kind(&self) -> HostAssetKind {
        self.kind
    }

    /// Store label.
    #[must_use]
    pub fn store_label(&self) -> &str {
        &self.store_label
    }

    /// Content identity.
    #[must_use]
    pub fn content_identity(&self) -> &str {
        &self.content_identity
    }

    /// Provenance label.
    #[must_use]
    pub fn provenance_label(&self) -> &str {
        &self.provenance_label
    }

    /// Trust state.
    #[must_use]
    pub const fn trust(&self) -> HostAssetTrust {
        self.trust
    }
}

impl HostAssetObservation {
    /// Asset id.
    #[must_use]
    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    /// Asset kind.
    #[must_use]
    pub const fn kind(&self) -> HostAssetKind {
        self.kind
    }

    /// Store label.
    #[must_use]
    pub fn store_label(&self) -> &str {
        &self.store_label
    }

    /// Content identity.
    #[must_use]
    pub fn content_identity(&self) -> &str {
        &self.content_identity
    }

    /// Observation id.
    #[must_use]
    pub fn observation_id(&self) -> &str {
        &self.observation_id
    }

    /// Enablement state.
    #[must_use]
    pub const fn enablement(&self) -> HostAssetEnablement {
        self.enablement
    }

    /// Trust state.
    #[must_use]
    pub const fn trust(&self) -> HostAssetTrust {
        self.trust
    }

    /// Host-supplied observation epoch.
    #[must_use]
    pub const fn observed_epoch(&self) -> u32 {
        self.observed_epoch
    }
}

impl HostAssetInventory {
    /// Validated manifest entries.
    #[must_use]
    pub fn manifests(&self) -> &[HostAssetManifestEntry] {
        &self.manifests
    }

    /// Validated installed observations.
    #[must_use]
    pub fn observations(&self) -> &[HostAssetObservation] {
        &self.observations
    }

    /// Host-supplied current epoch.
    #[must_use]
    pub const fn current_epoch(&self) -> u32 {
        self.current_epoch
    }
}

impl HostAssetDriftReport {
    /// Asset id.
    #[must_use]
    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    /// Asset kind.
    #[must_use]
    pub const fn kind(&self) -> HostAssetKind {
        self.kind
    }

    /// Drift class.
    #[must_use]
    pub const fn class(&self) -> HostAssetDriftClass {
        self.class
    }

    /// Manifest identity when present.
    #[must_use]
    pub fn manifest_identity(&self) -> Option<&str> {
        self.manifest_identity.as_deref()
    }

    /// Installed identity when present.
    #[must_use]
    pub fn installed_identity(&self) -> Option<&str> {
        self.installed_identity.as_deref()
    }

    /// Store label.
    #[must_use]
    pub fn store_label(&self) -> &str {
        &self.store_label
    }

    /// Diagnostic when classification produced one.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&HostAssetDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl HostAssetAction {
    /// Action id.
    #[must_use]
    pub fn action_id(&self) -> &str {
        &self.action_id
    }

    /// Asset id.
    #[must_use]
    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    /// Action kind.
    #[must_use]
    pub const fn kind(&self) -> HostAssetActionKind {
        self.kind
    }

    /// Required host capability label.
    #[must_use]
    pub fn required_capability(&self) -> &str {
        &self.required_capability
    }

    /// Content identity when the action is tied to one.
    #[must_use]
    pub fn content_identity(&self) -> Option<&str> {
        self.content_identity.as_deref()
    }
}

impl HostAssetReconciliationPlan {
    /// Plan status.
    #[must_use]
    pub const fn status(&self) -> HostAssetPlanStatus {
        self.status
    }

    /// Drift reports.
    #[must_use]
    pub fn reports(&self) -> &[HostAssetDriftReport] {
        &self.reports
    }

    /// Dry-run actions.
    #[must_use]
    pub fn actions(&self) -> &[HostAssetAction] {
        &self.actions
    }

    /// Plan diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[HostAssetDiagnostic] {
        &self.diagnostics
    }
}

impl HostAssetMutationAdmission {
    /// Admission status.
    #[must_use]
    pub const fn status(&self) -> HostAssetMutationStatus {
        self.status
    }

    /// Validated request when admitted.
    #[must_use]
    pub fn request(&self) -> Option<&HostAssetMutationRequest> {
        self.request.as_ref()
    }

    /// Diagnostic when rejected.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&HostAssetDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl HostAssetMutationRequest {
    /// Action id.
    #[must_use]
    pub fn action_id(&self) -> &str {
        &self.action_id
    }

    /// Asset id.
    #[must_use]
    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    /// Requested action kind.
    #[must_use]
    pub const fn kind(&self) -> HostAssetActionKind {
        self.kind
    }

    /// Capability label.
    #[must_use]
    pub fn capability_label(&self) -> &str {
        &self.capability_label
    }

    /// Plan identity.
    #[must_use]
    pub fn plan_identity(&self) -> &str {
        &self.plan_identity
    }
}

impl HostAssetReconciliationReceipt {
    /// Receipt status label.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Action hashes.
    #[must_use]
    pub fn action_hashes(&self) -> &[Text] {
        &self.action_hashes
    }

    /// Safe issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }
}

/// Validates a host asset inventory without filesystem or host-store access.
///
/// # Errors
///
/// Returns [`HostAssetDiagnostic`] when a field is unsafe, malformed, duplicate,
/// stale beyond the validation boundary, or outside bounded collection limits.
pub fn validate_host_asset_inventory(draft: &HostAssetInventoryDraft<'_>) -> AssetResult<HostAssetInventory> {
    validate_collection_limit("asset_manifests", draft.manifests.len())?;
    validate_collection_limit("asset_observations", draft.observations.len())?;
    if draft.manifests.is_empty() && draft.observations.is_empty() {
        return Err(HostAssetDiagnostic {
            code: HostAssetDiagnosticCode::EmptyCollection,
            asset_id: None,
            message: Text::from("asset_inventory"),
        });
    }
    let mut manifests = Collection::with_capacity(draft.manifests.len());
    for manifest in draft.manifests {
        let entry = validate_manifest_entry(manifest)?;
        if contains_asset_id(manifests.as_slice(), entry.asset_id()) {
            return Err(diagnostic(
                HostAssetDiagnosticCode::DuplicateAsset,
                Some(entry.asset_id()),
                "duplicate manifest asset id",
            ));
        }
        manifests.push(entry);
    }

    let mut observations = Collection::with_capacity(draft.observations.len());
    for observation in draft.observations {
        let entry = validate_observation(observation, draft.current_epoch)?;
        if contains_observation_id(observations.as_slice(), entry.asset_id()) {
            return Err(diagnostic(
                HostAssetDiagnosticCode::DuplicateAsset,
                Some(entry.asset_id()),
                "duplicate installed asset id",
            ));
        }
        observations.push(entry);
    }

    Ok(HostAssetInventory {
        manifests,
        observations,
        current_epoch: draft.current_epoch,
    })
}

/// Classifies manifest and installed observation drift deterministically.
#[must_use]
pub fn classify_host_asset_drift(inventory: &HostAssetInventory) -> Collection<HostAssetDriftReport> {
    let mut reports = Collection::with_capacity(inventory.manifests.len() + inventory.observations.len());
    for manifest in &inventory.manifests {
        let matching = find_observation(inventory.observations.as_slice(), manifest.asset_id());
        reports.push(classify_manifest_asset(manifest, matching, inventory.current_epoch));
    }
    for observation in &inventory.observations {
        if find_manifest(inventory.manifests.as_slice(), observation.asset_id()).is_none() {
            reports.push(report(HostAssetReportInput {
                asset_id: observation.asset_id(),
                kind: observation.kind(),
                class: HostAssetDriftClass::HostOnly,
                manifest_identity: None,
                installed_identity: Some(observation.content_identity()),
                store_label: observation.store_label(),
                diagnostic: None,
            }));
        }
    }
    reports
}

/// Builds a dry-run reconciliation plan. No host store is read or mutated.
#[must_use]
pub fn plan_host_asset_reconciliation(reports: &[HostAssetDriftReport]) -> HostAssetReconciliationPlan {
    let mut diagnostics = Collection::new();
    let mut actions = Collection::new();

    for report in reports {
        if let Some(item) = report.diagnostic() {
            diagnostics.push(item.clone());
        }
        if let Some(action) = action_for_report(report) {
            actions.push(action);
        }
    }

    let blocked = diagnostics.iter().any(is_blocking_diagnostic);
    let status = if blocked {
        HostAssetPlanStatus::Blocked
    } else if actions.is_empty() {
        HostAssetPlanStatus::Noop
    } else {
        HostAssetPlanStatus::DryRun
    };

    HostAssetReconciliationPlan {
        status,
        reports: reports.to_vec(),
        actions,
        diagnostics,
    }
}

/// Validates an explicit host-owned mutation request against a dry-run plan.
///
/// # Errors
///
/// This function does not return errors; rejected requests are represented as a
/// typed [`HostAssetMutationAdmission`] with a diagnostic.
#[must_use]
pub fn admit_host_asset_mutation(
    plan: &HostAssetReconciliationPlan,
    draft: &HostAssetMutationRequestDraft<'_>,
    service_status: HostAssetServiceStatus,
) -> HostAssetMutationAdmission {
    if service_status != HostAssetServiceStatus::Available {
        return rejected_admission(
            HostAssetMutationStatus::ServiceUnavailable,
            HostAssetDiagnosticCode::ServiceUnavailable,
            draft.asset_id,
            "host asset service unavailable",
        );
    }

    let action_id = match safe_text(ACTION_ID_FIELD, draft.action_id) {
        Ok(value) => value,
        Err(error) => {
            return rejected_from_diagnostic(HostAssetMutationStatus::UnsafeMetadata, error);
        }
    };
    let asset_id = match safe_text(ASSET_ID_FIELD, draft.asset_id) {
        Ok(value) => value,
        Err(error) => {
            return rejected_from_diagnostic(HostAssetMutationStatus::UnsafeMetadata, error);
        }
    };
    let capability_label = match safe_text(CAPABILITY_LABEL_FIELD, draft.capability_label) {
        Ok(value) => value,
        Err(error) => {
            return rejected_from_diagnostic(HostAssetMutationStatus::UnsafeMetadata, error);
        }
    };
    if capability_label.is_empty() {
        return rejected_admission(
            HostAssetMutationStatus::MissingCapability,
            HostAssetDiagnosticCode::MissingCapability,
            draft.asset_id,
            "mutation request requires a capability label",
        );
    }
    let plan_identity = match safe_hash(CONTENT_IDENTITY_FIELD, draft.plan_identity) {
        Ok(value) => value,
        Err(error) => {
            return rejected_from_diagnostic(HostAssetMutationStatus::UnsafeMetadata, error);
        }
    };

    if plan.status() != HostAssetPlanStatus::DryRun {
        return rejected_admission(
            HostAssetMutationStatus::PlanMismatch,
            HostAssetDiagnosticCode::PlanMismatch,
            asset_id.as_str(),
            "mutation requires a dry-run plan",
        );
    }

    let matching = plan.actions().iter().find(|action| {
        action.action_id() == action_id
            && action.asset_id() == asset_id
            && action.kind() == draft.kind
            && action.required_capability() == capability_label
    });
    if matching.is_none() {
        return rejected_admission(
            HostAssetMutationStatus::PlanMismatch,
            HostAssetDiagnosticCode::PlanMismatch,
            asset_id.as_str(),
            "mutation request does not match dry-run plan action",
        );
    }

    HostAssetMutationAdmission {
        status: HostAssetMutationStatus::Admitted,
        request: Some(HostAssetMutationRequest {
            action_id,
            asset_id,
            kind: draft.kind,
            capability_label,
            plan_identity,
        }),
        diagnostic: None,
    }
}

/// Validates a durable safe reconciliation receipt.
///
/// # Errors
///
/// Returns [`HostAssetDiagnostic`] when status, issue, or hashes include unsafe
/// metadata or malformed BLAKE3 identities.
pub fn validate_host_asset_receipt(
    draft: &HostAssetReconciliationReceiptDraft<'_>,
) -> AssetResult<HostAssetReconciliationReceipt> {
    validate_non_empty_collection_bound("asset_receipt_hashes", draft.action_hashes.len())?;
    let status = safe_text(RECEIPT_STATUS_FIELD, draft.status)?;
    let issue_code = draft.issue_code.map(|issue| safe_text(ISSUE_CODE_FIELD, issue)).transpose()?;
    let mut hashes = Collection::with_capacity(draft.action_hashes.len());
    for hash in draft.action_hashes {
        hashes.push(safe_hash(CONTENT_IDENTITY_FIELD, hash)?);
    }
    Ok(HostAssetReconciliationReceipt {
        status,
        action_hashes: hashes,
        issue_code,
    })
}

fn validate_manifest_entry(draft: &HostAssetManifestEntryDraft<'_>) -> AssetResult<HostAssetManifestEntry> {
    let asset_id = safe_text(ASSET_ID_FIELD, draft.asset_id)?;
    let store_label = safe_store_label(draft.store_label)?;
    let content_identity = safe_hash(CONTENT_IDENTITY_FIELD, draft.content_identity)?;
    let provenance_label = safe_text(PROVENANCE_FIELD, draft.provenance_label)?;
    Ok(HostAssetManifestEntry {
        asset_id,
        kind: draft.kind,
        store_label,
        content_identity,
        provenance_label,
        trust: draft.trust,
    })
}

fn validate_observation(
    draft: &HostAssetObservationDraft<'_>,
    _current_epoch: u32,
) -> AssetResult<HostAssetObservation> {
    let asset_id = safe_text(ASSET_ID_FIELD, draft.asset_id)?;
    let store_label = safe_store_label(draft.store_label)?;
    let content_identity = safe_hash(CONTENT_IDENTITY_FIELD, draft.content_identity)?;
    let observation_id = safe_text(OBSERVATION_ID_FIELD, draft.observation_id)?;
    Ok(HostAssetObservation {
        asset_id,
        kind: draft.kind,
        store_label,
        content_identity,
        observation_id,
        enablement: draft.enablement,
        trust: draft.trust,
        observed_epoch: draft.observed_epoch,
    })
}

fn classify_manifest_asset(
    manifest: &HostAssetManifestEntry,
    observation: Option<&HostAssetObservation>,
    current_epoch: u32,
) -> HostAssetDriftReport {
    if manifest.trust() == HostAssetTrust::Untrusted {
        return report(HostAssetReportInput {
            asset_id: manifest.asset_id(),
            kind: manifest.kind(),
            class: HostAssetDriftClass::UntrustedAsset,
            manifest_identity: Some(manifest.content_identity()),
            installed_identity: observation.map(HostAssetObservation::content_identity),
            store_label: manifest.store_label(),
            diagnostic: Some(diagnostic(
                HostAssetDiagnosticCode::UnsupportedStore,
                Some(manifest.asset_id()),
                "untrusted asset is advisory-only",
            )),
        });
    }

    match observation {
        None => report(HostAssetReportInput {
            asset_id: manifest.asset_id(),
            kind: manifest.kind(),
            class: HostAssetDriftClass::ManifestOnly,
            manifest_identity: Some(manifest.content_identity()),
            installed_identity: None,
            store_label: manifest.store_label(),
            diagnostic: None,
        }),
        Some(installed) => classify_installed_pair(manifest, installed, current_epoch),
    }
}

fn classify_installed_pair(
    manifest: &HostAssetManifestEntry,
    installed: &HostAssetObservation,
    current_epoch: u32,
) -> HostAssetDriftReport {
    if observation_is_stale(installed.observed_epoch(), current_epoch) {
        return report(HostAssetReportInput {
            asset_id: manifest.asset_id(),
            kind: manifest.kind(),
            class: HostAssetDriftClass::StaleObservation,
            manifest_identity: Some(manifest.content_identity()),
            installed_identity: Some(installed.content_identity()),
            store_label: manifest.store_label(),
            diagnostic: Some(diagnostic(
                HostAssetDiagnosticCode::StaleObservation,
                Some(manifest.asset_id()),
                "installed asset observation is stale",
            )),
        });
    }
    if manifest.store_label() != installed.store_label() {
        return report(HostAssetReportInput {
            asset_id: manifest.asset_id(),
            kind: manifest.kind(),
            class: HostAssetDriftClass::UnsupportedStore,
            manifest_identity: Some(manifest.content_identity()),
            installed_identity: Some(installed.content_identity()),
            store_label: manifest.store_label(),
            diagnostic: Some(diagnostic(
                HostAssetDiagnosticCode::UnsupportedStore,
                Some(manifest.asset_id()),
                "manifest and installed store labels differ",
            )),
        });
    }
    if manifest.content_identity() != installed.content_identity() {
        return report(HostAssetReportInput {
            asset_id: manifest.asset_id(),
            kind: manifest.kind(),
            class: HostAssetDriftClass::IdentityMismatch,
            manifest_identity: Some(manifest.content_identity()),
            installed_identity: Some(installed.content_identity()),
            store_label: manifest.store_label(),
            diagnostic: Some(diagnostic(
                HostAssetDiagnosticCode::InvalidHash,
                Some(manifest.asset_id()),
                "installed content identity differs from manifest identity",
            )),
        });
    }
    if installed.enablement() == HostAssetEnablement::Disabled {
        return report(HostAssetReportInput {
            asset_id: manifest.asset_id(),
            kind: manifest.kind(),
            class: HostAssetDriftClass::InstalledDisabled,
            manifest_identity: Some(manifest.content_identity()),
            installed_identity: Some(installed.content_identity()),
            store_label: manifest.store_label(),
            diagnostic: None,
        });
    }
    report(HostAssetReportInput {
        asset_id: manifest.asset_id(),
        kind: manifest.kind(),
        class: HostAssetDriftClass::InstalledMatchesManifest,
        manifest_identity: Some(manifest.content_identity()),
        installed_identity: Some(installed.content_identity()),
        store_label: manifest.store_label(),
        diagnostic: None,
    })
}

fn action_for_report(report: &HostAssetDriftReport) -> Option<HostAssetAction> {
    match report.class() {
        HostAssetDriftClass::ManifestOnly => {
            Some(action(report, HostAssetActionKind::Install, "asset-install", report.manifest_identity()))
        }
        HostAssetDriftClass::IdentityMismatch => {
            Some(action(report, HostAssetActionKind::Update, "asset-update", report.manifest_identity()))
        }
        HostAssetDriftClass::InstalledDisabled => {
            Some(action(report, HostAssetActionKind::Enable, "asset-enable", report.manifest_identity()))
        }
        HostAssetDriftClass::HostOnly => {
            Some(action(report, HostAssetActionKind::Remove, "asset-remove", report.installed_identity()))
        }
        HostAssetDriftClass::InstalledMatchesManifest
        | HostAssetDriftClass::DuplicateAsset
        | HostAssetDriftClass::StaleObservation
        | HostAssetDriftClass::UnsupportedStore
        | HostAssetDriftClass::UntrustedAsset => None,
    }
}

fn action(
    report: &HostAssetDriftReport,
    kind: HostAssetActionKind,
    required_capability: &str,
    content_identity: Option<&str>,
) -> HostAssetAction {
    HostAssetAction {
        action_id: Text::from(action_id_for(report.asset_id(), kind)),
        asset_id: Text::from(report.asset_id()),
        kind,
        required_capability: Text::from(required_capability),
        content_identity: content_identity.map(Text::from),
    }
}

fn action_id_for(asset_id: &str, kind: HostAssetActionKind) -> Text {
    let suffix = match kind {
        HostAssetActionKind::Install => "install",
        HostAssetActionKind::Update => "update",
        HostAssetActionKind::Enable => "enable",
        HostAssetActionKind::Disable => "disable",
        HostAssetActionKind::Remove => "remove",
        HostAssetActionKind::WriteManifest => "write-manifest",
    };
    format!("{asset_id}:{suffix}")
}

fn safe_text(field: FieldName, value: &str) -> AssetResult<Text> {
    let normalized = validation::normalized_adapter_field(field, value).map_err(|error| map_adapter_error(&error))?;
    let lower = normalized.to_ascii_lowercase();
    if UNSAFE_LABEL_MARKERS.iter().any(|marker| lower.contains(marker)) {
        return Err(diagnostic(HostAssetDiagnosticCode::UnsafeMetadata, None, "asset metadata contains unsafe marker"));
    }
    Ok(normalized)
}

fn safe_store_label(value: &str) -> AssetResult<Text> {
    let store = safe_text(STORE_LABEL_FIELD, value)?;
    if SUPPORTED_STORE_LABELS.contains(&store.as_str()) {
        Ok(store)
    } else {
        Err(diagnostic(HostAssetDiagnosticCode::UnsupportedStore, None, "unsupported asset store label"))
    }
}

fn safe_hash(field: FieldName, value: &str) -> AssetResult<Text> {
    let normalized = validation::normalized_adapter_field(field, value).map_err(|error| map_adapter_error(&error))?;
    if Hash::new(&normalized).is_ok() {
        Ok(normalized)
    } else {
        Err(diagnostic(HostAssetDiagnosticCode::InvalidHash, None, "hash must be lowercase BLAKE3 hex"))
    }
}

fn validate_collection_limit(field: &'static str, len: usize) -> AssetResult<()> {
    if len > MAX_ASSET_ITEMS {
        return Err(HostAssetDiagnostic {
            code: HostAssetDiagnosticCode::TooManyItems,
            asset_id: None,
            message: Text::from(field),
        });
    }
    Ok(())
}

fn validate_non_empty_collection_bound(field: &'static str, len: usize) -> AssetResult<()> {
    if len == 0 {
        return Err(HostAssetDiagnostic {
            code: HostAssetDiagnosticCode::EmptyCollection,
            asset_id: None,
            message: Text::from(field),
        });
    }
    validate_collection_limit(field, len)
}

fn contains_asset_id(entries: &[HostAssetManifestEntry], asset_id: &str) -> bool {
    entries.iter().any(|entry| entry.asset_id() == asset_id)
}

fn contains_observation_id(entries: &[HostAssetObservation], asset_id: &str) -> bool {
    entries.iter().any(|entry| entry.asset_id() == asset_id)
}

fn find_manifest<'a>(entries: &'a [HostAssetManifestEntry], asset_id: &str) -> Option<&'a HostAssetManifestEntry> {
    entries.iter().find(|entry| entry.asset_id() == asset_id)
}

fn find_observation<'a>(entries: &'a [HostAssetObservation], asset_id: &str) -> Option<&'a HostAssetObservation> {
    entries.iter().find(|entry| entry.asset_id() == asset_id)
}

fn observation_is_stale(observed_epoch: u32, current_epoch: u32) -> bool {
    current_epoch.saturating_sub(observed_epoch) > STALE_OBSERVATION_AGE_EPOCHS
}

fn is_blocking_diagnostic(diagnostic: &HostAssetDiagnostic) -> bool {
    matches!(
        diagnostic.code(),
        HostAssetDiagnosticCode::DuplicateAsset
            | HostAssetDiagnosticCode::UnsupportedStore
            | HostAssetDiagnosticCode::StaleObservation
            | HostAssetDiagnosticCode::UnsafeMetadata
            | HostAssetDiagnosticCode::InvalidHash
    )
}

struct HostAssetReportInput<'a> {
    asset_id: &'a str,
    kind: HostAssetKind,
    class: HostAssetDriftClass,
    manifest_identity: Option<&'a str>,
    installed_identity: Option<&'a str>,
    store_label: &'a str,
    diagnostic: Option<HostAssetDiagnostic>,
}

fn report(input: HostAssetReportInput<'_>) -> HostAssetDriftReport {
    HostAssetDriftReport {
        asset_id: Text::from(input.asset_id),
        kind: input.kind,
        class: input.class,
        manifest_identity: input.manifest_identity.map(Text::from),
        installed_identity: input.installed_identity.map(Text::from),
        store_label: Text::from(input.store_label),
        diagnostic: input.diagnostic,
    }
}

fn diagnostic(code: HostAssetDiagnosticCode, asset_id: Option<&str>, message: &str) -> HostAssetDiagnostic {
    HostAssetDiagnostic {
        code,
        asset_id: asset_id.map(Text::from),
        message: Text::from(message),
    }
}

fn map_adapter_error(error: &AdapterContractError) -> HostAssetDiagnostic {
    match error {
        AdapterContractError::EmptyField { field } => diagnostic(HostAssetDiagnosticCode::EmptyField, None, field),
        AdapterContractError::FieldTooLong { field, .. } => {
            diagnostic(HostAssetDiagnosticCode::FieldTooLong, None, field)
        }
        AdapterContractError::NulCharacter { field, .. } => {
            diagnostic(HostAssetDiagnosticCode::NulCharacter, None, field)
        }
        AdapterContractError::InvalidEvidenceHash { field, .. } => {
            diagnostic(HostAssetDiagnosticCode::InvalidHash, None, field)
        }
        AdapterContractError::EmptyCollection { field } => {
            diagnostic(HostAssetDiagnosticCode::EmptyCollection, None, field)
        }
        AdapterContractError::UnsupportedStatusState { field, .. }
        | AdapterContractError::UnsupportedStatusObservationKind { field, .. }
        | AdapterContractError::UnsupportedSessionReferenceKind { field, .. }
        | AdapterContractError::UnsupportedWorkspaceValue { field, .. }
        | AdapterContractError::UnsupportedDataspaceValue { field, .. }
        | AdapterContractError::UnsupportedRouterValue { field, .. } => {
            diagnostic(HostAssetDiagnosticCode::UnsafeMetadata, None, field)
        }
    }
}

fn rejected_admission(
    status: HostAssetMutationStatus,
    code: HostAssetDiagnosticCode,
    asset_id: &str,
    message: &str,
) -> HostAssetMutationAdmission {
    let safe_asset_id = safe_text(ASSET_ID_FIELD, asset_id).ok();
    HostAssetMutationAdmission {
        status,
        request: None,
        diagnostic: Some(HostAssetDiagnostic {
            code,
            asset_id: safe_asset_id,
            message: Text::from(message),
        }),
    }
}

fn rejected_from_diagnostic(
    status: HostAssetMutationStatus,
    diagnostic: HostAssetDiagnostic,
) -> HostAssetMutationAdmission {
    HostAssetMutationAdmission {
        status,
        request: None,
        diagnostic: Some(diagnostic),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASSET_ID: &str = "skill-alpha";
    const HOST_ONLY_ASSET_ID: &str = "host-only-skill";
    const STORE_LABEL: &str = "skill-store";
    const PROVENANCE: &str = "embedded-bundle-alpha";
    const OBSERVATION_ID: &str = "obs-alpha";
    const CURRENT_EPOCH: u32 = 7;
    const OBSERVED_EPOCH: u32 = 7;
    const STALE_EPOCH: u32 = 1;
    const ASSET_HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const ASSET_HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const PLAN_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const UNSAFE_GLOBAL_PATH: &str = "/home/user/.hermes/skills/secret";
    const RECEIPT_ISSUE: &str = "asset-enabled";

    fn manifest<'a>(asset_id: &'a str, hash: &'a str) -> HostAssetManifestEntryDraft<'a> {
        HostAssetManifestEntryDraft {
            asset_id,
            kind: HostAssetKind::Skill,
            store_label: STORE_LABEL,
            content_identity: hash,
            provenance_label: PROVENANCE,
            trust: HostAssetTrust::Trusted,
        }
    }

    fn observation<'a>(
        asset_id: &'a str,
        hash: &'a str,
        enablement: HostAssetEnablement,
    ) -> HostAssetObservationDraft<'a> {
        HostAssetObservationDraft {
            asset_id,
            kind: HostAssetKind::Skill,
            store_label: STORE_LABEL,
            content_identity: hash,
            observation_id: OBSERVATION_ID,
            enablement,
            trust: HostAssetTrust::Trusted,
            observed_epoch: OBSERVED_EPOCH,
        }
    }

    fn inventory(
        manifests: &[HostAssetManifestEntryDraft<'_>],
        observations: &[HostAssetObservationDraft<'_>],
    ) -> HostAssetInventory {
        validate_host_asset_inventory(&HostAssetInventoryDraft {
            manifests,
            observations,
            current_epoch: CURRENT_EPOCH,
        })
        .unwrap()
    }

    #[test]
    fn inventory_and_drift_classification_accept_store_neutral_assets() {
        // r[verify agent_harness_runtime.hermes_asset_reconcile.inventory]
        // r[verify agent_harness_runtime.hermes_asset_reconcile.drift]
        let manifests = [manifest(ASSET_ID, ASSET_HASH_A)];
        let observations = [observation(ASSET_ID, ASSET_HASH_A, HostAssetEnablement::Enabled)];
        let inventory = inventory(&manifests, &observations);
        let reports = classify_host_asset_drift(&inventory);
        let plan = plan_host_asset_reconciliation(&reports);

        assert_eq!(inventory.manifests().len(), 1);
        assert_eq!(inventory.observations().len(), 1);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].class(), HostAssetDriftClass::InstalledMatchesManifest);
        assert_eq!(plan.status(), HostAssetPlanStatus::Noop);
        assert!(PROHIBITED_ASSET_RECONCILIATION_EFFECTS.contains(&"symlink resolution"));
    }

    #[test]
    fn manifest_only_and_disabled_assets_plan_dry_run_actions() {
        // r[verify agent_harness_runtime.hermes_asset_reconcile.plan]
        let manifests = [
            manifest(ASSET_ID, ASSET_HASH_A),
            manifest("skill-disabled", ASSET_HASH_A),
        ];
        let observations = [observation(
            "skill-disabled",
            ASSET_HASH_A,
            HostAssetEnablement::Disabled,
        )];
        let inventory = inventory(&manifests, &observations);
        let reports = classify_host_asset_drift(&inventory);
        let plan = plan_host_asset_reconciliation(&reports);

        assert_eq!(plan.status(), HostAssetPlanStatus::DryRun);
        assert_eq!(plan.actions().len(), 2);
        assert!(
            plan.actions()
                .iter()
                .any(|action| { action.asset_id() == ASSET_ID && action.kind() == HostAssetActionKind::Install })
        );
        assert!(
            plan.actions().iter().any(|action| {
                action.asset_id() == "skill-disabled" && action.kind() == HostAssetActionKind::Enable
            })
        );
    }

    #[test]
    fn identity_mismatch_and_host_only_assets_are_reported_without_authority_choice() {
        // r[verify agent_harness_runtime.hermes_asset_reconcile.drift]
        let manifests = [manifest(ASSET_ID, ASSET_HASH_A)];
        let observations = [
            observation(ASSET_ID, ASSET_HASH_B, HostAssetEnablement::Enabled),
            observation(HOST_ONLY_ASSET_ID, ASSET_HASH_A, HostAssetEnablement::Enabled),
        ];
        let inventory = inventory(&manifests, &observations);
        let reports = classify_host_asset_drift(&inventory);
        let plan = plan_host_asset_reconciliation(&reports);

        assert!(reports.iter().any(|report| {
            report.asset_id() == ASSET_ID && report.class() == HostAssetDriftClass::IdentityMismatch
        }));
        assert!(reports.iter().any(|report| {
            report.asset_id() == HOST_ONLY_ASSET_ID && report.class() == HostAssetDriftClass::HostOnly
        }));
        assert_eq!(plan.status(), HostAssetPlanStatus::Blocked);
        assert!(!plan.actions().iter().any(|action| { action.kind() == HostAssetActionKind::WriteManifest }));
    }

    #[test]
    fn unsafe_global_paths_duplicate_assets_and_stale_observations_fail_safely() {
        // r[verify agent_harness_runtime.hermes_asset_reconcile.verification]
        let unsafe_manifest = [HostAssetManifestEntryDraft {
            asset_id: UNSAFE_GLOBAL_PATH,
            ..manifest(ASSET_ID, ASSET_HASH_A)
        }];
        let unsafe_error = validate_host_asset_inventory(&HostAssetInventoryDraft {
            manifests: &unsafe_manifest,
            observations: &[],
            current_epoch: CURRENT_EPOCH,
        })
        .expect_err("global paths must be rejected");
        let duplicate_manifests = [manifest(ASSET_ID, ASSET_HASH_A), manifest(ASSET_ID, ASSET_HASH_A)];
        let duplicate_error = validate_host_asset_inventory(&HostAssetInventoryDraft {
            manifests: &duplicate_manifests,
            observations: &[observation(ASSET_ID, ASSET_HASH_A, HostAssetEnablement::Enabled)],
            current_epoch: CURRENT_EPOCH,
        })
        .expect_err("duplicate asset ids must fail");
        let stale_observation = [HostAssetObservationDraft {
            observed_epoch: STALE_EPOCH,
            ..observation(ASSET_ID, ASSET_HASH_A, HostAssetEnablement::Enabled)
        }];
        let stale_inventory = validate_host_asset_inventory(&HostAssetInventoryDraft {
            manifests: &[manifest(ASSET_ID, ASSET_HASH_A)],
            observations: &stale_observation,
            current_epoch: CURRENT_EPOCH,
        })
        .unwrap();
        let stale_reports = classify_host_asset_drift(&stale_inventory);
        let stale_plan = plan_host_asset_reconciliation(&stale_reports);

        assert_eq!(unsafe_error.code(), HostAssetDiagnosticCode::UnsafeMetadata);
        assert_eq!(duplicate_error.code(), HostAssetDiagnosticCode::DuplicateAsset);
        assert_eq!(stale_reports[0].class(), HostAssetDriftClass::StaleObservation);
        assert_eq!(stale_plan.status(), HostAssetPlanStatus::Blocked);
    }

    #[test]
    fn mutation_admission_requires_host_service_capability_and_matching_plan() {
        // r[verify agent_harness_runtime.hermes_asset_reconcile.host_owned]
        // r[verify agent_harness_runtime.hermes_asset_reconcile.plan]
        let manifests = [manifest(ASSET_ID, ASSET_HASH_A)];
        let inventory = inventory(&manifests, &[]);
        let reports = classify_host_asset_drift(&inventory);
        let plan = plan_host_asset_reconciliation(&reports);
        let action = &plan.actions()[0];
        let draft = HostAssetMutationRequestDraft {
            action_id: action.action_id(),
            asset_id: action.asset_id(),
            kind: action.kind(),
            capability_label: action.required_capability(),
            plan_identity: PLAN_HASH,
        };
        let admitted = admit_host_asset_mutation(&plan, &draft, HostAssetServiceStatus::Available);
        let missing = admit_host_asset_mutation(&plan, &draft, HostAssetServiceStatus::Missing);
        let mismatched = admit_host_asset_mutation(
            &plan,
            &HostAssetMutationRequestDraft {
                kind: HostAssetActionKind::Remove,
                ..draft
            },
            HostAssetServiceStatus::Available,
        );

        assert_eq!(admitted.status(), HostAssetMutationStatus::Admitted);
        assert_eq!(missing.status(), HostAssetMutationStatus::ServiceUnavailable);
        assert_eq!(mismatched.status(), HostAssetMutationStatus::PlanMismatch);
    }

    #[test]
    fn receipts_accept_hashes_and_reject_leaky_issue_codes() {
        // r[verify agent_harness_runtime.hermes_asset_reconcile.verification]
        let receipt = validate_host_asset_receipt(&HostAssetReconciliationReceiptDraft {
            status: "applied",
            action_hashes: &[PLAN_HASH],
            issue_code: Some(RECEIPT_ISSUE),
        })
        .unwrap();
        let leaking = validate_host_asset_receipt(&HostAssetReconciliationReceiptDraft {
            status: "failed",
            action_hashes: &[PLAN_HASH],
            issue_code: Some("raw prompt leaked"),
        })
        .expect_err("receipt issue must be safe");

        assert_eq!(receipt.status(), "applied");
        assert_eq!(receipt.action_hashes().len(), 1);
        assert_eq!(leaking.code(), HostAssetDiagnosticCode::UnsafeMetadata);
    }
}
