//! Safe CLI shell for durable projected role-protocol operations.
//!
//! r[impl lattice.role_protocol.diagnostics]
#![allow(
    unknown_lints,
    assertion_density,
    path_segment_repetition,
    reason = "the bounded CLI dispatcher and public command name remain explicit and are covered by direct negative tests"
)]

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionCreateInput {
    pub choreography: choregraph_choreography::RoleChoreographySource,
    pub placements: Vec<crate::engine::protocol::RolePlacement>,
    pub nonce: crate::engine::protocol::Identity,
}

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum CliCommand {
    /// Create a session from one typed Nickel choreography, placement set, and nonce
    Create { plan: std::path::PathBuf },
    /// Inspect one durable session without payload or credential content
    Inspect { session: String },
    /// Settle one attempt from its admitted receiver observations
    Recover { session: String, attempt: String },
    /// Abandon an unknown attempt and plan one fresh attempt
    Retry {
        session: String,
        attempt: String,
        #[arg(long)]
        acknowledge_duplicate_risk: bool,
    },
    /// Close one unknown attempt without inventing its outcome
    Abandon {
        session: String,
        attempt: String,
        #[arg(long)]
        acknowledge_unknown_outcome: bool,
    },
    /// Replay durable protocol records without effects
    Replay { session: String },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
}

pub fn session_create_input_to_nickel(input: &SessionCreateInput) -> Result<String, crate::engine::protocol::Issue> {
    let value = serde_json::to_value(input).map_err(|_| crate::engine::protocol::Issue::InvalidPlacement)?;
    crate::orchestration::format::render_nickel_value(&value, 0)
        .map_err(|_| crate::engine::protocol::Issue::InvalidPlacement)
}

pub fn run(store: &crate::store::Store, args: &CommandArgs) -> CommandResult {
    let command = match command(args) {
        Ok(command) => command,
        Err(issue) => return failure(issue),
    };
    let authority = UnavailableAuthority;
    let adapters = crate::engine::protocol::shell::Adapters::new(
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let shell = crate::engine::protocol::shell::Runtime::new(store, &authority, &adapters);
    match super::shell::operator::execute(&shell, command) {
        Ok(output) => match serde_json::to_string_pretty(&output) {
            Ok(rendered) => CommandResult {
                success: true,
                output: rendered,
            },
            Err(_) => failure(crate::engine::protocol::Issue::UnsafeDiagnostic),
        },
        Err(crate::engine::protocol::Error::Issue(issue)) => failure(issue),
        Err(_) => failure(crate::engine::protocol::Issue::IdentityMismatch),
    }
}

fn command(args: &CommandArgs) -> Result<super::shell::operator::Command, super::model::Issue> {
    Ok(match &args.command {
        CliCommand::Create { plan } => super::shell::operator::Command::Create {
            plan: Box::new(load_plan(plan)?),
        },
        CliCommand::Inspect { session } => super::shell::operator::Command::Inspect {
            session: identity(session)?,
        },
        CliCommand::Recover { session, attempt } => super::shell::operator::Command::Recover {
            session: identity(session)?,
            attempt: identity(attempt)?,
        },
        CliCommand::Retry {
            session,
            attempt,
            acknowledge_duplicate_risk,
        } => super::shell::operator::Command::Retry {
            session: identity(session)?,
            attempt: identity(attempt)?,
            acknowledge_duplicate_risk: *acknowledge_duplicate_risk,
        },
        CliCommand::Abandon {
            session,
            attempt,
            acknowledge_unknown_outcome,
        } => super::shell::operator::Command::Abandon {
            session: identity(session)?,
            attempt: identity(attempt)?,
            acknowledge_unknown: *acknowledge_unknown_outcome,
        },
        CliCommand::Replay { session } => super::shell::operator::Command::Replay {
            session: identity(session)?,
        },
    })
}

fn load_plan(path: &std::path::Path) -> Result<crate::engine::protocol::SessionPlan, crate::engine::protocol::Issue> {
    let source_envelope = std::fs::read(path).map_err(|_| crate::engine::protocol::Issue::IncompleteArtifacts)?;
    let value = crate::nickel::evaluate_file(path).map_err(|_| crate::engine::protocol::Issue::InvalidPlacement)?;
    let input: SessionCreateInput =
        serde_json::from_value(value).map_err(|_| crate::engine::protocol::Issue::InvalidPlacement)?;
    let evaluated =
        serde_json::to_vec(&input.choreography).map_err(|_| crate::engine::protocol::Issue::IncompleteArtifacts)?;
    let admitted = choregraph_choreography::admit_source(input.choreography)
        .map_err(|_| crate::engine::protocol::Issue::IncompleteArtifacts)?;
    let projected = choregraph_choreography::project_all(&admitted)
        .map_err(|_| crate::engine::protocol::Issue::IncompleteArtifacts)?;
    let materials = choregraph_choreography::ProjectionMaterials::reviewed(&source_envelope, &evaluated);
    let bundle = choregraph_choreography::build_projection_bundle(&admitted, &materials)
        .map_err(|_| crate::engine::protocol::Issue::IncompleteArtifacts)?;
    let cohort = crate::engine::protocol::admit_projection(crate::engine::protocol::FrozenProjection {
        projected: &projected,
        bundle: &bundle,
    })
    .map_err(|_| crate::engine::protocol::Issue::IncompleteArtifacts)?;
    crate::engine::protocol::plan_session(cohort, input.placements, input.nonce)
        .map_err(|_| crate::engine::protocol::Issue::InvalidPlacement)
}

fn identity(value: &str) -> Result<crate::engine::protocol::Identity, crate::engine::protocol::Issue> {
    crate::engine::protocol::Identity::new(value).map_err(|_| crate::engine::protocol::Issue::IdentityMismatch)
}

fn failure(issue: crate::engine::protocol::Issue) -> CommandResult {
    let output = serde_json::json!({
        "schema": crate::engine::protocol::EVIDENCE_SCHEMA,
        "status": "denied",
        "issue_code": issue.code(),
    });
    CommandResult {
        success: false,
        output: output.to_string(),
    }
}

struct UnavailableAuthority;

impl crate::engine::protocol::shell::ProtocolAuthority for UnavailableAuthority {
    fn authorize(
        &self,
        _session: &crate::engine::protocol::SessionRecord,
        _envelope: &crate::engine::protocol::MessageEnvelope,
    ) -> Result<crate::engine::protocol::Identity, crate::engine::protocol::Issue> {
        Err(crate::engine::protocol::Issue::AuthorityDenied)
    }
}
