# Operation Packs Specification

## ADDED Requirements

### Requirement: Runbook catalogs are admitted through content-addressed packs

r[lattice.operation_packs.runbook_catalog] Lattice MUST let an operation pack carry workflow-template bundles as declared `workflow_bundle` content under `workflow-templates/<bundle-id>/`, each pinned to its workflow-template loader identity, with a pack identity that covers every bundle byte. Fetching a catalog repository MUST be an explicit operator command bound to one full commit, and every other catalog step MUST read local bytes only. Installation into the local `workflow-templates/` catalog MUST be create-only, per-bundle atomic and durable, MUST NOT follow symlinks or escape the catalog, and MUST grant no effect authority.

#### Scenario: Pack manifest declares bundle content

r[lattice.operation_packs.runbook_catalog.scenario.manifest]
- GIVEN a pack manifest that declares workflow bundles
- WHEN Lattice validates and identifies it
- THEN every `workflow_bundle` content item MUST lie under a declared bundle directory, every declared bundle MUST carry its `manifest.ncl` item, and no other role MAY use `workflow-templates/`
- AND a duplicate or unsafe bundle id, parent traversal, an undeclared file or a symlink MUST refuse the whole pack
- AND a change to any bundle file identity or declared bundle identity MUST change the pack identity, while packs without bundles MUST keep their identity

#### Scenario: Exact commit is fetched, locked and installed offline

r[lattice.operation_packs.runbook_catalog.scenario.install]
- GIVEN a catalog repository reachable through a `file://`, `https://` or `ssh://` locator and an exact commit
- WHEN the operator confirms `lattice pack catalog-fetch`, then locks and runs `lattice pack catalog-install` with `--confirm-create`
- THEN the fetch MUST publish only a tree the strict pack loader accepts, matching any expected pack identity
- AND lock, install and list MUST succeed with the repository removed and no `git` executable available
- AND each admitted bundle MUST be staged beside the catalog, re-verified by the workflow-template loader and renamed into the catalog only while its id is absent, and an existing id MUST be reported as a conflict without replacing bytes

#### Scenario: Inexact or unsafe sources are refused

r[lattice.operation_packs.runbook_catalog.scenario.fetch_denied]
- GIVEN a branch, tag or abbreviated commit, a locator outside the allowlisted schemes, an unexpected pack identity, or a tree with a symlink or submodule member
- WHEN `lattice pack catalog-fetch` runs
- THEN it MUST fail without publishing the destination and MUST remove its staging directory

#### Scenario: Fetch transfer and blob sizes are bounded

r[lattice.operation_packs.runbook_catalog.scenario.fetch_bounds]
- GIVEN a catalog repository with large blobs inside or outside the selected subdirectory, a server that may ignore partial-clone filters, and a SHA-1 or SHA-256 commit
- WHEN `lattice pack catalog-fetch` runs with a transfer budget
- THEN it MUST fetch the commit without blob content, fetch only the selected blobs by object name, cap every file `git` writes at the remaining budget, keep the staging repository's total within the budget, and disable lazy object fetching
- AND it MUST check every selected blob's type, size and the selected total against the pack limits before reading any blob content
- AND a transfer that would exceed the budget MUST end with a typed transfer-bound refusal and nothing published
- AND a 64-character commit MUST be fetched into a SHA-256 staging repository

#### Scenario: Staging is private and pinned through publication

r[lattice.operation_packs.runbook_catalog.scenario.pinned]
- GIVEN a fetch or installation staging directory
- WHEN its bytes are verified and published
- THEN the staging directory MUST be created mode 0700 through a handle on its parent
- AND the published entry MUST be renamed relative to pinned directory handles only while the staging path and the entry are still the inodes Lattice created and the target name is absent

#### Scenario: Pack byte drift refuses the pack and names every drifted bundle

r[lattice.operation_packs.runbook_catalog.scenario.drift]
- GIVEN a locked pack whose bundle bytes no longer match the pack declaration, before admission or after admission and before the first bundle is published
- WHEN `lattice pack catalog-install` runs
- THEN it MUST install nothing and exit non-zero
- AND the report MUST name each drifted bundle as `pack_bytes_drift` and every other bundle as `pack_refused`
- AND every bundle's bytes MUST be captured and verified before the first bundle is staged or published

#### Scenario: Installation keeps the loader's catalog limit

r[lattice.operation_packs.runbook_catalog.scenario.capacity]
- GIVEN a catalog whose bundle directories would exceed `MAX_CATALOG_BUNDLES` with another installed bundle
- WHEN `lattice pack catalog-install` previews or installs
- THEN each bundle beyond the remaining capacity MUST be rejected as `catalog_capacity`, checked again before each rename
- AND the existing catalog MUST stay loadable without a `limit_exceeded` diagnostic

#### Scenario: One bad bundle does not hide the others

r[lattice.operation_packs.runbook_catalog.scenario.partial]
- GIVEN an admitted pack in which one bundle has a wrong declared identity, a compatibility mismatch or a stale bundle-internal pin
- WHEN `lattice pack catalog-install` runs with `--confirm-create`
- THEN that bundle MUST be rejected with its own diagnostic code and none of its bytes published
- AND every other valid bundle MUST still be installed, and the command MUST exit non-zero

#### Scenario: Installation grants no authority

r[lattice.operation_packs.runbook_catalog.scenario.authority]
- GIVEN a bundle installed from a runbook catalog
- WHEN its workflow is saved and run without an operator-selected authority profile
- THEN authority inspection MUST still list the grants the workflow requires and the run MUST be refused before any effect
- AND fetch, install and list reports MUST state `grants_authority: false`
