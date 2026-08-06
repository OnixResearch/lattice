    #[test]
    fn topology_command_validates_supported_selector() {
        // r[verify agent_harness_runtime.host_control.topology_dtos.validation]
        let selector = pane_selector(LIVE_REVISION, SelectorLifetime::Live, HOST_ID);
        let snapshot = snapshot(
            core::slice::from_ref(&selector),
            &[OP_FOCUS, OP_RENAME],
            &[LABEL_MAIN],
        );

        let command = TopologyCommand::validate(
            &snapshot,
            TopologyCommandDraft {
                operation: OP_FOCUS,
                selector: Some(&selector),
                label: None,
            },
        )
        .expect("valid focus should parse");

        assert_eq!(command.operation(), HostOperation::Focus);
        assert_eq!(
            command
                .selector()
                .expect("selector should exist")
                .resource_id(),
            PANE_ID
        );
    }

    #[test]
    fn topology_resource_summarizes_context_focus_and_status() {
        // r[verify agent_harness_runtime.host_control.topology_dtos]
        let selector = pane_selector(LIVE_REVISION, SelectorLifetime::Live, HOST_ID);
        let resource = HostTopologyResource::validate(HostTopologyResourceDraft {
            selector: &selector,
            label: Some(LABEL_MAIN),
            cwd_summary: Some(CWD_SUMMARY),
            provenance_summary: Some(PROVENANCE_SUMMARY),
            focused: true,
            status: Some(STATUS_RUNNING),
        })
        .expect("resource summary should parse");
        let invalid_status = HostTopologyResource::validate(HostTopologyResourceDraft {
            status: Some(BAD_VALUE),
            ..resource_draft(&selector)
        })
        .expect_err("bad status should fail");
        let invalid_cwd = HostTopologyResource::validate(HostTopologyResourceDraft {
            cwd_summary: Some("bad\0cwd"),
            ..resource_draft(&selector)
        })
        .expect_err("nul cwd summary should fail");

        assert_eq!(resource.selector().resource_id(), PANE_ID);
        assert_eq!(resource.label(), Some(LABEL_MAIN));
        assert_eq!(resource.cwd_summary(), Some(CWD_SUMMARY));
        assert_eq!(resource.provenance_summary(), Some(PROVENANCE_SUMMARY));
        assert!(resource.focused());
        assert_eq!(resource.status(), Some(HostSemanticStatus::Running));
        assert!(matches!(
            invalid_status,
            HostControlError::UnsupportedValue { .. }
        ));
        assert!(matches!(invalid_cwd, HostControlError::Contract(_)));
    }

    #[test]
    fn topology_command_rejects_invalid_selectors_and_labels() {
        // r[verify agent_harness_runtime.host_control.topology_dtos.validation]
        let selector = pane_selector(LIVE_REVISION, SelectorLifetime::Live, HOST_ID);
        let stale = pane_selector(STALE_REVISION, SelectorLifetime::Live, HOST_ID);
        let cross_host = pane_selector(LIVE_REVISION, SelectorLifetime::Live, OTHER_HOST_ID);
        let snapshot = snapshot(
            core::slice::from_ref(&selector),
            &[OP_FOCUS, OP_RENAME],
            &[LABEL_MAIN],
        );

        let missing = TopologyCommand::validate(
            &snapshot,
            TopologyCommandDraft {
                operation: OP_FOCUS,
                selector: None,
                label: None,
            },
        )
        .expect_err("missing selector should fail");
        let stale = TopologyCommand::validate(
            &snapshot,
            TopologyCommandDraft {
                operation: OP_FOCUS,
                selector: Some(&stale),
                label: None,
            },
        )
        .expect_err("stale live selector should fail");
        let cross_host = TopologyCommand::validate(
            &snapshot,
            TopologyCommandDraft {
                operation: OP_FOCUS,
                selector: Some(&cross_host),
                label: None,
            },
        )
        .expect_err("cross-host selector should fail");
        let duplicate = TopologyCommand::validate(
            &snapshot,
            TopologyCommandDraft {
                operation: OP_RENAME,
                selector: Some(&selector),
                label: Some(LABEL_MAIN),
            },
        )
        .expect_err("duplicate label should fail");
        let rejected_operation = TopologyCommand::validate(
            &snapshot,
            TopologyCommandDraft {
                operation: OP_CLOSE,
                selector: Some(&selector),
                label: None,
            },
        )
        .expect_err("unsupported operation should fail");

        assert_eq!(missing, HostControlError::MissingSelector);
        assert_eq!(stale, HostControlError::StaleSelector);
        assert_eq!(cross_host, HostControlError::CrossHostSelector);
        assert_eq!(duplicate, HostControlError::DuplicateLabel);
        assert_eq!(rejected_operation, HostControlError::UnsupportedOperation);
    }

    #[test]
    fn durable_selector_survives_revision_change() {
        // r[verify agent_harness_runtime.host_control.selector_lifetime.not_durable]
        let durable = pane_selector(STALE_REVISION, SelectorLifetime::Durable, HOST_ID);
        let snapshot = snapshot(core::slice::from_ref(&durable), &[OP_FOCUS], &[]);

        let command = TopologyCommand::validate(
            &snapshot,
            TopologyCommandDraft {
                operation: OP_FOCUS,
                selector: Some(&durable),
                label: None,
            },
        )
        .expect("durable selector should survive revision change");

        assert_eq!(
            command.selector().expect("selector expected").lifetime(),
            SelectorLifetime::Durable
        );
    }
