let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 315,
  expected_missing_hash = "2b7906ee39473996244bcfbf795e9ee984704c8bbfe2db9017309647c4d8373c",
  reviewed = [
    {
      requirement_id = "n8nclone.repository_validation.clippy_scope",
      classification = "partial_evidence_only",
      evidence = [
        { path = "Cargo.toml", blake3 = "ae4d9a5744f9c07e2091ad48fae45862c6559488e5492a8cec0054b99cba11fa" },
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
      ],
      rationale_code = "gap.no-direct-vendor-lint-fixture",
    },
    {
      requirement_id = "n8nclone.repository_validation.clippy_scope.scenario.vendor-lints",
      classification = "partial_evidence_only",
      evidence = [
        { path = "Cargo.toml", blake3 = "ae4d9a5744f9c07e2091ad48fae45862c6559488e5492a8cec0054b99cba11fa" },
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
      ],
      rationale_code = "gap.no-triggering-vendor-lint-fixture",
    },
    {
      requirement_id = "n8nclone.repository_validation.flake_check",
      classification = "partial_evidence_only",
      evidence = [
        { path = "README.md", blake3 = "be65d085648a01ff0ff925d4b0ad1eae5c77ce3a0ae0a03c907ec4789d5adacc" },
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
      ],
      rationale_code = "gap.no-hostile-host-signing-check",
    },
    {
      requirement_id = "n8nclone.repository_validation.flake_check.scenario.missing-signing-key",
      classification = "partial_evidence_only",
      evidence = [
        { path = "README.md", blake3 = "be65d085648a01ff0ff925d4b0ad1eae5c77ce3a0ae0a03c907ec4789d5adacc" },
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
      ],
      rationale_code = "gap.no-missing-key-process-check",
    },
    {
      requirement_id = "n8nclone.repository_validation.flake_check.scenario.normal-signing",
      classification = "partial_evidence_only",
      evidence = [
        { path = "README.md", blake3 = "be65d085648a01ff0ff925d4b0ad1eae5c77ce3a0ae0a03c907ec4789d5adacc" },
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
      ],
      rationale_code = "gap.no-external-signing-policy-assertion",
    },
    {
      requirement_id = "n8nclone.repository_validation.format_scope",
      classification = "partial_evidence_only",
      evidence = [
        { path = "Cargo.toml", blake3 = "ae4d9a5744f9c07e2091ad48fae45862c6559488e5492a8cec0054b99cba11fa" },
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
      ],
      rationale_code = "gap.no-direct-vendor-format-fixture",
    },
    {
      requirement_id = "n8nclone.repository_validation.format_scope.scenario.vendor-format",
      classification = "partial_evidence_only",
      evidence = [
        { path = "Cargo.toml", blake3 = "ae4d9a5744f9c07e2091ad48fae45862c6559488e5492a8cec0054b99cba11fa" },
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
      ],
      rationale_code = "gap.no-upstream-format-difference-assertion",
    },
    {
      requirement_id = "n8nclone.repository_validation.test_source",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
        { path = "tests/fixtures/n8n/shell.json", blake3 = "f72d1067fa2a19983040d22454031baffd640068efcbb33617fc6853c02609d6" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "assertions.nix-nextest-compiles-tracked-test-fixtures",
    },
    {
      requirement_id = "n8nclone.repository_validation.test_source.scenario.json-fixture",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
        { path = "tests/fixtures/n8n/shell.json", blake3 = "f72d1067fa2a19983040d22454031baffd640068efcbb33617fc6853c02609d6" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "assertions.filtered-json-fixture-compiles",
    },
    {
      requirement_id = "n8nclone.repository_validation.vendor_snapshot",
      classification = "partial_evidence_only",
      evidence = [
        { path = "Cargo.toml", blake3 = "ae4d9a5744f9c07e2091ad48fae45862c6559488e5492a8cec0054b99cba11fa" },
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
        { path = "vendor-cargo-deps/animus/crates/animus-adapters/src/effect/result.rs", blake3 = "45238434efc4b7035133d4eabcdfdc5eb64308c96f1c49ffcab9f1f0120346b6" },
        { path = "vendor-cargo-deps/rats/source/ratcore/src/ratatui/coverage/baseline.rs", blake3 = "cd5972e5ba1d6feb7ce9f51b5d9fa9f05009dae30db90095b3617ac02dde693a" },
      ],
      rationale_code = "gap.no-nested-vendor-source-closure-assertion",
    },
    {
      requirement_id = "n8nclone.repository_validation.vendor_snapshot.scenario.nested-output-name",
      classification = "partial_evidence_only",
      evidence = [
        { path = "Cargo.toml", blake3 = "ae4d9a5744f9c07e2091ad48fae45862c6559488e5492a8cec0054b99cba11fa" },
        { path = "flake.nix", blake3 = "1b56551be094d259763966e52ab5a67e25f54f6a9846421b2c94a36ee2bfd7a8" },
        { path = "vendor-cargo-deps/animus/crates/animus-adapters/src/effect/result.rs", blake3 = "45238434efc4b7035133d4eabcdfdc5eb64308c96f1c49ffcab9f1f0120346b6" },
        { path = "vendor-cargo-deps/rats/source/ratcore/src/ratatui/coverage/baseline.rs", blake3 = "cd5972e5ba1d6feb7ce9f51b5d9fa9f05009dae30db90095b3617ac02dde693a" },
      ],
      rationale_code = "gap.no-git-and-nix-nested-path-assertion",
    },
  ],
  non_claims = [
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "partial evidence is not a near-complete correctness claim",
    "no exact evidence identified is bounded to the reviewed maintained roots and is not proof of global absence",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
  ],
} | Contracts.ReviewPolicy
