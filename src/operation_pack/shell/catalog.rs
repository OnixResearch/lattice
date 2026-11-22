//! Imperative shell for runbook catalogs carried by operation packs.
//!
//! Only `fetch` contacts a repository. Measurement, preview, installation and listing read local
//! files only and grant no effect authority.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "filesystem observations return typed catalog outcomes instead of terminating the process"
)]

pub mod fetch;
mod pinned;
mod placement;

const CONFIRM_FLAG: &str = "--confirm-create";

/// One admitted pack and the local `workflow-templates/` catalog that may receive its bundles.
#[derive(Debug, Clone, Copy)]
pub struct InstallRequest<'request> {
    pub pack_root: &'request std::path::Path,
    pub admitted: &'request super::super::model::AdmittedPack,
    pub catalog: &'request std::path::Path,
    pub confirm_create: bool,
}

/// Decide every bundle, then install each admitted one when the operator confirmed creation.
///
/// Before anything is staged, every declared bundle file is read and verified against the pack
/// declaration; any drift refuses the whole pack. The captured bytes are staged privately,
/// re-validated by the workflow-template loader, and each admitted bundle is renamed into the
/// catalog only while its id is absent and the catalog has room. A later per-bundle failure
/// rejects that bundle and never stops the others.
pub fn install(request: &InstallRequest<'_>) -> Result<InstallReport, super::super::Error> {
    super::installation::validate_directory(request.catalog)?;
    let parent = request.catalog.parent().ok_or(super::super::Error::Invalid("catalog parent"))?;
    super::installation::validate_directory(parent)?;
    let manifest = &request.admitted.candidate.manifest;
    if manifest.workflow_bundles.is_empty() {
        return Err(super::super::Error::Invalid("pack carries no workflow bundles"));
    }
    let catalog = cap_std::fs::Dir::open_ambient_dir(request.catalog, cap_std::ambient_authority())
        .map_err(|_| super::super::Error::Io("catalog authority"))?;
    let header = Header {
        pack_id: manifest.pack_id.clone(),
        pack_identity: request.admitted.candidate.pack_identity.clone(),
        lock_identity: Some(request.admitted.lock.identity.clone()),
    };
    if !request.confirm_create {
        let bundle_root = request.pack_root.join(super::super::catalog::BUNDLE_ROOT);
        let loaded = crate::workflow_template::load_catalog(&bundle_root)
            .map_err(|_| super::super::Error::Io("pack bundle root read"))?;
        let outcomes = placement::decide_against(manifest, &loaded, &catalog)?;
        return Ok(report(header, outcomes, false, Some(CONFIRM_FLAG)));
    }
    let (captured, drifted) = placement::capture(request);
    if !drifted.is_empty() {
        return Ok(report(header, refused_outcomes(manifest, &drifted), true, None));
    }
    let outcomes = placement::publish(request, parent, &catalog, &captured)?;
    Ok(report(header, outcomes, false, None))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InstallReport {
    pub schema: String,
    pub pack_id: String,
    pub pack_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_identity: Option<String>,
    pub bundles: Vec<super::super::catalog::decision::BundleOutcome>,
    pub installed_count: usize,
    pub rejected_count: usize,
    pub pack_refused: bool,
    pub grants_authority: bool,
    pub network_accessed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_flag: Option<String>,
}

impl InstallReport {
    /// Every bundle was admitted or installed; nothing was rejected or in conflict.
    pub fn is_complete(&self) -> bool {
        !self.pack_refused && self.rejected_count == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ListedBundle {
    pub id: String,
    pub bundle_identity: String,
    pub display_name: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ListedDiagnostic {
    pub bundle: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ListReport {
    pub bundles: Vec<ListedBundle>,
    pub diagnostics: Vec<ListedDiagnostic>,
    pub grants_authority: bool,
}

/// Measure every carried bundle file against its pack declaration and name each drifted bundle.
///
/// This reads only the local pack root. Any drift refuses the whole pack before admission, and
/// the report still names every drifted bundle.
pub fn drifted_bundles(
    root: &std::path::Path,
) -> Result<(super::super::model::Manifest, Vec<String>), super::super::Error> {
    let manifest = super::load_manifest(root)?;
    let mut drifted = Vec::with_capacity(manifest.workflow_bundles.len());
    for declaration in &manifest.workflow_bundles {
        let contents = super::super::catalog::bundle_contents(&manifest, &declaration.id);
        let has_drift = contents.values().any(|content| {
            super::read_bounded_regular(&root.join(&content.path), content.byte_length).map_or(true, |bytes| {
                u64::try_from(bytes.len()).ok() != Some(content.byte_length)
                    || super::super::identity::content_identity(&bytes) != content.content_identity
            })
        });
        if has_drift {
            drifted.push(declaration.id.clone());
        }
    }
    Ok((manifest, drifted))
}

/// Whole-pack refusal that still reports each bundle: drifted ones and the ones held back.
pub fn refusal_report(
    manifest: &super::super::model::Manifest,
    drifted: &[String],
) -> Result<InstallReport, super::super::Error> {
    let header = Header {
        pack_id: manifest.pack_id.clone(),
        pack_identity: super::super::identity::pack_identity(manifest)?,
        lock_identity: None,
    };
    Ok(report(header, refused_outcomes(manifest, drifted), true, None))
}

fn refused_outcomes(
    manifest: &super::super::model::Manifest,
    drifted: &[String],
) -> Vec<super::super::catalog::decision::BundleOutcome> {
    manifest
        .workflow_bundles
        .iter()
        .map(|declaration| {
            let is_drifted = drifted.contains(&declaration.id);
            super::super::catalog::decision::BundleOutcome {
                id: declaration.id.clone(),
                declared_identity: declaration.bundle_identity.clone(),
                status: super::super::catalog::decision::Status::Rejected,
                observed_identity: None,
                code: Some(if is_drifted { "pack_bytes_drift" } else { "pack_refused" }.to_string()),
                reason: Some(
                    if is_drifted {
                        "bundle bytes differ from the pack declaration"
                    } else {
                        "not installed because another bundle's bytes drifted and the pack is refused"
                    }
                    .to_string(),
                ),
            }
        })
        .collect()
}

/// List one local catalog through the workflow-template loader; no network, no authority.
pub fn list(catalog: &std::path::Path) -> Result<ListReport, super::super::Error> {
    super::installation::validate_directory(catalog)?;
    let loaded = crate::workflow_template::load_catalog(catalog)
        .map_err(|_| super::super::Error::Io("workflow-template catalog read"))?;
    Ok(ListReport {
        bundles: loaded
            .bundles
            .iter()
            .map(|bundle| ListedBundle {
                id: bundle.manifest.id.clone(),
                bundle_identity: format!("b3:{}", bundle.identity.as_str()),
                display_name: bundle.manifest.display_name.clone(),
                category: bundle.manifest.category.clone(),
            })
            .collect(),
        diagnostics: loaded
            .diagnostics
            .iter()
            .map(|diagnostic| ListedDiagnostic {
                bundle: diagnostic.bundle.clone(),
                code: diagnostic.code.as_str().to_string(),
                message: diagnostic.message.to_string(),
            })
            .collect(),
        grants_authority: false,
    })
}

/// Pack and lock identities that head an install report.
struct Header {
    pack_id: String,
    pack_identity: String,
    lock_identity: Option<String>,
}

fn report(
    header: Header,
    bundles: Vec<super::super::catalog::decision::BundleOutcome>,
    is_pack_refused: bool,
    required_flag: Option<&str>,
) -> InstallReport {
    let installed_count = bundles
        .iter()
        .filter(|bundle| bundle.status == super::super::catalog::decision::Status::Installed)
        .count();
    let rejected_count = bundles
        .iter()
        .filter(|bundle| {
            matches!(
                bundle.status,
                super::super::catalog::decision::Status::Rejected | super::super::catalog::decision::Status::Conflict
            )
        })
        .count();
    InstallReport {
        schema: super::super::catalog::decision::INSTALL_REPORT_SCHEMA.to_string(),
        pack_id: header.pack_id,
        pack_identity: header.pack_identity,
        lock_identity: header.lock_identity,
        bundles,
        installed_count,
        rejected_count,
        pack_refused: is_pack_refused,
        grants_authority: false,
        network_accessed: false,
        required_flag: required_flag.map(str::to_string),
    }
}
