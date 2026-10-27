//! Export source-bound inspection metadata; this output is not authority.
use std::io::{self, Write};

use basalt::nostr_identity::{
    NOSTR_AUTHENTICATION_RECEIPT_SCHEMA, NOSTR_COMPOSITION_RECEIPT_SCHEMA,
    NOSTR_IDENTITY_POLICY_SCHEMA, UCAN_NOSTR_CREDENTIAL_CONTRACT, UCAN_NOSTR_CREDENTIAL_REVISION,
    UCAN_SOURCE_REVISION, nostr_identity_implementation_identity, nostr_identity_schema_identity,
    parse_nostr_identity_policy,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let policy = parse_nostr_identity_policy(include_str!(
        "../contracts/nostr-identity/generated/policy.json"
    ))?;
    let identities = serde_json::json!({
        "schema": "basalt.nostr-identity-producer-identities.v1",
        "ucan_source_revision": UCAN_SOURCE_REVISION,
        "ucan_nostr_credential_revision": UCAN_NOSTR_CREDENTIAL_REVISION,
        "credential_contract": UCAN_NOSTR_CREDENTIAL_CONTRACT,
        "policy_schema": NOSTR_IDENTITY_POLICY_SCHEMA,
        "authentication_receipt_schema": NOSTR_AUTHENTICATION_RECEIPT_SCHEMA,
        "composition_receipt_schema": NOSTR_COMPOSITION_RECEIPT_SCHEMA,
        "producer_schema_blake3": nostr_identity_schema_identity().hex,
        "producer_implementation_blake3": nostr_identity_implementation_identity().hex,
        "policy_blake3": policy.policy_identity().hex,
    });
    let mut output = io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, &identities)?;
    writeln!(output)?;
    output.flush()?;
    Ok(())
}
