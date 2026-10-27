//! Test-only fake SOPS process for bounded provider fixtures.

use std::io::Write;
use std::time::Duration;

const MODE_ENV: &str = "ANIMUS_SOPS_FIXTURE_MODE";
const NONZERO_EXIT_CODE: i32 = 42;
const MISSING_CREDENTIAL_EXIT_CODE: i32 = 43;
const REQUIRED_CREDENTIAL_ENV: &str = "SOPS_AGE_KEY";
const SLEEP_SECONDS: u64 = 60;
const OVERSIZED_VALUE_BYTES: usize = 131_072;

fn main() {
    if std::env::var_os(REQUIRED_CREDENTIAL_ENV).is_none() {
        std::process::exit(MISSING_CREDENTIAL_EXIT_CODE);
    }
    match std::env::var(MODE_ENV).as_deref() {
        Ok("success") => print_json_value("fixture-sops-secret"),
        Ok("malformed") => print!("not-json"),
        Ok("timeout") => std::thread::sleep(Duration::from_secs(SLEEP_SECONDS)),
        Ok("overflow") => print_json_value(&"x".repeat(OVERSIZED_VALUE_BYTES)),
        _ => std::process::exit(NONZERO_EXIT_CODE),
    }
}

fn print_json_value(value: &str) {
    let output = serde_json::json!({ "SOPS_API_KEY": value }).to_string();
    if std::io::stdout().write_all(output.as_bytes()).is_err() {
        std::process::exit(NONZERO_EXIT_CODE);
    }
}
