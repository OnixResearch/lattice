# Octet baseline migration

The dependency pin changed the Octet profile and configuration identities. It replaced all accepted stable identities in both repository scopes.

A detached control worktree at parent revision `5491f4f` reproduced the drift before the storage-quarantine implementation:

- root baseline: 846 accepted identities became stale
- pure-core baseline: 23 accepted identities became stale
- root control extras outside the accepted baseline: 232
- pure-core control extras outside the accepted baseline: 8

The reviewed migration tool matched each old finding to a current semantic peer. It normalizes only line-sensitive assertion-density text, file-length text, and the expected module-file count change.

Migration result:

- root baseline: 846 to 846, with zero baseline growth
- pure-core baseline: 23 to 23, with zero baseline growth
- current root extras: 231
- current pure-core extras: 8
- root semantic improvement: one finding
- pure-core semantic growth: zero findings

The migration requires distinct current identities. Its first dry run exposed 23 duplicate-selection collisions and did not produce an accepted gate. Deterministic distinct peer selection corrected that defect.

Final `nix develop -c octet-precommit-check` result:

- root: 846 accepted, 846 current accepted, zero new, zero stale
- pure core: 23 accepted, 23 current accepted, zero new, zero stale
- verdict: PASS

The migration did not add a warning budget, disabled lint, or finding baseline entry.
