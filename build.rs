fn main() -> std::io::Result<()> {
    for (binding, path) in [
        ("LATTICE_DEPENDENCY_LOCK_BLAKE3", "Cargo.lock"),
        ("LATTICE_WORKFLOW_LOWERER_BLAKE3", "src/workflow_format/choregraph.rs"),
        ("LATTICE_SOURCE_ADAPTER_BLAKE3", "src/authoring/shell.rs"),
    ] {
        println!("cargo:rerun-if-changed={path}");
        let bytes = std::fs::read(path)?;
        println!("cargo:rustc-env={binding}={}", blake3::hash(&bytes).to_hex());
    }
    Ok(())
}
