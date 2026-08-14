mod common;

use lattice::engine::protocol::shell::SessionRepository;
use lattice::protocol_cli::CliCommand;
use lattice::protocol_cli::CommandArgs;
use lattice::protocol_cli::SessionCreateInput;

#[test]
fn cli_creates_and_inspects_a_producer_verified_session() {
    let directory = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let store = lattice::store::Store::new(directory.path().join("protocol.redb"))
        .unwrap_or_else(|error| panic!("store failed: {error}"));
    let source = common::protocol::native_source(common::protocol::end());
    let cohort = common::protocol::cohort(source.clone());
    let input = SessionCreateInput {
        choreography: source,
        placements: common::protocol::native_placements(&cohort),
        nonce: common::protocol::identity("cli-session"),
    };
    let path = directory.path().join("session.ncl");
    std::fs::write(
        &path,
        lattice::protocol_cli::session_create_input_to_nickel(&input)
            .unwrap_or_else(|error| panic!("encode failed: {error:?}")),
    )
    .unwrap_or_else(|error| panic!("write failed: {error}"));

    let created = lattice::protocol_cli::run(&store, &CommandArgs {
        command: CliCommand::Create { plan: path },
    });
    assert!(created.success, "{}", created.output);
    let output: serde_json::Value =
        serde_json::from_str(&created.output).unwrap_or_else(|error| panic!("decode failed: {error}"));
    let session = output["report"]["session"]
        .as_str()
        .unwrap_or_else(|| panic!("missing session identity"))
        .to_owned();
    assert!(!created.output.contains("credential-reference"));
    assert!(!created.output.contains(directory.path().to_string_lossy().as_ref()));

    let inspected = lattice::protocol_cli::run(&store, &CommandArgs {
        command: CliCommand::Inspect {
            session: session.clone(),
        },
    });
    assert!(inspected.success, "{}", inspected.output);
    assert!(inspected.output.contains(&session));
}

#[test]
fn cli_rejects_malformed_identity_and_missing_acknowledgements() {
    let directory = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let store = lattice::store::Store::new(directory.path().join("protocol.redb"))
        .unwrap_or_else(|error| panic!("store failed: {error}"));
    let malformed = lattice::protocol_cli::run(&store, &CommandArgs {
        command: CliCommand::Replay {
            session: "not-an-identity".to_owned(),
        },
    });
    assert!(!malformed.success);
    assert!(malformed.output.contains("protocol.identity-mismatch"));

    let current = common::protocol::unknown(&common::protocol::prepared_transfer());
    <lattice::store::Store as SessionRepository>::create(&store, &current)
        .unwrap_or_else(|error| panic!("persist failed: {error:?}"));
    let attempt = current.attempts[0].identity.as_str().to_owned();
    let retry = lattice::protocol_cli::run(&store, &CommandArgs {
        command: CliCommand::Retry {
            session: current.plan.session.as_str().to_owned(),
            attempt: attempt.clone(),
            acknowledge_duplicate_risk: false,
        },
    });
    assert!(!retry.success);
    assert!(retry.output.contains("protocol.acknowledgement-required"), "{}", retry.output);

    let abandon = lattice::protocol_cli::run(&store, &CommandArgs {
        command: CliCommand::Abandon {
            session: current.plan.session.as_str().to_owned(),
            attempt,
            acknowledge_unknown_outcome: false,
        },
    });
    assert!(!abandon.success);
    assert!(abandon.output.contains("protocol.acknowledgement-required"));
}
