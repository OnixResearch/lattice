# Bounded Exec Radicle source cutover evidence

Observed: 2026-07-25

## Accepted producer evidence

- Bounded Exec publication revision: `7f7bd84`
- Publication receipt type: `bounded-exec.radicle-publication.v1`
- Publication receipt BLAKE3: `6eab0250b4bfefb6f6e91ec75d9786a76ded20bc052e2e624b44c6bc5b1cc9f7`
- OnixOS publication acceptance revisions: `baaafde`, followed by focused-flake correction `b5ba554`
- OnixOS admitted catalog BLAKE3: `f132c01a9d0edb4141201146c1dd5dc1eacfe77a2b080e6f99b2c4181d856655`
- Lattice typed source-admission BLAKE3: `c7707d1fb3367e752ddde42ac1f89df1808551b9b7560b29564a959d911aabe3`
- Lattice cutover receipt BLAKE3: `a5a1c7dd49729ef8d131b1bea57f3557c6dc5ed665fc04d44caf9a67a2bf04f9`
- Canonical RID: `rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo`
- Reviewed revision: `29dac88ecded94457572db3fdfaaaab95fa91525`
- Source archive BLAKE3: `4fbbf8f0749262469f00748e04c775180488dba800303f139172656d25931927`

The producer receipt is staged with its Lattice consumer field `pending`; that
is the correct pre-cutover boundary. OnixOS accepted the receipt only after both
selective seeds and both single-seed outage drills matched the exact object and
source hash.

## Baseline before source edits

The existing GitHub-backed pin already named the reviewed revision. Before
changing declarations or locks:

- `cargo test --test test_repository_validation`: 6 passed
- `cargo test --test test_local_process`: 21 passed
- `checks.x86_64-linux.bounded-exec-source`: passed

No process policy or product source was changed by the cutover.

## Generated source transition

Cargo now names
`https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git` at the reviewed
revision. Nix names the same endpoint and revision through `git+https`, and
Crane matches Cargo's generated source identity exactly.

`Cargo.lock` was regenerated with Cargo:

```text
cargo update -p bounded-exec --precise 29dac88ecded94457572db3fdfaaaab95fa91525
```

Cargo reported removal of both GitHub-backed Bounded Exec packages and addition
of `bounded-exec` and `bounded-exec-core` from the Radicle-backed HTTPS URL at
the same revision.

`flake.lock` was regenerated with the targeted owning command:

```text
nix flake lock --update-input bounded-exec
```

The generated Nix node retains revision
`29dac88ecded94457572db3fdfaaaab95fa91525`, revision count `3`, and NAR hash
`sha256-BVmqyUYyoNpY6LfOABxwPO3DY88ZtXeKNg3TPoGCcL0=` while changing only the
Bounded Exec Git URL to the Radicle-backed HTTPS endpoint.

## Validation boundary

The pure repository validator compares Cargo, Cargo.lock, Nix, flake.lock,
Crane substitution, producer evidence, RID, endpoint, revision, NAR hash,
source archive hash, and no-fallback policy. Negative fixtures cover path
overrides, endpoint drift, revision drift, stale GitHub source, altered lock
identity, malformed producer evidence, and altered source identity. An
unavailable or substituted Radicle source fails closed with stable diagnostic
`source.bounded_exec_identity`; no alternate Bounded Exec URL is configured.
The thin Nix shell additionally requires both Bounded Exec crate manifests and
verifies the Nickel-authored admission JSON and BLAKE3.

## Post-cutover validation

The source-specific and behavior rails passed after lock regeneration:

- `cargo test --test test_repository_validation`: 6 passed
- `cargo test --test test_local_process`: 21 passed
- `checks.x86_64-linux.bounded-exec-source`: passed
- Cairn proposal/design/tasks gates and `cairn validate`: passed

The first broad `scripts/check.rs` run encountered a flaky simultaneous webhook
fixture; its exact rerun passed. A clean detached worktree at `d69ec49` then ran
the broad script without unrelated working-tree edits and reached a pre-existing
traceability fixture mismatch:

```text
all_repository_validation_rows_are_exact_byte_bound_and_unpromoted
left:  3d8e7511aa74823762bd852f80bb1b364acd4fc4ea77cfc9f4a259e640a303cd
right: f05f3ef4191db523c0902548398fa2680a6f9cbfaba0f5c25b43cbcf29d7f4c0
```

The clean-worktree Octet rail also retained its unrelated baseline blocker:
`892 accepted, 0 current accepted, 892 new, 892 stale`. The broad flake rail
therefore remains non-passing through existing Octet/traceability checks, while
the focused source, lock, format, behavior, and Cairn checks establish the
bounded cutover claim. No release-readiness claim is made.

## Non-claims

This cutover does not prove Radicle protocol correctness, Bounded Exec
correctness, host sandboxing, consumer release readiness, CI correctness,
third-party GitHub independence, or whole-stack GitHub independence. Historical
GitHub links outside current Bounded Exec acquisition remain documentation or
unrelated dependency inputs, not a fallback for this source.
