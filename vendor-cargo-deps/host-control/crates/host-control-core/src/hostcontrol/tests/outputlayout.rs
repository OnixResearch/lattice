    #[test]
    fn bounded_reads_truncate_and_hash_output() {
        // r[verify agent_harness_runtime.host_control.bounded_reads.truncated]
        let view = HostOutputView::new(HostOutputReadDraft {
            source: SOURCE_RECENT,
            body: OUTPUT_BODY,
            visible_line_budget: OUTPUT_LINE_BUDGET,
            redaction_status: REDACTION_CLEAN,
            full_output_reference: Some(SAFE_REFERENCE),
        })
        .expect("bounded output should parse");

        assert_eq!(view.source(), HostOutputSource::Recent);
        assert_eq!(view.visible_text(), OUTPUT_VISIBLE_TEXT);
        assert_eq!(view.truncation_status(), HostTruncationStatus::Truncated);
        assert_eq!(view.line_count(), OUTPUT_LINE_COUNT);
        assert!(view.content_hash().is_some());
        assert_eq!(view.full_output_reference(), Some(SAFE_REFERENCE));
    }

    #[test]
    fn bounded_reads_reject_unbounded_or_unsafe_inputs() {
        // r[verify agent_harness_runtime.host_control.bounded_reads.truncated]
        let unbounded = HostOutputView::new(HostOutputReadDraft {
            visible_line_budget: NO_TIMEOUT_TICKS,
            ..output_draft()
        })
        .expect_err("zero line budget should fail");
        let secret = HostOutputView::new(HostOutputReadDraft {
            body: SECRET_OUTPUT,
            ..output_draft()
        })
        .expect_err("secret output should fail");
        let malformed_source = HostOutputView::new(HostOutputReadDraft {
            source: BAD_VALUE,
            ..output_draft()
        })
        .expect_err("bad source should fail");
        let unsafe_reference = HostOutputView::new(HostOutputReadDraft {
            full_output_reference: Some(UNSAFE_REFERENCE),
            ..output_draft()
        })
        .expect_err("unsafe reference should fail");

        assert_eq!(unbounded, HostControlError::UnboundedRead);
        assert_eq!(secret, HostControlError::UnsafeOutput);
        assert!(matches!(
            malformed_source,
            HostControlError::UnsupportedValue { .. }
        ));
        assert_eq!(unsafe_reference, HostControlError::UnsafeOutput);
    }

    #[test]
    fn layout_apply_rejects_live_process_claims() {
        // r[verify agent_harness_runtime.host_control.layout_contract.apply]
        let selector = pane_selector(LIVE_REVISION, SelectorLifetime::Live, HOST_ID);
        let pane = LayoutNode::pane_with(LayoutPaneDraft {
            node_id: PANE_ID,
            label: Some(LABEL_MAIN),
            cwd_summary: Some(CWD_SUMMARY),
            command: Some(LayoutCommandDescriptorDraft {
                command: COMMAND_LABEL,
                args: &["--resume"],
            }),
            focused: true,
            zoomed: false,
        })
        .expect("pane should parse");
        let invalid_command = LayoutNode::pane_with(LayoutPaneDraft {
            command: Some(LayoutCommandDescriptorDraft {
                command: UNSAFE_REFERENCE,
                args: &[],
            }),
            ..pane_draft()
        })
        .expect_err("absolute command descriptor should fail");
        let invalid_split =
            LayoutNode::split("empty", AXIS_VERTICAL, vec![]).expect_err("empty split should fail");
        let layout =
            LayoutNode::split("root", AXIS_HORIZONTAL, vec![pane]).expect("split should parse");
        let request = LayoutApplyRequest::new(layout.clone(), false).expect("layout should parse");
        let response =
            LayoutApplyResponse::new(core::slice::from_ref(&selector), Some(&selector), false)
                .expect("layout response should parse");
        let invalid_request = LayoutApplyRequest::new(layout, true)
            .expect_err("live process preservation claim should fail");
        let invalid_response =
            LayoutApplyResponse::new(&[], None, true).expect_err("response claim should fail");

        assert!(!request.preserve_live_processes());
        assert_eq!(response.created_topology().len(), 1);
        assert_eq!(
            response
                .focused_selector()
                .expect("focused selector should exist")
                .resource_id(),
            PANE_ID
        );
        assert_eq!(invalid_command, HostControlError::InvalidLayout);
        assert_eq!(invalid_split, HostControlError::InvalidLayout);
        assert_eq!(invalid_request, HostControlError::InvalidLayout);
        assert_eq!(invalid_response, HostControlError::InvalidLayout);
    }
