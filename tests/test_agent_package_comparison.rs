//! Draft-schema fixture comparison of the NVIDIA agent-package proposal
//! (`NVIDIA/agent-package-spec` at `697872570744c8cfc1cf6b1d72e04db66e1a2c83`)
//! with Lattice bundle identities (Lattice #25, "NVIDIA research follow-up").
//!
//! This is a comparison, not materialization. No archive is read, extracted
//! or installed, no registry is contacted and no activation is performed. The
//! fixtures state the OCI manifest and Package Config as the draft shapes
//! them, plus the facts a materialization owner would report about the
//! payload; the harness compares those supplied facts and derives the
//! identities a Lattice adoption would bind. Provider-native OCI `sha256:`
//! digests are recorded verbatim and never become canonical identities; the
//! canonical identities are BLAKE3 under `content_identity_core` domains.
//! Registry, repository, tag and the version annotation are excluded.

// r[impl lattice.agent_revisions.package_comparison]
// r[impl lattice.agent_revisions.package_comparison.scenario.mapping]
// r[impl lattice.agent_revisions.package_comparison.scenario.mismatch]
// r[impl lattice.agent_revisions.package_comparison.scenario.lifecycle]
// r[verify lattice.agent_revisions.package_comparison]
// r[verify lattice.agent_revisions.package_comparison.scenario.mapping]
// r[verify lattice.agent_revisions.package_comparison.scenario.mismatch]
// r[verify lattice.agent_revisions.package_comparison.scenario.lifecycle]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use lattice_core::agent_revision::Refusal;
use lattice_core::agent_revision::facts::Bundle;
use lattice_core::agent_revision::ops::prepare;
use lattice_core::agent_revision::ops::selection as select;
use lattice_core::agent_revision::ops::sessions;
use lattice_core::agent_revision::ports as verdict;
use lattice_core::agent_revision::state::Lifecycle;
use lattice_core::agent_revision::state::SelectionKind;

const CONFIG_MEDIA_TYPE: &str = "application/vnd.agentpackage.config.v1+json";
const MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";
const PACKAGE_SCHEMA: &str = "agent.package/v1";
const PAYLOAD_ROOT: &str = "payload";
const REVISION_ANNOTATION: &str = "org.opencontainers.image.revision";
const FAMILY: &str = "rust-implementation";

/// The draft's two profiles: artifact type, payload kind and layer media type.
const PROFILES: [(&str, &str, &str); 2] = [
    (
        "application/vnd.agentpackage.skill.v1",
        "agent-skill",
        "application/vnd.agentpackage.skill.payload.v1.tar+gzip",
    ),
    (
        "application/vnd.agentpackage.plugin.v1",
        "agent-plugin",
        "application/vnd.agentpackage.plugin.payload.v1.tar+gzip",
    ),
];

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema: String,
    claim: String,
    draft: serde_json::Value,
    location: Location,
    resolved_manifest_digest: String,
    manifest: Manifest,
    config: Config,
    observation: Observation,
}

/// Where the package was fetched from; never part of an identity.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Location {
    registry: String,
    repository: String,
    tag: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    media_type: String,
    artifact_type: String,
    config: Descriptor,
    layers: Vec<Descriptor>,
    annotations: BTreeMap<String, String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Descriptor {
    media_type: String,
    digest: String,
    size: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Config {
    schema_version: String,
    payload: Payload,
    files: Vec<File>,
    annotations: BTreeMap<String, String>,
    extensions: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Payload {
    id: String,
    kind: String,
    spec: String,
    spec_version: String,
    root: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    path: String,
    digest: String,
    #[serde(default)]
    executable: bool,
}

/// What the materialization owner would report; supplied, never observed here.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    note: String,
    manifest_digest: String,
    config_digest: String,
    layer_digest: String,
    layer_bytes: u64,
    expanded_bytes: u64,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    kind: Kind,
    #[serde(default)]
    executable: bool,
    sha256: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Regular,
    Directory,
    Symlink,
    HardLink,
    Fifo,
    CharacterDevice,
    BlockDevice,
    Socket,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Bounds {
    max_entries: u64,
    max_expanded_bytes: u64,
    max_expansion_ratio: u64,
}

/// Comparison findings. The vocabulary is closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum Finding {
    UnsupportedEnvelope,
    KindMismatch,
    LayerCount,
    LayerMediaTypeMismatch,
    ManifestDigestMismatch,
    ConfigDigestMismatch,
    LayerDigestMismatch,
    /// The observed layer size differs from the manifest descriptor's.
    LayerSizeMismatch,
    UnsafePath,
    DuplicateInventoryPath,
    DuplicateEntry,
    PathCollision,
    Symlink,
    HardLink,
    SpecialFile,
    MissingFile,
    ExtraFile,
    ChangedFile,
    /// The reported SHA-256 differs from the digest of the supplied bytes.
    ReportedDigestMismatch,
    ExecutableMismatch,
    EntryCountExceeded,
    ExpandedBytesExceeded,
    ExpansionRatioExceeded,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Cases {
    schema: String,
    claim: String,
    bounds: Bounds,
    cases: Vec<Case>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    base: String,
    mutations: Vec<Mutation>,
    expected: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Mutation {
    RemoveEntry {
        path: String,
    },
    AddEntry {
        entry: Entry,
    },
    ReplaceContent {
        path: String,
        sha256: String,
        content: String,
    },
    DuplicateEntry {
        path: String,
    },
    DuplicateInventory {
        path: String,
    },
    Rename {
        path: String,
        to: String,
    },
    SetExpandedBytes {
        bytes: u64,
    },
    SetObservedLayerBytes {
        bytes: u64,
    },
    AddDirectories {
        count: u32,
    },
    SetArtifactType {
        value: String,
    },
    SetLayerMediaType {
        value: String,
    },
    AddLayer {},
    SetObservedLayerDigest {
        digest: String,
    },
    SetObservedConfigDigest {
        digest: String,
    },
    SetObservedManifestDigest {
        digest: String,
    },
    SetExecutable {
        path: String,
        value: bool,
    },
}

const CLAIM: &str = "draft_schema_fixture_comparison_not_materialization";

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/agent-package").join(name)
}

fn load(name: &str) -> Fixture {
    let fixture: Fixture =
        serde_json::from_slice(&std::fs::read(fixture_path(name)).expect("fixture read")).expect("fixture decodes");
    assert_eq!(fixture.schema, "lattice.agent-package-comparison-fixture.v1");
    assert_eq!(fixture.claim, CLAIM);
    assert!(!fixture.observation.note.is_empty());
    fixture
}

fn cases() -> Cases {
    let cases: Cases = serde_json::from_slice(&std::fs::read(fixture_path("negative-cases.json")).expect("cases read"))
        .expect("cases decode");
    assert_eq!(cases.schema, "lattice.agent-package-comparison-cases.v1");
    assert_eq!(cases.claim, CLAIM);
    cases
}

fn is_sha256(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')))
}

/// The `sha256:` digest of supplied bytes, as the draft spells it. SHA-256
/// comes from the hash provider of the *ring* TLS 1.3 suite
/// `TLS13_AES_128_GCM_SHA256` of `rustls`, an existing dev-dependency built
/// with its `ring` feature, so the harness adds no dependency.
fn sha256(content: &str) -> String {
    let suite = rustls::crypto::ring::cipher_suite::TLS13_AES_128_GCM_SHA256.tls13().expect("a TLS 1.3 suite");
    let hash = suite.common.hash_provider;
    assert_eq!(hash.algorithm(), rustls::crypto::hash::HashAlgorithm::SHA256);
    format!("sha256:{}", hex::encode(hash.hash(content.as_bytes()).as_ref()))
}

#[test]
fn the_sha256_provider_matches_the_standard_vectors() {
    let vectors = [
        ("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        ("abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        (
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ];
    for (input, expected) in vectors {
        assert_eq!(sha256(input), format!("sha256:{expected}"), "{input:?}");
    }
    for name in ["skill.json", "plugin.json"] {
        let fixture = load(name);
        let inventory = fixture
            .config
            .files
            .iter()
            .map(|file| (file.path.as_str(), file.digest.as_str()))
            .collect::<BTreeMap<_, _>>();
        for entry in fixture.observation.entries.iter().filter(|entry| entry.kind == Kind::Regular) {
            let computed = sha256(entry.content.as_deref().expect("regular content"));
            assert_eq!(entry.sha256.as_deref(), Some(computed.as_str()), "{name}: {}", entry.path);
            assert_eq!(inventory.get(entry.path.as_str()).copied(), Some(computed.as_str()), "{name}: {}", entry.path);
        }
    }
}

/// One skill's file tree under `prefix`: payload-relative paths with the
/// BLAKE3 of their bytes and their executable bit.
fn skill_tree(files: &BTreeSet<(String, String, bool)>, prefix: &str) -> serde_json::Value {
    let members = files
        .iter()
        .filter_map(|(path, digest, executable)| {
            Some(serde_json::json!([path.strip_prefix(prefix)?, digest, executable]))
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "files": members.len(),
        "identity": blake3_identity("lattice-agent-package-skill-tree-v1", &serde_json::json!(members)),
    })
}

/// The draft's path rules: relative, under `payload/`, `/`-separated, no
/// empty, `.` or `..` component, backslash, NUL or drive prefix.
fn is_safe_path(path: &str, kind: Kind) -> bool {
    if kind == Kind::Directory && (path == PAYLOAD_ROOT || path == "payload/") {
        return true;
    }
    let trimmed = if kind == Kind::Directory {
        path.strip_suffix('/').unwrap_or(path)
    } else {
        path
    };
    let Some(relative) = trimmed.strip_prefix("payload/") else {
        return false;
    };
    let is_clean = !path.contains('\\') && !path.contains('\0');
    is_clean && relative.split('/').all(|component| !component.is_empty() && component != "." && component != "..")
}

fn envelope_findings(fixture: &Fixture, findings: &mut BTreeSet<Finding>) {
    let (manifest, config) = (&fixture.manifest, &fixture.config);
    let is_supported = manifest.schema_version == 2
        && manifest.media_type == MANIFEST_MEDIA_TYPE
        && manifest.config.media_type == CONFIG_MEDIA_TYPE
        && config.schema_version == PACKAGE_SCHEMA
        && config.payload.root == PAYLOAD_ROOT
        && !config.annotations.contains_key(REVISION_ANNOTATION)
        && config.extensions.is_empty()
        && [&fixture.resolved_manifest_digest, &manifest.config.digest].iter().all(|digest| is_sha256(digest));
    if !is_supported {
        findings.insert(Finding::UnsupportedEnvelope);
    }
    let profile = PROFILES.iter().find(|(artifact_type, ..)| *artifact_type == manifest.artifact_type);
    if profile.is_none_or(|(_, kind, _)| *kind != config.payload.kind) {
        findings.insert(Finding::KindMismatch);
    }
    if manifest.layers.len() != 1 {
        findings.insert(Finding::LayerCount);
    }
    let layer = manifest.layers.first();
    if layer.is_none_or(|layer| profile.is_none_or(|(_, _, media_type)| *media_type != layer.media_type)) {
        findings.insert(Finding::LayerMediaTypeMismatch);
    }
    let observed = &fixture.observation;
    let digests = [
        (&fixture.resolved_manifest_digest, &observed.manifest_digest, Finding::ManifestDigestMismatch),
        (&manifest.config.digest, &observed.config_digest, Finding::ConfigDigestMismatch),
    ];
    for (declared, reported, finding) in digests {
        if declared != reported {
            findings.insert(finding);
        }
    }
    if layer.is_none_or(|layer| layer.digest != observed.layer_digest || !is_sha256(&layer.digest)) {
        findings.insert(Finding::LayerDigestMismatch);
    }
}

fn inventory_findings(config: &Config, findings: &mut BTreeSet<Finding>) {
    let mut seen = BTreeSet::new();
    for file in &config.files {
        if !is_safe_path(&file.path, Kind::Regular) {
            findings.insert(Finding::UnsafePath);
        }
        if !is_sha256(&file.digest) {
            findings.insert(Finding::UnsupportedEnvelope);
        }
        if !seen.insert(file.path.as_str()) {
            findings.insert(Finding::DuplicateInventoryPath);
        }
    }
}

fn entry_findings(fixture: &Fixture, findings: &mut BTreeSet<Finding>) {
    let inventory = fixture.config.files.iter().map(|file| (file.path.as_str(), file)).collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut folded = BTreeMap::new();
    for entry in &fixture.observation.entries {
        if !is_safe_path(&entry.path, entry.kind) {
            findings.insert(Finding::UnsafePath);
        }
        if !seen.insert(entry.path.as_str()) {
            findings.insert(Finding::DuplicateEntry);
        }
        let previous = folded.insert(entry.path.to_lowercase(), entry.path.as_str());
        if previous.is_some_and(|previous| previous != entry.path) {
            findings.insert(Finding::PathCollision);
        }
        match entry.kind {
            Kind::Regular => {
                // The bytes decide: a reported digest is checked against them,
                // never trusted in their place. Missing bytes cannot match.
                let computed = entry.content.as_deref().map(sha256);
                if computed.is_none() || entry.sha256 != computed {
                    findings.insert(Finding::ReportedDigestMismatch);
                }
                match inventory.get(entry.path.as_str()) {
                    None => {
                        findings.insert(Finding::ExtraFile);
                    }
                    Some(file) => {
                        if computed.as_deref() != Some(file.digest.as_str()) {
                            findings.insert(Finding::ChangedFile);
                        }
                        if entry.executable != file.executable {
                            findings.insert(Finding::ExecutableMismatch);
                        }
                    }
                }
            }
            Kind::Directory => {}
            Kind::Symlink => {
                findings.insert(Finding::Symlink);
            }
            Kind::HardLink => {
                findings.insert(Finding::HardLink);
            }
            Kind::Fifo | Kind::CharacterDevice | Kind::BlockDevice | Kind::Socket => {
                findings.insert(Finding::SpecialFile);
            }
        }
    }
    let regular = fixture
        .observation
        .entries
        .iter()
        .filter(|entry| entry.kind == Kind::Regular)
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    if inventory.keys().any(|path| !regular.contains(path)) {
        findings.insert(Finding::MissingFile);
    }
}

fn bound_findings(fixture: &Fixture, bounds: Bounds, findings: &mut BTreeSet<Finding>) {
    let observation = &fixture.observation;
    let entries = u64::try_from(observation.entries.len()).expect("entry count fits");
    if entries > bounds.max_entries {
        findings.insert(Finding::EntryCountExceeded);
    }
    if observation.expanded_bytes > bounds.max_expanded_bytes {
        findings.insert(Finding::ExpandedBytesExceeded);
    }
    // The ratio is accepted only over a compressed size both sides agree on;
    // on disagreement the smaller size bounds the expansion.
    let declared_bytes = fixture.manifest.layers.first().map(|layer| layer.size);
    if declared_bytes != Some(observation.layer_bytes) {
        findings.insert(Finding::LayerSizeMismatch);
    }
    let compressed_bytes =
        declared_bytes.map_or(observation.layer_bytes, |declared| declared.min(observation.layer_bytes));
    let ratio_bound = compressed_bytes.checked_mul(bounds.max_expansion_ratio);
    if ratio_bound.is_none_or(|bound| observation.expanded_bytes > bound) {
        findings.insert(Finding::ExpansionRatioExceeded);
    }
}

/// Compare the declared envelope and inventory with the supplied payload
/// facts. An empty set means the facts agree; it never means the payload
/// was safely materialized.
fn compare(fixture: &Fixture, bounds: Bounds) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    envelope_findings(fixture, &mut findings);
    inventory_findings(&fixture.config, &mut findings);
    entry_findings(fixture, &mut findings);
    bound_findings(fixture, bounds, &mut findings);
    findings
}

fn apply(fixture: &mut Fixture, mutation: &Mutation) {
    let observation = &mut fixture.observation;
    let entry = |entries: &[Entry], path: &str| -> usize {
        entries.iter().position(|entry| entry.path == path).expect("mutated entry exists")
    };
    match mutation {
        Mutation::RemoveEntry { path } => {
            let index = entry(&observation.entries, path);
            observation.entries.remove(index);
        }
        Mutation::AddEntry { entry } => observation.entries.push(entry.clone()),
        Mutation::ReplaceContent { path, sha256, content } => {
            let index = entry(&observation.entries, path);
            observation.entries[index].sha256 = Some(sha256.clone());
            observation.entries[index].content = Some(content.clone());
        }
        Mutation::DuplicateEntry { path } => {
            let index = entry(&observation.entries, path);
            let duplicate = observation.entries[index].clone();
            observation.entries.push(duplicate);
        }
        Mutation::DuplicateInventory { path } => {
            let file = fixture.config.files.iter().find(|file| file.path == *path).expect("inventory path").clone();
            fixture.config.files.push(file);
        }
        Mutation::Rename { path, to } => {
            let index = entry(&observation.entries, path);
            observation.entries[index].path.clone_from(to);
            let file = fixture.config.files.iter_mut().find(|file| file.path == *path).expect("inventory path");
            file.path.clone_from(to);
        }
        Mutation::SetExpandedBytes { bytes } => observation.expanded_bytes = *bytes,
        Mutation::SetObservedLayerBytes { bytes } => observation.layer_bytes = *bytes,
        Mutation::AddDirectories { count } => {
            for index in 0..*count {
                observation.entries.push(Entry {
                    path: format!("payload/d{index:03}"),
                    kind: Kind::Directory,
                    executable: false,
                    sha256: None,
                    content: None,
                });
            }
        }
        Mutation::SetArtifactType { value } => fixture.manifest.artifact_type.clone_from(value),
        Mutation::SetLayerMediaType { value } => fixture.manifest.layers[0].media_type.clone_from(value),
        Mutation::AddLayer {} => {
            let layer = fixture.manifest.layers[0].clone();
            fixture.manifest.layers.push(layer);
        }
        Mutation::SetObservedLayerDigest { digest } => observation.layer_digest.clone_from(digest),
        Mutation::SetObservedConfigDigest { digest } => observation.config_digest.clone_from(digest),
        Mutation::SetObservedManifestDigest { digest } => observation.manifest_digest.clone_from(digest),
        Mutation::SetExecutable { path, value } => {
            let index = entry(&observation.entries, path);
            observation.entries[index].executable = *value;
        }
    }
}

fn identity(domain: &str, part: &[u8]) -> String {
    let domain = content_identity_core::Domain::new(domain).expect("domain");
    let mut hasher = content_identity_core::IdentityHasher::new(domain);
    hasher.update_hashed_part(b"canonical", part);
    hasher.finish().to_hex().to_string()
}

fn blake3_identity(domain: &str, value: &serde_json::Value) -> String {
    format!("blake3:{}", identity(domain, &serde_json::to_vec(value).expect("canonical")))
}

/// The identities a Lattice adoption would bind, for facts that agree.
fn mapping(fixture: &Fixture) -> serde_json::Value {
    let mut inventory = fixture
        .config
        .files
        .iter()
        .map(|file| serde_json::json!([file.path, file.digest, file.executable]))
        .collect::<Vec<_>>();
    inventory.sort_by_key(|value| value[0].as_str().expect("path").to_owned());
    let inventory_identity = blake3_identity("lattice-agent-package-inventory-v1", &serde_json::json!(inventory));
    let files = fixture
        .observation
        .entries
        .iter()
        .filter(|entry| entry.kind == Kind::Regular)
        .map(|entry| {
            let relative = entry.path.strip_prefix("payload/").expect("safe path");
            let content = entry.content.as_deref().expect("regular content");
            (
                relative.to_owned(),
                format!("blake3:{}", blake3::hash(content.as_bytes()).to_hex()),
                entry.executable,
            )
        })
        .collect::<BTreeSet<_>>();
    let materialized_identity = blake3_identity("lattice-agent-package-materialized-v1", &serde_json::json!(files));
    let bundled = files
        .iter()
        .filter_map(|(path, _, _)| {
            let id = path.strip_prefix("skills/")?.strip_suffix("/SKILL.md")?;
            Some(serde_json::json!({ "id": id, "tree": skill_tree(&files, &format!("skills/{id}/")) }))
        })
        .collect::<Vec<_>>();
    let closure = serde_json::json!({ "external": [], "bundled_skills": bundled });
    let closure_identity = blake3_identity("lattice-agent-package-closure-v1", &closure);
    let subject = serde_json::json!({
        "artifact_type": fixture.manifest.artifact_type,
        "payload_id": fixture.config.payload.id,
        "payload_spec": [fixture.config.payload.spec, fixture.config.payload.spec_version],
        "manifest_digest": fixture.resolved_manifest_digest,
        "inventory_identity": inventory_identity,
        "materialized_identity": materialized_identity,
        "closure_identity": closure_identity,
    });
    let evidence_subject = blake3_identity("lattice-agent-package-subject-v1", &subject);
    serde_json::json!({
        "source_revision": fixture.manifest.annotations.get(REVISION_ANNOTATION),
        "provider_native": {
            "algorithm": "sha256",
            "manifest_digest": fixture.resolved_manifest_digest,
            "config_digest": fixture.manifest.config.digest,
            "layer_digest": fixture.manifest.layers[0].digest,
            "layer_bytes": fixture.manifest.layers[0].size,
            "file_digests": fixture.config.files.len(),
        },
        "excluded_from_identity": ["registry", "repository", "tag"],
        "version_annotation": "not a direct identity input; it is part of the manifest bytes, so changing it changes the manifest digest, the evidence subject and the lifecycle revision",
        "inventory_identity": inventory_identity,
        "materialized_identity": materialized_identity,
        "file_identities": files.iter().map(|(path, digest, executable)| serde_json::json!({
            "path": path, "blake3": digest, "executable": executable,
        })).collect::<Vec<_>>(),
        "dependency_closure": { "identity": closure_identity, "closure": closure, "composition_by_reference": "unsupported" },
        "skill_tree": if fixture.config.payload.kind == "agent-skill" { skill_tree(&files, "") } else { serde_json::Value::Null },
        "evidence_subject": evidence_subject,
    })
}

/// The comparison's lifecycle subject: a package definition per artifact
/// type and payload identifier, and the evidence subject as its revision.
fn lifecycle_bundle(fixture: &Fixture) -> Bundle {
    let mapped = mapping(fixture);
    let subject = mapped["evidence_subject"].as_str().expect("subject");
    let definition = serde_json::json!([fixture.manifest.artifact_type, fixture.config.payload.id]);
    Bundle {
        schema: lattice_core::agent_revision::facts::BUNDLE_SCHEMA.into(),
        definition: blake3_identity("lattice-agent-package-definition-v1", &definition),
        definition_name: fixture.config.payload.id.clone(),
        revision: subject.strip_prefix("blake3:").expect("bare subject").into(),
        adapter: "agent-package".into(),
        protocol_revision: PACKAGE_SCHEMA.into(),
        executable_identity: mapped["materialized_identity"].as_str().expect("materialized").into(),
        compute: lattice_core::execution_binding::request::ComputeRequirement::HostLocal {},
        task_families: Some(vec![FAMILY.into()]),
        tools: None,
        routes: Vec::new(),
        workflow: None,
    }
}

fn bounds() -> Bounds {
    cases().bounds
}

#[test]
fn reviewed_skill_and_plugin_fixtures_agree_and_map_onto_identities() {
    for name in ["skill.json", "plugin.json"] {
        let fixture = load(name);
        assert_eq!(compare(&fixture, bounds()), BTreeSet::new(), "{name}: supplied facts agree");
        let mapped = mapping(&fixture);
        assert!(mapped["source_revision"].is_string(), "{name}: source revision comes from the manifest");
        let encoded = mapped.to_string();
        let tag = format!("\"{}\"", fixture.location.tag);
        for location in [&fixture.location.registry, &fixture.location.repository, &tag] {
            assert!(!encoded.contains(location.as_str()), "{name}: location {location} is not identity");
        }
        assert!(mapped["evidence_subject"].as_str().is_some_and(lattice_core::agent_revision::is_reference));
        assert_ne!(mapped["inventory_identity"], mapped["materialized_identity"]);
        let bundle = lifecycle_bundle(&fixture);
        assert!(lattice_core::agent_revision::facts::identity(&bundle).is_ok(), "{name}: a lifecycle bundle");
    }
    let plugin_fixture = load("plugin.json");
    let plugin = mapping(&plugin_fixture);
    let bundled = &plugin["dependency_closure"]["closure"]["bundled_skills"];
    assert_eq!(bundled.as_array().map(Vec::len), Some(2));
    let skill = mapping(&load("skill.json"));
    assert_eq!(skill["skill_tree"]["files"], 3, "SKILL.md, references/policy.md and scripts/analyze.py");
    assert_eq!(bundled[0]["id"], "case-triage");
    assert_eq!(bundled[0]["tree"], skill["skill_tree"], "the bundled skill carries the reviewed skill's whole tree");
    let mut incomplete = plugin_fixture.clone();
    apply(&mut incomplete, &Mutation::RemoveEntry {
        path: "payload/skills/case-triage/references/policy.md".into(),
    });
    assert!(compare(&incomplete, bounds()).contains(&Finding::MissingFile));
    let partial = mapping(&incomplete);
    assert_ne!(
        partial["dependency_closure"]["closure"]["bundled_skills"][0]["tree"], skill["skill_tree"],
        "a bundled skill missing a referenced file is not the reviewed skill"
    );
    assert_ne!(plugin["evidence_subject"], skill["evidence_subject"], "the skill's approval is not the plugin's");
}

/// The findings one negative case actually produces.
fn case_findings(case: &Case, bounds: Bounds) -> BTreeSet<String> {
    let mut fixture = load(&format!("{}.json", case.base));
    for mutation in &case.mutations {
        apply(&mut fixture, mutation);
    }
    compare(&fixture, bounds)
        .into_iter()
        .map(|finding| serde_json::to_value(finding).expect("encodes").as_str().expect("code").to_owned())
        .collect()
}

#[test]
fn every_negative_case_reports_exactly_its_findings() {
    let cases = cases();
    for case in &cases.cases {
        let expected = case.expected.iter().cloned().collect::<BTreeSet<_>>();
        assert_eq!(case_findings(case, cases.bounds), expected, "{}", case.name);
    }
    assert_eq!(cases.cases.len(), 34);
}

fn republished(base: &Fixture) -> Fixture {
    let mut changed = base.clone();
    let digest = "sha256:232b309b1901de211e290cf532b1f0b8ca93e8368a97f7dc7b147f92b749637e";
    changed
        .config
        .files
        .iter_mut()
        .find(|file| file.path == "payload/SKILL.md")
        .expect("SKILL.md")
        .digest = digest.into();
    let content = "---\nname: case-triage\ndescription: Triage support cases using the reviewed procedure.\n---\n\n# Case triage\n\nSkip references/policy.md.\n";
    apply(&mut changed, &Mutation::ReplaceContent {
        path: "payload/SKILL.md".into(),
        sha256: digest.into(),
        content: content.into(),
    });
    changed.resolved_manifest_digest = "sha256:6628a483486226e5eefc4dec56db352291d24dfcaf11573a842b6e502ac84e95".into();
    changed.observation.manifest_digest.clone_from(&changed.resolved_manifest_digest);
    changed.manifest.config.digest = "sha256:612ee4cf7d307b30504a782d9e4f66c50471958dc1dc2fcbd55946c6a2ef230c".into();
    changed.observation.config_digest.clone_from(&changed.manifest.config.digest);
    changed.manifest.layers[0].digest =
        "sha256:9f80f7280d9860a21a5eef115a2c0fe21e6a0b08c69e67479843dc1a4738bb87".into();
    changed.observation.layer_digest.clone_from(&changed.manifest.layers[0].digest);
    changed
}

fn ok<T>(result: Result<lattice_core::agent_revision::ops::Applied<T>, Refusal>) -> (Lifecycle, T) {
    let applied = result.expect("transition applies");
    (applied.lifecycle, applied.outcome)
}

fn qualification_ref(bundle: &Bundle) -> String {
    format!("blake3:{}", blake3::hash(format!("evaluation:{}", bundle.revision).as_bytes()).to_hex())
}

fn answer(bundle: &Bundle, reference: &str) -> verdict::Verdict {
    verdict::Verdict::Qualified(verdict::Attachment {
        qualification_ref: reference.into(),
        subject: verdict::Subject {
            definition_identity: bundle.definition.clone(),
            profile_revision: bundle.revision.clone(),
            task_family: FAMILY.into(),
        },
        evidence_class: verdict::EvidenceClass::TaskFamilyEvaluation,
        protocol_identity: format!("blake3:{}", "a".repeat(64)),
        verifier_identity: format!("blake3:{}", "b".repeat(64)),
        observed_at_ms: 1_795_200_000_000,
        provenance: verdict::Provenance::Fixture,
    })
}

fn command(kind: SelectionKind, bundle: &Bundle, reference: &str, expected: u64) -> select::Command {
    select::Command {
        kind,
        request_id: format!("{kind:?}-{}", &bundle.revision[..8]),
        revision: bundle.revision.clone(),
        qualification_ref: reference.into(),
        policy: lattice_core::agent_revision::state::Policy {
            task_family: FAMILY.into(),
            required_class: verdict::EvidenceClass::TaskFamilyEvaluation,
        },
        expected_rollout_generation: expected,
    }
}

fn prepared(lifecycle: &Lifecycle, bundle: &Bundle) -> Lifecycle {
    let (desired, _) = ok(prepare::desire(lifecycle, bundle));
    let (pending, outcome) = ok(prepare::begin(&desired, &bundle.revision));
    let prepare::Begin::Started { ticket } = outcome else {
        panic!("expected a started preparation");
    };
    let observation = verdict::Prepared::Prepared {
        artifacts: vec![format!("blake3:{}", bundle.revision)],
    };
    ok(prepare::record(&pending, &ticket, &observation)).0
}

fn promoted(lifecycle: &Lifecycle, bundle: &Bundle) -> Lifecycle {
    let reference = qualification_ref(bundle);
    let promotion = command(SelectionKind::Promotion, bundle, &reference, lifecycle.rollout_generation);
    ok(select::select(lifecycle, &promotion, &answer(bundle, &reference))).0
}

fn admitted(lifecycle: &Lifecycle, session: &str, bundle: &Bundle) -> Lifecycle {
    ok(sessions::admit(lifecycle, sessions::Entry {
        session,
        revision: &bundle.revision,
    }))
    .0
}

#[test]
fn payload_change_relocation_revocation_and_rollback_use_the_existing_lifecycle() {
    let reviewed = load("skill.json");
    let r1 = lifecycle_bundle(&reviewed);
    let changed_fixture = republished(&reviewed);
    assert_eq!(compare(&changed_fixture, bounds()), BTreeSet::new(), "the republished package is self-consistent");
    let r2 = lifecycle_bundle(&changed_fixture);
    assert_eq!(changed_fixture.manifest.annotations, reviewed.manifest.annotations, "same name and version");
    assert_eq!(
        (r1.definition.as_str(), r1.definition_name.as_str()),
        (r2.definition.as_str(), r2.definition_name.as_str())
    );
    assert_ne!(r1.revision, r2.revision, "a changed digest is a new revision");

    let mut relocated = reviewed.clone();
    relocated.location = Location {
        registry: "mirror.invalid".into(),
        repository: "mirror/case-triage".into(),
        tag: "reviewed".into(),
    };
    assert_eq!(lifecycle_bundle(&relocated), r1, "relocation by the same digest keeps the revision");
    let mut relabelled = reviewed.clone();
    relabelled.manifest.annotations.insert("org.opencontainers.image.version".into(), "1.2.4".into());
    assert_eq!(lifecycle_bundle(&relabelled), r1, "the version annotation is not a direct identity input");
    let republished_manifest = "sha256:6628a483486226e5eefc4dec56db352291d24dfcaf11573a842b6e502ac84e95";
    relabelled.resolved_manifest_digest = republished_manifest.into();
    relabelled.observation.manifest_digest = republished_manifest.into();
    assert_eq!(compare(&relabelled, bounds()), BTreeSet::new());
    assert_ne!(
        lifecycle_bundle(&relabelled).revision,
        r1.revision,
        "the manifest digest it changes is an identity input"
    );

    let start = admitted(&promoted(&prepared(&Lifecycle::new(&r1.definition), &r1), &r1), "session-a", &r1);
    let (same, outcome) = ok(prepare::desire(&start, &lifecycle_bundle(&relocated)));
    assert!(matches!(outcome, prepare::Desire::Unchanged { .. }));
    assert_eq!(same, start);

    let with_r2 = prepared(&start, &r2);
    let r1_ref = qualification_ref(&r1);
    let stale = command(SelectionKind::Promotion, &r2, &r1_ref, 1);
    assert_eq!(select::select(&with_r2, &stale, &answer(&r1, &r1_ref)).unwrap_err(), Refusal::QualificationMismatch);
    let fresh = format!("blake3:{}", "c".repeat(64));
    let inherited = command(SelectionKind::Promotion, &r2, &fresh, 1);
    assert_eq!(select::select(&with_r2, &inherited, &answer(&r1, &fresh)).unwrap_err(), Refusal::NotQualified {
        reason: verdict::Unqualified::SubjectMismatch,
    });

    let r2_live = admitted(&promoted(&with_r2, &r2), "session-b", &r2);
    let (revoked, _) = ok(sessions::revoke(&r2_live, &sessions::Target::Revision(r2.revision.clone())));
    assert_eq!(sessions::effect_permitted(&revoked, "session-b").unwrap_err(), Refusal::RevisionRevoked);
    assert_eq!(
        sessions::admit(&revoked, sessions::Entry {
            session: "session-c",
            revision: &r2.revision
        })
        .unwrap_err(),
        Refusal::RevisionRevoked
    );
    let rollback = command(SelectionKind::Rollback, &r1, &r1_ref, revoked.rollout_generation);
    let (rolled, _) = ok(select::select(&revoked, &rollback, &answer(&r1, &r1_ref)));
    let with_c = admitted(&rolled, "session-c", &r1);
    assert_eq!(with_c.sessions["session-c"].revision, r1.revision, "new sessions use the rolled-back digest");
    assert_eq!(
        sessions::effect_permitted(&with_c, "session-b").unwrap_err(),
        Refusal::RevisionRevoked,
        "a historical pin never restores a revoked digest"
    );
    assert_eq!(sessions::effect_permitted(&with_c, "session-a").expect("A acts").revision, r1.revision);

    let (r1_revoked, _) = ok(sessions::revoke(&with_c, &sessions::Target::Revision(r1.revision.clone())));
    let back = command(SelectionKind::Rollback, &r2, &qualification_ref(&r2), r1_revoked.rollout_generation);
    assert_eq!(
        select::select(&r1_revoked, &back, &answer(&r2, &qualification_ref(&r2))).unwrap_err(),
        Refusal::RevisionRevoked
    );
    assert_eq!(
        prepare::desire(&r1_revoked, &r1).unwrap_err(),
        Refusal::RevisionRevoked,
        "no new activation of a revoked digest"
    );
}

#[test]
fn the_comparison_report_is_recorded() {
    let cases = cases();
    let packages = ["skill", "plugin"]
        .map(|name| {
            let fixture = load(&format!("{name}.json"));
            let bundle = lifecycle_bundle(&fixture);
            (
                name,
                serde_json::json!({
                    "findings": compare(&fixture, cases.bounds),
                    "mapping": mapping(&fixture),
                    "lifecycle": {
                        "definition": bundle.definition,
                        "revision": bundle.revision,
                        "bundle_identity": lattice_core::agent_revision::facts::identity(&bundle).expect("identity"),
                    },
                }),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let negative = cases
        .cases
        .iter()
        .map(|case| (case.name.clone(), case_findings(case, cases.bounds)))
        .collect::<BTreeMap<_, _>>();
    let changed = lifecycle_bundle(&republished(&load("skill.json")));
    let report = serde_json::json!({
        "schema": "lattice.agent-package-comparison-report.v1",
        "claim": CLAIM,
        "draft": load("skill.json").draft,
        "packages": packages,
        "republished_skill_revision": changed.revision,
        "negative_cases": negative,
        "live_interoperability": false,
        "decision": "useful_mapping_adoption_deferred",
    });
    let golden = fixture_path("comparison-report.json");
    let recorded: serde_json::Value =
        serde_json::from_slice(&std::fs::read(golden).expect("golden report read")).expect("golden decodes");
    assert_eq!(recorded, report, "computed: {}", serde_json::to_string_pretty(&report).expect("encode"));
}
