use super::*;

const TASK_ID: &str = "task-42";
const GROUP_CPU: &str = "cpu-heavy";
const GROUP_IO: &str = "io-heavy";
const COMMAND_LABEL: &str = "cargo-test-workspace";
const ENV_POLICY: &str = "env-allowlist-basic";
const TIMEOUT_POLICY: &str = "bounded-quick-wait";
const BACKEND_RUNNING: &str = "pueue-running";
const SAFE_REFERENCE: &str = "queue-log-ref:task-42";
const VISIBLE_LOG: &str = "running tests\nall good\n";
const SAFE_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const ZERO_COUNT: u32 = 0;
const MAX_LINES: u32 = 5;
const MAX_BYTES: u32 = 128;
const TOTAL_LINES: u32 = 30;
const TOTAL_BYTES: u32 = 900;
const VISIBLE_LINES: u32 = 2;
const VISIBLE_BYTES: u32 = 24;
const CPU_PARALLELISM: u32 = 2;
const IO_PARALLELISM: u32 = 4;
const CPU_PENDING: u32 = 3;
const CPU_RUNNING: u32 = 1;
const IO_RUNNING: u32 = 2;
const IO_COMPLETED: u32 = 8;

#[test]
fn enqueue_request_carries_safe_identity_without_shell_text() {
    let request = QueueEnqueueRequest::new(QueueEnqueueDraft {
        command_label: COMMAND_LABEL,
        group: Some(GROUP_CPU),
        environment_policy: ENV_POLICY,
        timeout_policy: TIMEOUT_POLICY,
        output_bounds: bounds(),
    })
    .expect("safe enqueue request should validate");

    assert_eq!(request.command_label(), COMMAND_LABEL);
    assert_eq!(request.group(), Some(GROUP_CPU));
    assert_eq!(request.environment_policy(), ENV_POLICY);
    assert_eq!(request.timeout_policy(), TIMEOUT_POLICY);
    assert_eq!(request.output_bounds().max_visible_lines(), MAX_LINES);
}

#[test]
fn injected_queue_service_can_return_task_observation() {
    let task = QueueTaskObservation::new(QueueTaskObservationDraft {
        task_id: TASK_ID,
        group: GROUP_CPU,
        status: QueueTaskStatus::Running,
        backend_status_label: Some(BACKEND_RUNNING),
    })
    .expect("task observation should validate");
    let mut adapter = FixtureQueueService {
        response: QueueServiceResponse::Task(task),
    };
    let request = QueueServiceRequest::Enqueue(
        QueueEnqueueRequest::new(QueueEnqueueDraft {
            command_label: COMMAND_LABEL,
            group: Some(GROUP_CPU),
            environment_policy: ENV_POLICY,
            timeout_policy: TIMEOUT_POLICY,
            output_bounds: bounds(),
        })
        .expect("enqueue request should validate"),
    );

    let outcome = crate::ComponentAdapter::call(&mut adapter, &request)
        .expect("fixture service should not fail");

    assert!(outcome.is_completed());
    assert_eq!(outcome.receipt().outcome(), crate::OutcomeKind::Completed);
}

#[test]
fn destructive_control_accepts_explicit_all_groups_selector() {
    let request = QueueControlRequest::new(QueueControlDraft {
        operation: QueueOperationKind::Clean,
        target: Some(QueueTargetDraft {
            kind: QueueTargetKind::AllGroups,
            label: None,
        }),
        parallelism: None,
    })
    .expect("explicit all-groups clean should validate");

    let target = request.target().expect("target should be retained");
    assert_eq!(request.operation(), QueueOperationKind::Clean);
    assert_eq!(target.kind(), QueueTargetKind::AllGroups);
}

#[test]
fn ambiguous_destructive_control_is_rejected() {
    let error = QueueControlRequest::new(QueueControlDraft {
        operation: QueueOperationKind::Cancel,
        target: None,
        parallelism: None,
    })
    .expect_err("destructive controls need explicit targets");

    assert_eq!(
        error,
        QueueContractError::AmbiguousTarget {
            operation: QueueOperationKind::Cancel
        }
    );
}

#[test]
fn group_parallelism_requires_exact_operation_target_and_positive_value() {
    let request = QueueControlRequest::new(QueueControlDraft {
        operation: QueueOperationKind::SetGroupParallelism,
        target: Some(QueueTargetDraft {
            kind: QueueTargetKind::Group,
            label: Some(GROUP_CPU),
        }),
        parallelism: Some(CPU_PARALLELISM),
    })
    .expect("group parallelism should validate");
    assert_eq!(request.parallelism(), Some(CPU_PARALLELISM));

    let missing = QueueControlRequest::new(QueueControlDraft {
        operation: QueueOperationKind::SetGroupParallelism,
        target: Some(QueueTargetDraft {
            kind: QueueTargetKind::Group,
            label: Some(GROUP_CPU),
        }),
        parallelism: None,
    })
    .expect_err("parallelism is required");
    assert_eq!(
        missing,
        QueueContractError::NonPositiveCount {
            field: PARALLELISM_FIELD
        }
    );

    let unexpected = QueueControlRequest::new(QueueControlDraft {
        operation: QueueOperationKind::Pause,
        target: Some(QueueTargetDraft {
            kind: QueueTargetKind::Group,
            label: Some(GROUP_CPU),
        }),
        parallelism: Some(CPU_PARALLELISM),
    })
    .expect_err("parallelism is operation-specific");
    assert_eq!(
        unexpected,
        QueueContractError::ParallelismMismatch {
            operation: QueueOperationKind::Pause
        }
    );
}

#[test]
fn group_status_preserves_parallelism_and_safe_backend_labels() {
    let cpu = QueueGroupStatus::new(QueueGroupStatusDraft {
        group: GROUP_CPU,
        parallelism_limit: CPU_PARALLELISM,
        counts: QueueStatusCounts::new(QueueStatusCountDraft {
            pending: CPU_PENDING,
            running: CPU_RUNNING,
            paused: ZERO_COUNT,
            stashed: ZERO_COUNT,
            completed: ZERO_COUNT,
        }),
        backend_labels: &["cpu", "nix-build"],
    })
    .expect("cpu group should validate");
    let io = QueueGroupStatus::new(QueueGroupStatusDraft {
        group: GROUP_IO,
        parallelism_limit: IO_PARALLELISM,
        counts: QueueStatusCounts::new(QueueStatusCountDraft {
            pending: ZERO_COUNT,
            running: IO_RUNNING,
            paused: ZERO_COUNT,
            stashed: ZERO_COUNT,
            completed: IO_COMPLETED,
        }),
        backend_labels: &["io", "download"],
    })
    .expect("io group should validate");

    assert_eq!(cpu.parallelism_limit(), CPU_PARALLELISM);
    assert_eq!(cpu.counts().pending(), CPU_PENDING);
    assert_eq!(io.parallelism_limit(), IO_PARALLELISM);
    assert_eq!(io.counts().completed(), IO_COMPLETED);
}

#[test]
fn zero_parallelism_is_rejected() {
    let error = QueueGroupStatus::new(QueueGroupStatusDraft {
        group: GROUP_CPU,
        parallelism_limit: ZERO_COUNT,
        counts: QueueStatusCounts::new(QueueStatusCountDraft {
            pending: ZERO_COUNT,
            running: ZERO_COUNT,
            paused: ZERO_COUNT,
            stashed: ZERO_COUNT,
            completed: ZERO_COUNT,
        }),
        backend_labels: &[],
    })
    .expect_err("zero parallelism should fail");

    assert_eq!(
        error,
        QueueContractError::NonPositiveCount {
            field: PARALLELISM_FIELD
        }
    );
}
