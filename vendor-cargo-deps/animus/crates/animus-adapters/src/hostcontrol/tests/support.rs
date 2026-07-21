    fn pane_selector(revision: u64, lifetime: SelectorLifetime, host_id: &str) -> HostSelector {
        HostSelector::validate(HostSelectorDraft {
            host_id,
            resource_id: PANE_ID,
            kind: KIND_PANE,
            lifetime: lifetime.as_str(),
            revision,
        })
        .expect("selector should parse")
    }

    fn agent_selector() -> HostSelector {
        HostSelector::validate(HostSelectorDraft {
            host_id: HOST_ID,
            resource_id: AGENT_ID,
            kind: KIND_AGENT,
            lifetime: LIFETIME_LIVE,
            revision: LIVE_REVISION,
        })
        .expect("agent selector should parse")
    }

    fn snapshot(
        selectors: &[HostSelector],
        supported_operations: &[&str],
        labels: &[&str],
    ) -> HostTopologySnapshot {
        HostTopologySnapshot::validate(HostTopologySnapshotDraft {
            host_id: HOST_ID,
            revision: LIVE_REVISION,
            selectors,
            labels,
            supported_operations,
        })
        .expect("snapshot should parse")
    }

    const fn resource_draft(selector: &HostSelector) -> HostTopologyResourceDraft<'_> {
        HostTopologyResourceDraft {
            selector,
            label: None,
            cwd_summary: None,
            provenance_summary: None,
            focused: false,
            status: None,
        }
    }

    const fn pane_draft<'a>() -> LayoutPaneDraft<'a> {
        LayoutPaneDraft {
            node_id: PANE_ID,
            label: None,
            cwd_summary: None,
            command: None,
            focused: false,
            zoomed: false,
        }
    }

    const fn output_draft<'a>() -> HostOutputReadDraft<'a> {
        HostOutputReadDraft {
            source: SOURCE_RECENT,
            body: OUTPUT_BODY,
            visible_line_budget: OUTPUT_LINE_BUDGET,
            redaction_status: REDACTION_CLEAN,
            full_output_reference: None,
        }
    }
