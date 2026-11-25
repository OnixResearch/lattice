# Credential provider implementation evidence

## Producer admission

- Accepted archive commit: `b1ce46d4533c9d80b629ca932d65d947a8720c89`.
- Hidden-lifecycle layout migration: `c85fdb35e4e546edcafb368fcb32e0353bf65fc4`.
- Exact inspected local `origin/main` tracking revision: `51d2683776ab73d4fd36d3d9e6ced2beadf9793d`.
- `git merge-base --is-ancestor b1ce46d4533c9d80b629ca932d65d947a8720c89 origin/main`
  completed with status 0 in Animus.
- Accepted `.cairn/specs/credentials/spec.md` contains all six
  `credentials.secretspec.*` requirements, including `provider_conformance`.
- Archived `evidence.md` records positive environment/systemd/SOPS delivery,
  negative provider/worker/redaction fixtures, feature isolation, Nix check,
  accepted-spec sync, and archive receipts. These are upstream recorded results,
  not new local test runs.
- SDK cohort is SecretSpec `0.17.0`, commit
  `a8794e46ec9664a0e1a3869cc3105d0853937e48`, checksum
  `68498f9695bb3662c157b8fd4b4665a594f1157de022ff5b0f891af4c7ec75d2`,
  Apache-2.0, defaults disabled, `sops` only. Bounded-exec is
  `29dac88ecded94457572db3fdfaaaab95fa91525`, AGPL-3.0-or-later.
- The integration owner subsequently imported the exact Animus revision through
  `scripts/sync_animus.rs`. The generated source metadata now records the archive
  BLAKE3, SDK/host licenses, reviewed SDK feature selection and bounded-exec
  cohort. This local import is not anonymous acquisition or runtime acceptance.

## Native lifecycle admission

Using cached native Cairn, explicit Lattice root and
`cairn-policy/generated/cairn-policy.json`, all three gates returned
`valid: true`, `verdict: PASS`, `issues: []` before source edits:


The local remote-tracking reference and archived publication record are not a
fresh anonymous source-acquisition proof. The integration owner has separately
reported authenticated-only declared GitHub acquisition and unavailable Radicle
seed projections; no new source pin or public-build claim is admitted by this
workstream's repository inspection.

| Gate | Receipt |
| --- | --- |
| proposal | `b6f1198447ba513eaed35b629f7a78636c2ad5a9296aec93e8d63d347e9e9e34` |
| design | `08da35e59d997b847ef169154b3814a9f0b1e2e6efab3925cb64e0ee43b2928f` |
| tasks | `4a48da5f0a98ddcda444d76d5979e29267f4c7806e89f98f533d7fb7bb26dcf1` |

Persistent Bash calls stalled even for native `--help`; the workstream cancelled
only its own three stalled jobs and used direct Bun.spawn with GNU timeout,
concurrently captured stdout/stderr, and the same exact native binary/policy.
This did not bypass lifecycle policy or change a dependency pin.

## Implemented boundary

`src/credentials/config.rs` contains the always-compiled closed provider/host
DTOs, explicit declaration/field mappings, provider allowlists and pure shape
validation. `src/credentials/secretspec.rs` contains optional bounded pure
request adaptation, fixed local diagnostic conversion, material-free probes,
the injected wrapper and the real accepted Animus bounded host composition.
Trusted `SecretSpecHostConfig` is separate from public provider metadata and
supports explicit environment names, absolute systemd coordinates and explicit
SOPS guard/executable/named provider-input coordinates. Factory policy and
manifest reads are capped at 1 MiB; metadata and identities are admitted before
provider input access. Host manifests are trusted operator-owned immutable files;
this wrapper does not claim resistance to concurrent malicious host mutation.

Contexts describe exact current node, trigger-authentication or
trigger-invocation bindings; they do not issue authority. Authentication does
not assert downstream effect admission. Existing Choregraph dispatch authority
remains required at actual node effects. Context-free SecretSpec resolution
fails closed. `SecretValue`, `ResolvedSecretEnv`, Agent `ExecutionContext`, and
receiving process/HTTP request guards provide unconditional cleanup ownership;
field transfers preserve that responsibility rather than relying on success-only
cleanup. Exact/product Agent dispatch seals the selected backend/configuration,
reference mapping and provider policy before authorizing material resolution.

`tests/test_secretspec_credentials.rs` adds paired pure/config/injected-host
boundary fixtures. `examples/secretspec_delivery.rs` exercises the actual host
with explicitly supplied deployment inputs and compares transient delivery
without printing/hashing material. `docs/secretspec-credentials.md` gives
operator prerequisites, exact smoke arguments, migration/rollback and non-claims.

## Coordinated security verification: 2026-10-16

- Native Nickel exported the canonical policy with the dedicated `lattice-agent`
  contract. Current policy BLAKE3 is
  `0f63ce4bb3e707f53ea634a48211dad5215e1aeeec1209b25992020d6d2459ec`;
  the embedded Rust and standalone-profile identities were migrated together.
- External Nickel admission accepted the exact Agent tuple and rejected process
  ability substitution, wrong resource, the old policy identity, duplicate node
  grants, an unknown grant field, and zero expiry. This exercises the contract,
  not embedded evaluator compatibility or runtime authority dispatch.
- A transient native allocator observer compiled the actual AST-selected
  `ResolvedSecretEnv` and Agent `ExecutionContext` owners and Drop implementations
  against the cached zeroize dependency. It observed initialized value bytes
  immediately before deallocation for ordinary return, error return, unwinding
  and owner transfer. Both owners cleared their environment and redaction copies
  in all four cases; each corresponding no-Drop control failed all four.
  This is an isolated ownership proof, not a whole-consumer build or erasure claim
  for transport, child, allocator or OS copies.
- New permanent regressions cover denied/unavailable/exact/product Agent dispatch,
  backend/request/reference/policy drift, value rotation, redacted echoes,
  partial multi-slot resolution, cancellation, transport failures and one-shot
  requests. Their source is present; their execution is not claimed.
- Before exact local intake, `cargo test --locked --offline --lib` exited 101
  because the declared `animus-std-secretspec/Cargo.toml` was absent. The later
  repository-owned import cleared that missing-manifest prerequisite; it did
  not bypass it with a source override or proposed API copy.
- After intake, the same full consumer command reached Rust compilation and
  exited 101 with 24 errors in other pending integrations: GitHub CI module
  composition, Tile dependencies, component runtime/verifier dependencies and
  revision binding, and a missing TUI validation import. The module composition
  and import were then repaired; compilation has not been repeated merely to
  confirm the still-unadmitted producer dependencies.
- Optional-feature execution, whole-consumer integration and credential-free
  public-build acceptance remain unclaimed. Exact source and operational
  prerequisites are not replaced by structural gates or isolated smokes.
- Lattice owns maintenance of the credential/Agent boundary and paired permanent
  regressions. Repeat them through the documented repository Cargo commands after
  exact producer acquisition, sync and provenance admission; local ownership and
  external-contract observations do not authorize lifecycle completion.

## Verified local intake: 2026-10-16

The actual repository-owned importer, typed provenance export, three
normalization regressions and script formatting passed. Locked offline Cargo
metadata resolves the optional exact Git SecretSpec cohort with only `sops`;
the default graph excludes the Animus host but retains a separate registry SDK
through Basalt. No normal dependency path reaches either SDK/host from the four
reusable workspace cores. This verifies dependency and normalization boundaries,
not provider delivery or whole-consumer execution. Exact commands, identities,
resolver rationale and outstanding prerequisites are recorded in
[`evidence/local-intake-20261016.json`](evidence/local-intake-20261016.json).

The post-intake proposal/design/tasks gates and root validation also returned
`valid: true`, no issues, with `PASS` on each gate. Their exact native command
and receipt data are in
[`evidence/local-intake-gates-20261016.json`](evidence/local-intake-gates-20261016.json).
These structural results do not close the runtime acceptance tasks.

## Verification ownership and outstanding evidence

At the frozen workstream handoff, no Cargo build, source tests, formatter,
Clippy, repository check or Nix check had been run mid-flight, per the coordinated
integration contract. Subsequent parent-owned observations are recorded above.
Runtime checkboxes remain unchecked until integration exercises the changed path.
The parent owns feature/dependency/pin admission, typed Nickel/AppConfig
composition, all executor and trigger callsite cutovers, TUI parity, root README,
integrated redaction/persistence/history/replay evidence, final focused/full
checks, accepted-spec sync and archive. The new example is an executable smoke
instruction, not a claimed live provider result.

The coordinated proposal, design and task gates returned `valid: true`,
`verdict: PASS`, `issues: []`; root validation returned `valid: true`,
`issues: []`. These are structural observations, not runtime acceptance. Exact
commands, native ownership controls, admission outcomes and the consumer-build
blocker are retained in
[`evidence/security-boundaries-20261016.json`](evidence/security-boundaries-20261016.json).
