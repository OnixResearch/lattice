let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 315,
  expected_missing_hash = "2b7906ee39473996244bcfbf795e9ee984704c8bbfe2db9017309647c4d8373c",
  reviewed = [
    {
      requirement_id = "n8nclone.repository_validation.clippy_scope",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "Cargo.toml", blake3 = "10e7b1591a94f5163b59f1d37963e0aa7de6b494a145bd2e0aea08c583d70137" },
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/fixtures/repository-validation/vendor-dependency/src/lib.rs", blake3 = "af923558b39ecfb1b88eeecf86211ccaf112680c98e1fd3ac2b5df94ef2fe456" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.package-clippy-no-deps-with-deny-fixture",
    },
    {
      requirement_id = "n8nclone.repository_validation.clippy_scope.scenario.vendor-lints",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "Cargo.toml", blake3 = "10e7b1591a94f5163b59f1d37963e0aa7de6b494a145bd2e0aea08c583d70137" },
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/fixtures/repository-validation/vendor-dependency/src/lib.rs", blake3 = "af923558b39ecfb1b88eeecf86211ccaf112680c98e1fd3ac2b5df94ef2fe456" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.vendor-clippy-deny-excluded",
    },
    {
      requirement_id = "n8nclone.repository_validation.flake_check",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "README.md", blake3 = "be65d085648a01ff0ff925d4b0ad1eae5c77ce3a0ae0a03c907ec4789d5adacc" },
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.check-only-signing-override-and-external-policy",
    },
    {
      requirement_id = "n8nclone.repository_validation.flake_check.scenario.missing-signing-key",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "README.md", blake3 = "be65d085648a01ff0ff925d4b0ad1eae5c77ce3a0ae0a03c907ec4789d5adacc" },
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.missing-key-overridden-for-full-check",
    },
    {
      requirement_id = "n8nclone.repository_validation.flake_check.scenario.normal-signing",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "README.md", blake3 = "be65d085648a01ff0ff925d4b0ad1eae5c77ce3a0ae0a03c907ec4789d5adacc" },
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.flake-signing-policy-absent",
    },
    {
      requirement_id = "n8nclone.repository_validation.format_scope",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/fixtures/repository-validation/vendor-dependency/src/coverage/result.rs", blake3 = "8b91c672aee50955e9bc64e92ea9718b924a62b696892e9e242c42520a810acf" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.package-format-excludes-vendor-fixture",
    },
    {
      requirement_id = "n8nclone.repository_validation.format_scope.scenario.vendor-format",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/fixtures/repository-validation/vendor-dependency/src/coverage/result.rs", blake3 = "8b91c672aee50955e9bc64e92ea9718b924a62b696892e9e242c42520a810acf" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.nonconforming-vendor-format-excluded",
    },
    {
      requirement_id = "n8nclone.repository_validation.test_source",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/fixtures/n8n/shell.json", blake3 = "f72d1067fa2a19983040d22454031baffd640068efcbb33617fc6853c02609d6" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.nix-nextest-compiles-tracked-test-fixtures",
    },
    {
      requirement_id = "n8nclone.repository_validation.test_source.scenario.json-fixture",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/fixtures/n8n/shell.json", blake3 = "f72d1067fa2a19983040d22454031baffd640068efcbb33617fc6853c02609d6" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.filtered-json-fixture-compiles",
    },
    {
      requirement_id = "n8nclone.repository_validation.vendor_snapshot",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "Cargo.toml", blake3 = "10e7b1591a94f5163b59f1d37963e0aa7de6b494a145bd2e0aea08c583d70137" },
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/fixtures/repository-validation/vendor-dependency/Cargo.toml", blake3 = "569835e588c912762f8c27c81a28801be2f9302ee9e03e525c093426e29ee1ae" },
        { path = "tests/fixtures/repository-validation/vendor-dependency/src/coverage/result.rs", blake3 = "8b91c672aee50955e9bc64e92ea9718b924a62b696892e9e242c42520a810acf" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.git-and-nix-nested-source-admission",
    },
    {
      requirement_id = "n8nclone.repository_validation.vendor_snapshot.scenario.nested-output-name",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "Cargo.toml", blake3 = "10e7b1591a94f5163b59f1d37963e0aa7de6b494a145bd2e0aea08c583d70137" },
        { path = "flake.nix", blake3 = "64a3b11138a008a3a088308f46c8aabcb695b01092372e86394e35612e4664fc" },
        { path = "tests/fixtures/repository-validation/vendor-dependency/Cargo.toml", blake3 = "569835e588c912762f8c27c81a28801be2f9302ee9e03e525c093426e29ee1ae" },
        { path = "tests/fixtures/repository-validation/vendor-dependency/src/coverage/result.rs", blake3 = "8b91c672aee50955e9bc64e92ea9718b924a62b696892e9e242c42520a810acf" },
        { path = "tests/test_repository_validation.rs", blake3 = "e99f6379ed945478fb024ca4b7b7a3d891a2497f8e02d85f75421a6e4bce5347" },
      ],
      rationale_code = "assertions.nested-output-name-admitted",
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
