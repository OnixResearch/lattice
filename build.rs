/// Sources whose bytes identify the captured-source adapter.
const SOURCE_ADAPTER: &[&str] = &[
    "src/authoring/shell.rs",
    "src/authoring/captured.rs",
    "src/authoring/construction.rs",
    "src/authoring/construction/lowering.rs",
    "src/authoring/construction/materialized.rs",
    "src/authoring/construction/store.rs",
];

/// Sources whose bytes identify the construction runtime implementation.
const RUNTIME_IMPLEMENTATION: &[&str] = &[
    "src/authoring/construction.rs",
    "src/authoring/construction/lowering.rs",
    "src/authoring/construction/materialized.rs",
    "src/authoring/construction/store.rs",
    "src/engine/executor.rs",
    "src/engine/effect/execution.rs",
    "src/engine/effect/execution/resume.rs",
    "src/engine/effect/choregraph.rs",
    "src/engine/effect/standalone/core.rs",
    "src/node_kind_provider.rs",
    "src/workflow_format.rs",
];

fn main() -> std::io::Result<()> {
    for (binding, path) in [
        ("LATTICE_DEPENDENCY_LOCK_BLAKE3", "Cargo.lock"),
        ("LATTICE_WORKFLOW_LOWERER_BLAKE3", "src/workflow_format/choregraph.rs"),
    ] {
        println!("cargo:rerun-if-changed={path}");
        let bytes = std::fs::read(path)?;
        println!("cargo:rustc-env={binding}={}", blake3::hash(&bytes).to_hex());
    }
    let source = hash_sources(SOURCE_ADAPTER)?;
    println!("cargo:rustc-env=LATTICE_SOURCE_ADAPTER_BLAKE3={}", source.to_hex());
    let runtime = hash_sources(RUNTIME_IMPLEMENTATION)?;
    println!("cargo:rustc-env=LATTICE_RUNTIME_IMPLEMENTATION_BLAKE3={}", runtime.to_hex());
    Ok(())
}

/// Length-framed BLAKE3 over each path and its bytes, in the listed order.
fn hash_sources(paths: &[&str]) -> std::io::Result<blake3::Hash> {
    let mut hasher = blake3::Hasher::new();
    for path in paths {
        println!("cargo:rerun-if-changed={path}");
        let bytes = std::fs::read(path)?;
        hasher.update(&(path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    Ok(hasher.finalize())
}
