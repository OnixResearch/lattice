use std::path::Path;

use lattice::operation::CapabilityClass;
use lattice::operation_pack::content_identity;
use lattice::operation_pack::manifest_identity;
use lattice::operation_pack::model::AdmissionProfile;
use lattice::operation_pack::model::AnalyzerDataView;
use lattice::operation_pack::model::AnalyzerManifest;
use lattice::operation_pack::model::AnalyzerScope;
use lattice::operation_pack::model::ContentItem;
use lattice::operation_pack::model::ContentRole;
use lattice::operation_pack::model::Manifest;
use lattice::operation_pack::model::OperationBinding;
use lattice::operation_pack::model::ReleaseChannel;
use lattice::operation_pack::model::ReleaseMetadata;
use lattice::operation_pack::model::RuntimeArtifact;
use lattice::operation_pack::model::SkillManifest;
use lattice::operation_pack::pack_identity;

const FIXTURE_ROOT: &str = "tests/fixtures/operation-packs/valid";
const OPERATION_TYPE_REVISION: &str = "b3:35bb448c27e5def0f6204d8fc3f14fa40275b73a735b1971a1c5954afa30bd2d";
const ANALYZER_OUTPUT_SCHEMA: &[u8] = b"lattice.operation-pack-analyzer-output.v1";
const ANALYZER_RETENTION: &[u8] = b"lattice.operation-pack-analyzer-retention.v1";
const OPERATION_TIMEOUT_SECONDS: u64 = 60;
const ANALYZER_TIMEOUT_SECONDS: u64 = 10;
const OPERATION_INPUT_BYTES: usize = 4_096;
const OPERATION_OUTPUT_BYTES: usize = 1_048_576;
const ANALYZER_INPUT_BYTES: usize = 65_536;
const ANALYZER_OUTPUT_BYTES: usize = 65_536;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(FIXTURE_ROOT);
    let entries = [
        ("operation-catalog", ContentRole::OperationCatalog, "operation-catalog.ncl", "nickel"),
        ("runtime-cairn", ContentRole::RuntimeArtifact, "cairn-read-only-fixture", "executable"),
        ("runtime-analyzer", ContentRole::RuntimeArtifact, "analyzer-fixture", "executable"),
        ("skill-body", ContentRole::SkillBody, "skill.md", "markdown"),
        ("skill-reference", ContentRole::SkillReference, "reference.md", "markdown"),
    ];
    let contents = entries
        .iter()
        .map(|(id, role, path, media_class)| {
            let bytes = std::fs::read(root.join(path))?;
            Ok(ContentItem {
                id: (*id).to_string(),
                role: *role,
                path: (*path).to_string(),
                content_identity: content_identity(&bytes),
                byte_length: u64::try_from(bytes.len())?,
                media_class: (*media_class).to_string(),
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let manifest = Manifest {
        schema: lattice::operation_pack::model::MANIFEST_SCHEMA.to_string(),
        pack_id: "lattice.cairn-inspection-pack".to_string(),
        description: "Exact read-only Cairn inspection with bounded analysis".to_string(),
        contents,
        operations: vec![OperationBinding {
            type_id: "cairn.repository".to_string(),
            type_revision: OPERATION_TYPE_REVISION.to_string(),
            catalog_content_id: "operation-catalog".to_string(),
            runtime_id: "cairn-runtime".to_string(),
            launch_label: "cairn-read-only-v1".to_string(),
            methods: vec!["change_list".to_string(), "validate".to_string()],
        }],
        runtimes: vec![
            RuntimeArtifact {
                id: "cairn-runtime".to_string(),
                content_id: "runtime-cairn".to_string(),
                kind: "bounded_process".to_string(),
                launch_labels: vec!["cairn-read-only-v1".to_string()],
                platforms: vec!["x86_64-linux".to_string(), "aarch64-linux".to_string()],
                capabilities: vec![CapabilityClass::ProcessRead],
                timeout_seconds: OPERATION_TIMEOUT_SECONDS,
                input_bytes: OPERATION_INPUT_BYTES,
                output_bytes: OPERATION_OUTPUT_BYTES,
                clear_environment: true,
                wasm: None,
            },
            RuntimeArtifact {
                id: "analyzer-runtime".to_string(),
                content_id: "runtime-analyzer".to_string(),
                kind: "bounded_process".to_string(),
                launch_labels: vec!["analyze".to_string()],
                platforms: vec!["x86_64-linux".to_string(), "aarch64-linux".to_string()],
                capabilities: vec![CapabilityClass::ProcessRead],
                timeout_seconds: ANALYZER_TIMEOUT_SECONDS,
                input_bytes: ANALYZER_INPUT_BYTES,
                output_bytes: ANALYZER_OUTPUT_BYTES,
                clear_environment: true,
                wasm: None,
            },
        ],
        analyzers: vec![AnalyzerManifest {
            id: "summary".to_string(),
            scope: AnalyzerScope::Method,
            data_view: vec![AnalyzerDataView::RunMetadata, AnalyzerDataView::ExactDataReferences],
            required: true,
            runtime_id: "analyzer-runtime".to_string(),
            launch_label: "analyze".to_string(),
            input_bytes: ANALYZER_INPUT_BYTES,
            timeout_seconds: ANALYZER_TIMEOUT_SECONDS,
            output_bytes: ANALYZER_OUTPUT_BYTES,
            output_schema_identity: content_identity(ANALYZER_OUTPUT_SCHEMA),
            markdown_allowed: true,
            maximum_markdown_bytes: ANALYZER_OUTPUT_BYTES,
            retention_identity: content_identity(ANALYZER_RETENTION),
            sensitivity: lattice::operation_data::SensitivityClass::Internal,
            required_non_claims: vec![
                "analyzer output does not strengthen the original operation observation".to_string(),
            ],
        }],
        skills: vec![SkillManifest {
            name: "lattice-operation-gateway".to_string(),
            description: "Route admitted typed operation work through safe Lattice entrypoints".to_string(),
            body_content_id: "skill-body".to_string(),
            reference_content_ids: vec!["skill-reference".to_string()],
            hosts: vec!["pi".to_string(), "herdr".to_string()],
            trust: "trusted".to_string(),
            positive_routes: vec![
                "Cairn lifecycle inspection".to_string(),
                "Mantle build inspection".to_string(),
                "OnixOS configuration inspection".to_string(),
                "Valence evidence inspection".to_string(),
            ],
            negative_routes: vec![
                "generic shell task".to_string(),
                "unavailable operation type".to_string(),
            ],
        }],
        dependencies: Vec::new(),
        platforms: vec!["x86_64-linux".to_string(), "aarch64-linux".to_string()],
        admission: AdmissionProfile {
            identity: content_identity(b"lattice.operation-pack-admission.read-only.v1"),
            required_evidence: Vec::new(),
            allowed_licenses: vec!["AGPL-3.0-or-later".to_string()],
            allowed_capabilities: vec![CapabilityClass::ProcessRead],
        },
        release: ReleaseMetadata {
            display_version: "1.0.0".to_string(),
            channel: ReleaseChannel::Beta,
            deprecated: false,
            superseded_by: None,
            publication_epoch_seconds: None,
        },
        licenses: vec!["AGPL-3.0-or-later".to_string()],
        probe: None,
        required_non_claims: lattice::operation_pack::model::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    };
    let manifest = lattice::operation_pack::validate_manifest(manifest)?;
    println!("{}", serde_json::to_string_pretty(&manifest)?);
    eprintln!("manifest_identity={}", manifest_identity(&manifest)?);
    eprintln!("pack_identity={}", pack_identity(&manifest)?);
    Ok(())
}
