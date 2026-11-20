//! `contracts/worker-execution-binding.ncl` admits the reviewed requirement,
//! declaration and capability shapes and refuses unsupported or malformed ones
//! before any effect.

// r[verify lattice.execution_binding.profile_facts.scenario.unsupported]
// r[verify lattice.worker_inventory.capability.scenario.malformed]
// r[verify lattice.worker_inventory.capability.scenario.undeclared]

const FIXTURES: &str = "tests/fixtures/execution-binding/nickel";

fn fixture(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES).join(format!("{relative}.ncl"))
}

#[test]
fn reviewed_configurations_satisfy_the_contract_and_the_strict_decoder() {
    for name in [
        "host-local-hosted-api",
        "provider-cpu-remote-model",
        "provider-direct-chip",
        "provider-cpu-declared-capability",
    ] {
        let configuration =
            lattice::execution_binding::format::load_configuration(&fixture(&format!("positive/{name}"))).expect(name);
        assert_eq!(configuration.schema, lattice::execution_binding::format::CONFIGURATION_SCHEMA);
    }
    let existing =
        lattice::execution_binding::format::load_configuration(&fixture("positive/provider-cpu-remote-model"))
            .expect("existing configuration");
    assert_eq!(existing.worker_capability, None, "existing configurations stay valid and undeclared");
    let declared =
        lattice::execution_binding::format::load_configuration(&fixture("positive/provider-cpu-declared-capability"))
            .expect("declared capability");
    let capability = declared.worker_capability.expect("declared");
    assert_eq!(capability.definition, "rust-implementer");
    assert_eq!(capability.tools, Some(lattice::execution_binding::core::capability::NameSet::Unavailable {}));
    let chip = lattice::execution_binding::format::load_configuration(&fixture("positive/provider-direct-chip"))
        .expect("direct chip");
    let lattice::execution_binding::core::request::ComputeRequirement::ProviderExecution {
        direct_accelerator: Some(accelerator),
    } = chip.worker_compute
    else {
        panic!("direct accelerator requirement expected");
    };
    assert_eq!(
        accelerator.minimum_enforcement,
        lattice::execution_binding::core::model::Enforcement::BackendAcknowledged
    );
}

#[test]
fn unsupported_configurations_fail_in_nickel() {
    for name in [
        "direct-accelerator-host-local",
        "zero-chips",
        "duplicate-route-names",
        "nine-routes",
        "unknown-granularity",
        "serving-kind-expectation",
        "noncanonical-route-identity",
        "capability-wrong-schema",
        "capability-invalid-definition",
        "capability-empty-set",
        "capability-too-many-names",
        "capability-duplicate-name",
        "capability-long-name",
        "capability-path-name",
        "capability-command-name",
        "capability-url-name",
        "capability-unknown-field",
        "capability-unknown-kind",
    ] {
        let path = fixture(&format!("negative/{name}"));
        assert!(path.is_file(), "{name} fixture exists");
        assert!(lattice::execution_binding::format::decode_fixture(&path).is_err(), "{name} must be refused");
    }
}

#[test]
fn the_strict_decoder_refuses_configurations_that_bypass_the_contract() {
    let directory = tempfile::tempdir().expect("scratch");
    let path = directory.path().join("bypass.ncl");
    std::fs::write(
        &path,
        r#"{
  schema = "lattice.worker-execution-binding.v1",
  worker_compute = { kind = "host_local" },
  worker_model = { kind = "no_model" },
  execution_binding = {
    schema = "lattice.execution-binding.v1",
    compute = { kind = "provider_allocation", allocation_id = "a", host_id = "h", consumer_id = "c", owner_generation = 1 },
    model_routes = { kind = "no_model" },
  },
}"#,
    )
    .expect("write bypass");
    let error = lattice::execution_binding::format::load_configuration(&path).expect_err("placement mismatch");
    assert!(error.contains("placement"), "{error}");
}

#[test]
fn the_strict_decoder_refuses_capability_declarations_that_bypass_the_contract() {
    let directory = tempfile::tempdir().expect("scratch");
    let path = directory.path().join("bypass.ncl");
    let configuration = |tools: &str| {
        format!(
            r#"{{
  schema = "lattice.worker-execution-binding.v1",
  worker_compute = {{ kind = "host_local" }},
  worker_model = {{ kind = "no_model" }},
  execution_binding = {{
    schema = "lattice.execution-binding.v1",
    compute = {{ kind = "host_local" }},
    model_routes = {{ kind = "no_model" }},
  }},
  worker_capability = {{ schema = "lattice.worker-capability.v1", definition = "rust-implementer", tools = {tools} }},
}}"#
        )
    };
    std::fs::write(&path, configuration(r#"{ kind = "declared", names = ["read"] }"#)).expect("write valid");
    assert!(
        lattice::execution_binding::format::load_configuration(&path).is_ok(),
        "uncontracted but well formed"
    );
    for tools in [
        r#"{ kind = "declared", names = ["/bin/sh"] }"#,
        r#"{ kind = "declared", names = [] }"#,
        r#"{ kind = "declared", names = ["read", "read"] }"#,
    ] {
        std::fs::write(&path, configuration(tools)).expect("write bypass");
        let error = lattice::execution_binding::format::load_configuration(&path).expect_err(tools);
        assert!(error.contains("worker capability declaration is unsupported"), "{error}");
        assert!(!error.contains("/bin/sh"), "the refusal names no value: {error}");
    }
}
