//! Private one-shot `SecretSpec` resolver worker.

use animus_std_secretspec::worker::{WORKER_EXIT_INTERNAL, WorkerFailure, run_worker_bytes};
use std::io::{Read, Write};

const REQUEST_LIMIT_BYTES: u64 = 65_537;

fn main() {
    let exit_code = match run() {
        Ok(()) => return,
        Err(code) => code,
    };
    std::process::exit(exit_code);
}

fn run() -> Result<(), i32> {
    let mut input = Vec::new();
    std::io::stdin()
        .take(REQUEST_LIMIT_BYTES)
        .read_to_end(&mut input)
        .map_err(|_| WORKER_EXIT_INTERNAL)?;
    let frame = run_worker_bytes(&input).map_err(WorkerFailure::exit_code)?;
    std::io::stdout()
        .write_all(frame.as_slice())
        .map_err(|_| WORKER_EXIT_INTERNAL)
}
