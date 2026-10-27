//! Immutable task and deterministic taskset contracts.

use alloc::{format, string::String, vec::Vec};

use crate::{EvaluationPolicy, hash_identity, validate_identity};

/// Taskset ordering contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskOrdering {
    /// Supplied order is stable and identity-bearing.
    Stable,
    /// Global shuffle is requested. It is not admitted by this initial core.
    GlobalShuffle,
}

/// Declared taskset finiteness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TasksetFiniteness {
    /// Exact finite membership.
    Finite,
    /// A finite lazy generator.
    LazyFinite,
    /// An unbounded lazy generator that requires a selection bound.
    Infinite,
}

/// Immutable task data and behavior identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskDescriptor<'a> {
    /// Public task identity.
    pub task_identity: &'a str,
    /// Canonical bounded task-data identity.
    pub data_identity: &'a str,
    /// Behavior implementation identity.
    pub behavior_identity: &'a str,
    /// Scorer implementation identity.
    pub scorer_identity: &'a str,
    /// Task policy identity.
    pub policy_identity: &'a str,
    /// Task schema version.
    pub version: &'a str,
}

/// Exact or lazy taskset source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TasksetSource<'a> {
    /// Exact finite members in stable order.
    Exact {
        /// Expected task identities.
        members: &'a [&'a str],
    },
    /// Caller-supplied generated prefix under an explicit generator contract.
    Lazy {
        /// Generator implementation identity.
        generator_identity: &'a str,
        /// Generator semantic version identity.
        generator_version: &'a str,
        /// Explicit deterministic seed.
        seed: u64,
        /// Declared finiteness.
        finiteness: TasksetFiniteness,
        /// Positive selected prefix bound.
        selection_bound: u32,
    },
}

/// Borrowed taskset descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TasksetDescriptor<'a> {
    /// Public taskset identity.
    pub taskset_identity: &'a str,
    /// Taskset schema version.
    pub version: &'a str,
    /// Ordering contract.
    pub ordering: TaskOrdering,
    /// Exact or lazy source.
    pub source: TasksetSource<'a>,
}

/// One validated task in taskset order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedTask {
    /// Task identity.
    pub task_identity: String,
    /// Canonical task descriptor identity.
    pub descriptor_identity: String,
    /// Zero-based stable order.
    pub order: u32,
}

/// Deterministic bounded taskset plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TasksetPlan {
    /// Source taskset identity.
    pub taskset_identity: String,
    /// Canonical taskset plan identity.
    pub identity: String,
    /// Ordered selected tasks.
    pub tasks: Vec<PlannedTask>,
    /// Whether generation was lazy.
    pub lazy: bool,
}

/// Task or taskset validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TasksetError {
    /// One identity or version is malformed.
    Identity,
    /// Membership is empty, duplicated, or does not match the declaration.
    Membership,
    /// The ordering mode is unsupported.
    Ordering,
    /// A lazy generator is missing required deterministic metadata.
    Generator,
    /// Selection is zero, over policy, or exceeds supplied generated facts.
    Bound,
}

/// Validate a task and return its canonical descriptor identity.
///
/// # Errors
/// Returns [`TasksetError::Identity`] for malformed task metadata.
pub fn task_descriptor_identity(task: &TaskDescriptor<'_>) -> Result<String, TasksetError> {
    for value in [
        task.task_identity,
        task.data_identity,
        task.behavior_identity,
        task.scorer_identity,
        task.policy_identity,
        task.version,
    ] {
        validate_identity(value).map_err(|_| TasksetError::Identity)?;
    }
    Ok(hash_identity(
        "animus.native-evaluation.task.v1",
        &[
            task.task_identity,
            task.data_identity,
            task.behavior_identity,
            task.scorer_identity,
            task.policy_identity,
            task.version,
        ],
    ))
}

/// Validate an exact or lazy taskset and produce an ordered bounded plan.
///
/// For lazy sources, `tasks` is a caller-supplied generated prefix. This pure
/// function never invokes or enumerates a generator.
///
/// # Errors
/// Returns a deterministic identity, membership, ordering, generator, or bound class.
pub fn plan_taskset(
    descriptor: &TasksetDescriptor<'_>,
    tasks: &[TaskDescriptor<'_>],
    policy: &EvaluationPolicy,
) -> Result<TasksetPlan, TasksetError> {
    validate_identity(descriptor.taskset_identity).map_err(|_| TasksetError::Identity)?;
    validate_identity(descriptor.version).map_err(|_| TasksetError::Identity)?;
    if descriptor.ordering != TaskOrdering::Stable {
        return Err(TasksetError::Ordering);
    }
    if tasks.is_empty() {
        return Err(TasksetError::Membership);
    }

    let selected_count = match descriptor.source {
        TasksetSource::Exact { members } => {
            if members.is_empty() || members.len() != tasks.len() {
                return Err(TasksetError::Membership);
            }
            for (member, task) in members.iter().zip(tasks) {
                if *member != task.task_identity {
                    return Err(TasksetError::Membership);
                }
            }
            tasks.len()
        }
        TasksetSource::Lazy {
            generator_identity,
            generator_version,
            seed: _,
            finiteness: _,
            selection_bound,
        } => {
            validate_identity(generator_identity).map_err(|_| TasksetError::Generator)?;
            validate_identity(generator_version).map_err(|_| TasksetError::Generator)?;
            if selection_bound == 0 || selection_bound > policy.max_tasks {
                return Err(TasksetError::Bound);
            }
            let selected = usize::try_from(selection_bound).map_err(|_| TasksetError::Bound)?;
            if tasks.len() < selected {
                return Err(TasksetError::Bound);
            }
            selected
        }
    };
    if selected_count > usize::try_from(policy.max_tasks).map_err(|_| TasksetError::Bound)? {
        return Err(TasksetError::Bound);
    }

    let mut planned = Vec::with_capacity(selected_count);
    for task in tasks.iter().take(selected_count) {
        if planned
            .iter()
            .any(|prior: &PlannedTask| prior.task_identity == task.task_identity)
        {
            return Err(TasksetError::Membership);
        }
        let descriptor_identity = task_descriptor_identity(task)?;
        let order = u32::try_from(planned.len()).map_err(|_| TasksetError::Bound)?;
        planned.push(PlannedTask {
            task_identity: task.task_identity.into(),
            descriptor_identity,
            order,
        });
    }

    let ordered_material = planned
        .iter()
        .map(|task| task.descriptor_identity.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let (lazy, generator_material) = match descriptor.source {
        TasksetSource::Exact { .. } => (false, String::from("exact")),
        TasksetSource::Lazy {
            generator_identity,
            generator_version,
            seed,
            finiteness,
            selection_bound,
        } => (
            true,
            format!(
                "{generator_identity}:{generator_version}:{seed}:{}:{selection_bound}",
                finiteness_label(finiteness)
            ),
        ),
    };
    let identity = hash_identity(
        "animus.native-evaluation.taskset-plan.v1",
        &[
            descriptor.taskset_identity,
            descriptor.version,
            "stable",
            &generator_material,
            &ordered_material,
            &policy.identity,
        ],
    );
    Ok(TasksetPlan {
        taskset_identity: descriptor.taskset_identity.into(),
        identity,
        tasks: planned,
        lazy,
    })
}

fn finiteness_label(value: TasksetFiniteness) -> &'static str {
    match value {
        TasksetFiniteness::Finite => "finite",
        TasksetFiniteness::LazyFinite => "lazy_finite",
        TasksetFiniteness::Infinite => "infinite",
    }
}
