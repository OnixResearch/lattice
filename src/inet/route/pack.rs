//! Create-only authoring of the opt-in exact `bounded_process` pack.
//!
//! The pack contains the profile's one-type operation catalog and an exact
//! copy of the selected Noble executable; its manifest binds both by BLAKE3
//! identity and byte length. Authoring never builds, downloads or selects an
//! executable: the caller supplies the pinned artifact, and ordinary pack
//! load, lock and admission decide whether it may run.

/// The pack identifier.
pub const PACK_ID: &str = "lattice.pure-inet-i64";
/// Lattice-owned catalog license.
pub const CATALOG_LICENSE: &str = "AGPL-3.0-or-later";
/// The Noble source declares no license; operators must admit this exact reference.
pub const PRODUCER_LICENSE: &str = "LicenseRef-noble-lang-undeclared";

const CATALOG_FILE: &str = "operation-catalog.ncl";
const EXECUTABLE_FILE: &str = "noble";
const RUNTIME_ID: &str = "noble-runtime";
const PLATFORM: &str = "x86_64-linux";
const ADMISSION_DOMAIN: &[u8] = b"lattice.pure-inet-i64.admission.v1";
#[cfg(unix)]
const EXECUTABLE_MODE: u32 = 0o555;

/// Author a new pack directory at `destination` around `executable`.
pub fn author(destination: &std::path::Path, executable: &std::path::Path) -> Result<(), crate::inet::Error> {
    let bytes = std::fs::read(executable).map_err(|_| crate::inet::Error::ArtifactUnavailable)?;
    std::fs::create_dir(destination).map_err(|_| crate::inet::Error::PackAuthoring)?;
    let runtime_path = destination.join(EXECUTABLE_FILE);
    write_new(&runtime_path, &bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&runtime_path, std::fs::Permissions::from_mode(EXECUTABLE_MODE))
            .map_err(|_| crate::inet::Error::PackAuthoring)?;
    }
    let catalog = crate::inet::route::catalog(&runtime_path)?;
    let catalog_source = nickel(&catalog)?;
    write_new(&destination.join(CATALOG_FILE), catalog_source.as_bytes())?;
    let manifest = manifest(&catalog, catalog_source.as_bytes(), &bytes)?;
    let manifest = crate::operation_pack::validate_manifest(manifest).map_err(|_| crate::inet::Error::PackAuthoring)?;
    write_new(&destination.join("manifest.ncl"), nickel(&manifest)?.as_bytes())
}

fn write_new(path: &std::path::Path, bytes: &[u8]) -> Result<(), crate::inet::Error> {
    use std::io::Write as _;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| crate::inet::Error::PackAuthoring)?;
    file.write_all(bytes).and_then(|()| file.sync_all()).map_err(|_| crate::inet::Error::PackAuthoring)
}

fn nickel(value: &impl serde::Serialize) -> Result<String, crate::inet::Error> {
    let value = serde_json::to_value(value).map_err(|_| crate::inet::Error::PackAuthoring)?;
    let body =
        crate::orchestration::format::render_nickel_value(&value, 0).map_err(|_| crate::inet::Error::PackAuthoring)?;
    Ok(format!("{body}\n"))
}

/// One declared pack content entry before measurement.
struct Declared {
    id: &'static str,
    role: crate::operation_pack::ContentRole,
    path: &'static str,
    media_class: &'static str,
}

const CATALOG_CONTENT: Declared = Declared {
    id: "operation-catalog",
    role: crate::operation_pack::ContentRole::OperationCatalog,
    path: CATALOG_FILE,
    media_class: "nickel",
};
const RUNTIME_CONTENT: Declared = Declared {
    id: "runtime-noble",
    role: crate::operation_pack::ContentRole::RuntimeArtifact,
    path: EXECUTABLE_FILE,
    media_class: "executable",
};

fn content(declared: &Declared, bytes: &[u8]) -> Result<crate::operation_pack::ContentItem, crate::inet::Error> {
    Ok(crate::operation_pack::ContentItem {
        id: declared.id.to_string(),
        role: declared.role,
        path: declared.path.to_string(),
        content_identity: crate::operation_pack::content_identity(bytes),
        byte_length: u64::try_from(bytes.len()).map_err(|_| crate::inet::Error::PackAuthoring)?,
        media_class: declared.media_class.to_string(),
    })
}

fn manifest(
    catalog: &crate::operation::Catalog,
    catalog_bytes: &[u8],
    executable_bytes: &[u8],
) -> Result<crate::operation_pack::Manifest, crate::inet::Error> {
    let definition = catalog.types.first().ok_or(crate::inet::Error::PackAuthoring)?;
    let capabilities = vec![crate::operation::model::CapabilityClass::ProcessRead];
    let licenses = vec![CATALOG_LICENSE.to_string(), PRODUCER_LICENSE.to_string()];
    debug_assert!(crate::inet::route::type_is_supported(definition));
    debug_assert!(!executable_bytes.is_empty() && !catalog_bytes.is_empty());
    Ok(crate::operation_pack::Manifest {
        schema: crate::operation_pack::model::MANIFEST_SCHEMA.to_string(),
        pack_id: PACK_ID.to_string(),
        description: "Opt-in exact pure interaction-net arithmetic through the pinned Noble net worker".to_string(),
        contents: vec![
            content(&CATALOG_CONTENT, catalog_bytes)?,
            content(&RUNTIME_CONTENT, executable_bytes)?,
        ],
        operations: vec![crate::operation_pack::OperationBinding {
            type_id: definition.id.clone(),
            type_revision: definition.revision.clone(),
            catalog_content_id: CATALOG_CONTENT.id.to_string(),
            runtime_id: RUNTIME_ID.to_string(),
            launch_label: crate::inet::route::RUNTIME_KEY.to_string(),
            methods: vec![crate::inet::route::METHOD.to_string()],
        }],
        runtimes: vec![crate::operation_pack::RuntimeArtifact {
            id: RUNTIME_ID.to_string(),
            content_id: RUNTIME_CONTENT.id.to_string(),
            kind: "bounded_process".to_string(),
            launch_labels: vec![crate::inet::route::RUNTIME_KEY.to_string()],
            platforms: vec![PLATFORM.to_string()],
            capabilities: capabilities.clone(),
            timeout_seconds: crate::inet::route::BOUNDS.timeout_seconds,
            input_bytes: crate::inet::route::BOUNDS.input_bytes,
            output_bytes: crate::inet::route::BOUNDS.output_bytes,
            clear_environment: true,
            wasm: None,
        }],
        analyzers: vec![],
        skills: vec![],
        dependencies: vec![],
        platforms: vec![PLATFORM.to_string()],
        admission: crate::operation_pack::AdmissionProfile {
            identity: crate::operation_pack::content_identity(ADMISSION_DOMAIN),
            required_evidence: vec![],
            allowed_licenses: licenses.clone(),
            allowed_capabilities: capabilities,
        },
        release: crate::operation_pack::ReleaseMetadata {
            display_version: "0.1.0".to_string(),
            channel: crate::operation_pack::ReleaseChannel::Beta,
            deprecated: false,
            superseded_by: None,
            publication_epoch_seconds: None,
        },
        licenses,
        probe: None,
        required_non_claims: crate::operation_pack::model::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    })
}
