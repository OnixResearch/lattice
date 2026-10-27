use super::*;
use crate::engine::effect::authority;

const NOW: u64 = 100;
const EXPIRY: u64 = 200;
const RUN: &str = "test-run";

fn profile() -> Profile {
    Profile {
        schema: PROFILE_SCHEMA.to_owned(),
        workflow_revision: authority::blake3_reference(b"test-workflow", &["original"]),
        machine_identity: authority::blake3_reference(b"test-machine", &["original"]),
        policy_ref: authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF.to_owned(),
        expires_at_unix_ms: EXPIRY,
        grants: vec![Grant {
            node_id: 0,
            operation: "lattice/shell".to_owned(),
            resource: "lattice://process/0".to_owned(),
            ability: "process/execute".to_owned(),
        }],
        roots: BTreeMap::from([("workspace".to_owned(), PathBuf::from("/workspace"))]),
    }
}

fn binding(admitted: &AdmittedProfile) -> authority::IntentBinding {
    serde_json::from_value(serde_json::json!({
        "schema": authority::CHOREGRAPH_EFFECT_AUTHORITY_BINDING_SCHEMA,
        "machine": admitted.profile.machine_identity,
        "workflow_revision": admitted.profile.workflow_revision,
        "run": RUN,
        "node": "lattice-node/0",
        "transition": START_TRANSITION,
        "operation": "lattice/shell",
        "resource": "lattice://process/0",
        "ability": "process/execute",
        "request": "request",
        "payload": "payload",
        "policy": admitted.profile.policy_ref,
        "authority": admitted.identity,
        "intent": "intent",
        "scope": {"kind": "unscoped"},
    }))
    .unwrap()
}

// r[verify lattice.standalone_effect_authority.currentness.scenario.expired]
#[test]
fn exact_grant_is_current_only_before_expiry() {
    let admitted = admit(profile(), NOW).unwrap();
    let binding = binding(&admitted);
    assert_eq!(check_binding(&admitted, &binding, RUN, EXPIRY - 1), Ok(&admitted.profile.grants[0]));
    assert_eq!(check_binding(&admitted, &binding, RUN, EXPIRY), Err(ProfileError::Expired));
    assert_eq!(admit(profile(), EXPIRY).unwrap_err(), ProfileError::Expired);
}

#[test]
fn every_identity_and_scope_boundary_is_required() {
    let admitted = admit(profile(), NOW).unwrap();
    let original = binding(&admitted);
    let mutations: &[fn(&mut authority::IntentBinding)] = &[
        |b| b.schema = "other".to_owned(),
        |b| b.machine = authority::MachineIdentity::new("other").unwrap(),
        |b| b.workflow_revision = authority::WorkflowRevisionIdentity::new("other").unwrap(),
        |b| b.run = authority::RunIdentity::new("other").unwrap(),
        |b| b.policy = authority::PolicyIdentity::new("other").unwrap(),
        |b| b.authority = authority::AuthorityIdentity::new("other").unwrap(),
        |b| b.transition = authority::TransitionIdentity::new("pending->start").unwrap(),
        |b| b.node = authority::NodeIdentity::new("lattice-node/00").unwrap(),
        |b| b.operation = authority::OperationIdentity::new("lattice/operation").unwrap(),
        |b| b.resource = authority::ResourceIdentity::new("lattice://process/1").unwrap(),
        |b| b.ability = authority::AbilityIdentity::new("process/*").unwrap(),
        |b| {
            b.scope = authority::CollaborationScopeBinding::Protected {
                scope: authority::ScopeIdentity::new("scope").unwrap(),
                revision: authority::ScopeRevisionIdentity::new("revision").unwrap(),
                membership_epoch: 1,
                audience: authority::AudienceIdentity::new("audience").unwrap(),
                required_consent: authority::ConsentDecisionIdentity::new("consent").unwrap(),
                snapshot: authority::ScopeSnapshotIdentity::new("snapshot").unwrap(),
            }
        },
    ];
    assert!(check_binding(&admitted, &original, RUN, NOW).is_ok());
    assert_eq!(check_binding(&admitted, &original, "another-run", NOW), Err(ProfileError::BindingMismatch));
    for mutate in mutations {
        let mut changed = original.clone();
        mutate(&mut changed);
        assert_eq!(check_binding(&admitted, &changed, RUN, NOW), Err(ProfileError::BindingMismatch));
    }
}

#[test]
fn closed_tuples_include_zero_and_all_reviewed_effect_classes() {
    for (operation, prefix, ability, contract) in [
        ("lattice/shell", PROCESS_RESOURCE_PREFIX, "process/execute", "lattice-process"),
        ("lattice/operation", PROCESS_RESOURCE_PREFIX, "process/execute", "lattice-process"),
        ("lattice/http", NETWORK_RESOURCE_PREFIX, "network/request", "lattice-network"),
        ("lattice/file", FILE_RESOURCE_PREFIX, "file/read", "lattice-file"),
        ("lattice/file", FILE_RESOURCE_PREFIX, "file/write", "lattice-file"),
        ("lattice/file", FILE_RESOURCE_PREFIX, "file/append", "lattice-file"),
        ("lattice/agent", AGENT_RESOURCE_PREFIX, "agent/execute", "lattice-agent"),
    ] {
        for node_id in [0, u64::MAX] {
            let mut grant = Grant {
                node_id,
                operation: operation.to_owned(),
                resource: format!("{prefix}{node_id}"),
                ability: ability.to_owned(),
            };
            assert_eq!(grant.contract_id(), Some(contract));
            grant.resource.push('*');
            assert_eq!(grant.contract_id(), None);
        }
    }
}

#[test]
fn malformed_profiles_cannot_be_admitted() {
    let mutations: &[fn(&mut Profile)] = &[
        |p| p.schema.push('x'),
        |p| p.policy_ref.push('x'),
        |p| p.workflow_revision = p.workflow_revision.to_uppercase(),
        |p| p.machine_identity = "blake3:00".to_owned(),
        |p| p.expires_at_unix_ms = 0,
        |p| p.grants.clear(),
        |p| p.grants.push(p.grants[0].clone()),
        |p| p.grants[0].operation = "lattice/unknown".to_owned(),
        |p| p.grants[0].resource = "lattice://process/*".to_owned(),
        |p| p.grants[0].ability = "process/*".to_owned(),
        |p| {
            p.roots.insert("relative".to_owned(), PathBuf::from("relative/path"));
        },
        |p| {
            p.roots.insert(String::new(), PathBuf::from("/absolute"));
        },
        |p| {
            p.roots.insert("x".repeat(MAX_ROOT_ID_BYTES + 1), PathBuf::from("/absolute"));
        },
    ];
    assert!(admit(profile(), NOW).is_ok());
    for mutate in mutations {
        let mut changed = profile();
        mutate(&mut changed);
        assert_eq!(admit(changed, NOW).unwrap_err(), ProfileError::InvalidProfile);
    }
    let mut value = serde_json::to_value(profile()).unwrap();
    value["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Profile>(value).is_err());
    let mut grant = serde_json::to_value(&profile().grants[0]).unwrap();
    grant["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Grant>(grant).is_err());
}

#[test]
fn collection_limits_admit_boundary_and_reject_overflow() {
    let mut bounded = profile();
    bounded.grants = (0..MAX_PROFILE_GRANTS as u64)
        .map(|node_id| Grant {
            node_id,
            resource: format!("{PROCESS_RESOURCE_PREFIX}{node_id}"),
            ..bounded.grants[0].clone()
        })
        .collect();
    bounded.roots =
        (0..MAX_PROFILE_ROOTS).map(|index| (format!("root-{index}"), PathBuf::from("/workspace"))).collect();
    assert!(admit(bounded.clone(), NOW).is_ok());
    let mut too_many_roots = bounded.clone();
    too_many_roots.roots.insert("extra".to_owned(), PathBuf::from("/workspace"));
    assert_eq!(admit(too_many_roots, NOW).unwrap_err(), ProfileError::InvalidProfile);
    bounded.grants.push(Grant {
        node_id: MAX_PROFILE_GRANTS as u64,
        resource: format!("{PROCESS_RESOURCE_PREFIX}{MAX_PROFILE_GRANTS}"),
        ..bounded.grants[0].clone()
    });
    assert_eq!(admit(bounded, NOW).unwrap_err(), ProfileError::InvalidProfile);
}

#[test]
fn semantic_edits_revoke_pinned_identity_but_source_formatting_does_not() {
    let original = admit(profile(), NOW).unwrap();
    let pretty = serde_json::to_string_pretty(&original.profile).unwrap();
    let reparsed = admit(serde_json::from_str(&pretty).unwrap(), NOW).unwrap();
    assert_eq!(original.identity, reparsed.identity);
    let mutations: &[fn(&mut Profile)] = &[
        |p| p.workflow_revision = authority::blake3_reference(b"workflow", &["changed import"]),
        |p| p.machine_identity = authority::blake3_reference(b"machine", &["changed import"]),
        |p| p.expires_at_unix_ms += 1,
        |p| p.grants[0].operation = "lattice/operation".to_owned(),
        |p| {
            p.roots.insert("workspace".to_owned(), PathBuf::from("/changed-import-root"));
        },
    ];
    for mutate in mutations {
        let mut changed = original.profile.clone();
        mutate(&mut changed);
        let changed = admit(changed, NOW).unwrap();
        assert_ne!(original.identity, changed.identity);
        assert_eq!(check_binding(&changed, &binding(&original), RUN, NOW), Err(ProfileError::BindingMismatch));
    }
}
