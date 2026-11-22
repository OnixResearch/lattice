//! Shared fixtures for runbook-catalog tests: the retained seed catalog, a local `file://`
//! git repository built from it, and pack-manifest regeneration for negative cases.
#![allow(dead_code, reason = "each test crate uses a different subset of the shared fixtures")]

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

pub const SEED_CATALOG: &str = "tests/fixtures/runbook-catalog";
pub const PACK_SUBDIRECTORY: &str = "packs/seed-runbooks";
pub const LICENSE: &str = "AGPL-3.0-or-later";
pub const HELLO: &str = "hello-runbook";
pub const STATUS: &str = "status-runbook";
pub const BUNDLE_ROOT: &str = "workflow-templates";

pub fn seed_catalog() -> PathBuf {
    std::fs::canonicalize(SEED_CATALOG).expect("seed catalog fixture")
}

pub fn seed_pack() -> PathBuf {
    seed_catalog().join(PACK_SUBDIRECTORY)
}

pub fn copy_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("copy destination");
    for entry in std::fs::read_dir(source).expect("copy source") {
        let entry = entry.expect("copy entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("copy entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy file");
        }
    }
}

/// A private copy of the seed pack at `<root>/pack`.
pub fn copy_seed_pack(root: &Path) -> PathBuf {
    let pack = root.join("pack");
    copy_tree(&seed_pack(), &pack);
    pack
}

pub fn git(directory: &Path, arguments: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=Catalog Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(arguments)
        .current_dir(directory)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("HOME", directory)
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).expect("git output").trim().to_string()
}

/// Commit the seed catalog tree into a new local repository and return it with the commit.
pub fn catalog_repository(root: &Path) -> (PathBuf, String) {
    catalog_repository_with(root, &Repository::default())
}

/// Options for a fixture catalog repository.
#[derive(Default)]
pub struct Repository<'options> {
    /// `sha256` for a SHA-256 repository; SHA-1 otherwise.
    pub object_format: Option<&'options str>,
    /// Let `upload-pack` honor `--filter`, as hosted servers do.
    pub allow_filter: bool,
    /// Extra committed files: repository-relative path and byte length of incompressible bytes.
    pub large_files: &'options [(&'options str, usize)],
}

pub fn catalog_repository_with(root: &Path, options: &Repository<'_>) -> (PathBuf, String) {
    let repository = root.join("catalog-repository");
    copy_tree(&seed_catalog(), &repository);
    let format = format!("--object-format={}", options.object_format.unwrap_or("sha1"));
    git(&repository, &["init", "--quiet", &format]);
    if options.allow_filter {
        git(&repository, &["config", "uploadpack.allowFilter", "true"]);
    }
    for (path, length) in options.large_files {
        let target = repository.join(path);
        std::fs::create_dir_all(target.parent().expect("large file parent")).expect("large file directory");
        let mut bytes = vec![0u8; *length];
        blake3::Hasher::new().update(path.as_bytes()).finalize_xof().fill(&mut bytes);
        std::fs::write(&target, bytes).expect("large file");
    }
    git(&repository, &["add", "--all"]);
    git(&repository, &["commit", "--quiet", "--message", "Publish the seed runbook catalog"]);
    let commit = git(&repository, &["rev-parse", "HEAD"]);
    (repository, commit)
}

pub fn file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

/// Identities the workflow-template loader computes for each valid bundle in a pack.
pub fn loader_identities(pack: &Path) -> BTreeMap<String, String> {
    let catalog = lattice::workflow_template::load_catalog(&pack.join(BUNDLE_ROOT)).expect("pack bundle root");
    catalog
        .bundles
        .iter()
        .map(|bundle| (bundle.manifest().id.clone(), format!("b3:{}", bundle.identity().as_str())))
        .collect()
}

/// Rewrite the pack manifest so every file under `workflow-templates/` is pinned again.
///
/// `bundles` lists each declared bundle id with its pinned identity; absent identities use the
/// loader's observation, or a fixed placeholder when the loader rejects the bundle.
pub fn regenerate_manifest(pack: &Path, bundles: &[(&str, Option<&str>)]) {
    let observed = loader_identities(pack);
    let mut files = Vec::new();
    collect_files(&pack.join(BUNDLE_ROOT), BUNDLE_ROOT, &mut files);
    files.sort();
    let contents = files
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let bytes = std::fs::read(pack.join(path)).expect("bundle file");
            format!(
                "{{ id = \"content-{index}\", role = \"workflow_bundle\", path = \"{path}\", content_identity = \"{}\", byte_length = {}, media_class = \"text\" }}",
                lattice::operation_pack::content_identity(&bytes),
                bytes.len()
            )
        })
        .collect::<Vec<_>>()
        .join(",\n    ");
    let declarations = bundles
        .iter()
        .map(|(id, identity)| {
            let pinned =
                identity.map(str::to_string).or_else(|| observed.get(*id).cloned()).unwrap_or_else(placeholder);
            format!("{{ id = \"{id}\", bundle_identity = \"{pinned}\" }}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(pack.join("manifest.ncl"), manifest_source(&contents, &declarations)).expect("pack manifest");
}

pub fn placeholder() -> String {
    format!("b3:{}", "0".repeat(64))
}

pub fn manifest_source(contents: &str, declarations: &str) -> String {
    format!(
        r#"{{
  schema = "lattice.operation-pack-manifest.v1",
  pack_id = "lattice.seed-runbooks",
  description = "Regenerated runbook catalog test pack",
  contents = [
    {contents}
  ],
  workflow_bundles = [{declarations}],
  operations = [],
  runtimes = [],
  analyzers = [],
  skills = [],
  dependencies = [],
  platforms = ["aarch64-darwin", "aarch64-linux", "x86_64-darwin", "x86_64-linux"],
  admission = {{
    identity = "{}",
    required_evidence = [],
    allowed_licenses = ["{LICENSE}"],
    allowed_capabilities = [],
  }},
  release = {{ display_version = "0.1.0", channel = "beta", deprecated = false }},
  licenses = ["{LICENSE}"],
  required_non_claims = [
    "pack identity proves exact admitted content only",
    "pack admission does not prove maintainer or source trust",
    "bounded process execution does not prove sandbox completeness",
    "pack evidence does not prove production or release readiness",
  ],
}}
"#,
        placeholder()
    )
}

fn collect_files(directory: &Path, relative: &str, files: &mut Vec<String>) {
    for entry in std::fs::read_dir(directory).expect("bundle directory") {
        let entry = entry.expect("bundle entry");
        let name = entry.file_name().into_string().expect("UTF-8 bundle path");
        let path = format!("{relative}/{name}");
        if entry.file_type().expect("bundle entry type").is_dir() {
            collect_files(&entry.path(), &path, files);
        } else {
            files.push(path);
        }
    }
}

/// Lock the pack exactly as `lattice pack lock --confirm-trust-local` does, and admit it.
pub fn admit(pack: &Path) -> lattice::operation_pack::AdmittedPack {
    let candidate = lattice::operation_pack::shell::load(pack).expect("catalog pack loads");
    let facts = lattice::operation_pack::AdmissionFacts {
        trusted_pack_identity: candidate.pack_identity.clone(),
        platform: lattice::operation_pack::shell::platform_label(),
        dependency_identities: BTreeMap::new(),
        allowed_licenses: vec![LICENSE.to_string()],
        allowed_capabilities: Vec::new(),
        evidence: BTreeMap::new(),
    };
    let lock = lattice::operation_pack::create_lock(&candidate, &facts).expect("catalog pack locks");
    lattice::operation_pack::admit(&candidate, &lock, &facts).expect("catalog pack admits")
}

/// A config directory with an empty `workflow-templates/` catalog.
pub fn config_root(root: &Path) -> (PathBuf, PathBuf) {
    let config_directory = root.join("config");
    let catalog = config_directory.join(BUNDLE_ROOT);
    std::fs::create_dir_all(&catalog).expect("catalog directory");
    let data = config_directory.join("data");
    let config = config_directory.join("config.ncl");
    std::fs::write(
        &config,
        format!(
            "{{ data_dir = {:?}, default_timeout_secs = 30, shell = \"sh\", max_parallelism = 1 }}",
            data.to_string_lossy()
        ),
    )
    .expect("config source");
    (config, catalog)
}
