use std::os::unix::fs::PermissionsExt as _;

const STAGING: &str = ".lattice-catalog-test.staging";
const ENTRY: &str = "bundle";
const MARKER: &str = "marker.txt";

fn staged(root: &std::path::Path) -> (super::Staging, super::Inode) {
    let staging = super::Staging::create(root, STAGING).expect("staging");
    staging.dir().create_dir(ENTRY).expect("entry");
    staging.dir().write(format!("{ENTRY}/{MARKER}"), b"verified").expect("marker");
    let inode = staging.entry_inode(ENTRY).expect("entry inode");
    (staging, inode)
}

fn target(root: &std::path::Path) -> cap_std::fs::Dir {
    std::fs::create_dir(root.join("catalog")).expect("target");
    cap_std::fs::Dir::open_ambient_dir(root.join("catalog"), cap_std::ambient_authority()).expect("target handle")
}

fn entry_in(catalog: &cap_std::fs::Dir) -> super::Target<'_> {
    super::Target {
        dir: catalog,
        name: ENTRY,
    }
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.pinned]
#[test]
fn staging_is_private_and_publishes_the_pinned_entry() {
    let root = tempfile::tempdir().expect("root");
    let (staging, inode) = staged(root.path());
    let mode = std::fs::metadata(staging.path()).expect("staging metadata").permissions().mode();
    assert_eq!(mode & 0o777, 0o700, "staging is private");
    let catalog = target(root.path());
    staging.publish(ENTRY, inode, entry_in(&catalog)).expect("publish");
    assert_eq!(std::fs::read(root.path().join("catalog").join(ENTRY).join(MARKER)).expect("published"), b"verified");
    assert!(staging.publish(ENTRY, inode, entry_in(&catalog)).is_err(), "nothing left to publish");
    staging.remove().expect("cleanup");
    assert!(!root.path().join(STAGING).exists());
    assert!(super::Staging::create(root.path(), STAGING).is_ok(), "a removed staging name is reusable");
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.pinned]
#[test]
fn a_swapped_staging_directory_or_entry_is_refused() {
    let root = tempfile::tempdir().expect("root");
    let (staging, inode) = staged(root.path());
    let catalog = target(root.path());

    // Swap the entry inside the pinned staging directory for different bytes.
    std::fs::rename(staging.path().join(ENTRY), root.path().join("moved-entry")).expect("move entry");
    std::fs::create_dir(staging.path().join(ENTRY)).expect("substitute entry");
    std::fs::write(staging.path().join(ENTRY).join(MARKER), b"unverified").expect("substitute bytes");
    assert!(staging.publish(ENTRY, inode, entry_in(&catalog)).is_err(), "substituted entry is refused");
    assert!(!root.path().join("catalog").join(ENTRY).exists());

    // Swap the whole staging directory at its path.
    std::fs::rename(staging.path(), root.path().join("moved-staging")).expect("move staging");
    std::fs::create_dir(staging.path()).expect("substitute staging");
    assert!(staging.ensure_pinned().is_err(), "substituted staging path is refused");
    assert!(staging.publish(ENTRY, inode, entry_in(&catalog)).is_err());
    assert!(!root.path().join("catalog").join(ENTRY).exists());

    // An existing target is never replaced.
    let (fresh, fresh_inode) = staged(&root.path().join("moved-entry"));
    std::fs::create_dir(root.path().join("catalog").join(ENTRY)).expect("existing target");
    assert!(fresh.publish(ENTRY, fresh_inode, entry_in(&catalog)).is_err(), "existing target is refused");
}
