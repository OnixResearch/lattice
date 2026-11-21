// r[verify lattice.implementation_attempt.lineage]
// r[verify lattice.implementation_attempt.lineage.scenario.exhausted]
// r[verify lattice.implementation_attempt.lineage.scenario.shared]

use super::Backpressure;
use super::Binding;
use super::BindingError;
use super::Blocker;
use super::Draws;
use super::Refusal;

fn hex(digit: char) -> String {
    digit.to_string().repeat(64)
}

pub(crate) fn binding(model_requests: u64, tool_invocations: u64) -> Binding {
    Binding {
        schema: super::LINEAGE_SCHEMA.into(),
        root: hex('1'),
        account: hex('2'),
        parent: Some(hex('1')),
        depth: 1,
        remaining: [
            (super::MODEL_REQUESTS.to_owned(), model_requests),
            (super::TOOL_INVOCATIONS.to_owned(), tool_invocations),
        ]
        .into_iter()
        .collect(),
        blocker: None,
        backpressure: Backpressure::Eligible,
    }
}

const fn draws(model_requests: u64, tool_invocations: u64) -> Draws {
    Draws {
        model_requests,
        tool_invocations,
    }
}

#[test]
fn a_well_formed_snapshot_validates_and_every_field_moves_its_identity() {
    let base = binding(4, 8);
    base.validate().expect("valid snapshot");
    let mut root = base.clone();
    root.account = root.root.clone();
    root.parent = None;
    root.depth = 0;
    root.validate().expect("root account");
    let changes: [fn(&mut Binding); 6] = [
        |value| value.depth = 2,
        |value| value.account = hex('3'),
        |value| value.blocker = Some(Blocker::Denied),
        |value| value.backpressure = Backpressure::None,
        |value| {
            value.remaining.insert("gpu_seconds".into(), 1);
        },
        |value| {
            value.remaining.insert(super::MODEL_REQUESTS.into(), 5);
        },
    ];
    for change in changes {
        let mut changed = base.clone();
        change(&mut changed);
        assert_ne!(changed.identity(), base.identity());
    }
    assert_eq!(base.identity(), binding(4, 8).identity(), "stable identity");
}

#[test]
fn malformed_snapshots_are_refused() {
    type Case = (fn(&mut Binding), BindingError);
    let cases: [Case; 9] = [
        (|value| value.schema = "kiln.lineage".into(), BindingError::Schema),
        (|value| value.root = "ABC".into(), BindingError::Identity),
        (|value| value.account = hex('A'), BindingError::Identity),
        (|value| value.parent = None, BindingError::Shape),
        (|value| value.depth = 0, BindingError::Shape),
        (|value| value.depth = super::MAX_LINEAGE_DEPTH + 1, BindingError::Shape),
        (|value| value.remaining.clear(), BindingError::Dimensions),
        (
            |value| {
                value.remaining.insert(String::new(), 1);
            },
            BindingError::Dimensions,
        ),
        (
            |value| {
                value.remaining.insert("a".repeat(super::MAX_DIMENSION_NAME_BYTES + 1), 1);
            },
            BindingError::Dimensions,
        ),
    ];
    for (change, expected) in cases {
        let mut changed = binding(4, 8);
        change(&mut changed);
        assert_eq!(changed.validate(), Err(expected));
    }
    let mut self_parent = binding(4, 8);
    self_parent.parent = Some(self_parent.account.clone());
    assert_eq!(self_parent.validate(), Err(BindingError::Shape));
}

#[test]
// r[verify lattice.implementation_attempt.lineage.scenario.exhausted]
fn the_gate_refuses_blockers_backpressure_and_exhaustion_before_spawn() {
    let lineage = binding(4, 8);
    lineage.gate(&Draws::default(), &draws(4, 8)).expect("exactly enough");
    assert_eq!(
        lineage.gate(&draws(1, 0), &draws(4, 1)),
        Err(Refusal::Exhausted {
            dimension: super::MODEL_REQUESTS.into()
        })
    );
    assert_eq!(
        lineage.gate(&draws(0, 8), &draws(1, 0)),
        Err(Refusal::Exhausted {
            dimension: super::TOOL_INVOCATIONS.into()
        }),
        "a zero remainder refuses even an empty reservation"
    );
    let mut unknown = binding(4, 8);
    unknown.remaining.insert("gpu_seconds".into(), 0);
    assert_eq!(
        unknown.gate(&Draws::default(), &draws(1, 1)),
        Err(Refusal::Exhausted {
            dimension: "gpu_seconds".into()
        })
    );
    unknown.remaining.insert("gpu_seconds".into(), 3);
    unknown.gate(&draws(3, 7), &draws(1, 1)).expect("an unaccounted dimension is never drawn");
    assert_eq!(unknown.unaccounted(), vec!["gpu_seconds".to_owned()]);
    let mut blocked = binding(4, 8);
    blocked.blocker = Some(Blocker::Cancelled);
    assert_eq!(
        blocked.gate(&Draws::default(), &draws(1, 1)),
        Err(Refusal::Blocked {
            blocker: Blocker::Cancelled
        })
    );
    for gate in [
        Backpressure::Waiting,
        Backpressure::Overloaded,
        Backpressure::Stale,
        Backpressure::Incomplete,
        Backpressure::Incompatible,
    ] {
        let mut pressured = binding(4, 8);
        pressured.backpressure = gate;
        assert_eq!(pressured.gate(&Draws::default(), &draws(1, 1)), Err(Refusal::Backpressure { gate }));
    }
    let mut ungated = binding(4, 8);
    ungated.backpressure = Backpressure::None;
    ungated.gate(&Draws::default(), &draws(1, 1)).expect("no serving owner");
}

#[test]
fn kiln_alias_names_are_admitted_and_unmapped_ones_stay_unaccounted() {
    // Kiln's `Alias::new` admits any non-empty name of at most 256 bytes.
    let mut kiln = binding(4, 8);
    for name in [
        "gpu-seconds",
        "Model Requests",
        &"g".repeat(super::MAX_DIMENSION_NAME_BYTES),
    ] {
        kiln.remaining.insert(name.to_owned(), 2);
    }
    kiln.validate().expect("every Kiln alias is admitted");
    let mut unaccounted = kiln.unaccounted();
    unaccounted.sort();
    let mut expected = vec![
        "Model Requests".to_owned(),
        "g".repeat(super::MAX_DIMENSION_NAME_BYTES),
        "gpu-seconds".to_owned(),
    ];
    expected.sort();
    assert_eq!(unaccounted, expected, "an unmapped dimension is reported, never rejected or drawn");
    kiln.gate(&draws(3, 7), &draws(1, 1)).expect("an unmapped dimension is never drawn");
    assert_eq!(kiln.overdrawn(&draws(4, 8)), None, "a draw equal to the remainder fits");
    assert_eq!(kiln.overdrawn(&draws(4, 9)), Some(super::TOOL_INVOCATIONS.to_owned()));
    assert_eq!(kiln.overdrawn(&draws(5, 9)), Some(super::MODEL_REQUESTS.to_owned()));
}

#[test]
// r[verify lattice.implementation_attempt.lineage.scenario.shared]
fn children_draw_from_the_parent_remainder_and_never_get_a_fresh_budget() {
    let parent = binding(10, 20);
    let mut child = binding(4, 8);
    child.root = parent.root.clone();
    child.parent = Some(parent.account.clone());
    child.account = hex('4');
    child.depth = 2;
    parent.admit_child(&draws(6, 12), &child).expect("fits the remainder");
    assert_eq!(
        parent.admit_child(&draws(7, 12), &child),
        Err(Refusal::FreshBudget {
            dimension: super::MODEL_REQUESTS.into()
        })
    );
    let mut omitted = child.clone();
    omitted.remaining.remove(super::TOOL_INVOCATIONS);
    assert_eq!(
        parent.admit_child(&Draws::default(), &omitted),
        Err(Refusal::FreshBudget {
            dimension: super::TOOL_INVOCATIONS.into()
        })
    );
    let mut extra = child.clone();
    extra.remaining.insert("gpu_seconds".into(), 1);
    assert_eq!(
        parent.admit_child(&Draws::default(), &extra),
        Err(Refusal::FreshBudget {
            dimension: "gpu_seconds".into()
        })
    );
    let crossed: [fn(&mut Binding); 3] = [
        |value| value.root = hex('9'),
        |value| value.parent = Some(hex('9')),
        |value| value.depth = 3,
    ];
    for change in crossed {
        let mut changed = child.clone();
        change(&mut changed);
        assert_eq!(parent.admit_child(&Draws::default(), &changed), Err(Refusal::CrossedChild));
    }
}

#[test]
fn draws_add_without_wrapping() {
    assert_eq!(draws(1, 2).saturating_add(&draws(3, 4)), draws(4, 6));
    assert_eq!(draws(u64::MAX, 0).saturating_add(&draws(1, 0)), draws(u64::MAX, 0));
    assert!(binding(4, 8).gate(&draws(u64::MAX, 0), &draws(1, 1)).is_err(), "a saturated draw exhausts");
}

#[test]
fn the_wire_shape_rejects_unknown_fields_and_kiln_reasons_outside_the_lineage_subset() {
    let value = serde_json::to_value(binding(1, 1)).expect("json");
    let decoded: Binding = serde_json::from_value(value.clone()).expect("round trip");
    assert_eq!(decoded, binding(1, 1));
    let mut extra = value.clone();
    extra["available"] = serde_json::json!({});
    assert!(serde_json::from_value::<Binding>(extra).is_err());
    let mut reason = value;
    reason["blocker"] = serde_json::Value::from("queue_full");
    assert!(serde_json::from_value::<Binding>(reason).is_err());
}
