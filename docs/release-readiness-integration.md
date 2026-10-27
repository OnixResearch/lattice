# Lattice release-readiness integration

Lattice owns the release selection; Cairn owns assessment and deterministic
receipt execution. The selection must be exported from the reviewed Nickel
source using an admitted immutable Cairn producer with
`Contracts.ReleaseReadinessPolicy` and the `mcp_agent_smoke` replay command kind.
Do not replace the pinned source with a sibling checkout or an unpublished
revision. The cached older native binary can inspect MCP/corpus inputs but cannot
implement the new consumer-selection contract.

## Observe actual MCP inputs

After all source, policy, dependency and lifecycle closeout edits, run from the
Lattice root inside the pinned development environment:

```sh
scripts/release-readiness-inputs.rs "$NATIVE_CAIRN"
```

`NATIVE_CAIRN` is the exact admitted native executable, not the developer Nix
wrapper. The script executes the actual native MCP call `cairn_validate`, with
explicit repository root and generated policy. It retains the raw MCP envelope,
stdout/stderr identities, exit status, executable digest, exact argv and a source
manifest in `target/release-readiness/`. It derives completion status only from
both the observed process status and `structuredContent.valid`. A CLI-shaped
receipt is not accepted as MCP evidence. Failure is retained as non-passing; no
passing status or heavyweight gate result is inferred.

The source manifest covers maintained source, crates, tests, scripts, examples,
docs, lifecycle, policy, release vectors and root manifests/lockfiles, excluding
nested build/VCS directories. Source and policy must remain unchanged while the
observation executes. The script removes old completion inputs before a new
attempt so interrupted or malformed observations cannot silently reuse them.
Regenerate after any covered source or policy edit. Generated target files are
local observations, not checked-in fixtures or completion authority.

The policy declares these actual input paths:

- `target/release-readiness/mcp-contract.json`
- `target/release-readiness/mcp-results.json`
- `fixtures/release-readiness/canonicalization.json`

The completion contract selects only the observed native MCP lifecycle
validation. Its BLAKE3 identities bind the exact command, policy, source manifest,
raw evidence and observed process facts. It does not claim that tests, Octet,
Nix, or any remote service ran. Cairn smoke separately executes its catalog,
completion-validation, invalid-argument and default mutation-denial flow against
these real inputs.

## Assess and replay

```sh
policy="$PWD/cairn-policy/generated/cairn-policy.json"
"$NATIVE_CAIRN" agent-receipt smoke-mcp --root "$PWD" --policy "$policy"
"$NATIVE_CAIRN" preserves-corpus update --path fixtures/release-readiness/canonicalization.json
"$NATIVE_CAIRN" determinism coverage-audit --root "$PWD" --policy "$policy"
"$NATIVE_CAIRN" release-readiness --root "$PWD" --policy "$policy"
"$NATIVE_CAIRN" determinism replay --root "$PWD" --policy "$policy" \
  --group release --out target/release-readiness/replay.json
```

Do not run final replay before the integrated tree and real MCP inputs are final.
The seven owned replay cases are status, validate, MCP catalog, MCP status, MCP
agent smoke, canonicalization freshness and release readiness. Status and validate
include cross-root comparison; target-input release cases run read-only repeated
comparisons in place. The consumer surface inventory describes those actual
Lattice release surfaces. It does not promise Cairn self-host legacy migration,
agent fixture, or demo-change fixtures that Lattice does not own. Producer
self-host defaults remain unchanged.

The maintained corpus is authored for Lattice workflow exchange values. Object
key order is canonical; node sequence order and absent-versus-null output retain
distinct identities. Assessment checks freshness without changing expected
hashes. A reviewed vector change may be refreshed separately using
`preserves-corpus update --execute`; that mutation is never part of readiness.

## Required external gates and limits

Readiness lists the actual maintained Lattice gates:

```sh
scripts/check.rs
octet-precommit-check
nix flake check -L --option secret-key-files ''
```

Listing is not execution evidence: run and retain each gate's own results.
Cairn's optional semantic ledger, stack/function provenance, authentication and
matrix checks retain explicit skipped/disabled or supplied-invalid reports.
`onix_release_ready` can truthfully remain false when authenticated release
requirements are not enabled/supplied, even if local assessment passes. No gate
is disabled to remove a failure, and stable replay of a failing receipt does not
make that receipt pass or authorize a release.
