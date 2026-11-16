//! Nonblocking, run-scoped executor observations.
//!
//! A single admitted DAG run emits at most one Started and one Finished per
//! graph node. The frontier removes terminal nodes from readiness; cache and
//! preparation failures emit only Finished, and cancellation emits Finished
//! only for nodes without a result. The approval loop visits each topological
//! node once and skips checkpoint nodes. Adapter retries stay inside that one
//! dispatch. The repository child runner constructs a fresh executor without
//! the parent's progress port. Thus 2*N slots hold the whole run without polling.
//!
//! Reusing a sender across runs is not supported. Violations remain explicit
//! observation faults; they never block dispatch, retry, or cancel execution.

const OBSERVATIONS_PER_NODE: usize = 2;
const EMPTY_RUN_CAPACITY: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapacityExceeded;

impl std::fmt::Display for CapacityExceeded {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("workflow exceeds the bounded executor observation capacity")
    }
}

impl std::error::Error for CapacityExceeded {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fault {
    pub has_overflowed: bool,
    pub is_closed: bool,
}

#[derive(Debug, Default)]
struct FaultState {
    has_overflowed: std::sync::atomic::AtomicBool,
    is_closed: std::sync::atomic::AtomicBool,
}

/// Completeness observation which does not keep the progress channel open.
#[derive(Debug, Clone)]
pub struct Observer {
    state: std::sync::Arc<FaultState>,
}

impl Observer {
    pub fn fault(&self) -> Option<Fault> {
        let has_overflowed = self.state.has_overflowed.load(std::sync::atomic::Ordering::Acquire);
        let is_closed = self.state.is_closed.load(std::sync::atomic::Ordering::Acquire);
        (has_overflowed || is_closed).then_some(Fault {
            has_overflowed,
            is_closed,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Sender {
    sender: tokio::sync::mpsc::Sender<super::NodeProgress>,
    observer: Observer,
}

impl Sender {
    pub fn observer(&self) -> Observer {
        self.observer.clone()
    }

    /// Synchronous producers must never wait on the async UI consumer.
    pub fn send(&self, observation: super::NodeProgress) -> Result<(), Fault> {
        match self.sender.try_send(observation) {
            Ok(()) => Ok(()),
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                self.observer.state.has_overflowed.store(true, std::sync::atomic::Ordering::Release);
                Err(Fault {
                    has_overflowed: true,
                    is_closed: false,
                })
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                self.observer.state.is_closed.store(true, std::sync::atomic::Ordering::Release);
                Err(Fault {
                    has_overflowed: false,
                    is_closed: true,
                })
            }
        }
    }
}

/// Allocate a whole-run budget, not a queue sized by UI polling speed.
pub fn channel(
    workflow: &crate::engine::workflow::Workflow,
) -> Result<(Sender, tokio::sync::mpsc::Receiver<super::NodeProgress>), CapacityExceeded> {
    let observation_count = capacity_for_nodes(workflow.graph.nodes().count())?;
    let (sender, receiver) = tokio::sync::mpsc::channel(observation_count);
    let observer = Observer {
        state: std::sync::Arc::new(FaultState::default()),
    };
    Ok((Sender { sender, observer }, receiver))
}

fn capacity_for_nodes(node_count: usize) -> Result<usize, CapacityExceeded> {
    let observation_count =
        node_count.checked_mul(OBSERVATIONS_PER_NODE).ok_or(CapacityExceeded)?.max(EMPTY_RUN_CAPACITY);
    if observation_count > tokio::sync::Semaphore::MAX_PERMITS {
        return Err(CapacityExceeded);
    }
    Ok(observation_count)
}

#[cfg(test)]
mod tests {
    use super::super::NodeProgress;
    use super::*;
    use crate::engine::workflow::Workflow;
    use crate::engine::workflow::WorkflowTimestamp;

    #[test]
    fn capacity_checks_zero_integer_overflow_and_tokio_limit() {
        assert_eq!(capacity_for_nodes(0), Ok(EMPTY_RUN_CAPACITY));
        assert_eq!(capacity_for_nodes(3), Ok(6));
        assert_eq!(capacity_for_nodes(usize::MAX), Err(CapacityExceeded));
        assert_eq!(capacity_for_nodes(tokio::sync::Semaphore::MAX_PERMITS), Err(CapacityExceeded));
    }

    #[test]
    fn full_and_closed_channels_remain_explicit_incomplete_observations() {
        let mut workflow = Workflow::new("bounded", WorkflowTimestamp::epoch());
        let node_id = workflow.add_shell_node("node", "true");
        let (sender, receiver) = channel(&workflow).expect("bounded channel");
        let observer = sender.observer();
        for _ in 0..OBSERVATIONS_PER_NODE {
            sender.send(NodeProgress::Started { node_id }).expect("whole-run slot");
        }
        assert_eq!(
            sender.send(NodeProgress::Started { node_id }),
            Err(Fault {
                has_overflowed: true,
                is_closed: false
            })
        );
        assert_eq!(
            observer.fault(),
            Some(Fault {
                has_overflowed: true,
                is_closed: false
            })
        );
        drop(receiver);
        assert_eq!(
            sender.send(NodeProgress::Started { node_id }),
            Err(Fault {
                has_overflowed: false,
                is_closed: true
            })
        );
        assert_eq!(
            observer.fault(),
            Some(Fault {
                has_overflowed: true,
                is_closed: true
            })
        );
    }

    #[tokio::test]
    async fn whole_run_finishes_without_consumer_polling_or_observer_keeping_channel_open() {
        let mut workflow = Workflow::new("lagging observer", WorkflowTimestamp::epoch());
        for _ in 0..64 {
            workflow.add_condition_node("pure", crate::control_flow::ConditionNodeConfig {
                branches: vec![crate::control_flow::ConditionBranch {
                    label: "match".to_string(),
                    predicate: crate::control_flow::ConditionPredicate::StringEquals("ready".to_string()),
                }],
            });
        }
        let (sender, mut receiver) = channel(&workflow).expect("bounded channel");
        let observer = sender.observer();
        let mut executor = super::super::Executor::new().with_progress_sender(sender);
        let (_, results) = executor.run(&workflow, &std::collections::HashMap::new()).await;
        assert_eq!(results.len(), 64);
        assert_eq!(observer.fault(), None);
        drop(executor);
        let mut observation_count = 0_usize;
        while receiver.recv().await.is_some() {
            observation_count = observation_count.saturating_add(1);
        }
        assert_eq!(observation_count, 128);
        assert_eq!(observer.fault(), None);
    }

    #[tokio::test]
    async fn empty_workflow_has_a_valid_channel_and_no_observations() {
        let workflow = Workflow::new("empty", WorkflowTimestamp::epoch());
        let (sender, mut receiver) = channel(&workflow).expect("empty channel");
        let observer = sender.observer();
        let mut executor = super::super::Executor::new().with_progress_sender(sender);
        let (_, results) = executor.run(&workflow, &std::collections::HashMap::new()).await;
        assert!(results.is_empty());
        drop(executor);
        assert!(receiver.recv().await.is_none());
        assert_eq!(observer.fault(), None);
    }

    #[tokio::test]
    async fn cached_node_emits_only_finished_with_no_consumer_polling() {
        let mut workflow = Workflow::new("cached", WorkflowTimestamp::epoch());
        let node_id = workflow.add_condition_node("pure", crate::control_flow::ConditionNodeConfig {
            branches: vec![crate::control_flow::ConditionBranch {
                label: "match".to_string(),
                predicate: crate::control_flow::ConditionPredicate::StringEquals("ready".to_string()),
            }],
        });
        let (_, first_results) = super::super::Executor::new().run(&workflow, &std::collections::HashMap::new()).await;
        let cache = first_results
            .into_iter()
            .filter_map(|step| match step.status {
                super::super::NodeStatus::Completed { outputs, .. } => Some((step.input_hash, outputs)),
                super::super::NodeStatus::Pending
                | super::super::NodeStatus::Running
                | super::super::NodeStatus::Failed { .. }
                | super::super::NodeStatus::Skipped => None,
            })
            .collect();
        let (sender, mut receiver) = channel(&workflow).expect("cached channel");
        let observer = sender.observer();
        let mut executor = super::super::Executor::new().with_progress_sender(sender);
        let (_, results) = executor.run(&workflow, &cache).await;
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, super::super::NodeStatus::Completed { cached: true, .. }));
        drop(executor);
        assert!(
            matches!(receiver.recv().await, Some(NodeProgress::Finished { node_id: observed, .. }) if observed == node_id)
        );
        assert!(receiver.recv().await.is_none());
        assert_eq!(observer.fault(), None);
    }
}
