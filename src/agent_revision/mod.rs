//! `lattice agent-revision`: the imperative shell of the agent revision
//! lifecycle (Lattice #25).
//!
//! The pure core is `lattice_core::agent_revision`. This shell resolves a
//! bundle from an exported implementation-attempt request, persists one
//! lifecycle per definition through the store's version-checked rows and
//! calls two injected ports: [`PreparationPort`] (runtime preparation and
//! cleanup, owned by Kiln #49) and [`QualificationPort`] (qualification
//! attachments, owned by Animus #18). The standalone CLI installs
//! [`UnavailablePreparation`] and [`UnavailableQualification`]: preparation
//! then records `unknown`, and promotion and rollback are refused as
//! `qualification_unavailable` with owner `animus#18`.

// r[impl lattice.agent_revisions.shell]
// r[impl lattice.agent_revisions.ports]
// r[impl lattice.agent_revisions.promotion.scenario.unavailable]

pub mod bundle;
mod commands;
mod decisions;
mod port;
pub mod seam;

pub use commands::Seed;
pub use commands::transact;

type Ticket = lattice_core::agent_revision::ops::prepare::Ticket;
type Bundle = lattice_core::agent_revision::facts::Bundle;
type Sweep = lattice_core::agent_revision::ops::retention::Sweep;
type Query = lattice_core::agent_revision::ports::Query;

/// Runtime preparation and cleanup by the deployment owner. Observations are
/// facts the owner reports; Lattice performs no preparation itself.
pub trait PreparationPort: Send {
    /// Prepare the bundle for exactly this ticket. Repeating a ticket must
    /// reconcile the same attempt, never start a second one.
    fn prepare(&mut self, ticket: &Ticket, bundle: &Bundle) -> lattice_core::agent_revision::ports::Prepared;
    /// Release exactly the listed artifacts.
    fn cleanup(&mut self, sweep: &Sweep) -> lattice_core::agent_revision::ports::Cleanup;
}

/// The shared qualification-port contract.
pub trait QualificationPort: Send {
    fn answer(&mut self, query: &Query) -> lattice_core::agent_revision::ports::Verdict;
}

/// No preparation owner is installed; every preparation and cleanup is unknown.
#[derive(Debug, Clone, Copy)]
pub struct UnavailablePreparation;

/// No qualification producer is installed.
#[derive(Debug, Clone, Copy)]
pub struct UnavailableQualification;

/// Scripted preparation and cleanup observations for deterministic tests.
#[derive(Debug, Default)]
pub struct FixturePreparation {
    prepared: std::collections::VecDeque<lattice_core::agent_revision::ports::Prepared>,
    cleanups: std::collections::VecDeque<lattice_core::agent_revision::ports::Cleanup>,
    tickets: Vec<Ticket>,
    sweeps: Vec<Sweep>,
}

/// Answers keyed by the exact query; any other subject is `subject_mismatch`.
#[derive(Debug, Default)]
pub struct FixtureQualification {
    answers: Vec<(Query, lattice_core::agent_revision::ports::Verdict)>,
    queries: Vec<Query>,
}

/// The ports of one command.
pub struct Ports<'a> {
    pub preparation: &'a mut dyn PreparationPort,
    pub qualification: &'a mut dyn QualificationPort,
}

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Print the inert bundle an exported implementation-attempt request resolves to.
    Resolve {
        #[arg(long)]
        request: std::path::PathBuf,
    },
    /// Record the request's revision as the definition's desired revision.
    Desire {
        #[arg(long)]
        request: std::path::PathBuf,
        #[arg(long)]
        store: std::path::PathBuf,
    },
    /// Prepare the desired revision through the preparation port.
    Prepare {
        #[arg(long)]
        store: std::path::PathBuf,
        #[arg(long)]
        definition: String,
        #[arg(long)]
        revision: String,
    },
    /// Select a prepared, qualified revision for new sessions.
    Promote(SelectArgs),
    /// Select an earlier selected, usable revision for new sessions.
    Rollback(SelectArgs),
    /// Revoke a revision, a qualification attachment or a session.
    Revoke(RevokeArgs),
    /// Pin a session to the revision selected for new sessions.
    AdmitSession {
        #[arg(long)]
        request: std::path::PathBuf,
        #[arg(long)]
        store: std::path::PathBuf,
        #[arg(long)]
        session: String,
    },
    /// Release unretained artifacts through the preparation port.
    Cleanup {
        #[arg(long)]
        store: std::path::PathBuf,
        #[arg(long)]
        definition: String,
    },
    /// Print the stored lifecycle and its slots without changing anything.
    Inspect {
        #[arg(long)]
        store: std::path::PathBuf,
        #[arg(long)]
        definition: String,
    },
}

#[derive(clap::Args, Debug)]
pub struct SelectArgs {
    #[arg(long)]
    pub store: std::path::PathBuf,
    #[arg(long)]
    pub definition: String,
    #[arg(long)]
    pub revision: String,
    #[arg(long)]
    pub qualification_ref: String,
    #[arg(long)]
    pub task_family: String,
    /// `declared_support`, `contract_conformance`, `task_family_evaluation` or
    /// `deployment_admission`.
    #[arg(long)]
    pub required_class: String,
    #[arg(long)]
    pub expected_generation: u64,
    #[arg(long)]
    pub request_id: String,
}

#[derive(clap::Args, Debug)]
#[command(group(clap::ArgGroup::new("target").required(true).args(["revision", "qualification_ref", "session"])))]
pub struct RevokeArgs {
    #[arg(long)]
    pub store: std::path::PathBuf,
    #[arg(long)]
    pub definition: String,
    #[arg(long)]
    pub revision: Option<String>,
    #[arg(long)]
    pub qualification_ref: Option<String>,
    #[arg(long)]
    pub session: Option<String>,
}

/// Run with the standalone CLI's unavailable ports.
pub fn run_command(args: &CommandArgs) -> Result<(), String> {
    let ports = Ports {
        preparation: &mut UnavailablePreparation,
        qualification: &mut UnavailableQualification,
    };
    run_command_with_ports(args, ports)
}

/// Run with embedder-supplied ports and print one JSON document.
pub fn run_command_with_ports(args: &CommandArgs, ports: Ports<'_>) -> Result<(), String> {
    let document = commands::execute(&args.command, ports)?;
    println!("{document}");
    match document.get("refusal") {
        Some(refusal) => Err(format!("agent revision command refused: {refusal}")),
        None => Ok(()),
    }
}
