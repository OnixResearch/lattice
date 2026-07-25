# Bounded Exec Radicle source cutover evidence

Observed: 2026-07-25

## Accepted producer evidence

- Bounded Exec publication revision: `7f7bd84`
- Publication receipt type: `bounded-exec.radicle-publication.v1`
- Publication receipt BLAKE3: `6eab0250b4bfefb6f6e91ec75d9786a76ded20bc052e2e624b44c6bc5b1cc9f7`
- OnixOS publication acceptance revisions: `baaafde`, followed by focused-flake correction `b5ba554`
- OnixOS admitted catalog BLAKE3: `f132c01a9d0edb4141201146c1dd5dc1eacfe77a2b080e6f99b2c4181d856655`
- Lattice typed source-admission BLAKE3: `c7707d1fb3367e752ddde42ac1f89df1808551b9b7560b29564a959d911aabe3`
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

Post-cutover focused results and final repository checks are recorded before the
change is archived.

## Non-claims

This cutover does not prove Radicle protocol correctness, Bounded Exec
correctness, host sandboxing, consumer release readiness, CI correctness,
third-party GitHub independence, or whole-stack GitHub independence. Historical
GitHub links outside current Bounded Exec acquisition remain documentation or
unrelated dependency inputs, not a fallback for this source.
