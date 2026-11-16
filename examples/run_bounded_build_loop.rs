//! Run one explicitly approved local bounded build loop.
#[path = "bounded-build-loop/mod.rs"]
mod bounded_build_loop;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[tokio::main]
async fn main() -> bounded_build_loop::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, root, mode] if command == "prepare" => {
            bounded_build_loop::prepare(Path::new(root), mode)?;
            println!("Prepared inert snapshot in {root}; review workflow.ncl, frozen.json, scripts and authority.json before granting local effects.");
        }
        [command, root, approval] if command == "run" && approval == "--approve-local-effects" => {
            let cancelled = Arc::new(AtomicBool::new(false));
            let signal = Arc::clone(&cancelled);
            let listener = tokio::spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() { signal.store(true, Ordering::Release); }
            });
            let result = bounded_build_loop::run(Path::new(root), cancelled).await;
            listener.abort();
            let report = result?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if report.outcome != "pass" { return Err(format!("bounded loop ended {}", report.outcome).into()); }
        }
        _ => return Err("usage: run_bounded_build_loop prepare NEW_DIR repair|pass|fail|cancel; run_bounded_build_loop run DIR --approve-local-effects".into()),
    }
    Ok(())
}
