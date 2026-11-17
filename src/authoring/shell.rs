//! Effect-free author compilation and explicit source/expanded export shells.

use super::*;

pub fn compile(program: &CapturedProgram) -> Result<crate::engine::Workflow, &'static str> {
    program.validate()?;
    debug_assert!(program.files.contains_key(&program.entry));
    debug_assert!(program.files.len() <= crate::workflow_template::MAX_BUNDLE_FILES);
    let source = std::str::from_utf8(&program.files[&program.entry]).map_err(|_| "entry is not UTF-8")?;
    super::admission::require_import_free(source)?;
    let value = match program.kind {
        ProgramKind::Workflow => {
            if !program.parameters.is_empty() {
                return Err("flat source has no declared parameters");
            }
            crate::nickel::evaluate_source(source).map_err(|_| "source evaluation failed")?
        }
        ProgramKind::Parameterized => {
            let declaration = format!("let program = (\n{source}\n) in program.parameters");
            let defaults =
                crate::nickel::evaluate_source(&declaration).map_err(|_| "parameter declaration evaluation failed")?;
            let mut parameters: std::collections::BTreeMap<String, serde_json::Value> =
                serde_json::from_value(defaults).map_err(|_| "source parameters must be a record")?;
            if parameters.len() > 64 || program.parameters.keys().any(|key| !parameters.contains_key(key)) {
                return Err("parameter is not explicitly declared by source");
            }
            parameters.extend(program.parameters.clone());
            // Native value printing escapes Nickel interpolation. The selected
            // convenience JSON-to-Nickel serializer does not escape `%{` strings.
            let value = serde_json::to_value(&parameters).map_err(|_| "parameter encoding failed")?;
            let parameters: nickel_lang_core::eval::value::NickelValue =
                serde_json::from_value(value).map_err(|_| "parameter literal conversion failed")?;
            let parameters = parameters.to_string();
            let application = format!("let program = (\n{source}\n) in program.build ({parameters})");
            super::admission::require_import_free(&application)?;
            crate::nickel::evaluate_source(&application).map_err(|_| "source evaluation failed")?
        }
    };
    admit_evaluated_snapshot(program, value, AssociationState::CapturedSnapshot)
}

/// Retention for existing bundle/file evaluation routes is not source-editing admission.
pub(crate) fn admit_evaluated_snapshot(
    program: &CapturedProgram,
    value: serde_json::Value,
    state: AssociationState,
) -> Result<crate::engine::Workflow, &'static str> {
    program.validate()?;
    let output = identity("lattice.author-evaluation-json.v1", &value)?;
    let mut workflow =
        crate::workflow_format::from_nickel_value(value, &program.files).map_err(|_| "workflow admission failed")?;
    let diagnostics = crate::workflow_validation::validate_workflow(&workflow);
    if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
        return Err("workflow validation failed");
    }
    workflow.authoring = Some(SourceAssociation {
        program: std::sync::Arc::new(program.clone()),
        source_revision: program.identity()?,
        compiler: CompilerIdentity::current()?,
        evaluated_output: output,
        expanded_artifact: expanded_identity(&workflow)?,
        state,
    });
    Ok(workflow)
}

/// Replaces only this draft's preview/diagnostic. The admitted workflow is untouched.
pub fn preview(draft: &mut SourceDraft) -> Result<(), &'static str> {
    draft.preview = None;
    draft.diagnostic = None;
    draft.validate()?;
    match compile(&draft.program) {
        Ok(candidate) if candidate.meta.name == draft.workflow_name => {
            let input_identity = draft.program.identity()?;
            debug_assert!(candidate.authoring.as_ref().is_some_and(|source| source.source_revision == input_identity));
            debug_assert!(
                candidate
                    .authoring
                    .as_ref()
                    .is_some_and(|source| source.state == AssociationState::CapturedSnapshot)
            );
            draft.preview = Some(SourcePreview {
                input_identity,
                compiler: CompilerIdentity::current()?,
                candidate,
            });
            Ok(())
        }
        Ok(_) => {
            draft.diagnostic = Some("source workflow name differs from draft target".into());
            Err("source workflow name differs from draft target")
        }
        Err(category) => {
            draft.diagnostic = Some(category.into());
            Err(category)
        }
    }
}

/// Capture one explicitly selected file. Ambient imports are never discovered or
/// claimed as captured modules. Bundle capture supplies its already bounded map.
pub fn capture_file(path: &std::path::Path, kind: ProgramKind) -> Result<CapturedProgram, &'static str> {
    let entry = path.file_name().and_then(|name| name.to_str()).ok_or("source filename is invalid")?;
    let bytes = read_member(path)?;
    let program = CapturedProgram {
        entry: entry.into(),
        files: std::collections::BTreeMap::from([(entry.into(), bytes)]),
        kind,
        parameters: std::collections::BTreeMap::new(),
    };
    program.validate()?;
    Ok(program)
}

pub fn read_member(path: &std::path::Path) -> Result<Vec<u8>, &'static str> {
    let entry = path.file_name().and_then(|name| name.to_str()).ok_or("source filename is invalid")?;
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    let root = crate::capability::open_root_path(parent).map_err(|_| "source directory is unavailable")?;
    let file = root.open(entry).map_err(|_| "source file is unavailable")?;
    let bytes = crate::capability::read_bounded(file, crate::workflow_template::MAX_FILE_BYTES)
        .map_err(|_| "source file exceeds capture limit or is unreadable")?;
    if bytes.len() > crate::workflow_template::MAX_FILE_BYTES {
        return Err("source file exceeds capture limit");
    }
    Ok(bytes)
}

/// An exact source archive, not a generated Nickel program and not a closure receipt.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorExport {
    pub schema: String,
    pub association: SourceAssociation,
}

// r[impl lattice.source_authoring.exports]
pub fn source_export(workflow: &crate::engine::Workflow) -> Result<Vec<u8>, &'static str> {
    let mut workflow = workflow.clone();
    reconcile_materialized(&mut workflow)?;
    let association = workflow.authoring.ok_or("workflow has no captured author source")?;
    serde_json::to_vec(&AuthorExport {
        schema: "lattice.author-source.v1".into(),
        association,
    })
    .map_err(|_| "source archive encoding failed")
}

pub fn read_source_archive(path: &std::path::Path) -> Result<CapturedProgram, &'static str> {
    let file = std::fs::File::open(path).map_err(|_| "source archive is unavailable")?;
    // JSON byte arrays can require four bytes per captured byte plus separators.
    let bytes = crate::capability::read_bounded(
        cap_std::fs::File::from_std(file),
        crate::workflow_template::MAX_BUNDLE_BYTES.checked_mul(8).ok_or("source archive bound overflows")?,
    )
    .map_err(|_| "source archive exceeds limit or is unreadable")?;
    let archive: AuthorExport = serde_json::from_slice(&bytes).map_err(|_| "source archive is invalid")?;
    if archive.schema != "lattice.author-source.v1" {
        return Err("source archive schema is unsupported");
    }
    archive.association.validate()?;
    Ok(std::sync::Arc::unwrap_or_clone(archive.association.program))
}

pub fn expanded_export(workflow: &crate::engine::Workflow) -> String {
    let source = workflow
        .authoring
        .as_ref()
        .map(|association| association.source_revision.as_str())
        .unwrap_or("none");
    format!(
        "# Artifact: lattice.expanded-workflow.v1; retained source revision: {source}\n{}",
        crate::workflow_format::workflow_to_nickel(workflow)
    )
}

/// Normal export preserves associated source. Flattening always requires an explicit flag.
pub fn default_export(workflow: &crate::engine::Workflow, expanded: bool) -> Result<Vec<u8>, &'static str> {
    if expanded || workflow.authoring.is_none() {
        Ok(expanded_export(workflow).into_bytes())
    } else {
        source_export(workflow)
    }
}

pub fn capture_members(
    program: &mut CapturedProgram,
    root: &std::path::Path,
    members: &[String],
) -> Result<(), &'static str> {
    let directory = crate::capability::open_root_path(root).map_err(|_| "capture directory is unavailable")?;
    for member in members {
        if !safe_path(member) || program.files.contains_key(member) {
            return Err("captured member path is invalid or duplicated");
        }
        let file = directory.open(member).map_err(|_| "captured member is unavailable")?;
        let bytes = crate::capability::read_bounded(file, crate::workflow_template::MAX_FILE_BYTES)
            .map_err(|_| "captured member exceeds limit or is unreadable")?;
        program.files.insert(member.clone(), bytes);
        program.validate()?;
    }
    Ok(())
}

/// Bounded metadata diff. Full configuration is available only in explicit expanded export.
pub fn preview_summary(
    previous: Option<&crate::engine::Workflow>,
    candidate: &crate::engine::Workflow,
) -> Result<String, &'static str> {
    let portable = crate::workflow_format::workflow_to_portable(candidate);
    let before = previous.map(crate::workflow_format::workflow_to_portable);
    let mut changed = Vec::with_capacity(portable.configs.len().min(64));
    let mut changed_count = 0usize;
    for config in portable
        .configs
        .iter()
        .filter(|config| before.as_ref().is_none_or(|before| !before.configs.contains(config)))
    {
        changed_count = changed_count.checked_add(1).ok_or("preview change count overflows")?;
        if changed.len() < 64 {
            changed.push(config.node_id);
        }
    }
    debug_assert!(changed.len() <= 64);
    debug_assert!(changed.len() <= changed_count);
    let before_nodes = before.as_ref().map_or(0, |workflow| workflow.nodes.len());
    let before_edges = before.as_ref().map_or(0, |workflow| workflow.edges.len());
    Ok(format!(
        "Preview only; no workloads, credential delivery or authority. Nodes {before_nodes} -> {}; edges {before_edges} -> {}; changed config nodes (first 64 of {}): {:?}; materialized identity {}. Complete import closure and origins unavailable.",
        portable.nodes.len(),
        portable.edges.len(),
        changed_count,
        changed,
        expanded_identity(candidate)?.as_str()
    ))
}

pub fn validate_export_path(
    workflow: &crate::engine::Workflow,
    expanded: bool,
    path: &std::path::Path,
) -> Result<(), &'static str> {
    if workflow.authoring.is_some()
        && !expanded
        && !path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.ends_with(".author.json"))
    {
        return Err(
            "captured author source exports require a .author.json archive target; use --expanded for flat .ncl",
        );
    }
    Ok(())
}
