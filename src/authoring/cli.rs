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
    Draft {
        workflow: String,
        id: String,
        #[arg(long)]
        input: Option<std::path::PathBuf>,
        #[arg(long, requires = "input")]
        archive: bool,
        #[arg(long, requires = "input", conflicts_with = "archive")]
        parameterized: bool,
        /// Explicit captured members relative to input's directory; not import discovery.
        #[arg(long, requires = "input", conflicts_with = "archive")]
        member: Vec<String>,
    },
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
    /// Preserve exact captured inputs unless --expanded explicitly requests flat Nickel.
    Export {
        workflow: String,
        output: std::path::PathBuf,
        #[arg(long)]
        expanded: bool,
    },
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
        Command::Draft {
            workflow: name,
            id,
            input,
            archive,
            parameterized,
            member,
        } => {
            if store.load_source_draft(id).map_err(|e| e.to_string())?.is_some() {
                return Err("draft identifier already exists; choose a distinct draft".into());
            }
            let current = store.load_workflow(name).map_err(|e| e.to_string())?;
            let kind = if *parameterized {
                ProgramKind::Parameterized
            } else {
                ProgramKind::Workflow
            };
            let program = captured_program(input.as_deref(), *archive, kind, member, current.as_ref())?;
            let draft = SourceDraft {
                id: id.clone(),
                workflow_name: name.clone(),
                expected_revision: current.and_then(|workflow| workflow.catalog_revision),
                program,
                diagnostic: None,
                preview: None,
            };
            store.save_source_draft(&draft).map_err(|e| e.to_string())?;
            println!("Captured source draft; input revision {} (snapshot only)", draft.program.identity()?.as_str());
        }
        Command::SetFile {
            draft: id,
            member,
            input,
        } => {
            let mut draft = draft(store, id)?;
            replace_member(&mut draft, member, input)?;
            store.save_source_draft(&draft).map_err(|e| e.to_string())?;
            println!("Updated captured member; preview invalidated");
        }
        Command::SetParameters { draft: id, input } => {
            let mut draft = draft(store, id)?;
            replace_parameters(&mut draft, input)?;
            store.save_source_draft(&draft).map_err(|e| e.to_string())?;
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
        Command::Export {
            workflow: name,
            output,
            expanded,
        } => export(&workflow(store, name)?, output, *expanded)?,
    }
    Ok(())
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
    if let Some(source) = &workflow.authoring {
        println!(
            "source_revision={}\nstate={:?}\ncompiler_current={}\nexpanded_artifact={}\ncomplete_module_provenance={}\norigins={}",
            source.source_revision.as_str(),
            source.state,
            source.compiler == CompilerIdentity::current()?,
            source.expanded_artifact.as_str(),
            COMPLETE_MODULE_PROVENANCE_UNAVAILABLE,
            ORIGIN_UNAVAILABLE
        );
    } else {
        println!("No captured source association (legacy materialized workflow)");
    }
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
