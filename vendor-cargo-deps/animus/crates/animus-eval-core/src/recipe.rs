//! Pure built-in evaluation recipe planning.

use alloc::{format, string::String, vec, vec::Vec};

use crate::{EvaluationPolicy, hash_identity, validate_identity};

const TWO_SEAT_RUNS: u32 = 2;

/// Built-in recipe kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeKind {
    /// One evaluated agent run.
    SingleAgent,
    /// Bounded sibling runs with deterministic pass-at-N reduction.
    BestOfN,
    /// Separate untrainable simulated-user and evaluated-assistant seats.
    UserSimulation,
    /// Solver, bounded artifact handoff, and distinct verifier run.
    IsolatedAgenticJudge,
    /// Shared-runtime compatibility judge whose result is advisory.
    SharedAdvisoryJudge,
}

impl RecipeKind {
    /// Stable policy label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SingleAgent => "single_agent",
            Self::BestOfN => "best_of_n",
            Self::UserSimulation => "user_simulation",
            Self::IsolatedAgenticJudge => "isolated_agentic_judge",
            Self::SharedAdvisoryJudge => "shared_advisory_judge",
        }
    }
}

/// Semantic seat role in an evaluation plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationSeatRole {
    /// Evaluated solver or assistant.
    Solver,
    /// Simulated user that cannot become training output.
    SimulatedUser,
    /// Separate verifier or judge.
    Verifier,
}

/// One deterministic planned seat.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationSeat {
    /// Stable seat identity.
    pub seat_identity: String,
    /// Stable run identity.
    pub run_identity: String,
    /// Semantic role.
    pub role: EvaluationSeatRole,
    /// Stable zero-based order.
    pub order: u32,
    /// Whether this seat can contribute evaluated training output.
    pub trainable: bool,
    /// Whether the seat requires a distinct runtime and workspace.
    pub isolated: bool,
}

/// Ordered pure recipe action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeActionKind {
    /// Request one solver execution from a future adapter.
    ExecuteSolver,
    /// Request one untrainable simulated-user turn source.
    ExecuteSimulatedUser,
    /// Project an allowlisted artifact handoff request.
    HandoffArtifacts,
    /// Request a distinct verifier execution.
    ExecuteVerifier,
    /// Reduce sibling outcomes in stable plan order.
    ReduceCandidates,
}

/// One ordered action in a pure recipe plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeAction {
    /// Stable action identity.
    pub action_identity: String,
    /// Action class.
    pub kind: RecipeActionKind,
    /// Zero-based action order.
    pub order: u32,
}

/// Exact policy and implementation bindings for a recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    clippy::struct_field_names,
    reason = "the identity suffix makes each cross-domain binding explicit"
)]
pub struct RecipeBindings<'a> {
    /// Task identity.
    pub task_identity: &'a str,
    /// Evaluated role identity.
    pub role_identity: &'a str,
    /// Model identity.
    pub model_identity: &'a str,
    /// Harness implementation identity.
    pub harness_identity: &'a str,
    /// Tool policy identity.
    pub tool_identity: &'a str,
    /// Solver environment identity.
    pub environment_identity: &'a str,
    /// Verifier implementation identity.
    pub verifier_identity: &'a str,
    /// Verifier environment identity.
    pub verifier_environment_identity: &'a str,
    /// Rubric identity.
    pub rubric_identity: &'a str,
    /// Resource policy identity.
    pub resource_identity: &'a str,
    /// Cancellation identity.
    pub cancellation_identity: &'a str,
    /// Deadline identity.
    pub deadline_identity: &'a str,
    /// Redaction policy identity.
    pub redaction_identity: &'a str,
    /// Artifact manifest identity.
    pub artifact_manifest_identity: &'a str,
}

/// Pure environment-plan input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnvironmentPlanRequest<'a> {
    /// Solver environment identity.
    pub solver_environment_identity: &'a str,
    /// Optional separate verifier environment identity.
    pub verifier_environment_identity: Option<&'a str>,
    /// Resource policy identity.
    pub resource_identity: &'a str,
    /// Network policy identity.
    pub network_identity: &'a str,
    /// Whether verifier isolation is required.
    pub require_verifier_isolation: bool,
}

/// Deterministic environment plan without execution authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentPlan {
    /// Canonical plan identity.
    pub identity: String,
    /// Solver environment identity.
    pub solver_environment_identity: String,
    /// Optional verifier environment identity.
    pub verifier_environment_identity: Option<String>,
    /// Whether distinct environment identities satisfy the requested isolation fact.
    pub verifier_isolated: bool,
}

/// Recipe planning request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeRequest<'a> {
    /// Recipe kind.
    pub kind: RecipeKind,
    /// Exact bindings.
    pub bindings: RecipeBindings<'a>,
    /// Candidate count for bounded sibling recipes.
    pub candidate_count: u32,
    /// Artifact reference count for handoff.
    pub artifact_count: u32,
    /// Whether a simulated user is incorrectly requested as trainable.
    pub simulated_user_trainable: bool,
    /// Whether a simulated user requests assistant-only tools.
    pub simulated_user_assistant_tools: bool,
}

/// Deterministic recipe plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipePlan {
    /// Canonical plan identity.
    pub identity: String,
    /// Planned recipe kind.
    pub kind: RecipeKind,
    /// Ordered seats.
    pub seats: Vec<EvaluationSeat>,
    /// Ordered effect-free actions.
    pub actions: Vec<RecipeAction>,
    /// Whether all verifier results from this recipe remain advisory.
    pub advisory: bool,
    /// Explicit verifier-isolation status.
    pub verifier_isolated: bool,
}

/// One ordered pass-at-N candidate observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassAtNObservation<'a> {
    /// Exact run identity.
    pub run_identity: &'a str,
    /// Whether admitted evaluation evidence passed its bounded criterion.
    pub passed: bool,
}

/// Deterministic pass-at-N reduction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PassAtNReduction {
    /// Canonical reduction identity.
    pub identity: String,
    /// Required passing candidate count.
    pub required_passes: u32,
    /// Observed passing candidate count.
    pub passed_count: u32,
    /// Whether the bounded criterion was met.
    pub passed: bool,
}

/// Recipe planning failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeError {
    /// A required identity is malformed.
    Identity,
    /// Policy does not admit this recipe.
    Unsupported,
    /// A run or artifact bound is invalid.
    Bound,
    /// Solver and verifier isolation is missing.
    Isolation,
    /// Simulated-user capabilities are unsafe.
    UserAuthority,
    /// A candidate observation is missing, duplicated, or out of order.
    Observation,
}

/// Plan explicit environment identities without opening or discovering a runtime.
///
/// # Errors
/// Returns an identity or isolation error for malformed or shared required environments.
pub fn plan_environment(
    request: &EnvironmentPlanRequest<'_>,
) -> Result<EnvironmentPlan, RecipeError> {
    validate_identity(request.solver_environment_identity).map_err(|_| RecipeError::Identity)?;
    validate_identity(request.resource_identity).map_err(|_| RecipeError::Identity)?;
    validate_identity(request.network_identity).map_err(|_| RecipeError::Identity)?;
    if let Some(verifier) = request.verifier_environment_identity {
        validate_identity(verifier).map_err(|_| RecipeError::Identity)?;
    }
    let verifier_isolated = request
        .verifier_environment_identity
        .is_some_and(|verifier| verifier != request.solver_environment_identity);
    if request.require_verifier_isolation && !verifier_isolated {
        return Err(RecipeError::Isolation);
    }
    let verifier = request.verifier_environment_identity.unwrap_or("none");
    let identity = hash_identity(
        "animus.native-evaluation.environment-plan.v1",
        &[
            request.solver_environment_identity,
            verifier,
            request.resource_identity,
            request.network_identity,
        ],
    );
    Ok(EnvironmentPlan {
        identity,
        solver_environment_identity: request.solver_environment_identity.into(),
        verifier_environment_identity: request.verifier_environment_identity.map(String::from),
        verifier_isolated,
    })
}

/// Produce a deterministic built-in recipe plan without dispatching effects.
///
/// # Errors
/// Returns a fail-closed identity, support, bound, isolation, or authority class.
pub fn plan_recipe(
    request: &RecipeRequest<'_>,
    policy: &EvaluationPolicy,
) -> Result<RecipePlan, RecipeError> {
    validate_bindings(&request.bindings)?;
    if !policy.supported_recipes.contains(&request.kind) {
        return Err(RecipeError::Unsupported);
    }
    if request.candidate_count == 0 || request.candidate_count > policy.max_runs {
        return Err(RecipeError::Bound);
    }
    if request.artifact_count > policy.max_artifacts {
        return Err(RecipeError::Bound);
    }
    if request.kind == RecipeKind::UserSimulation
        && (request.simulated_user_trainable || request.simulated_user_assistant_tools)
    {
        return Err(RecipeError::UserAuthority);
    }
    let distinct_verifier = request.bindings.role_identity != request.bindings.verifier_identity
        && request.bindings.environment_identity != request.bindings.verifier_environment_identity;
    if request.kind == RecipeKind::IsolatedAgenticJudge && !distinct_verifier {
        return Err(RecipeError::Isolation);
    }

    let run_count = match request.kind {
        RecipeKind::SingleAgent => 1,
        RecipeKind::BestOfN => request.candidate_count,
        RecipeKind::UserSimulation
        | RecipeKind::IsolatedAgenticJudge
        | RecipeKind::SharedAdvisoryJudge => TWO_SEAT_RUNS,
    };
    if run_count > policy.max_runs {
        return Err(RecipeError::Bound);
    }

    let binding_identity = binding_identity(&request.bindings);
    let count_material = format!("{}:{}", request.candidate_count, request.artifact_count);
    let identity = hash_identity(
        "animus.native-evaluation.recipe-plan.v1",
        &[
            request.kind.label(),
            &binding_identity,
            &count_material,
            &policy.identity,
        ],
    );
    let seats = build_seats(request, &identity)?;
    let actions = build_actions(request, &identity)?;
    let shared = request.kind == RecipeKind::SharedAdvisoryJudge;
    Ok(RecipePlan {
        identity,
        kind: request.kind,
        seats,
        actions,
        advisory: shared,
        verifier_isolated: request.kind == RecipeKind::IsolatedAgenticJudge && distinct_verifier,
    })
}

/// Reduce exact best-of-N candidate observations in planned order.
///
/// # Errors
/// Returns a bound or observation error for missing, reordered, or mismatched candidates.
pub fn reduce_pass_at_n(
    plan: &RecipePlan,
    observations: &[PassAtNObservation<'_>],
    required_passes: u32,
) -> Result<PassAtNReduction, RecipeError> {
    if plan.kind != RecipeKind::BestOfN || observations.len() != plan.seats.len() {
        return Err(RecipeError::Observation);
    }
    let observation_count = u32::try_from(observations.len()).map_err(|_| RecipeError::Bound)?;
    if required_passes == 0 || required_passes > observation_count {
        return Err(RecipeError::Bound);
    }
    let mut passed_count = 0u32;
    for (seat, observation) in plan.seats.iter().zip(observations) {
        validate_identity(observation.run_identity).map_err(|_| RecipeError::Identity)?;
        if seat.run_identity != observation.run_identity {
            return Err(RecipeError::Observation);
        }
        if observation.passed {
            passed_count = passed_count.checked_add(1).ok_or(RecipeError::Bound)?;
        }
    }
    let required_material = format!("{required_passes}");
    let passed_material = format!("{passed_count}");
    let identity = hash_identity(
        "animus.native-evaluation.pass-at-n.v1",
        &[&plan.identity, &required_material, &passed_material],
    );
    Ok(PassAtNReduction {
        identity,
        required_passes,
        passed_count,
        passed: passed_count >= required_passes,
    })
}

fn validate_bindings(bindings: &RecipeBindings<'_>) -> Result<(), RecipeError> {
    for identity in [
        bindings.task_identity,
        bindings.role_identity,
        bindings.model_identity,
        bindings.harness_identity,
        bindings.tool_identity,
        bindings.environment_identity,
        bindings.verifier_identity,
        bindings.verifier_environment_identity,
        bindings.rubric_identity,
        bindings.resource_identity,
        bindings.cancellation_identity,
        bindings.deadline_identity,
        bindings.redaction_identity,
        bindings.artifact_manifest_identity,
    ] {
        validate_identity(identity).map_err(|_| RecipeError::Identity)?;
    }
    Ok(())
}

fn binding_identity(bindings: &RecipeBindings<'_>) -> String {
    hash_identity(
        "animus.native-evaluation.recipe-bindings.v1",
        &[
            bindings.task_identity,
            bindings.role_identity,
            bindings.model_identity,
            bindings.harness_identity,
            bindings.tool_identity,
            bindings.environment_identity,
            bindings.verifier_identity,
            bindings.verifier_environment_identity,
            bindings.rubric_identity,
            bindings.resource_identity,
            bindings.cancellation_identity,
            bindings.deadline_identity,
            bindings.redaction_identity,
            bindings.artifact_manifest_identity,
        ],
    )
}

fn build_seats(
    request: &RecipeRequest<'_>,
    plan_identity: &str,
) -> Result<Vec<EvaluationSeat>, RecipeError> {
    const SOLVER_LOCAL: SeatSpec = SeatSpec {
        role: EvaluationSeatRole::Solver,
        trainable: true,
        isolated: false,
    };
    const SOLVER_ISOLATED: SeatSpec = SeatSpec {
        role: EvaluationSeatRole::Solver,
        trainable: true,
        isolated: true,
    };
    const USER_ISOLATED: SeatSpec = SeatSpec {
        role: EvaluationSeatRole::SimulatedUser,
        trainable: false,
        isolated: true,
    };
    const VERIFIER_ISOLATED: SeatSpec = SeatSpec {
        role: EvaluationSeatRole::Verifier,
        trainable: false,
        isolated: true,
    };
    const VERIFIER_SHARED: SeatSpec = SeatSpec {
        role: EvaluationSeatRole::Verifier,
        trainable: false,
        isolated: false,
    };

    let mut seats = Vec::new();
    match request.kind {
        RecipeKind::SingleAgent => push_seat(&mut seats, plan_identity, SOLVER_LOCAL)?,
        RecipeKind::BestOfN => {
            for _ in 0..request.candidate_count {
                push_seat(&mut seats, plan_identity, SOLVER_ISOLATED)?;
            }
        }
        RecipeKind::UserSimulation => {
            push_seat(&mut seats, plan_identity, USER_ISOLATED)?;
            push_seat(&mut seats, plan_identity, SOLVER_ISOLATED)?;
        }
        RecipeKind::IsolatedAgenticJudge => {
            push_seat(&mut seats, plan_identity, SOLVER_ISOLATED)?;
            push_seat(&mut seats, plan_identity, VERIFIER_ISOLATED)?;
        }
        RecipeKind::SharedAdvisoryJudge => {
            push_seat(&mut seats, plan_identity, SOLVER_LOCAL)?;
            push_seat(&mut seats, plan_identity, VERIFIER_SHARED)?;
        }
    }
    Ok(seats)
}

#[derive(Clone, Copy)]
struct SeatSpec {
    role: EvaluationSeatRole,
    trainable: bool,
    isolated: bool,
}

fn push_seat(
    seats: &mut Vec<EvaluationSeat>,
    plan_identity: &str,
    spec: SeatSpec,
) -> Result<(), RecipeError> {
    let order = u32::try_from(seats.len()).map_err(|_| RecipeError::Bound)?;
    let order_material = format!("{order}");
    let role_label = seat_role_label(spec.role);
    let seat_identity = hash_identity(
        "animus.native-evaluation.seat.v1",
        &[plan_identity, role_label, &order_material],
    );
    let run_identity = hash_identity(
        "animus.native-evaluation.run.v1",
        &[plan_identity, &seat_identity, &order_material],
    );
    seats.push(EvaluationSeat {
        seat_identity,
        run_identity,
        role: spec.role,
        order,
        trainable: spec.trainable,
        isolated: spec.isolated,
    });
    Ok(())
}

fn build_actions(
    request: &RecipeRequest<'_>,
    plan_identity: &str,
) -> Result<Vec<RecipeAction>, RecipeError> {
    let kinds = match request.kind {
        RecipeKind::SingleAgent => vec![RecipeActionKind::ExecuteSolver],
        RecipeKind::BestOfN => {
            let candidate_count =
                usize::try_from(request.candidate_count).map_err(|_| RecipeError::Bound)?;
            let mut values = vec![RecipeActionKind::ExecuteSolver; candidate_count];
            values.push(RecipeActionKind::ReduceCandidates);
            values
        }
        RecipeKind::UserSimulation => vec![
            RecipeActionKind::ExecuteSimulatedUser,
            RecipeActionKind::ExecuteSolver,
        ],
        RecipeKind::IsolatedAgenticJudge => vec![
            RecipeActionKind::ExecuteSolver,
            RecipeActionKind::HandoffArtifacts,
            RecipeActionKind::ExecuteVerifier,
        ],
        RecipeKind::SharedAdvisoryJudge => vec![
            RecipeActionKind::ExecuteSolver,
            RecipeActionKind::ExecuteVerifier,
        ],
    };
    let mut actions = Vec::with_capacity(kinds.len());
    for kind in kinds {
        let order = u32::try_from(actions.len()).map_err(|_| RecipeError::Bound)?;
        let order_material = format!("{order}");
        let action_identity = hash_identity(
            "animus.native-evaluation.recipe-action.v1",
            &[plan_identity, action_label(kind), &order_material],
        );
        actions.push(RecipeAction {
            action_identity,
            kind,
            order,
        });
    }
    Ok(actions)
}

fn seat_role_label(role: EvaluationSeatRole) -> &'static str {
    match role {
        EvaluationSeatRole::Solver => "solver",
        EvaluationSeatRole::SimulatedUser => "simulated_user",
        EvaluationSeatRole::Verifier => "verifier",
    }
}

fn action_label(kind: RecipeActionKind) -> &'static str {
    match kind {
        RecipeActionKind::ExecuteSolver => "execute_solver",
        RecipeActionKind::ExecuteSimulatedUser => "execute_simulated_user",
        RecipeActionKind::HandoffArtifacts => "handoff_artifacts",
        RecipeActionKind::ExecuteVerifier => "execute_verifier",
        RecipeActionKind::ReduceCandidates => "reduce_candidates",
    }
}
