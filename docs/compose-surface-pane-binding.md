# Pane-to-workflow metadata binding

Lattice can compose an explicit Tile pane observation with an exact workflow
identity without running that workflow. The pure `lattice-compose-binding` crate
is `no_std + alloc`; the std shell injects the observation and receipt publisher.
There is no executor, UI handle, ambient socket discovery, clock, or environment
access in the core. The only planned effect is `PublishMetadataReceipt`.

## Producer contract

The reviewed Tile contract revision is
`5e703953840bff72a91fec3e928682e9efb607ab`, consuming:

- `tile-sdk::TileClient::inspect_workspace`;
- `tile-protocol::{WorkspaceInspectRequest, WorkspaceInspectResponse}`;
- `WorkspaceInspectResponse::validate` and its bounded, revision-fenced,
  metadata-only pane projection.

Tile's archived `add-native-windowed-live-control-plane` change owns that
contract, especially its typed daemon workspace inspection decision and task.
The adapter validates the complete producer response before narrowing one exact
pane to lifecycle and focus. Required title, cwd, pane-content, raw-PTY,
persistence-field, socket-path, and secret omission flags must all be true.
`PaneReadResponse`, screen lines, perception output, and detection strings are
not consumed. Lattice does not copy the producer implementation or depend on a
sibling checkout.

`Cargo.toml` declares both producer crates from
`ssh://git@github.com/brittonr/tile.git` at that reviewed revision; `Cargo.lock`
retains the same exact source for `tile-sdk`, `tile-protocol`, and `tile-core`.
Acquisition through this authenticated SSH route was exercised. Anonymous HTTPS
acquisition was not established; building requires the declared source access or
an already admitted cache, not a sibling checkout.

Dependency-repair verification compiled this actual adapter source in a
disposable consumer, passed its three existing positive/negative tests, and ran
the real Tile projection with private title/cwd omission and Lattice request
admission/rejection checks. No daemon was started or contacted. This is adapter
and projection evidence, not a passing complete Lattice executable: the separate
component-runtime/verifier contract remains unresolved. Once that closure is
available, repeat the repository-owned adapter tests with
`cargo test --locked --package lattice --lib compose_surface::`. Lattice owns
this dependency binding and adapter; Tile retains observation authority.

The daemon authority BLAKE3 identity and connected client are explicit host
inputs. The source revision identifies the adapter's reviewed producer contract;
it is not cryptographic attestation of the running daemon binary. The injected
host remains responsible for binding its selected daemon authority to its
connection. Neither a response nor a receipt proves observation truth.

## Admission and identities

All DTO records are closed: missing and unknown fields fail. Requests are at
most 4096 bytes before JSON decoding. The exact producer revision, nonzero
session/pane/workspace revision, explicit daemon authority identity, canonical
workflow/capability/activation identities, and explicit metadata-binding
permission are required. Trigger shapes are closed: a matching lifecycle or a
focused pane. Stale/missing panes, source mismatches, unsatisfied triggers,
unknown/missing revisions, or denied capabilities cannot activate a binding.

The binding key is a domain-separated BLAKE3 hash of labelled, length-prefixed
source identity, workflow identity, trigger shape, and capability context.
JSON field order and supplied activation identity do not affect that key.
The metadata receipt additionally binds the activation identity and observed
lifecycle/focus, so different supplied activations have different receipt
identities. There is no ambient timestamp.

An accepted `activate` call invokes its publisher once. Failed admission invokes
it zero times. Publication failure is not retried automatically: it may be
uncertain whether an external sink retained the receipt. This is not a durable
exactly-once delivery claim. Receipts have a closed
`binding_activated_metadata_only` claim and a 4096-byte wire bound. They never
claim workflow correctness, observation truth, authorization to execute, or
product-effect success. `validate_receipt` compares every field with the exact
admitted plan, not merely a supplied hash.

## Real producer and consumer smoke

Run producer commands in the exact acquired Tile repository and consumer
commands in Lattice, using their pinned development environments. A Tile daemon
must already be explicitly started and owned by the operator; the consumer does
not start one or discover a default socket. Starting a daemon or creating its
pane is a producer setup effect, not an effect performed by Lattice composition.

Inspect an explicitly selected workspace through Tile's public daemon surface:

```sh
cargo run --locked -p tile --bin tile -- workspace --socket "$SOCKET" inspect \
  --session "$SESSION" --expected-workspace-revision "$REVISION"
```

Prepare a request from
`crates/lattice-compose-binding/fixtures/request.json`. Replace its daemon
authority, session, pane, workspace revision, workflow identity, capability
identity/permission, activation identity, and trigger with the exact reviewed
inputs. Fixture identities are examples, not producer or workflow authority.
Workflow identity names the binding target; this surface does not establish
that a workflow is installed or correct. A host requiring stored-workflow
admission must resolve and compare the exact workflow before calling activation.

The explicit std consumer uses the real pinned Tile SDK:

```sh
cargo run --locked --example compose-pane -- \
  "$SOCKET" "$DAEMON_BLAKE3_IDENTITY" "$REQUEST_JSON"
```

Successful output is exactly one bounded metadata-only receipt line. Changing
the expected workspace revision to a stale value, selecting an absent pane,
removing a required field, adding `raw_screen` or `effect_success`, clearing the
producer revision, using a different producer revision, denying metadata binding,
or selecting an unmatched lifecycle must fail without a receipt. A missing or
unknown field is rejected before connecting; source/revision/pane/trigger
rejection occurs before receipt publication. The connection and observation
have separate ten-second deadlines and sanitized producer failure diagnostics.

Focused regressions for the integration owner:

```sh
cargo test --locked -p lattice-compose-binding
cargo test --locked --lib compose_surface::tests
cargo check --locked -p lattice-compose-binding --lib
```

The matrix exercises all required-field removals and unknown-field additions,
source-identity boundaries, exact revision admission, canonical identity
sensitivity, plan determinism, activation identity separation, actual Tile
projection narrowing, stale/redaction/capability rejection, and receipt
claim/tamper rejection. A fixture check is not live producer/consumer evidence;
record the actual smoke result separately before lifecycle closeout.
