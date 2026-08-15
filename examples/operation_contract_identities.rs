use std::path::Path;

const DEFAULT_CATALOG_PATH: &str = "builtins/operations/catalog.ncl";
const DEFAULT_EXECUTOR_PATH: &str = "builtins/operations/cairn-read-only-fixture";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let catalog_path = std::env::args().nth(1).unwrap_or_else(|| DEFAULT_CATALOG_PATH.to_string());
    let executor_path = std::env::args().nth(2).unwrap_or_else(|| DEFAULT_EXECUTOR_PATH.to_string());
    let catalog = lattice::operation::format::decode_catalog(Path::new(&catalog_path))?;
    let revisions = catalog
        .types
        .iter()
        .map(|operation_type| {
            Ok(serde_json::json!({
                "id": operation_type.id,
                "declared_revision": operation_type.revision,
                "computed_revision": lattice::operation::identity::type_revision(operation_type)?,
            }))
        })
        .collect::<Result<Vec<_>, lattice::operation::Error>>()?;
    let executor_path = std::fs::canonicalize(executor_path)?;
    let output = serde_json::json!({
        "executor_identity": lattice::operation::identity::executor_file(&executor_path)?,
        "types": revisions,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
