use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::ComponentStageKind;
use crunch_wasm_component_core::MaterializationBundleRequest;
use crunch_wasm_component_core::REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM;
use crunch_wasm_component_core::REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM;
use crunch_wasm_component_core::StageReceiptReference;
use crunch_wasm_component_core::StoreObject;
use crunch_wasm_component_core::build_materialization_bundle;
use lattice::operation;
use lattice::operation_pack as pack;
use pack::wasm;

// Produce literal Nickel, rather than permitting imports into an admitted pack.
fn nickel(value: &impl serde::Serialize) -> String {
    fn literal(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(fields) => format!(
                "{{{}}}",
                fields
                    .iter()
                    .map(|(key, value)| format!("{} = {}", serde_json::to_string(key).unwrap(), literal(value)))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            serde_json::Value::Array(values) => {
                format!("[{}]", values.iter().map(literal).collect::<Vec<_>>().join(","))
            }
            value => value.to_string(),
        }
    }
    literal(&serde_json::to_value(value).expect("fixture value"))
}

// This is the real native byte-method carrier fixture: a named interface,
// canonical ABI string/list arguments and automatic post-return. The returned
// bytes are deliberately destroyed by cleanup, proving the runtime copies them.
fn component(action: &str, cleanup: &str) -> Vec<u8> {
    let result = serde_json::to_vec(&pack::wasm_output::Output::PureResult {
        schema: wasm::RESULT_SCHEMA.to_string(),
        result: serde_json::json!({"count": 7}),
    })
    .expect("component result");
    let data = result.iter().map(|byte| format!("\\{byte:02x}")).collect::<String>();
    format!(
        r#"(component
      (core module $m
        (memory (export "memory") 1)
        (data (i32.const 4096) "{data}")
        (global $heap (mut i32) (i32.const 1024))
        (func (export "realloc") (param i32 i32 i32) (param $size i32) (result i32)
          (local $ptr i32)
          global.get $heap local.tee $ptr local.get $size i32.add global.set $heap local.get $ptr)
        (func (export "run") (param i32 i32 i32 i32) (result i32)
          {action}
          i32.const 0 i32.const 4096 i32.store
          i32.const 4 i32.const {length} i32.store i32.const 0)
        (func (export "cleanup") (param i32)
          i32.const 4096 i32.const 0 i32.const {length} memory.fill {cleanup}))
      (core instance $i (instantiate $m))
      (alias core export $i "memory" (core memory $memory))
      (alias core export $i "realloc" (core func $realloc))
      (alias core export $i "run" (core func $run))
      (alias core export $i "cleanup" (core func $cleanup))
      (type $call (func (param "method" string) (param "input" (list u8)) (result (list u8))))
      (func $call (type $call) (canon lift (core func $run) (memory $memory)
        (realloc $realloc) (post-return $cleanup)))
      (instance $methods (export "invoke" (func $call)))
      (export "{interface}" (instance $methods)))"#,
        length = result.len(),
        interface = wasm::EXPORT_INTERFACE
    )
    .into_bytes()
}

struct Fixture {
    _root: tempfile::TempDir,
    pack_root: std::path::PathBuf,
    instances: std::path::PathBuf,
    config: lattice::config::AppConfig,
    admitted: pack::AdmittedPack,
}

impl Fixture {
    fn new(action: &str, cleanup: &str) -> Self {
        let root = tempfile::tempdir().expect("fixture root");
        let pack_root = root.path().join("pack");
        std::fs::create_dir(&pack_root).expect("pack root");
        let baseline_root = std::fs::canonicalize("tests/fixtures/operation-packs/valid").expect("baseline root");
        let baseline = pack::shell::load(&baseline_root).expect("baseline manifest");
        let mut manifest = baseline.manifest;
        manifest.pack_id = "test.native-component-pack".to_string();
        manifest.contents.clear();
        manifest.operations.clear();
        manifest.runtimes.clear();
        manifest.analyzers.clear();
        manifest.skills.clear();
        manifest.admission.required_evidence = vec![pack::EvidenceKind::MantleBuild];
        let platform = pack::shell::platform_label();
        manifest.platforms = vec![platform.clone()];
        let mut descriptor = wasm::Descriptor {
            wit_content_id: "wit".to_string(),
            bundle_content_id: "bundle".to_string(),
            wit_package: wasm::WIT_PACKAGE.to_string(),
            world: wasm::WORLD.to_string(),
            export_interface: wasm::EXPORT_INTERFACE.to_string(),
            invoke: wasm::INVOKE.to_string(),
            methods: vec![wasm::MethodMapping {
                type_id: "test.component".to_string(),
                method: "inspect".to_string(),
                guest_method: "inspect".to_string(),
            }],
            verifier_revision: wasm::VERIFIER_REVISION.to_string(),
            evidence_identities: BTreeMap::new(),
            runtime_revision: wasm::RUNTIME_REVISION.to_string(),
            runtime_profile_identity: String::new(),
            resources: wasm::Resources {
                fuel: 100_000,
                memory_bytes: 1_048_576,
                stack_bytes: 65_536,
                table_elements: 128,
                instances: 8,
                tables: 8,
                memories: 8,
                maximum_input_bytes: 4096,
                maximum_output_bytes: 4096,
                maximum_component_bytes: 65_536,
                maximum_wit_bytes: 4096,
                maximum_concurrency: 1,
                deadline_millis: 1000,
            },
            result_schema: wasm::RESULT_SCHEMA.to_string(),
            effect_intent_schema: wasm::EFFECT_INTENT_SCHEMA.to_string(),
            effect_intent_targets: vec![],
            maximum_effect_intents: 0,
            required_non_claims: wasm::REQUIRED_NON_CLAIMS.iter().map(|s| s.to_string()).collect(),
        };
        descriptor.runtime_profile_identity = wasm::runtime_profile_identity(&descriptor).expect("native profile");
        let final_portable = object(
            &pack_root,
            &mut manifest,
            "component",
            "component.wasm",
            pack::ContentRole::WasmComponent,
            &component(action, cleanup),
        );
        let wit = object(
            &pack_root,
            &mut manifest,
            "wit",
            "world.wit",
            pack::ContentRole::WitSource,
            wasm::FIRST_WORLD_WIT.as_bytes(),
        );
        let source = b"hello native source\n";
        object(&pack_root, &mut manifest, "source", "source/lib.rs", pack::ContentRole::AnalyzerAsset, source);
        // Frozen one-file native source-tree record; not a general consumer tree
        // hasher. A file hash is intentionally not this directory identity.
        let source_record = [
            b"file\0".as_slice(),
            &6_u64.to_le_bytes(),
            b"lib.rs",
            &(source.len() as u64).to_le_bytes(),
            source,
        ]
        .concat();
        let source_closure = StoreObject {
            logical_path: "/source".to_string(),
            digest_blake3: Blake3Identity::from_slice(&source_record),
            size_bytes: source.len() as u64,
        };
        let lock =
            object(&pack_root, &mut manifest, "wkg-lock", "wkg.lock", pack::ContentRole::AnalyzerAsset, b"locked\n");
        let stage_receipts = [
            ComponentStageKind::PackageResolution,
            ComponentStageKind::Lock,
            ComponentStageKind::BindingGeneration,
            ComponentStageKind::Compilation,
            ComponentStageKind::Composition,
            ComponentStageKind::Virtualization,
            ComponentStageKind::BuildValidation,
            ComponentStageKind::OctetValidation,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, kind)| {
            let receipt = object(
                &pack_root,
                &mut manifest,
                &format!("receipt-{index}"),
                &format!("receipts/{index}.json"),
                pack::ContentRole::AnalyzerAsset,
                format!("{{\"stage\":{index}}}").as_bytes(),
            );
            StageReceiptReference {
                stage_key: format!("stage-{index}"),
                kind,
                receipt_blake3: receipt.digest_blake3.clone(),
                receipt,
                artifact: None,
            }
        })
        .collect();
        let bundle = build_materialization_bundle(MaterializationBundleRequest {
            name: "lattice-native-component-fixture".to_string(),
            manifest_blake3: Blake3Identity::from_slice(b"native-fixture-manifest"),
            cohort_blake3: Blake3Identity::from_slice(b"native-fixture-cohort"),
            wit_inputs: vec![wit],
            package_inputs: vec![],
            source_closure,
            lock,
            final_portable,
            expected_octet_profile_blake3: Blake3Identity::from_slice(b"fixture-octet-profile"),
            expected_runtime_profile_blake3: Blake3Identity::parse(
                descriptor
                    .runtime_profile_identity
                    .strip_prefix("b3:")
                    .expect("profile identity prefix")
                    .to_string(),
            )
            .expect("native profile digest"),
            stage_receipts,
            wizer: None,
            aot: None,
            non_claims: vec![
                REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM.to_string(),
                REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM.to_string(),
            ],
        })
        .bundle
        .expect("native materialization bundle");
        let bundle_bytes = serde_json::to_vec(&bundle).expect("bundle bytes");
        let bundle_identity = pack::content_identity(&bundle_bytes);
        descriptor.evidence_identities.insert(pack::EvidenceKind::MantleBuild, bundle_identity.clone());
        object(&pack_root, &mut manifest, "bundle", "bundle.json", pack::ContentRole::MantleBundle, &bundle_bytes);
        let bounds = operation::Bounds {
            timeout_seconds: 1,
            input_bytes: 4096,
            output_bytes: 4096,
            artifact_bytes: 4096,
        };
        let mut definition: operation::OperationType = serde_json::from_value(serde_json::json!({
            "id":"test.component", "schema_version":2, "revision":"", "description":"Native import-free component fixture", "tags":[],
            "methods":[{"name":"inspect","description":"Read bounded component result",
                "arguments":{"fields":[],"deny_unknown_fields":true},
                "output":{"fields":[{"name":"count","value_type":"integer","required":true,"max_bytes":32}],"deny_unknown_fields":true},
                "preflight":["runtime_available"],"capabilities":["process_read"],"side_effect":"read_only"}],
            "runtime":{"kind":operation::runtime::component::RUNTIME_KIND,"key":operation::runtime::component::RUNTIME_KEY,
                "executor_identity":manifest.contents.iter().find(|c| c.id == "component").unwrap().content_identity},
            "execution_requirements":{"format":"wasm_component","filesystem":"no_filesystem","network":"network_denied",
                "process":"in_process","terminal":"no_terminal","peripheral":"no_peripheral","persistence":"ephemeral",
                "isolation":"mediated_isolation","resources":{"memory_bytes":1048576,"storage_bytes":4096,"cpu_milliseconds":1000,"wall_milliseconds":1000}},
            "capabilities":["process_read"],"credential_bindings":[],"bounds":bounds,
            "required_non_claims":operation::model::REQUIRED_NON_CLAIMS,
        })).expect("component operation");
        definition.revision = operation::identity::type_revision(&definition).expect("operation revision");
        let type_revision = definition.revision.clone();
        let catalog = operation::Catalog {
            schema: operation::model::CATALOG_SCHEMA.to_string(),
            types: vec![definition],
        };
        object(
            &pack_root,
            &mut manifest,
            "catalog",
            "catalog.ncl",
            pack::ContentRole::OperationCatalog,
            nickel(&catalog).as_bytes(),
        );
        manifest.operations = vec![pack::OperationBinding {
            type_id: "test.component".to_string(),
            type_revision: type_revision.clone(),
            catalog_content_id: "catalog".to_string(),
            runtime_id: "component".to_string(),
            launch_label: operation::runtime::component::RUNTIME_KEY.to_string(),
            methods: vec!["inspect".to_string()],
        }];
        manifest.runtimes = vec![pack::RuntimeArtifact {
            id: "component".to_string(),
            content_id: "component".to_string(),
            kind: wasm::RUNTIME_KIND.to_string(),
            launch_labels: vec![operation::runtime::component::RUNTIME_KEY.to_string()],
            platforms: vec![platform.clone()],
            capabilities: vec![operation::CapabilityClass::ProcessRead],
            timeout_seconds: 1,
            input_bytes: 4096,
            output_bytes: 4096,
            clear_environment: true,
            wasm: Some(descriptor),
        }];
        std::fs::write(pack_root.join("manifest.ncl"), nickel(&manifest)).expect("manifest");
        let candidate = pack::shell::load(&pack_root).expect("measured native component pack");
        let facts = pack::AdmissionFacts {
            trusted_pack_identity: candidate.pack_identity.clone(),
            platform,
            dependency_identities: BTreeMap::new(),
            allowed_licenses: manifest.admission.allowed_licenses.clone(),
            allowed_capabilities: manifest.admission.allowed_capabilities.clone(),
            evidence: BTreeMap::from([(pack::EvidenceKind::MantleBuild, pack::EvidenceFact {
                identity: bundle_identity,
                current: true,
                valid: true,
                overclaimed: false,
            })]),
        };
        let admitted = pack::admit(&candidate, &pack::create_lock(&candidate, &facts).expect("component lock"), &facts)
            .expect("component pack admission");
        let instances = root.path().join("instances.ncl");
        std::fs::write(
            &instances,
            nickel(&serde_json::json!({"schema":operation::model::INSTANCE_SCHEMA,
            "instances":[{"id":"component-instance","display_alias":"Component","type_id":"test.component",
                "type_revision":type_revision,"target":{},"tags":{},"method_defaults":{},"credentials":{}}]})),
        )
        .expect("instance");
        let config = lattice::config::AppConfig {
            data_dir: root.path().join("data"),
            ..Default::default()
        };
        Self {
            _root: root,
            pack_root,
            instances,
            config,
            admitted,
        }
    }

    fn run(&self, cancellation: Option<&AtomicBool>) -> Result<pack::shell::execution::Report, pack::Error> {
        pack::shell::execution::run(pack::shell::execution::Request {
            app_config: &self.config,
            pack_root: &self.pack_root,
            admitted: &self.admitted,
            instances_path: &self.instances,
            instance_id: "component-instance",
            method: "inspect",
            arguments_json: "{}",
            sequence: 1,
            prohibited_fragments: &[],
            cancellation,
        })
    }
}

fn object(
    root: &std::path::Path,
    manifest: &mut pack::Manifest,
    id: &str,
    path: &str,
    role: pack::ContentRole,
    bytes: &[u8],
) -> StoreObject {
    let target = root.join(path);
    std::fs::create_dir_all(target.parent().unwrap()).expect("member directory");
    std::fs::write(target, bytes).expect("member bytes");
    manifest.contents.push(pack::ContentItem {
        id: id.to_string(),
        role,
        path: path.to_string(),
        content_identity: pack::content_identity(bytes),
        byte_length: bytes.len() as u64,
        media_class: "fixture".to_string(),
    });
    StoreObject {
        logical_path: format!("/{path}"),
        digest_blake3: Blake3Identity::from_slice(bytes),
        size_bytes: bytes.len() as u64,
    }
}

#[test]
fn native_component_bundle_executes_and_fails_closed_without_process_fallback() {
    use pack::shell::component::Class;
    use pack::shell::component::PostReturn;
    let fixture = Fixture::new("", "");
    let report = fixture.run(None).expect("native component execution");
    assert_eq!(report.status, pack::shell::execution::Status::Completed);
    assert_eq!(report.operation_state, operation::RunState::Completed);
    let receipt = report.component.as_ref().expect("native component receipt");
    assert_eq!(receipt.outcome, Class::Completed);
    assert_eq!(receipt.post_return, PostReturn::Succeeded);
    assert_eq!(receipt.runtime_revision, wasm::RUNTIME_REVISION);
    assert_eq!(receipt.verifier_revision, wasm::VERIFIER_REVISION);
    assert!(receipt.store_dropped && receipt.observer_joined);
    assert!(!receipt.automatic_retry_permitted);
    let store_path = lattice::config::store_path(&fixture.config.data_dir);
    let plane = lattice::operation_data::Plane::open(&store_path, &fixture.config.data_dir).expect("operation data");
    let metadata = plane.list_metadata().expect("published component metadata");
    assert_eq!(metadata.len(), 1);
    assert_eq!(report.operation_data_version.as_deref(), Some(metadata[0].version_identity.as_str()));
    drop(plane);
    assert!(fixture.run(None).is_err(), "terminal component evidence must prevent automatic rerun");

    for (action, cleanup, expected) in [
        ("unreachable", "", Class::Trapped),
        ("", "unreachable", Class::Trapped),
        ("(loop $forever br $forever)", "", Class::FuelExhausted),
    ] {
        let fixture = Fixture::new(action, cleanup);
        let report = fixture.run(None).expect("durable failed component execution");
        assert_eq!(report.status, pack::shell::execution::Status::Failed);
        assert_eq!(report.operation_state, operation::RunState::Failed);
        assert!(report.operation_data_version.is_none());
        let receipt = report.component.expect("failed component receipt");
        assert_eq!(receipt.outcome, expected);
        assert!(receipt.store_dropped && receipt.observer_joined);
        assert!(!receipt.automatic_retry_permitted);
    }
    let fixture = Fixture::new("", "");
    let cancelled = AtomicBool::new(true);
    let report = fixture.run(Some(&cancelled)).expect("durable cancellation");
    assert_eq!(report.component.expect("cancellation receipt").outcome, Class::Cancelled);
    assert!(report.operation_data_version.is_none());

    let fixture = Fixture::new("", "");
    std::fs::write(fixture.pack_root.join("source/lib.rs"), b"other native source\n").expect("same-length drift");
    assert!(fixture.run(None).is_err(), "source-tree identity must be remeasured, not treated as a file digest");
    assert!(!fixture.config.data_dir.exists(), "bundle denial precedes durable execution");

    let fixture = Fixture::new("", "");
    std::fs::remove_file(fixture.pack_root.join("receipts/3.json")).expect("missing native stage receipt");
    assert!(fixture.run(None).is_err());
    assert!(!fixture.config.data_dir.exists());
}
