const CATALOG_PATH: &str = "catalog.ncl";
const CASE_DIRECTORY: &str = "cases";
const MAX_CATALOG_BYTES: usize = 1_048_576;
const MAX_CASE_ASSET_BYTES: usize = 1_048_576;
const MAX_PLAN_REQUEST_BYTES: usize = 1_048_576;

pub fn load_catalog(root_path: &std::path::Path) -> Result<super::model::AdmittedCatalog, super::Error> {
    let root = crate::capability::open_root_path(root_path).map_err(|_| super::Error::FileAccess)?;
    let catalog_bytes = read_regular_file(&root, std::path::Path::new(CATALOG_PATH), MAX_CATALOG_BYTES)?;
    let source = std::str::from_utf8(&catalog_bytes).map_err(|_| super::Error::InvalidCatalog)?;
    let value = crate::nickel::evaluate_source(source).map_err(|_| super::Error::NickelEvaluation)?;
    let catalog: super::model::Catalog = serde_json::from_value(value).map_err(|_| super::Error::InvalidCatalog)?;
    let assets = capture_case_assets(&root)?;
    super::catalog::validate(catalog, &assets)
}

pub fn load_plan_request(path: &std::path::Path) -> Result<super::planning::PlanRequest, super::Error> {
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let name = path.file_name().ok_or(super::Error::FileAccess)?;
    let root = crate::capability::open_root_path(parent).map_err(|_| super::Error::FileAccess)?;
    let bytes = read_regular_file(&root, std::path::Path::new(name), MAX_PLAN_REQUEST_BYTES)?;
    let source = std::str::from_utf8(&bytes).map_err(|_| super::Error::InvalidCatalog)?;
    let value = crate::nickel::evaluate_source(source).map_err(|_| super::Error::NickelEvaluation)?;
    serde_json::from_value(value).map_err(|_| super::Error::InvalidCatalog)
}

fn capture_case_assets(root: &cap_std::fs::Dir) -> Result<Vec<super::model::AssetFact>, super::Error> {
    let case_dir_metadata = root.symlink_metadata(CASE_DIRECTORY).map_err(|_| super::Error::FileAccess)?;
    if !case_dir_metadata.is_dir() || case_dir_metadata.file_type().is_symlink() {
        return Err(super::Error::FileAccess);
    }
    let case_dir = root.open_dir(CASE_DIRECTORY).map_err(|_| super::Error::FileAccess)?;
    let mut assets = Vec::with_capacity(super::model::MAX_ASSETS);
    for entry in case_dir.entries().map_err(|_| super::Error::FileAccess)? {
        let entry = entry.map_err(|_| super::Error::FileAccess)?;
        if assets.len() >= super::model::MAX_ASSETS {
            return Err(super::Error::AssetParity);
        }
        let file_type = entry.file_type().map_err(|_| super::Error::FileAccess)?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(super::Error::FileAccess);
        }
        let name = entry.file_name().into_string().map_err(|_| super::Error::FileAccess)?;
        let relative_path = std::path::Path::new(CASE_DIRECTORY).join(&name);
        if !super::identity::valid_safe_path(&relative_path.to_string_lossy()) {
            return Err(super::Error::FileAccess);
        }
        let bytes = read_regular_file(root, &relative_path, MAX_CASE_ASSET_BYTES)?;
        let byte_count = u64::try_from(bytes.len()).map_err(|_| super::Error::AssetParity)?;
        assets.push(super::model::AssetFact {
            relative_path: relative_path.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
            content_identity: super::identity::hash_bytes("lattice.agent-evaluation.case-asset.v1", &bytes),
            byte_count,
            regular_file: true,
            symlink: false,
        });
    }
    assets.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    debug_assert!(assets.len() <= super::model::MAX_ASSETS);
    debug_assert!(assets.iter().all(|asset| asset.regular_file));
    Ok(assets)
}

fn read_regular_file(
    root: &cap_std::fs::Dir,
    path: &std::path::Path,
    max_bytes: usize,
) -> Result<Vec<u8>, super::Error> {
    let metadata = root.symlink_metadata(path).map_err(|_| super::Error::FileAccess)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(super::Error::FileAccess);
    }
    let file = crate::capability::open_relative_read(root, path).map_err(|_| super::Error::FileAccess)?;
    let bytes = crate::capability::read_bounded(file, max_bytes).map_err(|_| super::Error::FileAccess)?;
    if bytes.len() > max_bytes || bytes.is_empty() {
        return Err(super::Error::FileAccess);
    }
    Ok(bytes)
}
