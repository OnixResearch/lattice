use std::time::Duration;

pub(super) const TARGET_ENV: &str = "CARGO_TARGET_DIR";
pub(super) const WRAPPER_ENV: &str = "RUSTC_WRAPPER";
pub(super) const GIT_FETCH_ENV: &str = "CARGO_NET_GIT_FETCH_WITH_CLI";
pub(super) const RECEIPT_DIR_ENV: &str = "RELEASE_VERIFY_RECEIPT_DIR";
pub(super) const POLICY_FRESH_ENV: &str = "RELEASE_VERIFY_POLICY_FRESH";
pub(super) const AUTHORITY_FRESH_ENV: &str = "RELEASE_VERIFY_AUTHORITY_FRESH";
pub(super) const SEMANTICS_FRESH_ENV: &str = "RELEASE_VERIFY_SEMANTICS_FRESH";
pub(super) const LICENSE_FRESH_ENV: &str = "RELEASE_VERIFY_LICENSE_FRESH";
pub(super) const ISSUER_FRESH_ENV: &str = "RELEASE_VERIFY_ISSUER_FRESH";
pub(super) const AST_GREP_FRESH_ENV: &str = "RELEASE_VERIFY_AST_GREP_FRESH";
pub(super) const CARGO_STEPS_ENV: &str = "RELEASE_VERIFY_CARGO_STEPS";
pub(super) const VENDORED_UCAN_TEST_BINARY: &str = "/tmp/basalt-vendored-ucan-tests";
pub(super) const VENDORED_UCAN_CHECK_BINARY: &str = "/tmp/basalt-vendored-ucan-check";
pub(super) const DEFAULT_TARGET_DIR: &str = "/tmp/basalt-release-target";
pub(super) const PACKAGE_TARGET_DIR: &str = "/tmp/basalt-release-package-target";
pub(super) const CLANKERS_TARGET_DIR: &str = "/tmp/basalt-clankers-consumer-target";
pub(super) const CLANKERS_CONSUMER_MANIFEST: &str =
    "../clankers/examples/basalt-consumer-fixture/Cargo.toml";
pub(super) const DEFAULT_RECEIPT_DIR: &str = "target/release-verify";
pub(super) const OUTPUT_TAIL_LIMIT: usize = 4_096;
pub(super) const REQUIRED_LICENSE: &str = "AGPL-3.0-or-later";
pub(super) const PRIVATE_PUBLISH_SETTING: &str = "publish = false";
pub(super) const POLICY_FRESH_STEP_NAME: &str = "policy freshness";
pub(super) const AUTHORITY_FRESH_STEP_NAME: &str = "authority-core fixture freshness";
pub(super) const AUTHORITY_REPLAY_STEP_NAME: &str = "authority-core fixture tests";
pub(super) const SEMANTICS_FRESH_STEP_NAME: &str = "UCAN semantics fixture freshness";
pub(super) const SEMANTICS_REPLAY_STEP_NAME: &str = "UCAN semantics contract tests";
pub(super) const LICENSE_FRESH_STEP_NAME: &str = "license-consumption fixture freshness";
pub(super) const LICENSE_REPLAY_STEP_NAME: &str = "license-consumption contract tests";
pub(super) const AST_GREP_FRESH_STEP_NAME: &str = "ast-grep policy-boundary audit";
pub(super) const ISSUER_FRESH_STEP_NAME: &str = "issuer-key fixture freshness";
pub(super) const ISSUER_REPLAY_STEP_NAME: &str = "issuer-key resolution contract tests";

pub(super) const REQUIRED_PACKAGE_FILES: &[&str] = &[
    "Cargo.lock",
    "Cargo.toml",
    "README.md",
    "contracts/generated/policy.json",
    "contracts/policy.ncl",
    "contracts/authority-core/contracts.ncl",
    "contracts/authority-core/fixtures/generated/negative/authority-overclaim.json",
    "contracts/authority-core/fixtures/generated/positive/direct-match.json",
    "contracts/authority-core/fixtures/negative/authority-overclaim.ncl",
    "contracts/authority-core/fixtures/positive/direct-match.ncl",
    "contracts/ucan-semantics/contracts.ncl",
    "contracts/ucan-semantics/fixtures/generated/negative/over-delegation.json",
    "contracts/ucan-semantics/fixtures/generated/positive/allow.json",
    "contracts/ucan-semantics/fixtures/negative/over-delegation.ncl",
    "contracts/ucan-semantics/fixtures/positive/allow.ncl",
    "contracts/license-consumption/contracts.ncl",
    "contracts/license-consumption/fixtures/base.ncl",
    "contracts/license-consumption/fixtures/generated/positive/acknowledged-agpl.json",
    "contracts/license-consumption/fixtures/generated/negative/missing-review.json",
    "contracts/license-consumption/fixtures/generated/negative/stale-license-metadata.json",
    "contracts/license-consumption/fixtures/generated/negative/weakened-non-claims.json",
    "contracts/license-consumption/fixtures/generated/negative/wrong-artifact-identity.json",
    "contracts/license-consumption/fixtures/generated/negative/wrong-license-expression.json",
    "contracts/nostr-identity/contracts.ncl",
    "contracts/nostr-identity/policy.ncl",
    "contracts/nostr-identity/generated/identities.json",
    "contracts/nostr-identity/generated/policy.json",
    "contracts/nostr-identity/fixtures/negative/ambiguous-key.json",
    "contracts/nostr-identity/fixtures/negative/secret-field.json",
    "contracts/nostr-identity/fixtures/negative/weakened-nonclaims.json",
    "docs/authority-core.md",
    "docs/ast-grep-policy-audits.md",
    "docs/cap-std-cli-policy-boundary.md",
    "docs/license-consumption.md",
    "docs/nostr-identity-bindings.md",
    "docs/policy-authoring.md",
    "docs/policy-identity.md",
    "docs/ucan-enforcement.md",
    "docs/ucan-semantics-contract.md",
    "docs/secretspec-issuer.md",
    "docs/vendored-ucan.md",
    "src/authority_core.rs",
    "src/authority_identity.rs",
    "src/ast_grep_audit.rs",
    "src/ast_grep_audit_identity.rs",
    "src/cli_fs.rs",
    "src/lib.rs",
    "src/license_consumption.rs",
    "src/license_consumption_identity.rs",
    "src/main.rs",
    "src/nostr_identity.rs",
    "src/nostr_identity/authentication.rs",
    "src/nostr_identity/authentication_model.rs",
    "src/nostr_identity/composition.rs",
    "src/nostr_identity/identity.rs",
    "src/nostr_identity/policy.rs",
    "src/nostr_identity/policy_model.rs",
    "src/nostr_identity/tests.rs",
    "src/policy_projection.rs",
    "src/policy_identity.rs",
    "src/secretspec_issuer.rs",
    "src/secretspec_issuer_identity.rs",
    "src/secretspec_issuer/shell.rs",
    "src/secretspec_issuer/tests.rs",
    "src/ucan_enforcement.rs",
    "src/ucan_enforcement/core.rs",
    "src/ucan_enforcement/error.rs",
    "src/ucan_enforcement/receipt.rs",
    "src/ucan_enforcement/shell.rs",
    "src/ucan_enforcement/types.rs",
    "src/ucan_enforcement/tests.rs",
    "src/ucan_enforcement/types.rs",
    "src/ucan_semantics.rs",
    "src/ucan_semantics_identity.rs",
    "tests/authority_core.rs",
    "tests/cli_feature.rs",
    "tests/consumer_boundary.rs",
    "tests/license_consumption.rs",
    "tests/nostr_identity.rs",
    "tests/nostr_verified_credential.rs",
    "tests/policy_artifact.rs",
    "tests/policy_projection.rs",
    "tests/policy_identity.rs",
    "tests/production_ucan_token_flow.rs",
    "tests/property_harness.rs",
    "tests/public_api.rs",
    "tests/secretspec_issuer.rs",
    "tests/steel_adapter_fixture.rs",
    "tests/ucan_enforcement.rs",
    "tests/ucan_enforcement/negative/mod.rs",
    "tests/ucan_enforcement/onixos_targets.rs",
    "tests/ucan_enforcement/order.rs",
    "tests/ucan_enforcement/positive.rs",
    "tests/ucan_enforcement/receipt.rs",
    "tests/ucan_enforcement/support/mod.rs",
    "tests/ucan_semantics_contract.rs",
    "tools/release-verify.rs",
    "tools/release_verify/execute.rs",
    "tools/release_verify/model.rs",
    "tools/release_verify/plan.rs",
    "tools/release_verify/receipts.rs",
    "tools/release_verify/tests.rs",
    "tools/release_verify/validators.rs",
    "tools/ast-grep/rules/no-direct-enforcement-bypass.yml",
    "tools/ast-grep/sgconfig.yml",
    "tools/ast-grep/fixtures/positive/direct-enforcement-helper.rs",
    "tools/ast-grep/fixtures/negative/direct-enforcement-bypass.rs",
    "tools/sync-vendored-ucan.rs",
    "tools/sync_vendored_ucan/filesystem.rs",
    "tools/sync_vendored_ucan/model.rs",
    "tools/sync_vendored_ucan/process.rs",
    "tools/sync_vendored_ucan/shell.rs",
    "tools/sync_vendored_ucan/tests.rs",
    "vendor/ucan-revision.txt",
    "vendor/ucan.snapshot",
];

pub(super) const REQUIRED_PRIVATE_SOURCE_FILES: &[&str] = &[
    "vendor/ucan-revision.txt",
    "vendor/ucan.snapshot",
    "vendor/ucan/Cargo.toml",
    "vendor/ucan/LICENSES/AGPL-3.0-or-later.txt",
    "vendor/ucan/LICENSES/MPL-2.0.txt",
    "vendor/ucan/crates/ucan-core/LICENSE",
    "vendor/ucan/crates/ucan-nostr/LICENSE",
    "vendor/ucan/vendor/verified-logic/LICENSE",
    "contracts/secretspec-issuer/fixtures/positive/issuer-key-declaration.json",
    "contracts/secretspec-issuer/fixtures/negative/provider-failure.json",
];

pub(super) struct Step {
    pub(super) name: &'static str,
    pub(super) program: &'static str,
    pub(super) args: &'static [&'static str],
    pub(super) target_dir: &'static str,
    pub(super) cargo_env: bool,
    pub(super) kind: StepKind,
}

#[derive(Clone, Copy)]
pub(super) enum StepKind {
    Normal,
    LicenseMetadata,
    PolicyFreshness,
    AuthorityFixtureFreshness,
    SemanticsFixtureFreshness,
    LicenseFixtureFreshness,
    IssuerFixtureFreshness,
    AstGrepAuditFreshness,
    PackageContents,
    ClankersConsumerFixture,
}

pub(super) struct StepReceipt {
    pub(super) name: &'static str,
    pub(super) command: String,
    pub(super) status: Option<i32>,
    pub(super) success: bool,
    pub(super) elapsed: Duration,
    pub(super) stdout_tail: String,
    pub(super) stderr_tail: String,
}
