use super::super::KeyModifier;
use super::super::KeySequenceInput;
use super::super::KeySequenceReducerOutcome;
use super::super::KeySequenceState;
use super::super::reduce_key_sequence;
use super::ACTION_SAVE;
use super::LAYER_APP;
use super::accepted_catalog;
use super::active;
use super::binding;
use super::registry;
use super::sequence;
use super::stroke;

fn prefix_catalog() -> Result<super::super::BindingCatalogSnapshot, String> {
    let sequence = sequence(vec![
        stroke('x', vec![KeyModifier::Control])?,
        stroke('s', vec![KeyModifier::Control])?,
    ])?;
    accepted_catalog(1, vec![binding("binding.save.sequence", ACTION_SAVE, sequence, LAYER_APP)])
}

fn stroke_input(stroke: super::super::LogicalKeystroke) -> KeySequenceInput {
    KeySequenceInput::Stroke {
        stroke,
        observed_catalog_generation: 1,
        observed_context_generation: 1,
    }
}

#[test]
fn reducer_resolves_prefix_and_preserves_pending_on_unsupported_input() -> Result<(), String> {
    let catalog = prefix_catalog()?;
    let active = active(&catalog, 1, &[LAYER_APP])?;
    let first = reduce_key_sequence(
        &KeySequenceState::default(),
        &stroke_input(stroke('x', vec![KeyModifier::Control])?),
        &catalog,
        &registry(),
        &active,
    );
    let unsupported = reduce_key_sequence(
        &first.state,
        &KeySequenceInput::UnsupportedKey {
            key: "MediaPlay".to_string(),
            observed_catalog_generation: 1,
            observed_context_generation: 1,
        },
        &catalog,
        &registry(),
        &active,
    );
    let second = reduce_key_sequence(
        &unsupported.state,
        &stroke_input(stroke('s', vec![KeyModifier::Control])?),
        &catalog,
        &registry(),
        &active,
    );

    assert!(matches!(first.outcome, KeySequenceReducerOutcome::Pending(_)));
    assert_eq!(unsupported.state, first.state);
    assert!(matches!(unsupported.outcome, KeySequenceReducerOutcome::Unsupported(_)));
    assert!(matches!(
        second.outcome,
        KeySequenceReducerOutcome::Resolved(ref item) if item.action_id == ACTION_SAVE
    ));
    assert!(second.state.pending_sequence().is_none());
    Ok(())
}

#[test]
fn cancellation_expiry_unbound_and_stale_inputs_fail_closed() -> Result<(), String> {
    let catalog = prefix_catalog()?;
    let active = active(&catalog, 1, &[LAYER_APP])?;
    let pending = reduce_key_sequence(
        &KeySequenceState::default(),
        &stroke_input(stroke('x', vec![KeyModifier::Control])?),
        &catalog,
        &registry(),
        &active,
    );
    let cancel = reduce_key_sequence(
        &pending.state,
        &KeySequenceInput::Cancel {
            observed_catalog_generation: 1,
            observed_context_generation: 1,
        },
        &catalog,
        &registry(),
        &active,
    );
    let expire = reduce_key_sequence(
        &pending.state,
        &KeySequenceInput::Expire {
            observed_catalog_generation: 1,
            observed_context_generation: 1,
        },
        &catalog,
        &registry(),
        &active,
    );
    let unbound = reduce_key_sequence(
        &KeySequenceState::default(),
        &stroke_input(stroke('z', Vec::new())?),
        &catalog,
        &registry(),
        &active,
    );
    let stale = reduce_key_sequence(
        &pending.state,
        &KeySequenceInput::Stroke {
            stroke: stroke('s', vec![KeyModifier::Control])?,
            observed_catalog_generation: 0,
            observed_context_generation: 1,
        },
        &catalog,
        &registry(),
        &active,
    );

    assert!(matches!(cancel.outcome, KeySequenceReducerOutcome::Cancelled));
    assert!(matches!(expire.outcome, KeySequenceReducerOutcome::Expired));
    assert!(matches!(unbound.outcome, KeySequenceReducerOutcome::Unbound));
    assert!(matches!(stale.outcome, KeySequenceReducerOutcome::Stale(_)));
    assert!(stale.state.pending_sequence().is_none());
    Ok(())
}

#[test]
fn accepted_snapshot_replacement_cancels_old_focus_prefix() -> Result<(), String> {
    let old_catalog = prefix_catalog()?;
    let old_active = active(&old_catalog, 1, &[LAYER_APP])?;
    let pending = reduce_key_sequence(
        &KeySequenceState::default(),
        &stroke_input(stroke('x', vec![KeyModifier::Control])?),
        &old_catalog,
        &registry(),
        &old_active,
    );
    let new_catalog = accepted_catalog(2, vec![binding(
        "binding.save.sequence",
        ACTION_SAVE,
        sequence(vec![
            stroke('x', vec![KeyModifier::Control])?,
            stroke('s', vec![KeyModifier::Control])?,
        ])?,
        LAYER_APP,
    )])?;
    let new_active = active(&new_catalog, 2, &[LAYER_APP])?;
    let replacement = reduce_key_sequence(
        &pending.state,
        &KeySequenceInput::AcceptedSnapshotReplacement {
            previous_catalog_generation: 1,
            previous_context_generation: 1,
        },
        &new_catalog,
        &registry(),
        &new_active,
    );

    assert!(matches!(replacement.outcome, KeySequenceReducerOutcome::Stale(_)));
    assert!(replacement.state.pending_sequence().is_none());
    Ok(())
}
