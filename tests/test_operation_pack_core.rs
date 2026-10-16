use std::collections::BTreeMap;

use lattice::operation_pack as pack;

const VALID_FIXTURE: &str = "tests/fixtures/operation-packs/valid";
const VALID_REGISTRY: &str = "tests/fixtures/operation-packs/registry-valid.ncl";
const AMBIGUOUS_REGISTRY: &str = "tests/fixtures/operation-packs/registry-ambiguous.ncl";
const UNKNOWN_FIELD_FIXTURE: &str = "tests/fixtures/operation-packs/invalid-unknown-field";
const VALID_POLICY_FIXTURE: &str = "tests/fixtures/operation-packs/manifest-policy-valid.ncl";
const INVALID_POLICY_FIXTURE: &str = "tests/fixtures/operation-packs/manifest-policy-invalid.ncl";
const CURRENT_EPOCH_SECONDS: i64 = 1_786_834_900;
const MAXIMUM_REGISTRY_AGE_SECONDS: u64 = 86_400;
const STALE_REGISTRY_EPOCH_SECONDS: i64 = 1_700_000_000;
#[cfg(unix)]
const NON_EXECUTABLE_MODE: u32 = 0o644;

fn fixture_root() -> std::path::PathBuf {
    std::fs::canonicalize(VALID_FIXTURE).expect("fixture root")
}

fn candidate() -> pack::Candidate {
    pack::shell::load(&fixture_root()).expect("valid operation pack")
}

fn facts(candidate: &pack::Candidate) -> pack::AdmissionFacts {
    pack::AdmissionFacts {
        trusted_pack_identity: candidate.pack_identity.clone(),
        platform: pack::shell::platform_label(),
        dependency_identities: BTreeMap::new(),
        allowed_licenses: vec!["MIT".to_string()],
        allowed_capabilities: candidate.manifest.admission.allowed_capabilities.clone(),
        evidence: BTreeMap::new(),
    }
}

fn rebuild(mut candidate: pack::Candidate) -> pack::Candidate {
    candidate.manifest = pack::validate_manifest(candidate.manifest).expect("normalized manifest");
    candidate.manifest_identity = pack::manifest_identity(&candidate.manifest).expect("manifest identity");
    candidate.pack_identity = pack::pack_identity(&candidate.manifest).expect("pack identity");
    candidate
}

#[test]
fn manifest_and_pack_identities_are_deterministic_and_ignore_display_metadata() {
    let first = candidate();
    let second = candidate();
    assert_eq!(first.manifest_identity, second.manifest_identity);
    assert_eq!(first.pack_identity, second.pack_identity);

    let mut display_changed = first.clone();
    display_changed.manifest.description = "Different display summary".to_string();
    display_changed.manifest.release.display_version = "9.9.9".to_string();
    display_changed.manifest.release.channel = pack::ReleaseChannel::Stable;
    display_changed.manifest.release.deprecated = true;
    display_changed.manifest.release.superseded_by = Some(pack::content_identity(b"display replacement"));
    display_changed.manifest.release.publication_epoch_seconds = Some(CURRENT_EPOCH_SECONDS);
    // r[verify lattice.operation_packs.identity.stable]
    assert_eq!(
        pack::manifest_identity(&display_changed.manifest).expect("display identity"),
        first.manifest_identity
    );
    assert_eq!(pack::pack_identity(&display_changed.manifest).expect("display pack identity"), first.pack_identity);

    let mut behavior_changed = first.clone();
    behavior_changed.manifest.runtimes[0].timeout_seconds -= 1;
    // r[verify lattice.operation_packs.identity.changed]
    assert_ne!(pack::pack_identity(&behavior_changed.manifest).expect("changed pack identity"), first.pack_identity);
}

#[test]
fn nickel_contract_accepts_the_exact_manifest_and_rejects_unknown_fields() {
    let valid = std::fs::canonicalize(VALID_POLICY_FIXTURE).expect("valid policy fixture");
    let valid_result = pack::shell::validate_manifest_contract(&valid);
    assert!(valid_result.is_ok(), "{valid_result:?}");
    let invalid = std::fs::canonicalize(INVALID_POLICY_FIXTURE).expect("invalid policy fixture");
    // r[verify lattice.operation_packs.manifest.invalid]
    assert!(pack::shell::validate_manifest_contract(&invalid).is_err());
}

#[test]
fn strict_manifest_and_exact_content_validation_fail_closed() {
    assert!(pack::shell::load(&std::fs::canonicalize(UNKNOWN_FIELD_FIXTURE).expect("invalid fixture root")).is_err());

    let source = fixture_root();
    let text_root = tempfile::tempdir().expect("manifest text root");
    let text_pack = text_root.path().join("pack");
    copy_pack(&source, &text_pack);
    let manifest_path = text_pack.join("manifest.ncl");
    let manifest_source = std::fs::read_to_string(&manifest_path).expect("manifest source");
    let display_changed = manifest_source.replace(
        "Exact read-only Cairn inspection with bounded analysis",
        "Important import guidance is display metadata",
    );
    std::fs::write(&manifest_path, display_changed).expect("display manifest");
    let display_candidate = pack::shell::load(&text_pack).expect("display import word is not syntax");
    assert_eq!(display_candidate.pack_identity, candidate().pack_identity);
    let imported_source = format!("let external = import \"../outside.ncl\" in\n{manifest_source}");
    std::fs::write(&manifest_path, imported_source).expect("importing manifest");
    assert!(pack::shell::load(&text_pack).is_err());

    let copy_root = tempfile::tempdir().expect("copy root");
    let copied = copy_root.path().join("pack");
    copy_pack(&source, &copied);
    let undeclared = copied.join("undeclared.txt");
    std::fs::write(&undeclared, "not declared").expect("undeclared content");
    assert!(pack::shell::load(&copied).is_err());
    std::fs::remove_file(undeclared).expect("remove undeclared content");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let analyzer = copied.join("analyzer-fixture");
        let executable_permissions = std::fs::metadata(&analyzer).expect("runtime metadata").permissions();
        std::fs::set_permissions(&analyzer, std::fs::Permissions::from_mode(NON_EXECUTABLE_MODE))
            .expect("remove executable mode");
        assert!(pack::shell::load(&copied).is_err());
        std::fs::set_permissions(&analyzer, executable_permissions).expect("restore executable mode");
    }
    std::fs::write(copied.join("skill.md"), "tampered body").expect("tamper body");
    assert!(pack::shell::load(&copied).is_err());

    let symlink_root = tempfile::tempdir().expect("symlink root");
    let linked = symlink_root.path().join("pack");
    copy_pack(&source, &linked);
    std::fs::remove_file(linked.join("skill.md")).expect("remove copied skill");
    #[cfg(unix)]
    std::os::unix::fs::symlink(source.join("skill.md"), linked.join("skill.md")).expect("create skill symlink");
    assert!(pack::shell::load(&linked).is_err());

    let mut unicode = candidate().manifest;
    unicode.skills[0].description = "Caf\u{e9}".to_string();
    assert!(pack::validate_manifest(unicode).is_err());
    let mut unsupported_runtime = candidate().manifest;
    unsupported_runtime.runtimes[0].kind = "node".to_string();
    // r[verify lattice.operation_packs.runtime.denied]
    assert!(pack::validate_manifest(unsupported_runtime).is_err());
    let mut contradictory_release = candidate().manifest;
    contradictory_release.release.superseded_by = Some(pack::content_identity(b"replacement"));
    assert!(pack::validate_manifest(contradictory_release).is_err());
    let mut cycle = candidate().manifest;
    cycle.dependencies.push(pack::Dependency {
        pack_id: cycle.pack_id.clone(),
        pack_identity: pack::content_identity(b"self dependency"),
    });
    assert!(pack::validate_manifest(cycle).is_err());
}

#[test]
fn lock_admission_binds_trust_platform_license_capability_and_closure() {
    let candidate = candidate();
    let admitted_facts = facts(&candidate);
    let lock = pack::create_lock(&candidate, &admitted_facts).expect("exact lock");
    let admitted = pack::admit(&candidate, &lock, &admitted_facts).expect("admitted pack");
    // r[verify lattice.operation_packs.admission.valid]
    assert_eq!(admitted.candidate.pack_identity, candidate.pack_identity);
    assert!(pack::validate_lock(&candidate, &lock).is_ok());

    let mut denied = admitted_facts.clone();
    denied.trusted_pack_identity = pack::content_identity(b"other pack");
    // r[verify lattice.operation_packs.admission.denied]
    assert!(pack::create_lock(&candidate, &denied).is_err());
    let mut denied = admitted_facts.clone();
    denied.platform = "unsupported-platform".to_string();
    assert!(pack::create_lock(&candidate, &denied).is_err());
    let mut denied = admitted_facts.clone();
    denied.allowed_licenses.clear();
    assert!(pack::create_lock(&candidate, &denied).is_err());
    let mut denied = admitted_facts.clone();
    denied.allowed_capabilities.clear();
    assert!(pack::create_lock(&candidate, &denied).is_err());

    let mut tampered = lock.clone();
    tampered.runtime_identities.clear();
    assert!(pack::admit(&candidate, &tampered, &admitted_facts).is_err());
    let mut unsafe_locator = lock.clone();
    unsafe_locator.source_locator = "private source path".to_string();
    assert!(pack::validate_lock(&candidate, &unsafe_locator).is_err());
    let mut other_platform = lock;
    other_platform.platform = "aarch64-linux".to_string();
    other_platform.identity = pack::identity::lock_identity(&other_platform).expect("other platform lock identity");
    assert!(pack::validate_lock(&candidate, &other_platform).is_ok());
    assert!(pack::admit(&candidate, &other_platform, &admitted_facts).is_err());
}

#[test]
fn dependencies_and_evidence_are_exact_and_overclaims_are_rejected() {
    let original = candidate();
    let dependency_identity = pack::content_identity(b"dependency");
    let mut dependent = original.clone();
    dependent.manifest.dependencies.push(pack::Dependency {
        pack_id: "lattice.dependency".to_string(),
        pack_identity: dependency_identity.clone(),
    });
    dependent.manifest.admission.required_evidence.push(pack::EvidenceKind::MantleBuild);
    let dependent = rebuild(dependent);
    let evidence_identity = pack::content_identity(b"evidence");
    let mut exact = facts(&dependent);
    exact.dependency_identities.insert("lattice.dependency".to_string(), dependency_identity);
    exact.evidence.insert(pack::EvidenceKind::MantleBuild, pack::EvidenceFact {
        identity: evidence_identity,
        current: true,
        valid: true,
        overclaimed: false,
    });
    assert!(pack::create_lock(&dependent, &exact).is_ok());

    let mut missing = exact.clone();
    missing.dependency_identities.clear();
    assert!(pack::create_lock(&dependent, &missing).is_err());
    let mut conflict = exact.clone();
    conflict
        .dependency_identities
        .insert("lattice.dependency".to_string(), pack::content_identity(b"wrong dependency"));
    assert!(pack::create_lock(&dependent, &conflict).is_err());
    let mut overclaimed = exact.clone();
    overclaimed.evidence.get_mut(&pack::EvidenceKind::MantleBuild).expect("evidence").overclaimed = true;
    assert!(pack::create_lock(&dependent, &overclaimed).is_err());
    let mut malformed = exact.clone();
    malformed.evidence.get_mut(&pack::EvidenceKind::MantleBuild).expect("evidence").identity =
        "not-an-identity".to_string();
    assert!(pack::create_lock(&dependent, &malformed).is_err());
    let mut stale = exact.clone();
    stale.evidence.get_mut(&pack::EvidenceKind::MantleBuild).expect("evidence").current = false;
    assert!(pack::create_lock(&dependent, &stale).is_err());
    let mut undeclared = exact;
    undeclared.evidence.insert(pack::EvidenceKind::ValenceRecordedOnly, pack::EvidenceFact {
        identity: pack::content_identity(b"undeclared optional evidence"),
        current: true,
        valid: true,
        overclaimed: false,
    });
    assert!(pack::create_lock(&dependent, &undeclared).is_err());
}

#[test]
fn higher_risk_capability_profile_uses_the_same_exact_evidence_boundary() {
    let mut higher_risk = candidate();
    higher_risk.manifest.runtimes[1]
        .capabilities
        .push(lattice::operation::CapabilityClass::CredentialRead);
    higher_risk
        .manifest
        .admission
        .allowed_capabilities
        .push(lattice::operation::CapabilityClass::CredentialRead);
    higher_risk.manifest.admission.required_evidence.push(pack::EvidenceKind::BasaltAuthority);
    let higher_risk = rebuild(higher_risk);
    let mut exact = facts(&higher_risk);
    exact.allowed_capabilities.push(lattice::operation::CapabilityClass::CredentialRead);
    exact.evidence.insert(pack::EvidenceKind::BasaltAuthority, pack::EvidenceFact {
        identity: pack::content_identity(b"Basalt authority evidence"),
        current: true,
        valid: true,
        overclaimed: false,
    });
    assert!(pack::create_lock(&higher_risk, &exact).is_ok());
    exact.evidence.clear();
    assert!(pack::create_lock(&higher_risk, &exact).is_err());
}

#[test]
fn locator_changes_do_not_change_authoritative_lock_identity() {
    let candidate = candidate();
    let admitted_facts = facts(&candidate);
    let first = pack::create_lock(&candidate, &admitted_facts).expect("first lock");
    let mut relocated = candidate.clone();
    relocated.source_locator = pack::content_identity(b"relocated display locator");
    let second = pack::create_lock(&relocated, &admitted_facts).expect("relocated lock");
    assert_ne!(first.source_locator, second.source_locator);
    assert_eq!(first.identity, second.identity);
    assert!(pack::validate_lock(&relocated, &first).is_ok());
}

#[test]
fn registry_search_freshness_and_forward_promotion_are_deterministic() {
    let valid = std::fs::canonicalize(VALID_REGISTRY).expect("registry path");
    let catalog = pack::shell::load_registry(&valid).expect("valid registry");
    let results = pack::registry::search(&catalog, "cairn");
    assert_eq!(results.len(), 1);
    assert_eq!(pack::registry::exact(&catalog, &results[0].pack_identity).expect("exact entry"), results[0]);
    let restore_root = tempfile::tempdir().expect("restore root");
    let restore =
        pack::registry::restore_plan(&catalog, &results[0].pack_identity, &restore_root.path().join("restored-pack"))
            .expect("restore plan");
    assert!(restore.destination_omitted);
    assert!(restore.requires_confirmation);
    assert!(!restore.network_contacted);
    assert!(!restore.pack_installed);
    assert!(!restore.lock_changed);
    let freshness = pack::registry::freshness(Some(&catalog), CURRENT_EPOCH_SECONDS, MAXIMUM_REGISTRY_AGE_SECONDS);
    assert!(freshness.current);
    assert!(!freshness.unavailable);
    let mut mirror = catalog.clone();
    mirror.entries[0].content_locator = "https://mirror.example/packs/cairn".to_string();
    assert!(pack::registry::validate_catalog(mirror).is_ok());
    let mut unsafe_mirror = catalog.clone();
    unsafe_mirror.entries[0].content_locator = "https://user:secret@mirror.example/packs/cairn".to_string();
    assert!(pack::registry::validate_catalog(unsafe_mirror).is_err());
    let promoted = pack::registry::promote(&catalog, &results[0].pack_identity, pack::ReleaseChannel::Rc)
        .expect("forward promotion");
    // r[verify lattice.operation_packs.registry.promote]
    assert_eq!(promoted.entries[0].channel, pack::ReleaseChannel::Rc);
    assert!(pack::registry::promote(&promoted, &results[0].pack_identity, pack::ReleaseChannel::Beta).is_err());

    let ambiguous = std::fs::canonicalize(AMBIGUOUS_REGISTRY).expect("ambiguous registry path");
    assert!(pack::shell::load_registry(&ambiguous).is_err());
    // r[verify lattice.operation_packs.registry.advisory]
    assert!(pack::registry::freshness(None, CURRENT_EPOCH_SECONDS, MAXIMUM_REGISTRY_AGE_SECONDS).unavailable);
}

#[test]
fn stale_registry_and_supersession_remain_visible_without_implicit_mutation() {
    let valid = std::fs::canonicalize(VALID_REGISTRY).expect("registry path");
    let catalog = pack::shell::load_registry(&valid).expect("valid registry");
    let mut lifecycle = catalog.clone();
    lifecycle.generated_epoch_seconds = STALE_REGISTRY_EPOCH_SECONDS;
    lifecycle.entries[0].deprecated = true;
    lifecycle.entries[0].superseded_by = Some(pack::content_identity(b"replacement pack"));
    let lifecycle = pack::registry::validate_catalog(lifecycle).expect("lifecycle registry");
    let before = lifecycle.clone();
    let freshness = pack::registry::freshness(Some(&lifecycle), CURRENT_EPOCH_SECONDS, MAXIMUM_REGISTRY_AGE_SECONDS);
    assert!(!freshness.current);
    assert_eq!(lifecycle, before);
    let selected = pack::registry::search(&lifecycle, "cairn");
    assert!(selected[0].deprecated);
    assert!(selected[0].superseded_by.is_some());
}

fn copy_pack(source: &std::path::Path, destination: &std::path::Path) {
    std::fs::create_dir(destination).expect("create copied pack");
    for entry in std::fs::read_dir(source).expect("read source pack") {
        let entry = entry.expect("source entry");
        let destination_path = destination.join(entry.file_name());
        std::fs::copy(entry.path(), &destination_path).expect("copy pack member");
        std::fs::set_permissions(&destination_path, entry.metadata().expect("entry metadata").permissions())
            .expect("copy permissions");
    }
}
