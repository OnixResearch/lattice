const UCAN_MANIFEST: &str = include_str!("../vendor/ucan/Cargo.toml");
const UCAN_CORE_MANIFEST: &str = include_str!("../vendor/ucan/crates/ucan-core/Cargo.toml");
const UCAN_NOSTR_MANIFEST: &str = include_str!("../vendor/ucan/crates/ucan-nostr/Cargo.toml");
const BASALT_LOCK: &str = include_str!("../Cargo.lock");

const BLAKE3_DEPENDENCY_PREFIX: &str = "blake3 = ";
const BLAKE3_PACKAGE_NAME: &str = "blake3";
const EXPECTED_BLAKE3_REQUIREMENT: &str = "\"=1.8.5\"";
const EXPECTED_BLAKE3_LOCK_VERSION: &str = "1.8.5";
const DRIFTED_BLAKE3_REQUIREMENT: &str = "\"=1.8.4\"";
const DRIFTED_BLAKE3_LOCK_VERSION: &str = "1.8.4";
const ZEROIZE_DEPENDENCY_PREFIX: &str = "zeroize = ";
const ZEROIZE_PACKAGE_NAME: &str = "zeroize";
const EXPECTED_ZEROIZE_REQUIREMENT: &str = "\"=1.9.0\"";
const EXPECTED_ZEROIZE_LOCK_VERSION: &str = "1.9.0";
const DRIFTED_ZEROIZE_REQUIREMENT: &str = "\"=1.8.2\"";
const PACKAGE_HEADER: &str = "[[package]]";
const EXPECTED_DECLARATION_COUNT: usize = 1;
const EXPECTED_LOCK_ENTRY_COUNT: usize = 1;

#[test]
fn reviewed_ucan_consumers_resolve_one_blake3_cohort() {
    assert_eq!(
        validate_blake3_cohort(UCAN_CORE_MANIFEST, UCAN_NOSTR_MANIFEST, BASALT_LOCK),
        Ok(())
    );
    assert_eq!(
        blake3_lock_versions(BASALT_LOCK),
        vec![EXPECTED_BLAKE3_LOCK_VERSION]
    );
}

#[test]
fn drifted_pin_and_duplicate_lock_cohort_fail_closed() {
    let drifted_core = UCAN_CORE_MANIFEST.replacen(
        EXPECTED_BLAKE3_REQUIREMENT,
        DRIFTED_BLAKE3_REQUIREMENT,
        EXPECTED_DECLARATION_COUNT,
    );
    let pin_error = validate_blake3_cohort(&drifted_core, UCAN_NOSTR_MANIFEST, BASALT_LOCK)
        .expect_err("a drifted ucan-core pin must be rejected");
    assert!(pin_error.contains("ucan-core"), "{pin_error}");

    let duplicate_lock = format!(
        "{BASALT_LOCK}\n{PACKAGE_HEADER}\nname = \"{BLAKE3_PACKAGE_NAME}\"\nversion = \"{DRIFTED_BLAKE3_LOCK_VERSION}\"\n"
    );
    let lock_error =
        validate_blake3_cohort(UCAN_CORE_MANIFEST, UCAN_NOSTR_MANIFEST, &duplicate_lock)
            .expect_err("multiple BLAKE3 lock entries must be rejected");
    assert!(lock_error.contains("lock"), "{lock_error}");
    assert!(
        lock_error.contains(DRIFTED_BLAKE3_LOCK_VERSION),
        "{lock_error}"
    );
}

#[test]
fn reviewed_zeroize_consumers_resolve_one_exact_cohort() {
    assert_eq!(
        validate_zeroize_cohort(UCAN_MANIFEST, UCAN_CORE_MANIFEST, BASALT_LOCK),
        Ok(())
    );
    assert_eq!(
        lock_versions(BASALT_LOCK, ZEROIZE_PACKAGE_NAME),
        vec![EXPECTED_ZEROIZE_LOCK_VERSION]
    );
}

#[test]
fn drifted_zeroize_pin_fails_closed() {
    let drifted_core = UCAN_CORE_MANIFEST.replacen(
        EXPECTED_ZEROIZE_REQUIREMENT,
        DRIFTED_ZEROIZE_REQUIREMENT,
        EXPECTED_DECLARATION_COUNT,
    );
    let error = validate_zeroize_cohort(UCAN_MANIFEST, &drifted_core, BASALT_LOCK)
        .expect_err("a drifted ucan-core Zeroize pin must be rejected");
    assert!(error.contains("ucan-core"), "{error}");
    assert!(error.contains("Zeroize"), "{error}");
}

fn validate_blake3_cohort(
    core_manifest: &str,
    nostr_manifest: &str,
    lock: &str,
) -> Result<(), String> {
    validate_exact_pin(
        "ucan-core",
        core_manifest,
        BLAKE3_DEPENDENCY_PREFIX,
        EXPECTED_BLAKE3_REQUIREMENT,
        "BLAKE3",
    )?;
    validate_exact_pin(
        "ucan-nostr",
        nostr_manifest,
        BLAKE3_DEPENDENCY_PREFIX,
        EXPECTED_BLAKE3_REQUIREMENT,
        "BLAKE3",
    )?;

    let versions = blake3_lock_versions(lock);
    if versions.len() != EXPECTED_LOCK_ENTRY_COUNT || versions[0] != EXPECTED_BLAKE3_LOCK_VERSION {
        return Err(format!(
            "Cargo lock must contain one BLAKE3 {EXPECTED_BLAKE3_LOCK_VERSION} entry; found {versions:?}"
        ));
    }
    Ok(())
}

fn validate_zeroize_cohort(
    root_manifest: &str,
    core_manifest: &str,
    lock: &str,
) -> Result<(), String> {
    validate_exact_pin(
        "ucan",
        root_manifest,
        ZEROIZE_DEPENDENCY_PREFIX,
        EXPECTED_ZEROIZE_REQUIREMENT,
        "Zeroize",
    )?;
    validate_exact_pin(
        "ucan-core",
        core_manifest,
        ZEROIZE_DEPENDENCY_PREFIX,
        EXPECTED_ZEROIZE_REQUIREMENT,
        "Zeroize",
    )?;

    let versions = lock_versions(lock, ZEROIZE_PACKAGE_NAME);
    if versions.len() != EXPECTED_LOCK_ENTRY_COUNT || versions[0] != EXPECTED_ZEROIZE_LOCK_VERSION {
        return Err(format!(
            "Cargo lock must contain one Zeroize {EXPECTED_ZEROIZE_LOCK_VERSION} entry; found {versions:?}"
        ));
    }
    Ok(())
}

fn validate_exact_pin(
    owner: &str,
    manifest: &str,
    prefix: &str,
    expected_requirement: &str,
    dependency: &str,
) -> Result<(), String> {
    let declarations = manifest
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with(prefix))
        .collect::<Vec<_>>();
    if declarations.len() != EXPECTED_DECLARATION_COUNT
        || !declarations[0].contains(expected_requirement)
    {
        return Err(format!(
            "{owner} must declare one exact {dependency} {expected_requirement} dependency"
        ));
    }
    Ok(())
}

fn blake3_lock_versions(lock: &str) -> Vec<&str> {
    lock_versions(lock, BLAKE3_PACKAGE_NAME)
}

fn lock_versions<'a>(lock: &'a str, package_name: &str) -> Vec<&'a str> {
    lock.split(PACKAGE_HEADER)
        .filter_map(|package| {
            let name = package_value(package, "name")?;
            (name == package_name).then(|| package_value(package, "version"))?
        })
        .collect()
}

fn package_value<'a>(package: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key} = \"");
    package
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(&prefix)?.strip_suffix('"'))
}
