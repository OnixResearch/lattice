//! Exact captured-only author evaluation. No ambient import or source rewriting.

fn evaluate(
    files: &std::collections::BTreeMap<String, Vec<u8>>,
    entry: &str,
) -> Result<super::EvaluatorObservation, &'static str> {
    let (result, worker_artifact) =
        crate::nickel::evaluate_captured(entry, files).map_err(|_| "captured evaluation failed")?;
    Ok(super::EvaluatorObservation {
        modules: files.clone(),
        entry: entry.to_owned(),
        result,
        worker_artifact,
    })
}

fn wrapper_name(program: &super::CapturedProgram) -> Result<(String, String), &'static str> {
    let (parent, basename) = program.entry.rsplit_once('/').unwrap_or(("", &program.entry));
    let wrapper = if parent.is_empty() {
        "lattice-author-application.ncl".to_owned()
    } else {
        format!("{parent}/lattice-author-application.ncl")
    };
    if program.files.contains_key(&wrapper) {
        return Err("reserved author application module already exists");
    }
    Ok((wrapper, format!("./{basename}")))
}

pub(super) fn compile(
    program: &super::CapturedProgram,
) -> Result<(serde_json::Value, super::EvaluatorObservation), &'static str> {
    program.validate()?;
    let mut files = program.files.clone();
    if super::construction::is_native_kind(program.kind) {
        select_construction_library(&mut files)?;
    }
    let entry = match program.kind {
        super::ProgramKind::Workflow | super::ProgramKind::Construction => flat_entry(program)?,
        super::ProgramKind::Parameterized | super::ProgramKind::ParameterizedConstruction => {
            apply_parameters(program, &mut files)?
        }
    };
    let observation = evaluate(&files, &entry)?;
    let value = serde_json::from_str(&observation.result.json).map_err(|_| "evaluation output is not JSON")?;
    Ok((value, observation))
}

/// Compiles an observed construction evaluation into its workflow and retained facts.
pub(super) fn compile_construction(
    observation: &super::EvaluatorObservation,
) -> Result<(crate::engine::Workflow, super::ConstructionAssociation), &'static str> {
    let (workflow, projection, binding, origins) = super::construction::lowering::compile(observation)?;
    let construction = super::ConstructionAssociation {
        projection_json: serde_json::to_vec(&projection).map_err(|_| "construction projection encoding failed")?,
        binding_json: serde_json::to_vec(&binding).map_err(|_| "construction binding encoding failed")?,
        origins,
        port_map: super::construction::lowering::port_map(&workflow, &projection)?,
    };
    Ok((workflow, construction))
}

fn flat_entry(program: &super::CapturedProgram) -> Result<String, &'static str> {
    if !program.parameters.is_empty() {
        return Err("flat source has no declared parameters");
    }
    Ok(program.entry.clone())
}

fn select_construction_library(files: &mut std::collections::BTreeMap<String, Vec<u8>>) -> Result<(), &'static str> {
    select_module(
        files,
        choregraph_nickel::construction::model::profile::LIBRARY_MODULE,
        choregraph_nickel::construction::model::profile::LIBRARY_BYTES,
    )?;
    select_module(
        files,
        choregraph_nickel::construction::model::profile::CONTRACTS_MODULE,
        choregraph_nickel::construction::model::profile::CONTRACTS_BYTES,
    )
}

/// Adds a selected construction module; captured bytes under that name must be identical.
fn select_module(
    files: &mut std::collections::BTreeMap<String, Vec<u8>>,
    name: &str,
    bytes: &[u8],
) -> Result<(), &'static str> {
    if files.get(name).is_some_and(|existing| existing.as_slice() != bytes) {
        return Err("captured construction library differs from selected bytes");
    }
    files.insert(name.into(), bytes.to_vec());
    Ok(())
}

/// Evaluates declared parameter defaults, then applies them through a generated
/// application module and returns that module as the evaluation entry.
fn apply_parameters(
    program: &super::CapturedProgram,
    files: &mut std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<String, &'static str> {
    let (wrapper, relative) = wrapper_name(program)?;
    let import = crate::nickel::string_literal(&relative);
    files.insert(wrapper.clone(), format!("let program = import {import} in program.parameters").into_bytes());
    let defaults = evaluate(files, &wrapper)?;
    let selected = selected_parameters(program, &defaults.result.json)?;
    let literal =
        crate::nickel::value_literal(&serde_json::to_value(selected).map_err(|_| "parameter encoding failed")?)
            .map_err(|_| "parameter literal conversion failed")?;
    files.insert(wrapper.clone(), format!("let program = import {import} in program.build ({literal})").into_bytes());
    Ok(wrapper)
}

/// Declared defaults overridden only by explicitly declared parameters.
fn selected_parameters(
    program: &super::CapturedProgram,
    defaults: &str,
) -> Result<std::collections::BTreeMap<String, serde_json::Value>, &'static str> {
    let mut selected: std::collections::BTreeMap<String, serde_json::Value> =
        serde_json::from_str(defaults).map_err(|_| "source parameters must be a record")?;
    if selected.len() > 64 || program.parameters.keys().any(|key| !selected.contains_key(key)) {
        return Err("parameter is not explicitly declared by source");
    }
    let declared = selected.len();
    selected.extend(program.parameters.clone());
    debug_assert_eq!(selected.len(), declared, "overrides only replace declared parameters");
    Ok(selected)
}
