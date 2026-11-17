pub(super) fn catalog(executable: &std::path::Path) -> anyhow::Result<crate::operation::model::Catalog> {
    let mut definition = crate::operation::model::OperationType {
        id: super::TYPE_ID.into(),
        schema_version: crate::operation::model::TYPE_SCHEMA_VERSION,
        revision: format!("blake3:{}", "0".repeat(64)),
        description: "Aspen artifact-bound tiled render consumer".into(),
        tags: vec!["render".into()],
        methods: methods(),
        runtime: crate::operation::model::RuntimeRequirement {
            kind: super::RUNTIME_KIND.into(),
            key: super::RUNTIME_KEY.into(),
            executor_identity: crate::operation::identity::executor_file(executable)?,
        },
        execution_requirements: super::requirements(),
        capabilities: vec![crate::operation::model::CapabilityClass::ProcessRead],
        credential_bindings: vec![],
        bounds: crate::operation::model::Bounds {
            timeout_seconds: 600,
            input_bytes: 256 * 1024,
            output_bytes: 64 * 1024,
            artifact_bytes: 4_194_304,
        },
        required_non_claims: crate::operation::model::REQUIRED_NON_CLAIMS.iter().map(|s| (*s).into()).collect(),
    };
    definition.revision = crate::operation::identity::type_revision(&definition)?;
    assert_eq!(definition.methods.len(), super::METHODS.len(), "closed render method inventory");
    assert!(super::type_is_supported(&definition), "authored render catalog must match the runtime profile");
    Ok(crate::operation::model::Catalog {
        schema: crate::operation::model::CATALOG_SCHEMA.into(),
        types: vec![definition],
    })
}

pub(super) fn instance(
    catalog: &crate::operation::model::Catalog,
    repository: &std::path::Path,
    binding_file: &std::path::Path,
    binding: molten_batch_core::Id,
) -> crate::operation::model::InstanceFile {
    crate::operation::model::InstanceFile {
        schema: crate::operation::model::INSTANCE_SCHEMA.into(),
        instances: vec![crate::operation::model::OperationInstance {
            id: "admitted-render".into(),
            display_alias: "Exact admitted render".into(),
            type_id: super::TYPE_ID.into(),
            type_revision: catalog.types[0].revision.clone(),
            target: instance_target(repository, binding_file, binding),
            tags: std::collections::BTreeMap::new(),
            method_defaults: std::collections::BTreeMap::new(),
            credentials: std::collections::BTreeMap::new(),
        }],
    }
}

fn methods() -> Vec<crate::operation::model::Method> {
    super::METHODS
        .iter()
        .map(|name| crate::operation::model::Method {
            name: (*name).into(),
            description: format!("Identity-bound render {name}; mutation requires the configured effect authority"),
            arguments: crate::operation::model::RecordContract {
                fields: vec![],
                deny_unknown_fields: true,
            },
            output: super::output_contract(),
            preflight: super::PREFLIGHT_CHECKS.to_vec(),
            capabilities: vec![crate::operation::model::CapabilityClass::ProcessRead],
            side_effect: crate::operation::model::SideEffectClass::Mutation,
        })
        .collect()
}

fn instance_target(
    repository: &std::path::Path,
    binding_file: &std::path::Path,
    binding: molten_batch_core::Id,
) -> std::collections::BTreeMap<String, serde_json::Value> {
    std::collections::BTreeMap::from([
        (crate::operation::runtime::REPOSITORY_ROOT_TARGET.into(), serde_json::json!(repository)),
        (super::FILE_TARGET.into(), serde_json::json!(binding_file)),
        (super::ID_TARGET.into(), serde_json::json!(binding.to_string())),
    ])
}
