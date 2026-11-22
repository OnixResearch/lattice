# Design: Runbook catalogs carried by operation packs

## Context

Baseline: Lattice `0862c017`. Lattice #27 changes the bundle manifest schema (typed `params`, `mode`) in parallel, so this change treats a bundle manifest as opaque pinned bytes and verifies bundles only through the existing loader entry point `workflow_template::load_catalog`.

## Decisions

### Decision: Bundle files are ordinary pack content plus a declared bundle identity

**Choice:** Every bundle file is a `workflow_bundle` content item at `workflow-templates/<bundle-id>/<member>`. `Manifest.workflow_bundles` lists `{ id, bundle_identity }`, where `bundle_identity` is the loader's identity with a `b3:` prefix. The pure `catalog::validate_declarations` enforces placement, the bundle manifest item, and id uniqueness and syntax (lowercase ASCII, digits and `-`, at most 64 bytes, a subset of the loader's identifiers that is also one safe path component). `workflow_bundles` joins the manifest identity projection only when it is non-empty.

**Rationale:** Reusing content items keeps every byte inside the existing measured, length-bound, symlink-free and undeclared-file-free pack load, and inside the pack identity. The declared loader identity lets a pack pin exactly what the loader validates. Because bundle manifests are pinned as bytes, fields added later (#27) are covered without a schema dependency. The conditional projection preserves every existing pack and lock identity.

### Decision: Two levels of refusal

**Choice:** `catalog-install` first measures every bundle file against its declaration. If any file has drifted, it refuses the whole pack, and the report names each drifted bundle as `pack_bytes_drift` and every other bundle as `pack_refused`. Admission then proceeds through the strict existing loader and lock check. After admission, the pure `catalog::decision::decide` decides each bundle independently from the loader's catalog over the pack's `workflow-templates/` directory:

- a loader diagnostic, reported with its own code;
- `pack_content_mismatch`, when the loader-measured manifest or file identities differ from the pack declaration;
- `identity_mismatch`, when the loader identity differs from the declared one;
- `catalog_conflict`, when any entry already exists at `<catalog>/<id>`.

**Rationale:** The operator trusts one exact pack identity. A tree whose bytes differ from that identity was modified after it was locked or fetched. Installing its other bundles would trust a source that is known to be tampered, so the whole pack is refused. The report still names each drifted bundle, so the drift does not hide which bundles are affected. Per-bundle defects inside an untampered pack are publisher defects that affect one bundle only: a compatibility mismatch, a stale bundle-internal pin or a wrong declared identity. Isolating them lets the good bundles install. That satisfies "one bad bundle does not hide the others" without weakening the pack-level identity.

### Decision: Per-bundle staged installation, verified twice

**Choice:** For each admitted bundle, Lattice creates `.lattice-catalog-<id>.staging` in the catalog's parent directory. It copies each file from `shell::read_content`, which re-checks the declared identity, through a `cap_std` capability directory with `create_new`. It syncs every file and directory. It runs `load_catalog` on the staging directory and requires the same decision to admit the bundle again. It renames `<staging>/<id>` to `<catalog>/<id>` only while that path is absent, then syncs the catalog and the parent directory. The staging directory is removed on every path.

**Rationale:** The second loader pass binds the published bytes to the pack declaration, not to the earlier preview read, which closes the gap between preview and copy. The capability directory cannot follow a symlink or leave the staging root. The rename makes each bundle appear whole. Staging outside the catalog keeps a crash residue out of **Browse Workflow Templates**. The absence check before the rename has the same check-then-rename window as the existing `install_pack_create_only`. It is documented as create-only, not as a lock.

### Decision: Fetch is the only networked step, through bounded git

**Choice:** `catalog-fetch` requires `--confirm-fetch`, a full lowercase commit and a `file://`, `https://` or `ssh://` locator. It runs `git` through `process::shell::run_bounded_clear_environment` with an isolated configuration: `GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL=/dev/null`, `HOME` set to the staging directory, `GIT_ALLOW_PROTOCOL=file:https:ssh` and `GIT_TERMINAL_PROMPT=0`. It forwards only `PATH`, `SSL_CERT_FILE`, `SSL_CERT_DIR` and `SSH_AUTH_SOCK`. The fetch runs with `transfer.fsckObjects=true` and `--depth=1`. Lattice then verifies `rev-parse <commit>^{commit}`. The pure `catalog::source::parse_tree` reads `ls-tree -r -z`, refuses symlink and submodule modes, unsafe paths and entries outside the subdirectory, and bounds the listing. Each blob is read with `cat-file` and written create-only. The strict `shell::load` must accept the tree, and an optional `--pack-identity` must match, before the tree is renamed into a new destination.

**Rationale:** Reading blobs instead of checking out a work tree never creates links. It also never runs filters or hooks from the fetched repository. No Cargo dependency is added. `file://` URLs give hermetic tests.

## Tests

- `tests/test_runbook_catalog_core.rs`: the seed manifest satisfies the Nickel contract, and a malformed bundle identity does not. The seed pins are current, and changing any content or bundle identity changes the pack identity. Duplicate ids, traversal, an undeclared bundle, a missing bundle manifest, another role under the root, an unsafe id and a symlink member refuse the pack. A wrong identity, a compatibility mismatch and a stale internal pin reject only that bundle while the other installs. An installed workflow still has authority grants to review and fails without a profile. Tree-listing, commit and locator validation is covered.
- `tests/test_runbook_catalog_cli.rs`: a fetch preview writes nothing. Fetch, lock, preview, install and list run over a `file://` repository, then run with the repository removed and an empty `PATH`. A second install conflicts without changing bytes. Inexact commits, disallowed locators, a wrong pack identity and a symlink member are refused. Tampered bytes refuse the pack and name each bundle.

## Risks / Trade-offs

- Installation is per-bundle atomic, not a transaction across bundles. A crash can leave some bundles installed and a `.lattice-catalog-*.staging` directory in the config directory, which the next run reports as a staging conflict for that bundle until it is removed.
- The create-only absence check before the rename is not atomic against a concurrent writer to the same catalog path.
- `https://` fetching relies on the host's `git` and certificate configuration and gets no credential helper, so private repositories need `ssh://` with an agent.
- A pack holds at most 128 content items, which bounds the bundles in one pack.
