//! Animus consumer conformance for the pinned standalone host-control core.

const HOST_ID: &str = "host:animus";
const OTHER_HOST_ID: &str = "host:other";
const PANE_ID: &str = "pane:main";
const KIND_PANE: &str = "pane";
const LIFETIME_LIVE: &str = "live";
const OP_FOCUS: &str = "focus";
const SOURCE_RECENT: &str = "recent";
const REDACTION_CLEAN: &str = "clean";
const OUTPUT: &str = "first\nsecond\nthird";
const LIVE_REVISION: u64 = 7;
const STALE_REVISION: u64 = 6;
const LINE_BUDGET: u32 = 2;

fn selector(host_id: &str, revision: u64) -> Result<animus_adapters::HostSelector, animus_adapters::HostControlError> {
    animus_adapters::HostSelector::validate(animus_adapters::HostSelectorDraft {
        host_id,
        resource_id: PANE_ID,
        kind: KIND_PANE,
        lifetime: LIFETIME_LIVE,
        revision,
    })
}

#[test]
fn compatibility_facade_preserves_topology_and_bounded_output_behavior() -> Result<(), animus_adapters::HostControlError>
{
    let selected = selector(HOST_ID, LIVE_REVISION)?;
    let snapshot = animus_adapters::HostTopologySnapshot::validate(animus_adapters::HostTopologySnapshotDraft {
        host_id: HOST_ID,
        revision: LIVE_REVISION,
        selectors: core::slice::from_ref(&selected),
        labels: &[],
        supported_operations: &[OP_FOCUS],
    })?;
    let command = animus_adapters::TopologyCommand::validate(&snapshot, animus_adapters::TopologyCommandDraft {
        operation: OP_FOCUS,
        selector: Some(&selected),
        label: None,
    })?;
    let output = animus_adapters::HostOutputView::new(animus_adapters::HostOutputReadDraft {
        source: SOURCE_RECENT,
        body: OUTPUT,
        visible_line_budget: LINE_BUDGET,
        redaction_status: REDACTION_CLEAN,
        full_output_reference: None,
    })?;

    assert_eq!(command.selector(), Some(&selected));
    assert_eq!(selected.lifetime(), animus_adapters::SelectorLifetime::Live);
    assert_eq!(output.source(), animus_adapters::HostOutputSource::Recent);
    assert_eq!(output.truncation_status(), animus_adapters::HostTruncationStatus::Truncated);
    assert!(output.content_hash().is_some());
    Ok(())
}

#[test]
fn compatibility_facade_rejects_stale_cross_host_and_unbounded_inputs() -> Result<(), animus_adapters::HostControlError>
{
    let selected = selector(HOST_ID, LIVE_REVISION)?;
    let stale = selector(HOST_ID, STALE_REVISION)?;
    let cross_host = selector(OTHER_HOST_ID, LIVE_REVISION)?;
    let snapshot = animus_adapters::HostTopologySnapshot::validate(animus_adapters::HostTopologySnapshotDraft {
        host_id: HOST_ID,
        revision: LIVE_REVISION,
        selectors: core::slice::from_ref(&selected),
        labels: &[],
        supported_operations: &[OP_FOCUS],
    })?;

    for (candidate, expected) in [
        (&stale, animus_adapters::HostControlError::StaleSelector),
        (&cross_host, animus_adapters::HostControlError::CrossHostSelector),
    ] {
        let Err(error) = animus_adapters::TopologyCommand::validate(&snapshot, animus_adapters::TopologyCommandDraft {
            operation: OP_FOCUS,
            selector: Some(candidate),
            label: None,
        }) else {
            return Err(expected);
        };
        assert_eq!(error, expected);
    }

    let Err(unbounded) = animus_adapters::HostOutputView::new(animus_adapters::HostOutputReadDraft {
        source: SOURCE_RECENT,
        body: OUTPUT,
        visible_line_budget: 0,
        redaction_status: REDACTION_CLEAN,
        full_output_reference: None,
    }) else {
        return Err(animus_adapters::HostControlError::UnboundedRead);
    };
    assert_eq!(unbounded, animus_adapters::HostControlError::UnboundedRead);
    Ok(())
}
