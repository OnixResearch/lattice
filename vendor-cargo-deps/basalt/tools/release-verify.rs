use std::process::ExitCode;

#[path = "../src/authority_identity.rs"]
mod authority_identity;
#[path = "../src/ast_grep_audit_identity.rs"]
mod ast_grep_audit_identity;
#[path = "release_verify/execute.rs"]
mod execute;
#[path = "release_verify/model.rs"]
mod model;
#[path = "release_verify/plan.rs"]
mod plan;
#[path = "release_verify/receipts.rs"]
mod receipts;
#[path = "../src/license_consumption_identity.rs"]
mod license_consumption_identity;
#[path = "../src/secretspec_issuer_identity.rs"]
mod secretspec_issuer_identity;
#[path = "../src/ucan_semantics_identity.rs"]
mod ucan_semantics_identity;
#[path = "release_verify/validators.rs"]
mod validators;

#[cfg(test)]
#[path = "release_verify/tests.rs"]
mod tests;

fn main() -> ExitCode {
    execute::run()
}
