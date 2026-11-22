#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    unbounded_collection_growth,
    path_segment_repetition,
    reason = "the command shell maps explicit file inputs to the restore boundary and prints one JSON document"
)]

//! `lattice application-checkpoint`: the process boundary Kiln consumes (OD-1).

use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use serde::Serialize;

use super::CheckpointBinding;
use super::CheckpointRoot;
use super::Refusal;
use super::RestartReceipt;
use super::RestartRequest;

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Adopt a committed checkpoint into this data directory's fresh store; `workflow-exchange
    /// serve --checkpoint-binding` on the same data directory then runs it
    Restore {
        /// Typed Nickel workflow exchange handler profile of the target
        #[arg(long)]
        profile: PathBuf,
        /// `lattice.application-checkpoint-binding.v1` JSON for the target route
        #[arg(long)]
        binding: PathBuf,
        /// Checkpoint root holding the committed manifest and its objects
        #[arg(long)]
        source: PathBuf,
        /// `lattice.application-restart-request.v1` JSON issued with Kiln's admission
        #[arg(long)]
        request: PathBuf,
    },
    /// List committed manifests and any drain acknowledgement in a checkpoint root
    Inspect {
        #[arg(long)]
        root: PathBuf,
    },
}

#[derive(Debug, Serialize)]
struct RestoreReport {
    receipt: RestartReceipt,
    execution_state: crate::workflow_exchange::DurableExecutionState,
    nodes: Vec<NodeReport>,
}

#[derive(Debug, Serialize)]
struct NodeReport {
    workflow_node_id: u64,
    state: lattice_workflow_contract::NodeState,
}

#[derive(Debug, Serialize)]
struct Refused {
    refused: Refusal,
}

#[derive(Debug, Serialize)]
struct CommittedManifest {
    publication: u64,
    manifest: lattice_workflow_contract::Identity,
    producing_incarnation: u32,
    sequence: u64,
    completed_nodes: usize,
    uncertain_nodes: usize,
}

#[derive(Debug, Serialize)]
struct InspectReport {
    committed: Vec<CommittedManifest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    drained: Option<lattice_workflow_contract::Identity>,
}

pub fn run(app_config: &crate::config::AppConfig, args: &CommandArgs) -> ExitCode {
    let outcome = match &args.command {
        Command::Restore {
            profile,
            binding,
            source,
            request,
        } => restore(app_config, profile, binding, source, request).and_then(|report| print(&report)),
        Command::Inspect { root } => inspect(&CheckpointRoot::new(root.clone())).and_then(|report| print(&report)),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(refused) => {
            if print(&Refused { refused }).is_err() {
                eprintln!("application checkpoint refused: {refused}");
            }
            ExitCode::FAILURE
        }
    }
}

fn print<T: Serialize>(value: &T) -> Result<(), Refusal> {
    let text = serde_json::to_string(value).map_err(|_| Refusal::Encoding)?;
    println!("{text}");
    Ok(())
}

/// Read a `lattice.application-checkpoint-binding.v1` JSON file and validate it.
pub fn read_binding(path: &Path) -> Result<CheckpointBinding, Refusal> {
    let binding: CheckpointBinding = read_json(path)?;
    super::validate_binding(&binding)?;
    Ok(binding)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Refusal> {
    let bytes = std::fs::read(path).map_err(|error| {
        eprintln!("{}: {error}", path.display());
        Refusal::Io
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        eprintln!("{}: {error}", path.display());
        Refusal::Schema
    })
}

// r[impl lattice.application_checkpoint.restore]
fn restore(
    app_config: &crate::config::AppConfig,
    profile: &Path,
    binding: &Path,
    source: &Path,
    request: &Path,
) -> Result<RestoreReport, Refusal> {
    let profile = crate::workflow_exchange::load_profile(profile).map_err(|error| {
        eprintln!("{}: {error}", profile.display());
        Refusal::Schema
    })?;
    let binding: CheckpointBinding = read_json(binding)?;
    let request: RestartRequest = read_json(request)?;
    let store =
        crate::store::Store::new(crate::config::store_path(&app_config.data_dir)).map_err(|_| Refusal::Store)?;
    let mut handler =
        crate::workflow_exchange::ApplicationHandler::new(store.clone(), profile, app_config.effective_agent_runtime())
            .map_err(|error| {
                eprintln!("{error}");
                Refusal::Schema
            })?
            .with_application_checkpoint(binding)
            .map_err(|_| Refusal::CrossedRoute)?;
    let receipt = handler.restore_checkpoint(&CheckpointRoot::new(source), &request, super::now_unix_ms())?;
    let record = store
        .load_workflow_exchange(&request.execution)
        .map_err(|_| Refusal::Store)?
        .ok_or(Refusal::Store)?;
    Ok(RestoreReport {
        receipt,
        execution_state: record.state,
        nodes: record
            .nodes
            .iter()
            .map(|node| NodeReport {
                workflow_node_id: node.workflow_node_id,
                state: node.state,
            })
            .collect(),
    })
}

fn inspect(root: &CheckpointRoot) -> Result<InspectReport, Refusal> {
    let mut committed = Vec::new();
    for (publication, identity) in root.committed()? {
        let Some(bytes) = root.manifest_bytes(&identity)? else {
            continue;
        };
        let Ok(manifest) = serde_json::from_slice::<super::Manifest>(&bytes) else {
            continue;
        };
        committed.push(CommittedManifest {
            publication,
            manifest: identity,
            producing_incarnation: manifest.producing_incarnation,
            sequence: manifest.execution.sequence,
            completed_nodes: manifest.completed_nodes.len(),
            uncertain_nodes: manifest.uncertain_nodes.len(),
        });
    }
    Ok(InspectReport {
        committed,
        drained: root.drain_acknowledgement()?.map(|acknowledgement| acknowledgement.manifest),
    })
}
