    #[test]
    fn waits_share_event_semantics_and_timeout() {
        // r[verify agent_harness_runtime.host_control.events_waits.timeout]
        let selector = agent_selector();
        let request = HostWaitRequest::new(
            CORRELATION_ID,
            selector.clone(),
            STATUS_DONE,
            DEFAULT_TIMEOUT_TICKS,
        )
        .expect("wait request should parse");
        let running_event = HostEvent::agent_status(
            CORRELATION_ID,
            selector.clone(),
            HostSemanticStatus::Running,
        )
        .expect("running event should parse");
        let output_event = HostEvent::output_match(CORRELATION_ID, selector.clone(), "done marker")
            .expect("output match should parse");
        let lifecycle_event =
            HostEvent::pane_lifecycle(CORRELATION_ID, selector.clone(), "created")
                .expect("lifecycle event should parse");
        let done_event =
            HostEvent::agent_status(CORRELATION_ID, selector.clone(), HostSemanticStatus::Done)
                .expect("done event should parse");
        let subscription = HostEventSubscription::new(
            CORRELATION_ID,
            Some(&selector),
            &[EVENT_OUTPUT_MATCH, EVENT_PANE_LIFECYCLE],
        )
        .expect("subscription should parse");
        let filtered = filter_host_events(
            &subscription,
            &[running_event.clone(), output_event, lifecycle_event],
        );
        let matched = resolve_host_wait(&request, &[running_event, done_event]);
        let cancelled = resolve_host_wait(
            &request,
            &[HostEvent::cancelled(CORRELATION_ID, selector.clone())
                .expect("cancel event should parse")],
        );
        let host_error = resolve_host_wait(
            &request,
            &[HostEvent::host_error(
                CORRELATION_ID,
                selector.clone(),
                HostTypedError::new(ERROR_CODE, ERROR_MESSAGE).expect("typed error should parse"),
            )
            .expect("host error event should parse")],
        );
        let elapsed_wait_result = resolve_host_wait(&request, &[]);
        let zero_ticks_error = HostWaitRequest::new(
            CORRELATION_ID,
            agent_selector(),
            STATUS_DONE,
            NO_TIMEOUT_TICKS,
        )
        .expect_err("zero timeout should fail");
        let unknown_event = HostEventKind::parse(BAD_VALUE).expect_err("unknown event should fail");

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].kind(), HostEventKind::OutputMatch);
        assert_eq!(filtered[1].kind(), HostEventKind::PaneLifecycle);
        assert_eq!(matched.kind(), HostWaitOutcomeKind::Matched);
        assert_eq!(cancelled.kind(), HostWaitOutcomeKind::Cancelled);
        assert_eq!(host_error.kind(), HostWaitOutcomeKind::HostError);
        assert_eq!(elapsed_wait_result.kind(), HostWaitOutcomeKind::Timeout);
        assert_eq!(zero_ticks_error, HostControlError::InvalidTimeout);
        assert!(matches!(
            unknown_event,
            HostControlError::UnsupportedValue { .. }
        ));
    }

    #[test]
    fn transport_envelope_preserves_unknown_features_without_ipc() {
        // r[verify agent_harness_runtime.host_control.transport_neutral.no_ipc_dependency]
        // r[verify agent_harness_runtime.host_control.verification]
        // r[verify agent_harness_runtime.host_control.verification.closeout]
        let selector = pane_selector(LIVE_REVISION, SelectorLifetime::Live, HOST_ID);
        let snapshot = snapshot(core::slice::from_ref(&selector), &[OP_FOCUS], &[]);
        let command = TopologyCommand::validate(
            &snapshot,
            TopologyCommandDraft {
                operation: OP_FOCUS,
                selector: Some(&selector),
                label: None,
            },
        )
        .expect("command should parse without IPC");

        let envelope = HostControlEnvelope::new(
            CORRELATION_ID,
            PROTOCOL_VERSION,
            command,
            &[FEATURE_LAYOUT, BAD_VALUE],
        )
        .expect("envelope should parse");
        let unknown_fields = HostUnknownFields::validate(&[UNKNOWN_FIELD_NAME])
            .expect("unknown fields should be preserved");
        let typed_error =
            HostTypedError::new(ERROR_CODE, ERROR_MESSAGE).expect("typed error should parse");
        let bad_version =
            HostControlEnvelope::new(CORRELATION_ID, BAD_VALUE, envelope.command().clone(), &[])
                .expect_err("bad protocol version should fail");
        let invalid_unknown_field =
            HostUnknownFields::validate(&[""]).expect_err("empty unknown field should fail");

        assert_eq!(
            envelope.accepted_features(),
            &[String::from(FEATURE_LAYOUT)]
        );
        assert_eq!(envelope.unknown_features(), &[String::from(BAD_VALUE)]);
        assert_eq!(unknown_fields.fields(), &[String::from(UNKNOWN_FIELD_NAME)]);
        assert_eq!(typed_error.code(), ERROR_CODE);
        assert_eq!(typed_error.message(), ERROR_MESSAGE);
        assert_eq!(bad_version, HostControlError::UnsupportedVersion);
        assert!(matches!(
            invalid_unknown_field,
            HostControlError::Contract(_)
        ));
    }
