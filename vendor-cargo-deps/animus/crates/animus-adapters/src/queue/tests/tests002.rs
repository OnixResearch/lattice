#[test]
fn bounded_log_viewport_records_truncation_and_hash() {
    let viewport = QueueLogViewport::new(
        QueueLogViewportDraft {
            task_id: TASK_ID,
            visible_text: VISIBLE_LOG,
            total_line_count: TOTAL_LINES,
            total_byte_count: TOTAL_BYTES,
            visible_line_count: VISIBLE_LINES,
            visible_byte_count: VISIBLE_BYTES,
            truncated: true,
            full_output_hash: Some(SAFE_HASH),
            safe_reference: Some(SAFE_REFERENCE),
        },
        bounds(),
    )
    .expect("bounded truncated viewport should validate");

    assert!(viewport.truncated());
    assert_eq!(viewport.visible_text(), VISIBLE_LOG);
    assert_eq!(viewport.total_line_count(), TOTAL_LINES);
    assert_eq!(
        viewport
            .full_output_hash()
            .expect("hash should be retained")
            .as_str(),
        SAFE_HASH
    );
    assert_eq!(viewport.safe_reference(), Some(SAFE_REFERENCE));
}

#[test]
fn oversized_visible_log_window_is_rejected() {
    let error = QueueLogViewport::new(
        QueueLogViewportDraft {
            task_id: TASK_ID,
            visible_text: VISIBLE_LOG,
            total_line_count: TOTAL_LINES,
            total_byte_count: TOTAL_BYTES,
            visible_line_count: TOTAL_LINES,
            visible_byte_count: VISIBLE_BYTES,
            truncated: true,
            full_output_hash: Some(SAFE_HASH),
            safe_reference: Some(SAFE_REFERENCE),
        },
        bounds(),
    )
    .expect_err("visible lines beyond bounds should fail");

    assert_eq!(
        error,
        QueueContractError::LogWindowTooLarge {
            field: VISIBLE_LINES_FIELD,
            max: MAX_LINES,
            actual: TOTAL_LINES,
        }
    );
}

#[test]
fn partial_log_without_truncation_marker_is_rejected() {
    let error = QueueLogViewport::new(
        QueueLogViewportDraft {
            task_id: TASK_ID,
            visible_text: VISIBLE_LOG,
            total_line_count: TOTAL_LINES,
            total_byte_count: TOTAL_BYTES,
            visible_line_count: VISIBLE_LINES,
            visible_byte_count: VISIBLE_BYTES,
            truncated: false,
            full_output_hash: Some(SAFE_HASH),
            safe_reference: Some(SAFE_REFERENCE),
        },
        bounds(),
    )
    .expect_err("partial logs require truncation marker");

    assert_eq!(error, QueueContractError::MissingTruncationMarker);
}

#[test]
fn send_input_requires_advertised_capability_and_safe_hash() {
    let request = QueueInteractionRequest::new(QueueInteractionDraft {
        kind: QueueInteractionKind::SendInput,
        task_id: TASK_ID,
        timeout_policy: TIMEOUT_POLICY,
        capabilities: QueueCapabilities::all(),
        input_hash: Some(SAFE_HASH),
    })
    .expect("send-input with capability and hash should validate");

    assert_eq!(request.kind(), QueueInteractionKind::SendInput);
    assert_eq!(request.task_id(), TASK_ID);
    assert_eq!(
        request
            .input_hash()
            .expect("send input stores only hash")
            .as_str(),
        SAFE_HASH
    );
}

#[test]
fn unsupported_send_input_fails_without_backend_fallback() {
    let error = QueueInteractionRequest::new(QueueInteractionDraft {
        kind: QueueInteractionKind::SendInput,
        task_id: TASK_ID,
        timeout_policy: TIMEOUT_POLICY,
        capabilities: QueueCapabilities::wait_stream_poll(),
        input_hash: Some(SAFE_HASH),
    })
    .expect_err("unsupported send-input should fail closed");

    assert_eq!(
        error,
        QueueContractError::UnsupportedCapability {
            capability: QueueCapabilityKind::SendInput
        }
    );
}

#[test]
fn send_input_omitting_hash_is_rejected() {
    let error = QueueInteractionRequest::new(QueueInteractionDraft {
        kind: QueueInteractionKind::SendInput,
        task_id: TASK_ID,
        timeout_policy: TIMEOUT_POLICY,
        capabilities: QueueCapabilities::all(),
        input_hash: None,
    })
    .expect_err("send-input cannot carry raw input or omit identity hash");

    assert_eq!(error, QueueContractError::MissingInputHash);
}

struct FixtureQueueService {
    response: QueueServiceResponse,
}

impl crate::ComponentAdapter<QueueServiceRequest, crate::ServiceOutcome<QueueServiceResponse>>
    for FixtureQueueService
{
    type Error = QueueContractError;

    fn call(
        &mut self,
        _request: &QueueServiceRequest,
    ) -> Result<crate::ServiceOutcome<QueueServiceResponse>, Self::Error> {
        Ok(crate::ServiceOutcome::completed(
            self.response.clone(),
            crate::Receipt::new(crate::ReceiptDraft::new(
                crate::ServiceKind::HostEffect,
                crate::CapabilityClass::HostEffect,
                crate::OutcomeKind::Completed,
                crate::Reason::Completed,
            )),
        ))
    }
}

fn bounds() -> QueueLogBounds {
    QueueLogBounds::new(MAX_LINES, MAX_BYTES).expect("test bounds should be positive")
}
