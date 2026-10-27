//! Read-only bounded SOPS executable guard.

use animus_std_secretspec::sops_guard::run_sops_guard;
use std::io::Write;

fn main() {
    let args = std::env::args_os().skip(1).collect();
    match run_sops_guard(args) {
        Ok(output) => {
            if std::io::stdout().write_all(&output).is_err() {
                std::process::exit(exit_failure());
            }
        }
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "{}", error.marker());
            std::process::exit(error.exit_code());
        }
    }
}

const fn exit_failure() -> i32 {
    const FAILURE_EXIT_CODE: i32 = 74;
    FAILURE_EXIT_CODE
}
