use std::path::Path;

const DEFAULT_CATALOG_PATH: &str = "builtins/operations/catalog.ncl";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap_or_else(|| DEFAULT_CATALOG_PATH.to_string());
    let catalog = lattice::operation::format::decode_catalog(Path::new(&path))?;
    for operation_type in catalog.types {
        let revision = lattice::operation::identity::type_revision(&operation_type)?;
        println!("{} {revision}", operation_type.id);
    }
    Ok(())
}
