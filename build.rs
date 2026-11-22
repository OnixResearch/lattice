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

/// Files and source trees whose bytes identify checkpoint publication, admission, restore and
/// the continued run (Lattice #12). Directories are walked recursively in sorted path order.
const CHECKPOINT_IMPLEMENTATION: &[&str] = &[
    "Cargo.lock",
    "src/checkpoint",
    "src/engine/executor/choice.rs",
    "src/headless/shell/checkpoint.rs",
    "src/headless/shell/record.rs",
    "src/workflow/gate/shell/checkpoint.rs",
    "src/workflow_exchange",
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
    let checkpoint = hash_sources(&expand_sources(CHECKPOINT_IMPLEMENTATION)?)?;
    println!("cargo:rustc-env=LATTICE_CHECKPOINT_IMPLEMENTATION_BLAKE3={}", checkpoint.to_hex());
    Ok(())
}

/// Every file under the listed paths, directories expanded recursively, in sorted path order.
fn expand_sources(paths: &[&str]) -> std::io::Result<Vec<String>> {
    let nested = paths
        .iter()
        .map(|path| expand_path(std::path::Path::new(path)))
        .collect::<std::io::Result<Vec<_>>>()?;
    let mut files = nested.concat();
    files.sort();
    Ok(files)
}

fn expand_path(path: &std::path::Path) -> std::io::Result<Vec<String>> {
    if !path.is_dir() {
        let name = path.to_str().ok_or_else(|| std::io::Error::other("non-UTF-8 source path"))?;
        return Ok(vec![name.replace('\\', "/")]);
    }
    println!("cargo:rerun-if-changed={}", path.display());
    let entries = std::fs::read_dir(path)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    let nested = entries.iter().map(|entry| expand_path(entry)).collect::<std::io::Result<Vec<_>>>()?;
    Ok(nested.concat())
}

/// Length-framed BLAKE3 over each path and its bytes, in the listed order.
fn hash_sources(paths: &[impl AsRef<str>]) -> std::io::Result<blake3::Hash> {
    let mut hasher = blake3::Hasher::new();
    for path in paths.iter().map(AsRef::as_ref) {
        println!("cargo:rerun-if-changed={path}");
        let bytes = std::fs::read(path)?;
        hasher.update(&(path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    Ok(hasher.finalize())
}
