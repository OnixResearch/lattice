//! Source-draft commands; this route never executes a workflow.

use super::*;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Capture an editable draft; omitting input copies the admitted source snapshot.
    Draft(DraftArgs),
    /// Replace exact captured member bytes without touching the admitted workflow.
    SetFile {
        draft: String,
        member: String,
        input: std::path::PathBuf,
    },
    /// Set overrides as a JSON object; preview rejects names not declared by source.
    SetParameters { draft: String, input: std::path::PathBuf },
    /// Evaluate/admit graph structure only, persist failures, optionally write flat preview.
    Preview {
        draft: String,
        #[arg(long)]
        expanded_output: Option<std::path::PathBuf>,
    },
    /// Atomically admit the exact successful durable preview using expected-revision CAS.
    Admit { draft: String },
    /// Report association facts without dumping captured source or configuration.
    Inspect { workflow: String },
    /// Inspect a selected graph node or port's verified logical source location.
    Origin {
        workflow: String,
        node: u64,
        #[arg(long)]
        port: Option<u64>,
    },
    /// Preserve exact captured inputs unless --expanded explicitly requests flat Nickel.
    Export {
        workflow: String,
        output: std::path::PathBuf,
        #[arg(long)]
        expanded: bool,
    },
}

#[derive(Debug, clap::Args)]
pub struct DraftArgs {
    pub workflow: String,
    pub id: String,
    #[arg(long)]
    pub input: Option<std::path::PathBuf>,
    #[arg(long, requires = "input")]
    pub archive: bool,
    #[arg(long, requires = "input", conflicts_with = "archive")]
    pub parameterized: bool,
    /// Compile the selected captured construction profile rather than a flat workflow.
    #[arg(long, requires = "input", conflicts_with = "archive")]
    pub construction: bool,
    /// Explicit captured members relative to input's directory; not import discovery.
    #[arg(long, requires = "input", conflicts_with = "archive")]
    pub member: Vec<String>,
}

impl DraftArgs {
    const fn kind(&self) -> ProgramKind {
        match (self.construction, self.parameterized) {
            (false, false) => ProgramKind::Workflow,
            (false, true) => ProgramKind::Parameterized,
            (true, false) => ProgramKind::Construction,
            (true, true) => ProgramKind::ParameterizedConstruction,
        }
    }
}

fn draft(store: &crate::store::Store, id: &str) -> Result<SourceDraft, String> {
    store
        .load_source_draft(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "source draft not found".into())
}
fn workflow(store: &crate::store::Store, name: &str) -> Result<crate::engine::Workflow, String> {
    store.load_workflow(name).map_err(|e| e.to_string())?.ok_or_else(|| "workflow not found".into())
}

pub fn run(store: &crate::store::Store, args: &Args) -> Result<(), String> {
    match &args.command {
        Command::Draft(request) => capture_draft(store, request)?,
        Command::SetFile {
            draft: id,
            member,
            input,
        } => {
            edit_draft(store, id, |draft| replace_member(draft, member, input))?;
            println!("Updated captured member; preview invalidated");
        }
        Command::SetParameters { draft: id, input } => {
            edit_draft(store, id, |draft| replace_parameters(draft, input))?;
            println!("Updated parameter overrides; preview invalidated");
        }
        Command::Preview {
            draft: id,
            expanded_output,
        } => preview(store, id, expanded_output.as_deref())?,
        Command::Admit { draft: id } => {
            let admitted = store.admit_source_draft(&draft(store, id)?).map_err(|e| e.to_string())?;
            debug_assert!(admitted.catalog_revision.is_some());
            debug_assert!(admitted.authoring.is_some());
            println!(
                "Admitted catalog revision {}",
                admitted.catalog_revision.as_ref().ok_or("revision missing")?.as_str()
            );
        }
        Command::Inspect { workflow: name } => inspect(&workflow(store, name)?)?,
        Command::Origin {
            workflow: name,
            node,
            port,
        } => origin(&workflow(store, name)?, *node, *port)?,
        Command::Export {
            workflow: name,
            output,
            expanded,
        } => export(&workflow(store, name)?, output, *expanded)?,
    }
    Ok(())
}

fn capture_draft(store: &crate::store::Store, request: &DraftArgs) -> Result<(), String> {
    if store.load_source_draft(&request.id).map_err(|e| e.to_string())?.is_some() {
        return Err("draft identifier already exists; choose a distinct draft".into());
    }
    let current = store.load_workflow(&request.workflow).map_err(|e| e.to_string())?;
    let program =
        captured_program(request.input.as_deref(), request.archive, request.kind(), &request.member, current.as_ref())?;
    let draft = SourceDraft {
        id: request.id.clone(),
        workflow_name: request.workflow.clone(),
        expected_revision: current.and_then(|workflow| workflow.catalog_revision),
        program,
        diagnostic: None,
        preview: None,
    };
    store.save_source_draft(&draft).map_err(|e| e.to_string())?;
    println!("Captured source draft; input revision {} (snapshot only)", draft.program.identity()?.as_str());
    Ok(())
}

fn edit_draft(
    store: &crate::store::Store,
    id: &str,
    edit: impl FnOnce(&mut SourceDraft) -> Result<(), String>,
) -> Result<(), String> {
    let mut draft = draft(store, id)?;
    edit(&mut draft)?;
    store.save_source_draft(&draft).map_err(|e| e.to_string())
}

// r[impl lattice.graph_construction.origins.navigation]
fn origin(workflow: &crate::engine::Workflow, node: u64, port: Option<u64>) -> Result<(), String> {
    let element = origin_element(workflow, node, port)?;
    let origin = construction::lowering::origin_for(workflow, &element).ok_or(ORIGIN_UNAVAILABLE)?;
    let location = origin.location.map_or_else(
        || "unavailable".to_owned(),
        |span| format!("{}:{}..{}", span.logical_module, span.start, span.end),
    );
    println!(
        "template={} instance={} generation={} location={}",
        origin.template,
        origin.instance_path.join("/"),
        origin.generation.as_deref().unwrap_or("unavailable"),
        location
    );
    Ok(())
}

/// The selected node, or one of its own ports, as a native element reference.
fn origin_element(workflow: &crate::engine::Workflow, node: u64, port: Option<u64>) -> Result<String, String> {
    let node_id = crate::engine::workflow::NodeId(node);
    if workflow.graph.node(node_id).is_none() {
        return Err("selected node does not exist".into());
    }
    let Some(port) = port else {
        return Ok(format!("node/{node}"));
    };
    if workflow.graph.port_owner(crate::engine::workflow::PortId(port)) != Some(node_id) {
        return Err("selected port does not belong to node".into());
    }
    Ok(format!("port/{port}"))
}

fn captured_program(
    input: Option<&std::path::Path>,
    is_archive: bool,
    kind: ProgramKind,
    members: &[String],
    current: Option<&crate::engine::Workflow>,
) -> Result<CapturedProgram, String> {
    let Some(input) = input else {
        return Ok(current
            .and_then(|workflow| workflow.authoring.as_ref())
            .ok_or("workflow has no captured source; provide --input")?
            .program
            .as_ref()
            .clone());
    };
    if is_archive {
        return Ok(shell::read_source_archive(input)?);
    }
    let mut program = shell::capture_file(input, kind)?;
    let root = input.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    shell::capture_members(&mut program, root, members)?;
    debug_assert!(program.files.contains_key(&program.entry));
    debug_assert!(program.files.len() <= crate::workflow_template::MAX_BUNDLE_FILES);
    Ok(program)
}

fn replace_member(draft: &mut SourceDraft, member: &str, input: &std::path::Path) -> Result<(), String> {
    if !draft.program.files.contains_key(member) {
        return Err("member is not in captured revision".into());
    }
    draft.program.files.insert(member.to_owned(), shell::read_member(input)?);
    draft.preview = None;
    draft.diagnostic = None;
    Ok(())
}

fn replace_parameters(draft: &mut SourceDraft, input: &std::path::Path) -> Result<(), String> {
    draft.program.parameters =
        serde_json::from_slice(&shell::read_member(input)?).map_err(|_| "parameter overrides must be a JSON object")?;
    draft.preview = None;
    draft.diagnostic = None;
    Ok(())
}

fn preview(store: &crate::store::Store, id: &str, output: Option<&std::path::Path>) -> Result<(), String> {
    let mut draft = draft(store, id)?;
    store.preview_source_draft(&mut draft).map_err(|e| e.to_string())?;
    let candidate = &draft.preview.as_ref().ok_or("preview is unavailable")?.candidate;
    debug_assert_eq!(candidate.meta.name, draft.workflow_name);
    debug_assert!(candidate.authoring.is_some());
    println!(
        "{}",
        shell::preview_summary(
            store.load_workflow(&draft.workflow_name).map_err(|e| e.to_string())?.as_ref(),
            candidate
        )?
    );
    if let Some(path) = output {
        std::fs::write(path, shell::expanded_export(candidate)).map_err(|_| "expanded preview could not be written")?;
    }
    Ok(())
}

fn inspect(workflow: &crate::engine::Workflow) -> Result<(), String> {
    let Some(source) = &workflow.authoring else {
        println!("No captured source association (legacy materialized workflow)");
        return Ok(());
    };
    let provenance = source
        .observation
        .as_ref()
        .map_or(COMPLETE_MODULE_PROVENANCE_UNAVAILABLE, |_| "native-observed-closure");
    let origins = source.construction.as_ref().map_or(ORIGIN_UNAVAILABLE, |_| "native-template-instance-map");
    println!(
        "source_revision={}\nstate={:?}\ncompiler_current={}\nexpanded_artifact={}\ncomplete_module_provenance={provenance}\norigins={origins}",
        source.source_revision.as_str(),
        source.state,
        source.compiler == CompilerIdentity::current()?,
        source.expanded_artifact.as_str(),
    );
    Ok(())
}

fn export(workflow: &crate::engine::Workflow, output: &std::path::Path, is_expanded: bool) -> Result<(), String> {
    shell::validate_export_path(workflow, is_expanded, output)?;
    std::fs::write(output, shell::default_export(workflow, is_expanded)?).map_err(|_| "export could not be written")?;
    println!(
        "Exported {} artifact",
        if is_expanded || workflow.authoring.is_none() {
            "expanded"
        } else {
            "author-source"
        }
    );
    Ok(())
}
