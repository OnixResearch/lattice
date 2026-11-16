//! Product-fact and real live-read conflict regressions for issue #19.

use std::collections::BTreeMap;

use lattice::control_flow::ConditionBranch;
use lattice::control_flow::ConditionNodeConfig;
use lattice::control_flow::ConditionPredicate;
use lattice::control_flow::MergeNodeConfig;
use lattice::control_flow::MergePolicy;
use lattice::engine::frontier;
use lattice::engine::workflow::NodeConfig;
use lattice::file_node::FileOperationKind;
use lattice_core::scheduling::ActiveReservations;
use lattice_core::scheduling::Blocker;
use lattice_core::scheduling::ConflictState;

const FACTS: frontier::ProductFacts = frontier::ProductFacts {
    is_suspended: false,
    is_cancelled: false,
};

fn live_reads() -> [NodeConfig; 2] {
    [
        NodeConfig::file(FileOperationKind::Read, "/root/live.txt"),
        NodeConfig::image_input("/root/live.png"),
    ]
}

fn two_slot_policy() -> lattice_core::scheduling::CapacityPolicy {
    frontier::capacity_policy(2, &frontier::uniform_lane_limits(2)).expect("all-lane two-slot policy")
}

// r[verify lattice.workflow_execution.ready_frontier.live_reads]
// r[verify lattice.workflow_execution.ready_frontier.live_reads.scenario.candidates]
#[test]
fn product_live_reads_and_mutations_cannot_share_two_slots_in_either_order() {
    let shell = NodeConfig::shell("printf mutation");
    for read in live_reads() {
        for (first, second) in [(&shell, &read), (&read, &shell)] {
            let candidates = [
                frontier::candidate(1, Some(first), FACTS),
                frontier::candidate(2, Some(second), FACTS),
            ];
            assert_eq!(candidates[0].conflict, ConflictState::Unresolved);
            assert_eq!(candidates[1].conflict, ConflictState::Unresolved);
            let planned =
                lattice_core::scheduling::plan(&two_slot_policy(), &ActiveReservations::default(), &candidates)
                    .expect("product candidates plan");
            assert_eq!(planned.selected_nodes, vec![1]);
            assert_eq!(planned.blocked_nodes[0].node_id, 2);
            assert_eq!(planned.blocked_nodes[0].reason, Blocker::UnresolvedConflict);
        }
    }
}

// r[verify lattice.workflow_execution.ready_frontier.live_reads.scenario.reservations]
#[test]
fn active_product_reservations_exclude_live_reads_and_mutations_in_either_order() {
    let shell = NodeConfig::shell("printf mutation");
    for read in live_reads() {
        for (reserved, pending) in [(&shell, &read), (&read, &shell)] {
            let active = frontier::active_reservations(&BTreeMap::from([(1, frontier::reservation(Some(reserved)))]));
            assert!(active.unresolved_conflict);
            let candidate = frontier::candidate(2, Some(pending), FACTS);
            let planned = lattice_core::scheduling::plan(&two_slot_policy(), &active, &[candidate])
                .expect("active product reservation plans");
            assert!(planned.selected_nodes.is_empty());
            assert_eq!(planned.blocked_nodes[0].reason, Blocker::UnresolvedConflict);
        }
    }
}

// r[verify lattice.workflow_execution.ready_frontier.live_reads.scenario.pure]
#[test]
fn condition_and_merge_remain_pure_in_both_candidate_and_reservation_orders() {
    let shell = NodeConfig::shell("printf mutation");
    let condition = NodeConfig::condition(ConditionNodeConfig {
        branches: vec![ConditionBranch {
            label: "always".to_string(),
            predicate: ConditionPredicate::Always,
        }],
    });
    let merge = NodeConfig::merge(MergeNodeConfig {
        policy: MergePolicy::All,
        required_inputs: vec!["input".to_string()],
    });
    for pure in [&condition, &merge] {
        let pure_candidate = frontier::candidate(2, Some(pure), FACTS);
        assert_eq!(pure_candidate.conflict, ConflictState::Pure);
        assert_eq!(pure_candidate.lane, frontier::PURE_LANE);
        for (first, second) in [(&shell, pure), (pure, &shell)] {
            let candidates = [
                frontier::candidate(1, Some(first), FACTS),
                frontier::candidate(2, Some(second), FACTS),
            ];
            let planned =
                lattice_core::scheduling::plan(&two_slot_policy(), &ActiveReservations::default(), &candidates)
                    .expect("pure overlap plans");
            assert_eq!(planned.selected_nodes, vec![1, 2]);
            let active = frontier::active_reservations(&BTreeMap::from([(1, frontier::reservation(Some(first)))]));
            let planned = lattice_core::scheduling::plan(&two_slot_policy(), &active, &candidates[1..])
                .expect("pure active overlap plans");
            assert_eq!(planned.selected_nodes, vec![2]);
        }
    }
}

// r[verify lattice.workflow_execution.ready_frontier.live_reads.scenario.bounds]
#[test]
fn live_read_lanes_capacity_and_cancellation_remain_independent_blockers() {
    for (read, expected_lane) in live_reads().into_iter().zip([frontier::FILESYSTEM_LANE, frontier::IMAGE_LANE]) {
        let candidate = frontier::candidate(2, Some(&read), FACTS);
        assert_eq!(candidate.lane, expected_lane);
        let mut pure_reservation = frontier::reservation(Some(&read));
        // Isolate lane/total accounting from the separately tested conflict fact.
        pure_reservation.conflict = ConflictState::Pure;
        let active = frontier::active_reservations(&BTreeMap::from([(1, pure_reservation)]));
        let serial = lattice_core::scheduling::plan(
            &frontier::serial_capacity_policy(),
            &active,
            std::slice::from_ref(&candidate),
        )
        .expect("capacity-one product read plans");
        assert_eq!(serial.blocked_nodes[0].reason, Blocker::TotalCapacity);
        let mut policy = two_slot_policy();
        policy.lane_slots.insert(expected_lane.to_string(), 1);
        let lane_limited = lattice_core::scheduling::plan(&policy, &active, &[candidate]).expect("lane bound plans");
        assert_eq!(lane_limited.blocked_nodes[0].reason, Blocker::ResourceCapacity);
        let cancelled = frontier::candidate(2, Some(&read), frontier::ProductFacts {
            is_cancelled: true,
            ..FACTS
        });
        let planned = lattice_core::scheduling::plan(&two_slot_policy(), &ActiveReservations::default(), &[cancelled])
            .expect("cancelled product read plans");
        assert!(planned.selected_nodes.is_empty());
        assert_eq!(planned.blocked_nodes[0].reason, Blocker::Cancellation);
    }
}

#[cfg(unix)]
mod executor {
    use std::collections::HashMap;
    use std::io::Read;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::task::Poll;
    use std::time::Duration;

    use lattice::engine::Executor;
    use lattice::engine::Workflow;
    use lattice::engine::executor::NodeStatus;
    use lattice::engine::value::ImageValue;
    use lattice::engine::workflow::NodeConfig;
    use lattice::file_node::FileError;
    use lattice::file_node::FileOperationKind;
    use lattice::file_node::FileOperationPlan;
    use lattice::file_node::FileOperationResult;
    use lattice::file_node::FileSystem;
    use lattice::file_node::LocalFs;
    use lattice::image_node::ImageError;
    use lattice::image_node::ImageInputPlan;
    use lattice::image_node::ImageIo;
    use lattice::image_node::LocalImageIo;
    use tokio::io::unix::AsyncFd;

    use super::two_slot_policy;

    const DEADLINE: Duration = Duration::from_secs(10);
    const FILE_FINAL: &[u8] = b"complete mutation";
    const IMAGE_FINAL: &[u8] = b"\x89PNG\r\n\x1a\ncomplete mutation";

    #[derive(Clone, Default)]
    struct Observation {
        started: Arc<AtomicBool>,
        bytes: Arc<Mutex<Option<Vec<u8>>>>,
    }

    struct ObservedFile(Observation);

    impl FileSystem for ObservedFile {
        fn execute(&self, plan: &FileOperationPlan) -> Result<FileOperationResult, FileError> {
            self.0.started.store(true, Ordering::SeqCst);
            let result = LocalFs.execute(plan)?;
            if let FileOperationResult::Read { content, .. } = &result {
                *self.0.bytes.lock().expect("file observation") = Some(content.as_bytes().to_vec());
            }
            Ok(result)
        }
    }

    struct ObservedImage(Observation);

    impl ImageIo for ObservedImage {
        fn load(&self, plan: &ImageInputPlan) -> Result<ImageValue, ImageError> {
            self.0.started.store(true, Ordering::SeqCst);
            let image = LocalImageIo.load(plan)?;
            *self.0.bytes.lock().expect("image observation") = Some(image.bytes.clone());
            Ok(image)
        }
    }

    // Read/write nonblocking FIFO handles keep setup and teardown nonblocking,
    // even if a subprocess fails. No sleeps or elapsed-time ordering assertions.
    struct MutationBarrier {
        ready: AsyncFd<std::fs::File>,
        release: std::fs::File,
    }

    impl MutationBarrier {
        fn new(root: &Path) -> Self {
            let ready_path = root.join("ready.fifo");
            let release_path = root.join("release.fifo");
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(&ready_path)
                    .arg(&release_path)
                    .status()
                    .expect("mkfifo available")
                    .success()
            );
            let open = |path| {
                std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open(path)
                    .expect("open barrier FIFO")
            };
            Self {
                ready: AsyncFd::new(open(&ready_path)).expect("register ready FIFO"),
                release: open(&release_path),
            }
        }

        async fn entered(&self) {
            loop {
                let mut ready = self.ready.readable().await.expect("ready FIFO readable");
                match ready.try_io(|fd| {
                    let mut byte = [0];
                    let mut file = fd.get_ref();
                    file.read_exact(&mut byte)
                }) {
                    Ok(result) => {
                        result.expect("mutation entered barrier");
                        return;
                    }
                    Err(_would_block) => {}
                }
            }
        }

        fn release(&mut self) {
            self.release.write_all(b"release\n").expect("release mutation barrier");
        }
    }

    impl Drop for MutationBarrier {
        fn drop(&mut self) {
            let _ = self.release.write_all(b"release\n");
        }
    }

    fn quote(path: &Path) -> String {
        format!("'{}'", path.to_str().expect("UTF-8 fixture path").replace('\'', "'\\''"))
    }

    #[derive(Clone, Copy, Debug)]
    enum ConflictFact {
        Missing,
        SameAlias,
        Dependency,
    }

    async fn mutation_excludes_live_read(image: bool, fact: ConflictFact, cancel_pending: bool) {
        let directory = tempfile::tempdir().expect("capability root");
        let root = directory.path();
        let live = root.join(if image { "live.png" } else { "live.txt" });
        let final_path = root.join("final-bytes");
        let expected = if image { IMAGE_FINAL } else { FILE_FINAL };
        std::fs::write(&live, expected).expect("initial live file");
        std::fs::write(&final_path, expected).expect("final mutation bytes");
        let mut barrier = MutationBarrier::new(root);
        let command = format!(
            "set -eu; printf intermediate > {live}; printf R > {ready}; IFS= read -r release < {release}; cp {final_path} {live}; printf '%s' {live}",
            live = quote(&live),
            ready = quote(&root.join("ready.fifo")),
            release = quote(&root.join("release.fifo")),
            final_path = quote(&final_path),
        );
        let mut workflow = Workflow::new("live-read-barrier", lattice::engine::workflow::WorkflowTimestamp::epoch());
        let mutation = workflow.add_shell_node("mutation", command);
        let read_path = if matches!(fact, ConflictFact::Dependency) {
            let output = workflow.graph.node(mutation).expect("mutation node").output_ports[0].id;
            format!("{{{{node:{}:port:{}}}}}", mutation.0, output.0)
        } else {
            live.to_str().expect("live path").to_string()
        };
        let roots = vec![root.to_str().expect("root path").to_string()];
        let read = if image {
            let node = workflow.add_image_input_node("live image", &read_path);
            workflow.set_node_config(node, NodeConfig::ImageInput {
                path: read_path,
                allowed_roots: roots,
                max_bytes: lattice::image_node::DEFAULT_MAX_IMAGE_BYTES,
            });
            node
        } else {
            let node = workflow.add_file_node("live file", FileOperationKind::Read, &read_path);
            workflow.set_node_config(node, NodeConfig::File {
                operation: FileOperationKind::Read,
                path: read_path,
                content: None,
                overwrite: false,
                allowed_roots: roots,
            });
            node
        };
        let observation = Observation::default();
        let executor = if image {
            Executor::new_with_image_io(Box::new(ObservedImage(observation.clone())))
        } else {
            Executor::new_with_file_system(Box::new(ObservedFile(observation.clone())))
        };
        let cancellation = Arc::new(AtomicBool::new(false));
        let mut executor = executor
            .with_frontier_policy(two_slot_policy())
            .expect("two slots")
            .with_cancellation(cancellation.clone());
        if matches!(fact, ConflictFact::SameAlias) {
            executor = executor
                .with_resolved_resource_alias(mutation, "live-resource")
                .expect("mutation alias")
                .with_resolved_resource_alias(read, "live-resource")
                .expect("read alias");
        }
        let cache = HashMap::new();
        let started_while_mutating;
        let steps;
        {
            let mut run = Box::pin(executor.run(&workflow, &cache));
            tokio::time::timeout(DEADLINE, async {
                tokio::select! {
                    _result = &mut run => panic!("workflow completed before mutation barrier"),
                    () = barrier.entered() => {}
                }
            })
            .await
            .expect("subprocess reaches explicit barrier");
            // Drive every currently runnable selected future to Pending while the
            // mutation is definitely held. This catches even a read dispatched
            // before the child reports entry, without relying on task timing.
            std::future::poll_fn(|cx| {
                assert!(run.as_mut().poll(cx).is_pending(), "held mutation cannot complete");
                Poll::Ready(())
            })
            .await;
            started_while_mutating = observation.started.load(Ordering::SeqCst);
            if cancel_pending {
                cancellation.store(true, Ordering::SeqCst);
            }
            barrier.release();
            (_, steps) = tokio::time::timeout(DEADLINE, run).await.expect("released workflow completes");
        }
        assert!(!started_while_mutating, "live read started during mutation: image={image}, fact={fact:?}");
        if cancel_pending {
            assert!(!observation.started.load(Ordering::SeqCst), "cancelled pending read must not start");
            assert!(observation.bytes.lock().expect("cancelled read observation").is_none());
            let read_step = steps.iter().find(|step| step.node_id == read).expect("cancelled read result");
            assert!(!matches!(read_step.status, NodeStatus::Completed { .. }));
            return;
        }
        crate::assert_completed(&steps);
        assert!(observation.started.load(Ordering::SeqCst));
        assert_eq!(observation.bytes.lock().expect("observed live bytes").as_deref(), Some(expected));
        assert_eq!(executor.observed_completion_order(), &[mutation, read]);
        assert_eq!(steps.iter().map(|step| step.node_id).collect::<Vec<_>>(), vec![mutation, read]);
    }

    // r[verify lattice.workflow_execution.ready_frontier.live_reads.scenario.executor]
    #[tokio::test]
    async fn barrier_controlled_file_read_waits_for_unresolved_mutation() {
        mutation_excludes_live_read(false, ConflictFact::Missing, false).await;
    }

    // r[verify lattice.workflow_execution.ready_frontier.live_reads.scenario.executor]
    #[tokio::test]
    async fn barrier_controlled_image_read_waits_for_unresolved_mutation() {
        mutation_excludes_live_read(true, ConflictFact::Missing, false).await;
    }

    // r[verify lattice.workflow_execution.ready_frontier.live_reads.scenario.explicit_order]
    #[tokio::test]
    async fn admitted_aliases_and_dependencies_still_order_real_live_reads() {
        for image in [false, true] {
            for fact in [ConflictFact::SameAlias, ConflictFact::Dependency] {
                mutation_excludes_live_read(image, fact, false).await;
            }
        }
    }

    // r[verify lattice.workflow_execution.ready_frontier.live_reads.scenario.cancellation]
    #[tokio::test]
    async fn cancellation_while_mutation_is_held_prevents_pending_live_reads() {
        for image in [false, true] {
            mutation_excludes_live_read(image, ConflictFact::Missing, true).await;
        }
    }
}
