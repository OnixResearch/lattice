//! Q0: the published C1/C2 fixtures, their identities and every refusal code.

// r[verify lattice.vm_profile.admission]
// r[verify lattice.vm_profile.binding]
// r[verify lattice.vm_profile.identity]

use super::*;

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/vm-profile")
}

fn value(relative: &str) -> serde_json::Value {
    let bytes = std::fs::read(root().join(relative)).unwrap_or_else(|error| panic!("{relative}: {error}"));
    serde_json::from_slice(&bytes).unwrap_or_else(|error| panic!("{relative}: {error}"))
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    serde_json::from_value(value(relative)).unwrap_or_else(|error| panic!("{relative}: {error}"))
}

/// Fixture files of one directory whose names start with `prefix`, sorted.
fn named(directory: &str, prefix: &str) -> Vec<(String, String)> {
    let mut names: Vec<(String, String)> = std::fs::read_dir(root().join(directory))
        .expect("fixture directory")
        .map(|entry| entry.expect("fixture entry").file_name().into_string().expect("UTF-8 name"))
        .filter_map(|name| {
            let stem = name.strip_suffix(".json")?.strip_prefix(prefix)?.to_owned();
            Some((format!("{directory}/{name}"), stem))
        })
        .collect();
    names.sort();
    names
}

pub(crate) fn profile() -> ExecutionProfile {
    read("execution-profile/admitted.json")
}

pub(crate) fn binding() -> AttemptBinding {
    read("attempt-binding/admitted.json")
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    profile: String,
    envelope: String,
    attempt_id: String,
    fence: u64,
    guest_image: String,
    environment: String,
    execution: String,
    workspace: WorkspaceBinding,
    deadline_unix_ms: u64,
}

fn expected<'a>(context: &'a Context, profile: &'a ExecutionProfile) -> Expected<'a> {
    Expected {
        profile,
        envelope: Some(&context.envelope),
        attempt_id: Some(&context.attempt_id),
        fence: Some(context.fence),
        guest_image: Some(&context.guest_image),
        environment: Some(&context.environment),
        execution: Some(&context.execution),
        workspace: Some(&context.workspace),
        deadline_unix_ms: Some(context.deadline_unix_ms),
    }
}

#[test]
fn admitted_fixtures_reproduce_the_published_canonical_bytes_and_identities() {
    let vectors = value("vectors.json");
    for name in ["profile", "pinned_profile"] {
        let vector = &vectors[name];
        let profile: ExecutionProfile = read(vector["fixture"].as_str().expect("fixture"));
        assert_eq!(serde_json::to_string(&profile).expect("canonical"), vector["canonical"].as_str().expect("bytes"));
        assert_eq!(admit_profile(&profile).as_deref(), Ok(vector["identity"].as_str().expect("identity")));
        assert_eq!(vector["domain"], PROFILE_SCHEMA);
    }
    let context: Context = read("attempt-binding/context.json");
    let profile: ExecutionProfile = read(&context.profile);
    let binding = binding();
    let vector = &vectors["binding"];
    assert_eq!(serde_json::to_string(&binding).expect("canonical"), vector["canonical"].as_str().expect("bytes"));
    let admitted = admit_binding(&binding, &expected(&context, &profile)).expect("admitted binding");
    assert_eq!(admitted.binding_identity, vector["identity"].as_str().expect("identity"));
    assert_eq!(admitted.profile_identity, vectors["profile"]["identity"].as_str().expect("identity"));
    assert_eq!(admitted.disposition, DrainDisposition::Migrated);
    for (mode, disposition) in vectors["drain_dispositions"].as_object().expect("table") {
        let mode: Mobility = serde_json::from_value(serde_json::Value::from(mode.as_str())).expect("mode");
        assert_eq!(serde_json::to_value(drain_disposition(mode)).expect("disposition"), *disposition);
    }
}

#[test]
fn every_refused_profile_fixture_names_its_typed_refusal() {
    let refused = named("execution-profile", "refused-");
    assert!(refused.len() >= 14, "{refused:?}");
    for (path, code) in refused {
        let profile: ExecutionProfile = read(&path);
        let refusal = admit_profile(&profile).expect_err(&path);
        assert_eq!(refusal.code(), code, "{path}");
        assert_eq!(serde_json::to_value(refusal).expect("code"), serde_json::Value::from(code.as_str()));
    }
    for (path, _) in named("execution-profile", "invalid-") {
        assert!(serde_json::from_value::<ExecutionProfile>(value(&path)).is_err(), "{path}");
    }
}

// r[verify lattice.vm_profile.binding.scenario.crossed]
#[test]
fn every_crossed_binding_field_is_refused_before_effect() {
    let context: Context = read("attempt-binding/context.json");
    let profile: ExecutionProfile = read(&context.profile);
    let refused = named("attempt-binding", "refused-");
    let codes: Vec<&str> = refused.iter().map(|(_, code)| code.as_str()).collect();
    for field in [
        "crossed_profile",
        "crossed_guest_image",
        "crossed_environment",
        "crossed_envelope",
        "crossed_attempt",
        "crossed_fence",
        "crossed_execution",
        "crossed_workspace",
        "crossed_deadline",
        "mobility_not_offered",
    ] {
        assert!(codes.contains(&field), "missing fixture for {field}");
    }
    for (path, code) in &refused {
        let binding: AttemptBinding = read(path);
        let refusal = admit_binding(&binding, &expected(&context, &profile)).expect_err(path);
        assert_eq!(refusal.code(), code, "{path}");
    }
    for (path, _) in named("attempt-binding", "invalid-") {
        assert!(serde_json::from_value::<AttemptBinding>(value(&path)).is_err(), "{path}: placement never binds");
    }
}

/// The guest knows only the profile, envelope, attempt and fence; Kiln
/// checks the rest. The binding identity names no placement, so it is the
/// same before and after a move.
#[test]
fn guest_verifier_checks_what_it_knows_and_identity_ignores_placement() {
    let profile = profile();
    let binding = binding();
    let guest = Expected {
        profile: &profile,
        envelope: Some(&binding.envelope),
        attempt_id: Some(&binding.attempt_id),
        fence: Some(binding.fence),
        guest_image: None,
        environment: None,
        execution: None,
        workspace: None,
        deadline_unix_ms: None,
    };
    let admitted = admit_binding(&binding, &guest).expect("guest admission");
    let mut crossed = binding.clone();
    crossed.execution = "a".repeat(64);
    assert!(admit_binding(&crossed, &guest).is_ok(), "the guest does not know Kiln's execution");
    assert_ne!(binding_identity(&crossed), Ok(admitted.binding_identity.clone()));
    crossed = binding.clone();
    crossed.envelope = "9".repeat(64);
    assert_eq!(admit_binding(&crossed, &guest), Err(Refusal::CrossedEnvelope));
    let mut other_profile = profile.clone();
    other_profile.resources.vcpus = 4;
    assert_eq!(
        admit_binding(&binding, &Expected {
            profile: &other_profile,
            ..guest
        }),
        Err(Refusal::CrossedProfile)
    );
    assert_eq!(binding_identity(&binding.clone()), Ok(admitted.binding_identity));
}

// r[verify lattice.vm_profile.admission.scenario.mobility]
#[test]
fn requested_modes_outside_the_profile_and_live_precopy_are_refused() {
    let pinned: ExecutionProfile = read("execution-profile/admitted-pinned.json");
    assert_eq!(admit_mobility(&pinned, Mobility::Pinned), Ok(DrainDisposition::Pinned));
    for mode in [
        Mobility::StoppedTransfer,
        Mobility::LivePrecopy,
        Mobility::ApplicationCheckpoint,
    ] {
        assert_eq!(admit_mobility(&pinned, mode), Err(Refusal::MobilityNotOffered), "{mode:?}");
    }
    let profile = profile();
    assert_eq!(admit_mobility(&profile, Mobility::ApplicationCheckpoint), Ok(DrainDisposition::Checkpointed));
    let mut live = profile;
    live.mobility = vec![Mobility::Pinned, Mobility::LivePrecopy];
    assert_eq!(admit_mobility(&live, Mobility::Pinned), Err(Refusal::LivePrecopyUnqualified));
}

// r[verify lattice.vm_profile.admission.scenario.nickel]
#[test]
fn nickel_profiles_evaluate_to_the_published_fixture_and_negatives_are_refused() {
    let profiles = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("profiles/vm-execution");
    let evaluated = crate::nickel::evaluate_file(&profiles.join("admitted.ncl")).expect("positive profile");
    let admitted: ExecutionProfile = serde_json::from_value(evaluated).expect("C1 record");
    assert_eq!(admitted, profile());
    for (name, admission) in [
        ("refused-guest-network.ncl", None),
        ("refused-gpu-device.ncl", None),
        ("refused-unknown-field.ncl", None),
        ("refused-live-precopy.ncl", Some(Refusal::LivePrecopyUnqualified)),
        ("refused-checkpoint-required.ncl", Some(Refusal::CheckpointRequired)),
    ] {
        let evaluated = crate::nickel::evaluate_file(&profiles.join(name));
        match admission {
            None => assert!(evaluated.is_err(), "{name} must fail its Nickel contract"),
            Some(refusal) => {
                let profile: ExecutionProfile = serde_json::from_value(evaluated.expect(name)).expect(name);
                assert_eq!(admit_profile(&profile), Err(refusal), "{name}");
            }
        }
    }
}
