const ADMITTED_ADAPTER: &str = "https://seed.radicle.garden/z3kGDT73ja8rewvGLwJFLF5MTx1ih.git";
const REVIEWED_OBJECT: &str = "a0de79303f3f45c18a34fff535b690a85cb3cf59";
const EXPECTED_PACKAGE_COUNT: usize = 2;
const MANIFEST: &str = include_str!("../Cargo.toml");
const LOCKFILE: &str = include_str!("../Cargo.lock");

#[test]
fn both_cores_use_one_exact_radicle_object() {
    for package in ["replay-ledger-core", "revocation-view-core"] {
        let expected = format!(
            "{package} = {{ git = \"{ADMITTED_ADAPTER}\", rev = \"{REVIEWED_OBJECT}\", optional = true }}"
        );
        assert!(
            MANIFEST.contains(&expected),
            "missing exact source for {package}"
        );
    }
    let locked_source = format!("git+{ADMITTED_ADAPTER}?rev={REVIEWED_OBJECT}#{REVIEWED_OBJECT}");
    assert_eq!(
        LOCKFILE.matches(&locked_source).count(),
        EXPECTED_PACKAGE_COUNT
    );
}

#[test]
fn fallback_sources_are_absent() {
    for prohibited in [
        "github.com/OnixResearch/durable-authority-state",
        "path = \"../durable-authority-state\"",
        "branch =",
        "tag =",
        "rev = \"main\"",
    ] {
        assert!(
            !MANIFEST.contains(prohibited),
            "prohibited Durable Authority State fallback is present: {prohibited}"
        );
    }
}
