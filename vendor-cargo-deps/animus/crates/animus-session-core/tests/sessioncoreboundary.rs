//! Manifest boundary checks for the pure session core.

const ALLOWED_PRODUCTION_DEPENDENCY_COUNT: usize = 2;
const ALLOWED_PRODUCTION_DEPENDENCIES: [&str; ALLOWED_PRODUCTION_DEPENDENCY_COUNT] =
    ["animus-adapters", "blake3"];
const EXPECTED_PRODUCTION_DEPENDENCY_COUNT: usize = ALLOWED_PRODUCTION_DEPENDENCIES.len();
const DEPENDENCIES_SECTION: &str = "dependencies";
const BUILD_DEPENDENCIES_SECTION: &str = "build-dependencies";
const TARGET_SECTION: &str = "target";

#[derive(Debug, Eq, PartialEq)]
enum BoundaryError {
    MalformedManifest,
    MissingDependencies,
    ForbiddenProductionDependency(String),
    BuildDependency(String),
}

#[derive(Debug, Eq, PartialEq)]
struct BoundaryReport {
    production_dependencies: Vec<String>,
}

fn validate_manifest_text(source: &str) -> Result<BoundaryReport, BoundaryError> {
    let manifest = source
        .parse::<toml::Value>()
        .map_err(|_| BoundaryError::MalformedManifest)?;
    let root = manifest
        .as_table()
        .ok_or(BoundaryError::MalformedManifest)?;
    let dependencies = root
        .get(DEPENDENCIES_SECTION)
        .and_then(toml::Value::as_table)
        .ok_or(BoundaryError::MissingDependencies)?;
    if dependencies.is_empty() {
        return Err(BoundaryError::MissingDependencies);
    }

    let production_dependencies = validate_dependencies(dependencies)?;
    validate_build_dependencies(root.get(BUILD_DEPENDENCIES_SECTION))?;
    validate_target_dependencies(root.get(TARGET_SECTION))?;

    debug_assert!(!production_dependencies.is_empty());
    debug_assert!(production_dependencies.len() <= ALLOWED_PRODUCTION_DEPENDENCY_COUNT);
    Ok(BoundaryReport {
        production_dependencies,
    })
}

fn validate_dependencies(
    dependencies: &toml::map::Map<String, toml::Value>,
) -> Result<Vec<String>, BoundaryError> {
    for dependency in dependencies.keys() {
        if !ALLOWED_PRODUCTION_DEPENDENCIES.contains(&dependency.as_str()) {
            return Err(BoundaryError::ForbiddenProductionDependency(
                dependency.clone(),
            ));
        }
    }
    debug_assert!(dependencies.len() <= ALLOWED_PRODUCTION_DEPENDENCY_COUNT);
    let mut admitted = dependencies.keys().cloned().collect::<Vec<_>>();
    admitted.sort();
    Ok(admitted)
}

fn validate_build_dependencies(value: Option<&toml::Value>) -> Result<(), BoundaryError> {
    let Some(build_dependencies) = value else {
        return Ok(());
    };
    let table = build_dependencies
        .as_table()
        .ok_or(BoundaryError::MalformedManifest)?;
    if let Some(dependency) = table.keys().next() {
        return Err(BoundaryError::BuildDependency(dependency.clone()));
    }
    Ok(())
}

fn validate_target_dependencies(value: Option<&toml::Value>) -> Result<(), BoundaryError> {
    let Some(targets) = value else {
        return Ok(());
    };
    let targets = targets.as_table().ok_or(BoundaryError::MalformedManifest)?;
    for target in targets.values() {
        let table = target.as_table().ok_or(BoundaryError::MalformedManifest)?;
        if let Some(dependencies) = table.get(DEPENDENCIES_SECTION) {
            let dependencies = dependencies
                .as_table()
                .ok_or(BoundaryError::MalformedManifest)?;
            validate_dependencies(dependencies)?;
        }
        validate_build_dependencies(table.get(BUILD_DEPENDENCIES_SECTION))?;
    }
    Ok(())
}

fn fixture_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn read_manifest(relative: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(fixture_path(relative))
}

#[test]
fn admitted_manifest_contains_only_pure_production_dependencies() {
    // r[agent_harness_runtime.session_core_crate.dependencies]
    // r[agent_harness_runtime.session_core_crate.validation]
    let source_result = read_manifest("tests/fixtures/session-core-boundary/positive/Cargo.toml");
    assert!(source_result.is_ok(), "positive fixture must be readable");
    let Ok(source) = source_result else {
        return;
    };
    let validation = validate_manifest_text(&source);
    assert!(validation.is_ok(), "positive manifest must pass");
    let Ok(report) = validation else {
        return;
    };
    assert_eq!(
        report.production_dependencies.len(),
        EXPECTED_PRODUCTION_DEPENDENCY_COUNT
    );
    assert_eq!(
        report.production_dependencies,
        ALLOWED_PRODUCTION_DEPENDENCIES.map(String::from)
    );
}

#[test]
fn repository_manifest_obeys_the_same_boundary() {
    let source_result = read_manifest("Cargo.toml");
    assert!(source_result.is_ok(), "crate manifest must be readable");
    let Ok(source) = source_result else {
        return;
    };
    let validation = validate_manifest_text(&source);
    assert!(validation.is_ok(), "crate manifest must pass");
    let Ok(report) = validation else {
        return;
    };
    assert_eq!(
        report.production_dependencies,
        ALLOWED_PRODUCTION_DEPENDENCIES.map(String::from)
    );
}

#[test]
fn host_dependency_fails_closed() {
    // r[agent_harness_runtime.session_core_crate.dependencies]
    let source_result =
        read_manifest("tests/fixtures/session-core-boundary/invalid-host/Cargo.toml");
    assert!(source_result.is_ok(), "invalid fixture must be readable");
    let Ok(source) = source_result else {
        return;
    };
    let error_result = validate_manifest_text(&source);
    assert!(error_result.is_err(), "host dependency must fail");
    let Err(error) = error_result else {
        return;
    };
    assert_eq!(
        error,
        BoundaryError::ForbiddenProductionDependency(String::from("reqwest"))
    );
}

#[test]
fn malformed_manifest_fails_closed() {
    // r[agent_harness_runtime.session_core_crate.validation]
    let source_result =
        read_manifest("tests/fixtures/session-core-boundary/malformed/manifest.toml");
    assert!(source_result.is_ok(), "malformed fixture must be readable");
    let Ok(source) = source_result else {
        return;
    };
    let error_result = validate_manifest_text(&source);
    assert!(error_result.is_err(), "malformed manifest must fail");
    let Err(error) = error_result else {
        return;
    };
    assert_eq!(error, BoundaryError::MalformedManifest);
}
