#![cfg(feature = "oidc-exchange")]

use basalt::oidc_exchange::{
    BOUNDED_HTTP_REVISION, BOUNDED_HTTP_RID, JSONWEBTOKEN_REVISION, JSONWEBTOKEN_VERSION,
};

const BOUNDED_HTTP_URL: &str = "https://seed.radicle.garden/z4ARxEt93T6gJTrTZpUyKHMAemPiv.git";
const JSONWEBTOKEN_CHECKSUM: &str =
    "881733cbc631fc9e472e24447ce32a64bedf2da498d6d8570b08edc87de71f65";

#[test]
fn cargo_and_runtime_bind_the_same_reviewed_sources() {
    let manifest = include_str!("../Cargo.toml");
    let lock = include_str!("../Cargo.lock");
    let source = include_str!("../contracts/oidc-exchange/source-boundary.ncl");
    let source_contracts = include_str!("../contracts/oidc-exchange/source-contracts.ncl");
    for expected in [
        BOUNDED_HTTP_RID,
        BOUNDED_HTTP_URL,
        BOUNDED_HTTP_REVISION,
        JSONWEBTOKEN_VERSION,
        JSONWEBTOKEN_REVISION,
        JSONWEBTOKEN_CHECKSUM,
    ] {
        assert!(
            manifest.contains(expected)
                || lock.contains(expected)
                || source.contains(expected)
                || source_contracts.contains(expected),
            "missing reviewed source identity {expected}"
        );
    }
    assert!(manifest.contains("default-features = false, features = [\"rust_crypto\"]"));
    assert!(lock.contains(&format!(
        "rev={BOUNDED_HTTP_REVISION}#{BOUNDED_HTTP_REVISION}"
    )));
}

#[test]
fn bounded_http_has_no_floating_or_local_fallback() {
    let manifest = include_str!("../Cargo.toml");
    assert!(!manifest.contains("github.com/OnixResearch/bounded-http"));
    assert!(!manifest.contains("path = \"../bounded-http\""));
    assert!(!manifest.contains("branch ="));
}
