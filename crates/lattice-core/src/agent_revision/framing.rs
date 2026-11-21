//! `lattice.agent-revision-components.v1`: the issue #25 section 1
//! components the profile revision does not frame yet.
//!
//! The implementation-attempt profile revision already frames the harness
//! and executable pins, the model route, compute, workspace paths and the
//! optional capability declaration. It does not frame the memory policy,
//! checkpoint semantics, the harness tool protocol, model requirements or
//! producer/consumer compatibility. An operator may declare them here. The
//! revision frames the [`digest`] only when a declaration is present, so
//! every existing revision keeps its bytes, the way capability does.

// r[impl lattice.agent_revisions.framing]
// r[impl lattice.agent_revisions.framing.scenario.undeclared]
// r[impl lattice.agent_revisions.framing.scenario.declared]

pub const COMPONENTS_SCHEMA: &str = "lattice.agent-revision-components.v1";
pub const COMPONENTS_DOMAIN: &str = "lattice-agent-revision-components-v1";
/// Tool names or model requirements one declaration may carry.
pub const MAX_ITEMS: u32 = 32;

/// Optional components; an absent one frames nothing.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Components {
    pub schema: String,
    #[serde(default = "unset", skip_serializing_if = "Option::is_none")]
    pub memory: Option<Memory>,
    #[serde(default = "unset", skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<Checkpoint>,
    #[serde(default = "unset", skip_serializing_if = "Option::is_none")]
    pub tools: Option<Tools>,
    #[serde(default = "unset", skip_serializing_if = "Option::is_none")]
    pub model_requirements: Option<Vec<Requirement>>,
    #[serde(default = "unset", skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<Compatibility>,
}

/// Session memory. The OMP attempt worker runs single-shot `--no-session`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Memory {
    SingleShotNoSession {},
    SessionScoped { schema: String },
}

/// Supported checkpoint semantics; support itself is owned by Lattice #12.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Checkpoint {
    Unsupported {},
    Supported { schema: String },
}

/// The tool executor protocol and the exact tool names the harness exposes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tools {
    pub protocol: String,
    pub names: Vec<String>,
}

/// One route's model requirement document, by canonical reference.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub route: String,
    pub requirement_ref: String,
}

/// The producer and consumer contracts the revision is compatible with.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    pub producer: String,
    pub consumer: String,
}

/// A declaration refused before any identity is computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "defect", rename_all = "snake_case")]
pub enum Defect {
    Schema,
    Name {
        field: &'static str,
    },
    Reference {
        field: &'static str,
    },
    Count {
        field: &'static str,
    },
    Duplicate {
        field: &'static str,
    },
    /// The shared framing core refused a part; no identity exists.
    Framing,
}

#[must_use]
pub const fn unset<T>() -> Option<T> {
    None
}

/// An absent declaration; existing requests decode to it.
#[must_use]
pub const fn absent() -> Option<Components> {
    None
}

/// A capability-grammar name, or the given defect.
fn name(value: &str, defect: Defect) -> Result<(), Defect> {
    if crate::execution_binding::capability::is_name(value) {
        Ok(())
    } else {
        Err(defect)
    }
}

fn distinct<'a>(values: impl Iterator<Item = &'a str> + Clone, field: &'static str) -> Result<(), Defect> {
    let count = values.clone().count();
    if count == 0 || !super::is_within(count, MAX_ITEMS) {
        return Err(Defect::Count { field });
    }
    if values.collect::<std::collections::BTreeSet<_>>().len() != count {
        return Err(Defect::Duplicate { field });
    }
    Ok(())
}

/// Check the schema and every declared component.
pub fn validate(components: &Components) -> Result<(), Defect> {
    if components.schema != COMPONENTS_SCHEMA {
        return Err(Defect::Schema);
    }
    if let Some(Memory::SessionScoped { schema }) = &components.memory {
        name(schema, Defect::Name { field: "memory" })?;
    }
    if let Some(Checkpoint::Supported { schema }) = &components.checkpoint {
        name(schema, Defect::Name { field: "checkpoint" })?;
    }
    if let Some(tools) = &components.tools {
        let defect = Defect::Name { field: "tools" };
        name(&tools.protocol, defect)?;
        tools.names.iter().try_for_each(|tool| name(tool, defect))?;
        distinct(tools.names.iter().map(String::as_str), "tools")?;
    }
    if let Some(requirements) = &components.model_requirements {
        let defect = Defect::Name {
            field: "model_requirements",
        };
        requirements.iter().try_for_each(|requirement| name(&requirement.route, defect))?;
        if !requirements.iter().all(|requirement| super::is_reference(&requirement.requirement_ref)) {
            return Err(Defect::Reference {
                field: "model_requirements",
            });
        }
        distinct(requirements.iter().map(|requirement| requirement.route.as_str()), "model_requirements")?;
    }
    if let Some(compatibility) = &components.compatibility {
        let defect = Defect::Name { field: "compatibility" };
        name(&compatibility.producer, defect)?;
        name(&compatibility.consumer, defect)?;
    }
    debug_assert_eq!(components.schema, COMPONENTS_SCHEMA);
    debug_assert!(components.tools.as_ref().is_none_or(|tools| !tools.names.is_empty()));
    Ok(())
}

fn memory_text(memory: &Memory) -> String {
    match memory {
        Memory::SingleShotNoSession {} => "single_shot_no_session".to_owned(),
        Memory::SessionScoped { schema } => format!("session_scoped:{schema}"),
    }
}

fn checkpoint_text(checkpoint: &Checkpoint) -> String {
    match checkpoint {
        Checkpoint::Unsupported {} => "unsupported".to_owned(),
        Checkpoint::Supported { schema } => format!("supported:{schema}"),
    }
}

/// The `lattice-agent-revision-components-v1` digest, spelled `blake3:<hex>`,
/// or `None` when nothing is declared. Tool names and requirements are
/// sorted, so their order never changes the digest.
pub fn digest(components: Option<&Components>) -> Result<Option<String>, Defect> {
    let Some(components) = components else {
        return Ok(None);
    };
    validate(components)?;
    let framing_failed = |_| Defect::Framing;
    let domain = content_identity_core::Domain::new(COMPONENTS_DOMAIN).map_err(framing_failed)?;
    let mut hasher = content_identity_core::IdentityHasher::new(domain);
    let memory = components.memory.as_ref().map(memory_text);
    let checkpoint = components.checkpoint.as_ref().map(checkpoint_text);
    let protocol = components.tools.as_ref().map(|tools| tools.protocol.as_str());
    let mut tools = components.tools.as_ref().map_or_else(Vec::new, |tools| tools.names.clone());
    tools.sort_unstable();
    let mut requirements = components
        .model_requirements
        .iter()
        .flatten()
        .map(|requirement| format!("{}={}", requirement.route, requirement.requirement_ref))
        .collect::<Vec<_>>();
    requirements.sort_unstable();
    let producer = components.compatibility.as_ref().map(|compatibility| compatibility.producer.as_str());
    let consumer = components.compatibility.as_ref().map(|compatibility| compatibility.consumer.as_str());
    let tool_items = tools.iter().map(String::as_str).collect::<Vec<_>>();
    let requirement_items = requirements.iter().map(String::as_str).collect::<Vec<_>>();
    hasher
        .update_tagged_str(b"schema", &components.schema)
        .map_err(framing_failed)?
        .update_optional_str(b"memory", memory.as_deref())
        .map_err(framing_failed)?
        .update_optional_str(b"checkpoint", checkpoint.as_deref())
        .map_err(framing_failed)?
        .update_optional_str(b"tool-protocol", protocol)
        .map_err(framing_failed)?
        .update_ordered_strs(b"tools", &tool_items)
        .map_err(framing_failed)?
        .update_ordered_strs(b"model-requirements", &requirement_items)
        .map_err(framing_failed)?
        .update_optional_str(b"producer", producer)
        .map_err(framing_failed)?
        .update_optional_str(b"consumer", consumer)
        .map_err(framing_failed)?;
    debug_assert_eq!(tool_items.len(), tools.len());
    debug_assert_eq!(requirement_items.len(), requirements.len());
    Ok(Some(format!("blake3:{}", hasher.finish().to_hex())))
}
