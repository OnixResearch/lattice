// r[verify lattice.implementation_attempt.operator_repair.scenario.checks]

use super::Error;

fn pinned(name: &str) -> crate::candidate_worker::broker::Executable {
    crate::candidate_worker::broker::Executable {
        path: format!("/nix/store/{}-{name}/bin/{name}", "0".repeat(32)).into(),
        blake3: "a".repeat(64),
    }
}

fn catalog() -> serde_json::Value {
    serde_json::json!({
        "schema": super::SCHEMA,
        "sandbox": pinned("bwrap"),
        "checks": [
            {"name": "lint", "program": pinned("lint"), "args": ["--strict"], "failure_class": "lint"},
            {"name": "unit", "program": pinned("unit"), "args": [], "failure_class": "test"},
        ],
    })
}

fn parse(value: &serde_json::Value) -> Result<super::Pins, Error> {
    super::parse(&serde_json::to_vec(value).expect("catalog bytes"))
}

fn policy(catalog: &super::Pins) -> crate::implementation_attempt::repair::Policy {
    crate::implementation_attempt::repair::Policy {
        schema: crate::implementation_attempt::repair::POLICY_SCHEMA.into(),
        failure_classes: vec!["test".into()],
        max_iterations: 1,
        max_elapsed_ms: 10_000,
        max_model_requests: 2,
        max_tool_invocations: 6,
        step_tool_invocations: 1,
        check_executor: catalog.executor(),
        checks: catalog
            .checks
            .iter()
            .map(|entry| crate::implementation_attempt::repair::Check {
                name: entry.name.clone(),
                configuration: entry.configuration(),
            })
            .collect(),
    }
}

#[test]
fn a_pinned_catalog_is_admitted_for_exactly_its_policy() {
    let admitted = parse(&catalog()).expect("a pinned catalog");
    let policy = policy(&admitted);
    admitted.admit(&policy).expect("the catalog the policy names");
    let mut other_executor = policy.clone();
    other_executor.check_executor = "b".repeat(64);
    let mut reconfigured = policy.clone();
    reconfigured.checks[1].configuration = "c".repeat(64);
    let mut missing = policy.clone();
    missing.checks.pop();
    for policy in [other_executor, reconfigured, missing] {
        assert_eq!(admitted.admit(&policy), Err(Error::Unmatched));
    }
    // Every pinned fact is framed: a changed argument, program pin, failure
    // class or sandbox moves the identities the policy names.
    let mut changed = catalog();
    changed["checks"][1]["args"] = serde_json::json!(["--fast"]);
    let changed = parse(&changed).expect("changed catalog");
    assert_ne!(changed.checks[1].configuration(), admitted.checks[1].configuration());
    assert_eq!(changed.admit(&policy), Err(Error::Unmatched));
    let mut sandbox = catalog();
    sandbox["sandbox"]["blake3"] = serde_json::json!("d".repeat(64));
    assert_ne!(parse(&sandbox).expect("other sandbox").executor(), admitted.executor());
}

/// One edit of a valid catalog and the refusal it must cause.
type Mutation = (fn(&mut serde_json::Value), Error);

#[test]
fn unpinned_oversized_and_malformed_catalogs_are_refused() {
    let mutations: [Mutation; 9] = [
        (|c| c["checks"][0]["program"]["path"] = "/usr/bin/lint".into(), Error::Unpinned),
        (|c| c["checks"][0]["program"]["path"] = "/nix/store/../usr/bin/lint".into(), Error::Unpinned),
        (|c| c["checks"][0]["program"]["blake3"] = "A".repeat(64).into(), Error::Unpinned),
        (|c| c["sandbox"]["path"] = "bwrap".into(), Error::Unpinned),
        (|c| c["checks"] = serde_json::json!([]), Error::Entries),
        (|c| c["checks"].as_array_mut().expect("checks").reverse(), Error::Entries),
        (|c| c["checks"][0]["failure_class"] = "Lint!".into(), Error::Entries),
        (|c| c["checks"][0]["args"] = serde_json::json!(vec!["x"; super::MAX_ARGS + 1]), Error::Arguments),
        (|c| c["checks"][0]["timeout"] = 5.into(), Error::Malformed),
    ];
    for (mutate, expected) in mutations {
        let mut value = catalog();
        mutate(&mut value);
        assert_eq!(parse(&value).map(|_| ()), Err(expected), "{value}");
    }
    let mut schema = catalog();
    schema["schema"] = "lattice.implementation-attempt-check-catalog.v0".into();
    assert_eq!(parse(&schema).map(|_| ()), Err(Error::Malformed));
    let mut long = catalog();
    long["checks"][0]["args"] = serde_json::json!(["x".repeat(super::MAX_ARG_BYTES + 1)]);
    assert_eq!(parse(&long).map(|_| ()), Err(Error::Arguments));
    let oversized = vec![b' '; super::MAX_CATALOG_BYTES + 1];
    assert_eq!(super::parse(&oversized).map(|_| ()), Err(Error::Oversized));
}

#[test]
fn only_an_exit_on_an_unchanged_copy_decides_a_check() {
    use crate::candidate_worker::Outcome;
    use crate::implementation_attempt::repair::checks::Outcome as Check;
    let failed = |blocker: &str| Outcome::Failed {
        blocker: blocker.into(),
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    let exited = failed("worker exited with status Some(1)");
    assert_eq!(super::outcome(&exited, "test"), Check::Failed { class: "test".into() });
    for unknown in [
        failed("worker exceeded its admitted deadline; termination observed"),
        failed("worker exceeded its admitted output limit; termination observed"),
        failed("worker exited with status None"),
        Outcome::Blocked {
            blocker: "sandbox".into(),
        },
        Outcome::Cancelled {
            stdout: Vec::new(),
            stderr: Vec::new(),
        },
    ] {
        assert_eq!(super::outcome(&unknown, "test"), Check::Unknown, "{unknown:?}");
    }
}
