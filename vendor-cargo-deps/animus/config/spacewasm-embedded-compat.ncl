# Typed non-default diagnostic compatibility profile; generic Wasm support is independent.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let Profile = {
  schema | NonEmpty,
  profile_id | NonEmpty,
  enabled | Bool,
  producer | { target | NonEmpty, rust_version | NonEmpty, opaque_lowering | Bool },
  guest | { source | NonEmpty, crates | Array NonEmpty, features | Array NonEmpty, exports | Array NonEmpty, imports | Array NonEmpty },
  runtime | { repository | NonEmpty, revision | NonEmpty, cohort_identity | NonEmpty, source_lock_identity | NonEmpty, mantle_bundle | NonEmpty },
  octet | { profile_id | NonEmpty, role | [| 'static_boundary |] },
  limits | { artifact_bytes | Positive, input_bytes | Positive, output_bytes | Positive, fuel | Positive },
  expected_outcomes | Array NonEmpty,
  excluded_capabilities | Array NonEmpty,
  non_claims | Array NonEmpty,
} in
{
  spacewasm_embedded_compat | Profile = {
    schema = "animus.spacewasm-embedded-compat.v1",
    profile_id = "spacewasm-e24-hostless-run-v1",
    enabled = true,
    producer = { target = "wasm32v1-none", rust_version = "1.91.1", opaque_lowering = false },
    guest = {
      source = "fixtures/spacewasm/guest.rs",
      crates = ["animus-core", "animus-adapters"],
      features = [],
      exports = ["run"],
      imports = [],
    },
    runtime = {
      repository = "https://github.com/nasa/spacewasm",
      revision = "e24cf09355a90497148eb5029fdb8e3400bd63e3",
      cohort_identity = "b3:7c537eef7bff66d0a90508715de08de79c9aaac23352e1007a6d47e7fb4674a2",
      source_lock_identity = "b3:4e6a07910125d4c921a62cec03d8bc9acb3bf341ebd1a11999df08d6c6c5c5f1",
      mantle_bundle = "mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3",
    },
    octet = { profile_id = "spacewasm-mvp", role = 'static_boundary },
    limits = { artifact_bytes = 1048576, input_bytes = 4096, output_bytes = 4096, fuel = 1000000 },
    expected_outcomes = ["returned", "expected-trap", "resource-exhausted"],
    excluded_capabilities = ["wasi", "filesystem", "process", "network", "credentials", "clock", "terminal", "provider", "tool", "persistence", "plugin"],
    non_claims = ["not-general-wasm-runtime-support", "not-plugin-activation", "not-runtime-correctness", "not-sandbox-completeness", "not-production-or-release-readiness"],
  },
}
