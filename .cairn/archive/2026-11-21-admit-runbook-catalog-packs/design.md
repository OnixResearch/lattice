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

### Decision: Per-bundle staged installation, verified twice (superseded in part by the review decision below)

**Choice:** For each admitted bundle, Lattice creates `.lattice-catalog-<id>.staging` in the catalog's parent directory. It copies each file from `shell::read_content`, which re-checks the declared identity, through a `cap_std` capability directory with `create_new`. It syncs every file and directory. It runs `load_catalog` on the staging directory and requires the same decision to admit the bundle again. It renames `<staging>/<id>` to `<catalog>/<id>` only while that path is absent, then syncs the catalog and the parent directory. The staging directory is removed on every path.

**Rationale:** The second loader pass binds the published bytes to the pack declaration, not to the earlier preview read, which closes the gap between preview and copy. The capability directory cannot follow a symlink or leave the staging root. The rename makes each bundle appear whole. Staging outside the catalog keeps a crash residue out of **Browse Workflow Templates**. The absence check before the rename has the same check-then-rename window as the existing `install_pack_create_only`. It is documented as create-only, not as a lock.

The review decision below replaces the per-bundle staging and path-based rename with one private, handle-pinned staging directory, filled from bytes captured before anything is staged. The fetch description below is likewise extended by the bounded two-step transfer.

### Decision: Fetch is the only networked step, through bounded git

**Choice:** `catalog-fetch` requires `--confirm-fetch`, a full lowercase commit and a `file://`, `https://` or `ssh://` locator. It runs `git` through `process::shell::run_bounded_clear_environment` with an isolated configuration: `GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL=/dev/null`, `HOME` set to the staging directory, `GIT_ALLOW_PROTOCOL=file:https:ssh` and `GIT_TERMINAL_PROMPT=0`. It forwards only `PATH`, `SSL_CERT_FILE`, `SSL_CERT_DIR` and `SSH_AUTH_SOCK`. The fetch runs with `transfer.fsckObjects=true` and `--depth=1`. Lattice then verifies `rev-parse <commit>^{commit}`. The pure `catalog::source::parse_tree` reads `ls-tree -r -z`, refuses symlink and submodule modes, unsafe paths and entries outside the subdirectory, and bounds the listing. Each blob is read with `cat-file` and written create-only. The strict `shell::load` must accept the tree, and an optional `--pack-identity` must match, before the tree is renamed into a new destination.

**Rationale:** Reading blobs instead of checking out a work tree never creates links. It also never runs filters or hooks from the fetched repository. No Cargo dependency is added. `file://` URLs give hermetic tests.

### Decision: Review of `182667cd` — bounded transfer, capacity, pinned staging, capture-first, SHA-256

**Context:** The independent review (`CatalogReview`) found five defects:

- (P1) `--depth=1` still downloaded every blob of the commit, and each selected blob could reach the 64 MiB output limit before any total was checked.
- (P1) Installing a 65th bundle directory reported `installed`, while the loader then showed zero bundles with `limit_exceeded`.
- (P1) Staging directories inherited the umask and were verified by path and renamed by path.
- (P2) `publish_bundle` reread source bytes after admission, so mid-install drift rejected only one bundle.
- (P2) A 64-character commit was fetched into a SHA-1 repository.

**Choice:**

- **Transfer.** The staging repository is `git init --bare --template= --object-format=<sha1|sha256>`, with one promisor remote `origin` (`partialclonefilter=blob:none`). Every command runs with `GIT_NO_LAZY_FETCH=1`. The first fetch takes the commit and its trees with `--filter=blob:none --depth=1`. After `ls-tree` and `parse_tree`, a second fetch takes exactly the listed blob names on stdin, with the noop negotiation that lazy fetching uses. Both fetches run as `sh -c 'ulimit -f "$1" && shift && exec "$@"'` with `fetch.unpackLimit=1` and `transfer.unpackLimit=1`, so each fetch writes one pack. `RLIMIT_FSIZE` caps each file at the remaining budget, in 512-byte POSIX blocks, rounded down. The repository's total and largest file are measured after each fetch. A file that reached the cap, or a total over the budget, is the typed `catalog fetch transfer bound` refusal. The pure `catalog::transfer::validate_object_sizes` checks the `cat-file --batch-check` type and size of every selected object, and the selected total, before `cat-file blob` reads it. Each read is then bounded to that object's exact size. `--maximum-transfer-bytes` defaults to twice the pack content limit and cannot go below 64 KiB. Behavior was confirmed against the pinned git 2.53.0:
  - `ulimit -f` counts 512-byte blocks in `sh`;
  - SIGXFSZ ends `index-pack`, and the fetch fails;
  - protocol v2 serves unadvertised reachable blob names without `allowAnySHA1InWant`;
  - `GIT_NO_LAZY_FETCH` reports a missing promisor blob instead of fetching it;
  - SHA-256 `file://` fetches work.
- **Capacity.** `decision::Facts.free_slots` comes from the catalog's directory count, which uses the loader's own rule: directories, not symlinks. The decision rejects admitted bundles past the free slots as `catalog_capacity`. Placement recounts through the catalog handle before each rename.
- **Pinned staging.** `shell/catalog/pinned.rs::Staging` creates the staging directory mode 0700 through a handle on its parent and records its device and inode. Before and after each path-based reader (the strict pack loader, `load_catalog`) and before each rename, it checks that both the path and the parent-handle entry still name that inode. Each publication checks the entry's recorded inode and the target's absence, then uses `Dir::rename` relative to the staging handle and the target handle. Fetch writes its tree and installation writes its bundles only through handles.
- **Capture first.** With `--confirm-create`, `placement::capture` reads every declared bundle file through `read_content` into memory, verifying identity and length, before anything is staged. Any failure refuses the whole pack. The captured bytes are staged once into `.lattice-catalog-<pack identity prefix>.staging`, verified by `load_catalog` there, decided, and published bundle by bundle. The pack directory is never reread.

**Rationale:** A filter is a request, not a bound. A dishonest or filter-ignoring server can send anything, so the hard bound is the kernel's per-file size limit plus the measured total. The filter keeps honest large repositories cheap. Checking sizes before content keeps memory use within the pack limits. Mode 0700 keeps other users out of staging, and the inode checks refuse a swapped directory or entry instead of publishing it. Capturing every byte before the first publication makes drift a property of the whole pack again.

## Tests

- `tests/test_runbook_catalog_core.rs`: the seed manifest satisfies the Nickel contract, and a malformed bundle identity does not. The seed pins are current, and changing any content or bundle identity changes the pack identity. Duplicate ids, traversal, an undeclared bundle, a missing bundle manifest, another role under the root, an unsafe id and a symlink member refuse the pack. A wrong identity, a compatibility mismatch and a stale internal pin reject only that bundle while the other installs. An installed workflow still has authority grants to review and fails without a profile. Tree-listing, commit and locator validation is covered.
- `tests/test_runbook_catalog_cli.rs`: a fetch preview writes nothing. Fetch, lock, preview, install and list run over a `file://` repository, then run with the repository removed and an empty `PATH`. A second install conflicts without changing bytes. Inexact commits, disallowed locators, a wrong pack identity and a symlink member are refused. Tampered bytes refuse the pack and name each bundle.
- `tests/test_runbook_catalog_fetch.rs`:
  - A 4 MiB blob outside the selected pack, from a filter-honoring server with a 1 MiB budget, is never downloaded: the fetch succeeds with less than 1 MiB transferred.
  - The same blob from a filter-ignoring server, and a 4 MiB blob inside the selected pack, end with `catalog fetch transfer bound` and nothing published.
  - A budget below the minimum is refused.
  - A SHA-256 repository fetches into SHA-256 staging and yields the seed pack identity, and a SHA-256 name against a SHA-1 repository fails.
  - `validate_object_sizes` refuses missing objects, non-blobs, an oversized manifest, file or total, short output and unexpected objects.
- `tests/test_runbook_catalog_install.rs`: with 63 directories, preview and install admit one bundle and reject the other as `catalog_capacity`. The catalog then holds 64 directories and loads without `limit_exceeded`. A full catalog takes nothing and is unchanged. Bytes that drift after admission refuse the whole pack with nothing published and no staging left, while an undrifted copy installs both bundles.
- `src/operation_pack/shell/catalog/pinned/tests.rs`:
  - Staging is mode 0700 and publishes the pinned entry.
  - A substituted entry, a substituted staging directory and an existing target are each refused, and nothing is published.

## Risks / Trade-offs

- Installation is per-bundle atomic, not a transaction across bundles. A crash can leave some bundles installed and a `.lattice-catalog-<pack identity prefix>.staging` directory in the config directory. The next installation of that pack then fails with a staging conflict until the directory is removed.
- The create-only absence and capacity checks before each rename are not atomic against a concurrent writer to the same catalog. The inode checks refuse a swapped staging directory or entry, but a process running as the same user can still change files inside the 0700 staging directory.
- While a fetch runs, disk use can briefly reach the budget once per file `git` writes (pack, index and reverse index) before the total is measured. The bound relies on a POSIX `sh` whose `ulimit -f` counts 512-byte blocks, as confirmed for the pinned shell.
- Partial-clone filters and blob wants were exercised only against a local `file://` `upload-pack` with `uploadpack.allowFilter`. Hosted `https://` and `ssh://` servers were not tested.
- `https://` fetching relies on the host's `git` and certificate configuration and gets no credential helper, so private repositories need `ssh://` with an agent.
- A pack holds at most 128 content items, which bounds the bundles in one pack.
