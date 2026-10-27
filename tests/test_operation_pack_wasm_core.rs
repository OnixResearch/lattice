use std::collections::BTreeMap;

use lattice::operation::CapabilityClass;
use lattice::operation_pack as pack;
use pack::wasm;

const PLATFORM: &str = "x86_64-linux";
const RUNTIME: &str = "component";
const TYPE_ID: &str = "test.component";
const METHOD: &str = "inspect";

fn identity(value: &str) -> String {
    pack::content_identity(value.as_bytes())
}

fn resources() -> wasm::Resources {
    wasm::Resources {
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
    }
}

fn manifest() -> pack::Manifest {
    let contents = [
        ("catalog", pack::ContentRole::OperationCatalog),
        ("component", pack::ContentRole::WasmComponent),
        ("wit", pack::ContentRole::WitSource),
        ("bundle", pack::ContentRole::MantleBundle),
        ("process", pack::ContentRole::RuntimeArtifact),
    ]
    .into_iter()
    .map(|(id, role)| pack::ContentItem {
        id: id.to_string(),
        role,
        path: id.to_string(),
        content_identity: identity(id),
        byte_length: 64,
        media_class: "test-content".to_string(),
    })
    .collect();
    let descriptor = wasm::Descriptor {
        wit_content_id: "wit".to_string(),
        bundle_content_id: "bundle".to_string(),
        wit_package: wasm::WIT_PACKAGE.to_string(),
        world: wasm::WORLD.to_string(),
        export_interface: wasm::EXPORT_INTERFACE.to_string(),
        invoke: wasm::INVOKE.to_string(),
        methods: vec![wasm::MethodMapping {
            type_id: TYPE_ID.to_string(),
            method: METHOD.to_string(),
            guest_method: "inspect_guest".to_string(),
        }],
        verifier_revision: wasm::VERIFIER_REVISION.to_string(),
        runtime_revision: "1".repeat(40),
        runtime_profile_identity: identity("runtime-profile"),
        resources: resources(),
        result_schema: wasm::RESULT_SCHEMA.to_string(),
        effect_intent_schema: wasm::EFFECT_INTENT_SCHEMA.to_string(),
        evidence_identities: BTreeMap::from([(pack::EvidenceKind::MantleBuild, identity("bundle-evidence"))]),
        effect_intent_targets: vec![wasm::EffectTarget {
            type_id: "test.process".to_string(),
            method: "inspect".to_string(),
        }],
        maximum_effect_intents: 1,
        required_non_claims: wasm::REQUIRED_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    let component_runtime = pack::RuntimeArtifact {
        id: RUNTIME.to_string(),
        content_id: "component".to_string(),
        kind: wasm::RUNTIME_KIND.to_string(),
        launch_labels: vec!["component-v1".to_string()],
        platforms: vec![PLATFORM.to_string(), "aarch64-linux".to_string()],
        capabilities: vec![CapabilityClass::ProcessRead],
        timeout_seconds: 1,
        input_bytes: 4096,
        output_bytes: 4096,
        clear_environment: true,
        wasm: Some(descriptor),
    };
    let process_runtime = pack::RuntimeArtifact {
        id: "process".to_string(),
        content_id: "process".to_string(),
        kind: "bounded_process".to_string(),
        launch_labels: vec!["process-v1".to_string()],
        platforms: vec![PLATFORM.to_string(), "aarch64-linux".to_string()],
        capabilities: vec![CapabilityClass::ProcessRead],
        timeout_seconds: 1,
        input_bytes: 4096,
        output_bytes: 4096,
        clear_environment: true,
        wasm: None,
    };
    pack::Manifest {
        schema: pack::MANIFEST_SCHEMA.to_string(),
        pack_id: "test.component-pack".to_string(),
        description: "Pure core admission fixture".to_string(),
        contents,
        operations: vec![
            pack::OperationBinding {
                type_id: TYPE_ID.to_string(),
                type_revision: identity("component-operation"),
                catalog_content_id: "catalog".to_string(),
                runtime_id: RUNTIME.to_string(),
                launch_label: "component-v1".to_string(),
                methods: vec![METHOD.to_string()],
            },
            pack::OperationBinding {
                type_id: "test.process".to_string(),
                type_revision: identity("process-operation"),
                catalog_content_id: "catalog".to_string(),
                runtime_id: "process".to_string(),
                launch_label: "process-v1".to_string(),
                methods: vec!["inspect".to_string()],
            },
        ],
        runtimes: vec![component_runtime, process_runtime],
        analyzers: vec![],
        skills: vec![],
        dependencies: vec![],
        platforms: vec![PLATFORM.to_string(), "aarch64-linux".to_string()],
        admission: pack::AdmissionProfile {
            identity: identity("admission"),
            required_evidence: vec![pack::EvidenceKind::MantleBuild],
            allowed_licenses: vec!["AGPL-3.0-or-later".to_string()],
            allowed_capabilities: vec![CapabilityClass::ProcessRead],
        },
        release: pack::ReleaseMetadata {
            display_version: "1.0.0".to_string(),
            channel: pack::ReleaseChannel::Beta,
            deprecated: false,
            superseded_by: None,
            publication_epoch_seconds: None,
        },
        licenses: vec!["AGPL-3.0-or-later".to_string()],
        probe: None,
        required_non_claims: pack::REQUIRED_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    }
}

fn candidate(manifest: pack::Manifest) -> pack::Candidate {
    let manifest = pack::validate_manifest(manifest).expect("valid complete component manifest");
    pack::Candidate {
        manifest_identity: pack::manifest_identity(&manifest).expect("manifest identity"),
        pack_identity: pack::pack_identity(&manifest).expect("pack identity"),
        measured_contents: manifest
            .contents
            .iter()
            .map(|content| pack::MeasuredContent {
                id: content.id.clone(),
                path: content.path.clone(),
                content_identity: content.content_identity.clone(),
                byte_length: content.byte_length,
            })
            .collect(),
        manifest,
        source_locator: identity("non-authoritative locator"),
    }
}

fn facts(candidate: &pack::Candidate) -> pack::AdmissionFacts {
    pack::AdmissionFacts {
        trusted_pack_identity: candidate.pack_identity.clone(),
        platform: PLATFORM.to_string(),
        dependency_identities: BTreeMap::new(),
        allowed_licenses: candidate.manifest.admission.allowed_licenses.clone(),
        allowed_capabilities: candidate.manifest.admission.allowed_capabilities.clone(),
        evidence: BTreeMap::from([(pack::EvidenceKind::MantleBuild, pack::EvidenceFact {
            identity: identity("bundle-evidence"),
            current: true,
            valid: true,
            overclaimed: false,
        })]),
    }
}

fn admitted() -> pack::AdmittedPack {
    let candidate = candidate(manifest());
    let facts = facts(&candidate);
    let lock = pack::create_lock(&candidate, &facts).expect("exact component lock");
    pack::admit(&candidate, &lock, &facts).expect("exact component admission")
}

fn dispatch_facts(admitted: &pack::AdmittedPack) -> wasm::DispatchFacts {
    let manifest = &admitted.candidate.manifest;
    let runtime = manifest.runtimes.iter().find(|runtime| runtime.id == RUNTIME).expect("component runtime");
    let descriptor = runtime.wasm.as_ref().expect("component descriptor");
    let operation = manifest.operations.iter().find(|operation| operation.type_id == TYPE_ID).expect("operation");
    let content = |id: &str| {
        manifest
            .contents
            .iter()
            .find(|content| content.id == id)
            .expect("declared content")
            .content_identity
            .clone()
    };
    wasm::DispatchFacts {
        platform: PLATFORM.to_string(),
        runtime_available: true,
        runtime_revision: descriptor.runtime_revision.clone(),
        runtime_profile_identity: descriptor.runtime_profile_identity.clone(),
        bundle: wasm::BundleFacts {
            bundle_identity: content(&descriptor.bundle_content_id),
            component_identity: content(&runtime.content_id),
            wit_identity: content(&descriptor.wit_content_id),
            verifier_revision: wasm::VERIFIER_REVISION.to_string(),
            valid: true,
            current: true,
            overclaimed: false,
        },
        world: wasm::WorldFacts {
            wit_package: wasm::WIT_PACKAGE.to_string(),
            world: wasm::WORLD.to_string(),
            export_interface: wasm::EXPORT_INTERFACE.to_string(),
            invoke: wasm::INVOKE.to_string(),
            imports: vec![],
        },
        resources: resources(),
        input_bytes: 4096,
        authority: wasm::AuthorityFacts {
            pack_identity: admitted.candidate.pack_identity.clone(),
            lock_identity: admitted.lock.identity.clone(),
            type_id: TYPE_ID.to_string(),
            type_revision: operation.type_revision.clone(),
            method: METHOD.to_string(),
            capabilities: runtime.capabilities.clone(),
            operation_admitted: true,
        },
    }
}

fn descriptor(manifest: &mut pack::Manifest) -> &mut wasm::Descriptor {
    manifest
        .runtimes
        .iter_mut()
        .find(|runtime| runtime.id == RUNTIME)
        .expect("runtime")
        .wasm
        .as_mut()
        .expect("descriptor")
}

#[test]
fn complete_component_cohort_has_deterministic_exact_lock_and_plan() {
    // r[verify lattice.operation_packs_wasm.manifest]
    // r[verify lattice.operation_packs_wasm.lock]
    // r[verify lattice.operation_packs_wasm.functional_core]
    let admitted = admitted();
    let again = pack::create_lock(&admitted.candidate, &facts(&admitted.candidate)).expect("repeat lock");
    assert_eq!(again, admitted.lock);
    let facts = dispatch_facts(&admitted);
    let plan = wasm::admit_dispatch(&admitted, RUNTIME, TYPE_ID, METHOD, &facts).expect("dispatch plan");
    assert_eq!(wasm::admit_dispatch(&admitted, RUNTIME, TYPE_ID, METHOD, &facts), Ok(plan.clone()));
    assert_eq!(plan.guest_method, "inspect_guest");
    assert_eq!(plan.cohort_identity, admitted.lock.component_cohort_identities[RUNTIME]);
    let mut relocated = admitted.clone();
    relocated.candidate.source_locator = identity("other locator");
    relocated.lock.source_locator = identity("third locator");
    assert_eq!(wasm::admit_dispatch(&relocated, RUNTIME, TYPE_ID, METHOD, &facts), Ok(plan));
}

#[test]
fn malformed_missing_duplicate_crosspack_and_overclaimed_descriptors_fail() {
    // r[verify lattice.operation_packs_wasm.manifest]
    // r[verify lattice.operation_packs_wasm.nonclaims]
    let valid = manifest();
    let mut missing = valid.clone();
    missing.runtimes[0].wasm = None;
    assert!(pack::validate_manifest(missing).is_err());
    for field in [
        "wit_content_id",
        "resources",
        "runtime_revision",
        "required_non_claims",
        "effect_intent_targets",
    ] {
        let mut value = serde_json::to_value(&valid).expect("JSON");
        value["runtimes"][0]["wasm"].as_object_mut().expect("descriptor").remove(field);
        assert!(serde_json::from_value::<pack::Manifest>(value).is_err(), "missing {field}");
    }
    for path in ["descriptor", "resource", "mapping", "effect_target"] {
        let mut value = serde_json::to_value(&valid).expect("JSON");
        let target = match path {
            "descriptor" => &mut value["runtimes"][0]["wasm"],
            "resource" => &mut value["runtimes"][0]["wasm"]["resources"],
            "mapping" => &mut value["runtimes"][0]["wasm"]["methods"][0],
            _ => &mut value["runtimes"][0]["wasm"]["effect_intent_targets"][0],
        };
        target["ambient_authority"] = serde_json::json!(true);
        assert!(serde_json::from_value::<pack::Manifest>(value).is_err(), "unknown {path}");
    }
    let mutations: &[fn(&mut pack::Manifest)] = &[
        |manifest| {
            let descriptor = descriptor(manifest);
            descriptor.methods.push(descriptor.methods[0].clone());
        },
        |manifest| descriptor(manifest).wit_content_id = "other.pack/wit".to_string(),
        |manifest| descriptor(manifest).bundle_content_id = "missing".to_string(),
        |manifest| descriptor(manifest).methods[0].type_id = "other.component".to_string(),
        |manifest| descriptor(manifest).methods[0].method = "undeclared".to_string(),
        |manifest| descriptor(manifest).required_non_claims.pop().map(|_| ()).expect("nonclaim"),
        |manifest| {
            descriptor(manifest).required_non_claims[0] = "component evidence proves operation correctness".to_string()
        },
        |manifest| descriptor(manifest).world = "other".to_string(),
        |manifest| descriptor(manifest).verifier_revision = "2".repeat(40),
        |manifest| descriptor(manifest).result_schema = "open-json".to_string(),
        |manifest| descriptor(manifest).effect_intent_schema = "open-intents".to_string(),
        |manifest| descriptor(manifest).runtime_revision = "floating-main".to_string(),
        |manifest| descriptor(manifest).evidence_identities.clear(),
        |manifest| descriptor(manifest).resources.maximum_concurrency = 2,
        |manifest| descriptor(manifest).resources.maximum_component_bytes = 63,
        |manifest| descriptor(manifest).resources.maximum_wit_bytes = 63,
        |manifest| descriptor(manifest).resources.maximum_input_bytes -= 1,
        |manifest| descriptor(manifest).resources.fuel = 0,
        |manifest| descriptor(manifest).resources.deadline_millis = 999,
        |manifest| descriptor(manifest).effect_intent_targets[0].type_id = TYPE_ID.to_string(),
        |manifest| descriptor(manifest).effect_intent_targets[0].method = "undeclared".to_string(),
        |manifest| descriptor(manifest).maximum_effect_intents = 0,
        |manifest| {
            let descriptor = descriptor(manifest);
            descriptor.effect_intent_targets.push(descriptor.effect_intent_targets[0].clone());
        },
        |manifest| manifest.runtimes[1].capabilities.push(CapabilityClass::CredentialRead),
        |manifest| {
            manifest.contents.iter_mut().find(|content| content.id == "bundle").expect("bundle").byte_length =
                pack::MAXIMUM_MANIFEST_BYTES + 1
        },
        |manifest| {
            manifest.contents.iter_mut().find(|content| content.id == "wit").expect("wit").role =
                pack::ContentRole::RuntimeArtifact
        },
    ];
    for (index, mutate) in mutations.iter().enumerate() {
        let mut invalid = valid.clone();
        mutate(&mut invalid);
        assert!(pack::validate_manifest(invalid).is_err(), "invalid mutation {index}");
    }
}

#[test]
fn every_authoritative_component_cohort_change_denies_the_old_lock() {
    // r[verify lattice.operation_packs_wasm.lock]
    let admitted = admitted();
    let mutations: &[fn(&mut pack::Manifest)] = &[
        |manifest| {
            manifest
                .contents
                .iter_mut()
                .find(|content| content.id == "component")
                .expect("component")
                .content_identity = identity("changed component")
        },
        |manifest| {
            manifest.contents.iter_mut().find(|content| content.id == "wit").expect("wit").content_identity =
                identity("changed WIT")
        },
        |manifest| {
            manifest
                .contents
                .iter_mut()
                .find(|content| content.id == "bundle")
                .expect("bundle")
                .content_identity = identity("changed bundle")
        },
        |manifest| descriptor(manifest).runtime_revision = "2".repeat(40),
        |manifest| descriptor(manifest).runtime_profile_identity = identity("other profile"),
        |manifest| descriptor(manifest).resources.fuel += 1,
        |manifest| descriptor(manifest).methods[0].guest_method = "other_guest_method".to_string(),
        |manifest| descriptor(manifest).maximum_effect_intents = 2,
        |manifest| manifest.admission.identity = identity("other authority profile"),
        |manifest| manifest.operations[0].type_revision = identity("other operation revision"),
        |manifest| {
            manifest.runtimes[0].capabilities.push(CapabilityClass::CredentialRead);
            manifest.admission.allowed_capabilities.push(CapabilityClass::CredentialRead);
        },
    ];
    for (index, mutate) in mutations.iter().enumerate() {
        let mut changed = admitted.candidate.manifest.clone();
        mutate(&mut changed);
        let changed = candidate(changed);
        assert_ne!(changed.pack_identity, admitted.candidate.pack_identity, "cohort mutation {index}");
        assert!(pack::validate_lock(&changed, &admitted.lock).is_err(), "old lock mutation {index}");
        let new_lock = pack::create_lock(&changed, &facts(&changed)).expect("new explicit lock");
        assert_ne!(new_lock.component_cohort_identities[RUNTIME], admitted.lock.component_cohort_identities[RUNTIME]);
        assert_ne!(new_lock.runtime_identities[RUNTIME], admitted.lock.runtime_identities[RUNTIME]);
    }
    let mut evidence_changed = facts(&admitted.candidate);
    evidence_changed.evidence.get_mut(&pack::EvidenceKind::MantleBuild).expect("evidence").identity =
        identity("new evidence");
    assert!(pack::admit(&admitted.candidate, &admitted.lock, &evidence_changed).is_err());
    assert!(pack::create_lock(&admitted.candidate, &evidence_changed).is_err());
    let mut updated_manifest = admitted.candidate.manifest.clone();
    descriptor(&mut updated_manifest)
        .evidence_identities
        .insert(pack::EvidenceKind::MantleBuild, identity("new evidence"));
    let updated_candidate = candidate(updated_manifest);
    evidence_changed.trusted_pack_identity = updated_candidate.pack_identity.clone();
    assert_ne!(updated_candidate.pack_identity, admitted.candidate.pack_identity);
    assert!(pack::validate_lock(&updated_candidate, &admitted.lock).is_err());
    let new_lock = pack::create_lock(&updated_candidate, &evidence_changed).expect("new evidence lock");
    assert_ne!(new_lock.component_cohort_identities[RUNTIME], admitted.lock.component_cohort_identities[RUNTIME]);
    let mut platform_changed = facts(&admitted.candidate);
    platform_changed.platform = "aarch64-linux".to_string();
    assert!(pack::admit(&admitted.candidate, &admitted.lock, &platform_changed).is_err());
    let new_lock = pack::create_lock(&admitted.candidate, &platform_changed).expect("new platform lock");
    assert_ne!(new_lock.component_cohort_identities[RUNTIME], admitted.lock.component_cohort_identities[RUNTIME]);
}

#[test]
fn bundle_world_import_runtime_resource_and_authority_failures_are_ordered() {
    // r[verify lattice.operation_packs_wasm.world]
    // r[verify lattice.operation_packs_wasm.functional_core]
    // r[verify lattice.operation_packs_wasm.fallback]
    let admitted = admitted();
    let exact = dispatch_facts(&admitted);
    let cases: &[(fn(&mut wasm::DispatchFacts), wasm::Blocker)] = &[
        (|facts| facts.runtime_available = false, wasm::Blocker::RuntimeUnavailable),
        (|facts| facts.platform = "aarch64-linux".to_string(), wasm::Blocker::Platform),
        (|facts| facts.bundle.bundle_identity = identity("other bundle"), wasm::Blocker::Bundle),
        (|facts| facts.bundle.component_identity = identity("other component"), wasm::Blocker::Bundle),
        (|facts| facts.bundle.wit_identity = identity("other WIT"), wasm::Blocker::Bundle),
        (|facts| facts.bundle.overclaimed = true, wasm::Blocker::Bundle),
        (|facts| facts.bundle.valid = false, wasm::Blocker::Bundle),
        (|facts| facts.bundle.current = false, wasm::Blocker::Bundle),
        (|facts| facts.bundle.verifier_revision = "2".repeat(40), wasm::Blocker::VerifierRevision),
        (|facts| facts.runtime_revision = "2".repeat(40), wasm::Blocker::RuntimeRevision),
        (|facts| facts.runtime_profile_identity = identity("other profile"), wasm::Blocker::RuntimeProfile),
        (|facts| facts.world.world = "other".to_string(), wasm::Blocker::World),
        (|facts| facts.world.export_interface = "other".to_string(), wasm::Blocker::World),
        (|facts| facts.world.imports.push("wasi:filesystem/types@0.2.0".to_string()), wasm::Blocker::Imports),
        (|facts| facts.resources.fuel += 1, wasm::Blocker::Resources),
        (|facts| facts.input_bytes += 1, wasm::Blocker::Resources),
        (|facts| facts.authority.operation_admitted = false, wasm::Blocker::Authority),
        (|facts| facts.authority.pack_identity = identity("other pack"), wasm::Blocker::Authority),
        (|facts| facts.authority.lock_identity = identity("other lock"), wasm::Blocker::Authority),
        (|facts| facts.authority.type_revision = identity("other operation"), wasm::Blocker::Authority),
        (|facts| facts.authority.capabilities.push(CapabilityClass::CredentialRead), wasm::Blocker::Authority),
    ];
    for (mutate, expected) in cases {
        let mut denied = exact.clone();
        mutate(&mut denied);
        assert_eq!(wasm::admit_dispatch(&admitted, RUNTIME, TYPE_ID, METHOD, &denied), Err(vec![*expected]));
    }
    let mut denied = exact.clone();
    for (mutate, _) in cases {
        mutate(&mut denied);
    }
    assert_eq!(
        wasm::admit_dispatch(&admitted, RUNTIME, TYPE_ID, METHOD, &denied),
        Err(vec![
            wasm::Blocker::RuntimeUnavailable,
            wasm::Blocker::Platform,
            wasm::Blocker::Bundle,
            wasm::Blocker::VerifierRevision,
            wasm::Blocker::RuntimeRevision,
            wasm::Blocker::RuntimeProfile,
            wasm::Blocker::World,
            wasm::Blocker::Imports,
            wasm::Blocker::Resources,
            wasm::Blocker::Authority,
        ])
    );
    assert_eq!(
        wasm::admit_dispatch(&admitted, "process", TYPE_ID, METHOD, &exact),
        Err(vec![wasm::Blocker::RuntimeDeclaration])
    );
    assert_eq!(
        wasm::admit_dispatch(&admitted, RUNTIME, TYPE_ID, "unknown", &exact),
        Err(vec![wasm::Blocker::Method, wasm::Blocker::Authority])
    );
}

#[test]
fn pure_result_components_need_no_capability_and_cannot_use_process_only_routes() {
    let mut pure = manifest();
    pure.operations.retain(|operation| operation.type_id == TYPE_ID);
    pure.runtimes.retain(|runtime| runtime.id == RUNTIME);
    pure.runtimes[0].capabilities.clear();
    pure.admission.allowed_capabilities.clear();
    descriptor(&mut pure).effect_intent_targets.clear();
    descriptor(&mut pure).maximum_effect_intents = 0;
    let candidate = candidate(pure.clone());
    let facts = facts(&candidate);
    let lock = pack::create_lock(&candidate, &facts).expect("effect-free component lock");
    let admitted = pack::admit(&candidate, &lock, &facts).expect("effect-free component admission");
    let dispatch = dispatch_facts(&admitted);
    let plan = wasm::admit_dispatch(&admitted, RUNTIME, TYPE_ID, METHOD, &dispatch).expect("effect-free plan");
    assert!(plan.descriptor.effect_intent_targets.is_empty());
    pure.probe = Some(pack::ProbeManifest {
        runtime_id: RUNTIME.to_string(),
        launch_label: "component-v1".to_string(),
        effectful: false,
        uses_production_credentials: false,
    });
    assert!(pack::validate_manifest(pure).is_err());
    let mut empty_process = manifest();
    empty_process.runtimes[1].capabilities.clear();
    assert!(pack::validate_manifest(empty_process).is_err());
}

#[test]
fn component_analyzers_are_rejected_before_any_process_route() {
    let mut manifest = manifest();
    manifest.analyzers.push(pack::AnalyzerManifest {
        id: "component-analyzer".to_string(),
        scope: pack::AnalyzerScope::Method,
        data_view: vec![pack::AnalyzerDataView::RunMetadata],
        required: true,
        runtime_id: RUNTIME.to_string(),
        launch_label: "component-v1".to_string(),
        input_bytes: 4096,
        timeout_seconds: 1,
        output_bytes: 4096,
        output_schema_identity: identity("analyzer output"),
        markdown_allowed: false,
        maximum_markdown_bytes: 1,
        retention_identity: identity("retention"),
        sensitivity: lattice::operation_data::SensitivityClass::Internal,
        required_non_claims: vec!["analyzer output does not strengthen the original operation observation".to_string()],
    });
    let error = pack::validate_manifest(manifest).expect_err("unsupported component analyzer");
    assert!(matches!(error, pack::Error::Invalid("component analyzer unsupported")));
}

fn output_fixture() -> (pack::AdmittedPack, lattice::operation::InvocationPlan) {
    use lattice::operation;
    use operation::backend::*;
    let bounds = operation::Bounds {
        timeout_seconds: 1,
        input_bytes: 4096,
        output_bytes: 4096,
        artifact_bytes: 4096,
    };
    let mut definition = operation::OperationType {
        id: TYPE_ID.to_string(),
        schema_version: operation::model::TYPE_SCHEMA_VERSION,
        revision: String::new(),
        description: "Import-free output admission".to_string(),
        tags: vec![],
        methods: vec![operation::model::Method {
            name: METHOD.to_string(),
            description: "Bounded result and proposals".to_string(),
            arguments: operation::model::RecordContract {
                fields: vec![],
                deny_unknown_fields: true,
            },
            output: operation::model::RecordContract {
                fields: vec![operation::model::FieldContract {
                    name: "count".to_string(),
                    value_type: operation::model::ValueType::Integer,
                    required: true,
                    max_bytes: 32,
                }],
                deny_unknown_fields: true,
            },
            preflight: vec![operation::model::PreflightCheck::RuntimeAvailable],
            capabilities: vec![CapabilityClass::ProcessRead],
            side_effect: operation::model::SideEffectClass::ReadOnly,
        }],
        runtime: operation::model::RuntimeRequirement {
            kind: operation::runtime::component::RUNTIME_KIND.to_string(),
            key: operation::runtime::component::RUNTIME_KEY.to_string(),
            executor_identity: identity("component"),
        },
        execution_requirements: operation::ExecutionRequirements {
            format: ExecutionFormat::WasmComponent,
            filesystem: FilesystemSemantics::NoFilesystem,
            network: NetworkPosture::NetworkDenied,
            process: ProcessModel::InProcess,
            terminal: TerminalNeed::NoTerminal,
            peripheral: PeripheralNeed::NoPeripheral,
            persistence: PersistenceNeed::Ephemeral,
            isolation: IsolationClaim::MediatedIsolation,
            resources: operation::ResourceRequirements {
                memory_bytes: 1_048_576,
                storage_bytes: 4096,
                cpu_milliseconds: 1000,
                wall_milliseconds: 1000,
            },
        },
        capabilities: vec![CapabilityClass::ProcessRead],
        credential_bindings: vec![],
        bounds,
        required_non_claims: operation::model::REQUIRED_NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect(),
    };
    definition.revision = operation::identity::type_revision(&definition).expect("component type revision");
    let revision = definition.revision.clone();
    let catalog = operation::catalog::validate(operation::Catalog {
        schema: operation::model::CATALOG_SCHEMA.to_string(),
        types: vec![definition],
    })
    .expect("component catalog");
    let mut instances = operation::instance::validate(
        operation::InstanceFile {
            schema: operation::model::INSTANCE_SCHEMA.to_string(),
            instances: vec![operation::OperationInstance {
                id: "component-instance".to_string(),
                display_alias: "Component".to_string(),
                type_id: TYPE_ID.to_string(),
                type_revision: revision.clone(),
                target: BTreeMap::new(),
                tags: BTreeMap::new(),
                method_defaults: BTreeMap::new(),
                credentials: BTreeMap::new(),
            }],
        },
        &catalog,
    )
    .expect("empty-target import-free instance");
    let instance = instances.remove("component-instance").expect("component instance");
    let plan = operation::planning::plan(operation::PlanRequest {
        catalog: &catalog,
        instance: &instance,
        method: METHOD,
        arguments: BTreeMap::new(),
        expected_catalog_identity: &catalog.identity,
        expected_instance_identity: &instance.identity,
        runtime: operation::RuntimeFact {
            kind: operation::runtime::component::RUNTIME_KIND.to_string(),
            key: operation::runtime::component::RUNTIME_KEY.to_string(),
            executor_identity: identity("component"),
            executable_path: "/offline/pack/component.wasm".to_string(),
            available: true,
            maximum_bounds: bounds,
        },
        policy: operation::CapabilityPolicy {
            allowed: vec![CapabilityClass::ProcessRead],
            allow_mutation: false,
        },
        preflight: operation::PreflightFacts {
            executable_available: false,
            repository_root_admitted: false,
            credential_references_valid: true,
        },
    })
    .expect("existing operation planner admits genuine component profile");
    operation::planning::validate_plan_admission(&plan, &catalog, &instance).expect("operation admission");
    let mut manifest = manifest();
    manifest
        .operations
        .iter_mut()
        .find(|binding| binding.type_id == TYPE_ID)
        .expect("component binding")
        .type_revision = revision;
    let candidate = candidate(manifest);
    let host = facts(&candidate);
    let lock = pack::create_lock(&candidate, &host).expect("component lock");
    (pack::admit(&candidate, &lock, &host).expect("admitted component pack"), plan)
}

#[test]
fn component_output_is_closed_schema_bound_and_does_not_turn_intents_into_effect_success() {
    // r[verify lattice.operation_packs_wasm.world]
    // r[verify lattice.operation_packs_wasm.effects]
    // r[verify lattice.operation_packs_wasm.nonclaims]
    use pack::wasm_output::Blocker;
    use pack::wasm_output::EffectIntent;
    use pack::wasm_output::Output;
    use pack::wasm_output::admit;
    let (admitted, plan) = output_fixture();
    let descriptor = admitted
        .candidate
        .manifest
        .runtimes
        .iter()
        .find_map(|runtime| runtime.wasm.as_ref())
        .expect("descriptor");
    let result = serde_json::json!({"count":7});
    let pure = serde_json::to_vec(&Output::PureResult {
        schema: wasm::RESULT_SCHEMA.to_string(),
        result: result.clone(),
    })
    .expect("pure bytes");
    let output = admit(&pure, &admitted, descriptor, &plan, &[]).expect("typed pure result");
    assert_eq!(output.result(), &result);
    assert!(output.intents().is_empty());
    let intent = EffectIntent {
        type_id: "test.process".to_string(),
        type_revision: identity("process-operation"),
        instance_id: "read-only-instance".to_string(),
        method: "inspect".to_string(),
        arguments: BTreeMap::new(),
    };
    let proposed = Output::EffectIntents {
        schema: wasm::EFFECT_INTENT_SCHEMA.to_string(),
        intents: vec![intent.clone()],
        result,
    };
    let bytes = serde_json::to_vec(&proposed).expect("proposal bytes");
    let proposal = admit(&bytes, &admitted, descriptor, &plan, &[]).expect("typed proposal");
    assert_eq!(proposal.intents(), &[intent]);
    let invalid_rows = [
        (
            serde_json::json!({"kind":"pure_result","schema":wasm::RESULT_SCHEMA,"result":{"count":"wrong"}}),
            Blocker::ResultContract,
        ),
        (serde_json::json!({"kind":"pure_result","schema":"unknown","result":{"count":7}}), Blocker::Schema),
        (
            serde_json::json!({"kind":"pure_result","schema":wasm::RESULT_SCHEMA,"result":{"count":7,"release_eligible":true}}),
            Blocker::ProhibitedContent,
        ),
        (
            serde_json::json!({"kind":"pure_result","schema":wasm::RESULT_SCHEMA,"result":{"count":7},"authority":true}),
            Blocker::Encoding,
        ),
        (
            serde_json::json!({"kind":"effect_intents","schema":wasm::EFFECT_INTENT_SCHEMA,"intents":[],"result":{"count":7}}),
            Blocker::IntentCount,
        ),
        (
            serde_json::json!({"kind":"effect_intents","schema":wasm::EFFECT_INTENT_SCHEMA,"intents":[{"type_id":TYPE_ID,"type_revision":plan.type_revision,"instance_id":"component-instance","method":METHOD,"arguments":{}}],"result":{"count":7}}),
            Blocker::IntentTarget,
        ),
    ];
    for (value, blocker) in invalid_rows {
        assert_eq!(
            admit(&serde_json::to_vec(&value).expect("negative bytes"), &admitted, descriptor, &plan, &[]),
            Err(blocker)
        );
    }
    assert_eq!(admit(&pure, &admitted, descriptor, &plan, &["count".to_string()]), Err(Blocker::ProhibitedContent));
    let mut no_authority = plan.clone();
    no_authority.capabilities.clear();
    no_authority.backend_route = lattice::operation::backend::plan_route(&lattice::operation::backend::RouteContext {
        requirements: &no_authority.execution_requirements,
        runtime: &no_authority.runtime,
        method_identity: &no_authority.method_identity,
        capabilities: &no_authority.capabilities,
        credentials: &no_authority.credentials,
        bounds: no_authority.bounds,
    })
    .expect("no-authority route");
    no_authority.identity = lattice::operation::identity::invocation(&no_authority).expect("no-authority identity");
    assert_eq!(admit(&bytes, &admitted, descriptor, &no_authority, &[]), Err(Blocker::IntentTarget));
}

#[test]
fn native_profile_binds_resources_and_refuses_lossy_limit_conversions() {
    let mut fixture = manifest();
    let profile = descriptor(&mut fixture);
    let original = wasm::runtime_profile_identity(profile).expect("exact native profile");
    profile.resources.fuel += 1;
    let changed = wasm::runtime_profile_identity(profile).expect("changed explicit fuel");
    assert_ne!(original, changed, "a bundle for the old fuel budget must not admit the new one");
    profile.resources.table_elements = u64::from(u32::MAX);
    let exact_boundary = wasm::runtime_profile_identity(profile).expect("exact native table boundary");
    assert_ne!(changed, exact_boundary);
    profile.resources.table_elements += 1;
    assert!(wasm::runtime_profile_identity(profile).is_err(), "table narrowing must not clamp or truncate");
    profile.resources.table_elements = resources().table_elements;
    profile.resources.memory_bytes += 1;
    assert!(wasm::runtime_profile_identity(profile).is_err(), "memory bytes must be exact Wasm pages");
    profile.resources.memory_bytes = 0;
    assert!(wasm::runtime_profile_identity(profile).is_err(), "zero memory cannot acquire a positive page cap");
}
