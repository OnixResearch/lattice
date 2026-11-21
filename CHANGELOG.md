# Changelog

All notable changes to Lattice are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
Semantic Versioning. This project is pre-1.0, so minor releases may change
interfaces.

## [Unreleased]

### Added

- Qualification attachments for agent revisions (Animus #18, Lattice #25).
  `lattice agent-revision promote` and `rollback` accept a repeatable
  `--qualification-attachment PATH`. Each file must be a regular,
  non-symlinked file of at most 16 KiB. The attachment port answers through
  the vendored `animus_runtime::qualification::qualify` for the exact
  definition, bare revision, task family and required class. `Qualified`
  keeps its reference, class, protocol, verifier, observation time and
  `fixture`/`live` provenance label (recorded, never gating), and
  each producer refusal maps to the same `not_qualified` reason. The named
  `--qualification-ref` must equal the answer's reference. A qualified answer
  now carries the subject it was produced for, taken from the decoded
  attachment, and promotion and rollback refuse an answer for another
  definition or revision (`subject_mismatch`) or task family
  (`scope_mismatch`) even when it names the requested reference. Without
  attachments the CLI still refuses as `qualification_unavailable` (owner
  `animus#18`). The vendored Animus snapshot moves to
  `8d6f67d38509899402df4d62e2d54321ff047f06`, adding only the qualification
  module. A favorable attachment is evidence, not promotion authority, and
  live efficacy is not claimed. See `docs/agent-revisions.md`.
- Immutable agent revision lifecycle (#25, sections 1 to 5). `lattice
  agent-revision` resolves an inert bundle from an exported
  implementation-attempt request, records the desired revision, prepares it
  through an injected preparation port, promotes or rolls back the revision
  selected for new sessions with an exact `qualification_ref`, an expected
  rollout generation and an idempotent request identifier, revokes revisions,
  attachments and sessions, pins sessions and cleans up unretained artifacts.
  The revision is the existing profile revision, used verbatim; the pure core
  is `lattice_core::agent_revision`, and lifecycles persist in a
  version-checked `agent_revision_lifecycles_v1` table. Bounds count only
  what is still referenced (live sessions, a sliding window of receipts,
  revisions nothing references are forgotten), so admission, desire,
  promotion and rollback never exhaust for good, and revoking a referenced
  target is never refused. Every refusal, including a refused
  `admit-session`, exits non-zero. The session-admission seam takes the
  admission mode its caller records, so sessions admitted before a
  definition became managed stay unmanaged. The standalone CLI
  reports preparation as `unknown` and refuses promotion as
  `qualification_unavailable` (owner `animus#18`). Implementation attempts,
  their revisions and the inventory are unchanged. See
  `docs/agent-revisions.md`.
- Coordinator transport for Kiln #45's `kiln-lattice-dispatch --worker-request`
  (#6). Implementation-attempt requests may declare `network:false` and the
  coordinator's `dispatch_plan` identity; `network:true` is refused. A run
  records its attempt per work and fence before any binding or intent, and
  stale fences, crossed attempts and substituted request bytes are refused with
  a typed `lattice.implementation-attempt-refusal.v1` record before any effect.
  Recovery that finds no intent closes the attempt, so a late or duplicate run
  can never dispatch. Every observation, including binding refusals and the new
  `not_started_closed_by_recovery` status, carries the exact envelope and
  request identity. A `cairn_artifact` source must name its exact
  `change/stage/artifact`; issue and task identity bytes are unchanged. See
  `docs/candidate-worker.md`.
- Worker inventory v2 facts for Kiln #60 (#24, #25 slots only). Each profile
  carries a definition identity that is stable across revisions
  (`lattice-worker-definition-v1`). It also carries declared `task_families`
  and `tools`, which stay `unknown` when undeclared and are never empty.
  Lattice #25's desired, prepared, qualified, selected and pinned revisions
  appear as owned unknowns. Profiles and non-terminal instances carry bounded
  closed-vocabulary wait reasons that name only published routes. Instances
  carry their definition identity and single-shot liveness (`not_started`,
  `exited` or `unknown` with `unobserved`/`stale`), never idle or live. An
  optional typed `lattice.worker-capability.v1` declaration in
  `contracts/worker-execution-binding.ncl` and the implementation-attempt
  request supplies the definition name and capability sets. It is part of the
  profile revision only when present, and malformed declarations are refused
  before any effect. See `docs/worker-inventory.md`.
- Default-off declared preference factors on projected role-protocol choices
  (#16). `admit_annotated_projection` admits a published Choregraph
  preference-annotation cohort, verifies its receipt and annotated local set, and
  records factors and world variables only under `record-declared-factors`.
  Exact-label admission, cursors, ownership and authority are unchanged. The
  default profile reports `factors-ignored-profile-off` and leaves the cohort and
  session bytes identical to the unannotated intake. The game mapping is reported
  as not formalized.
- An optional typed policy operation (#23): a versioned profile binds Animus's
  frozen-policy leaf (vendored at `1c370d50`) to one declared Condition node,
  validates receipts independently, resolves typed outcomes through a declared
  fallback, persists bounded decision records in run details (including runs
  that suspend at and resume from approval gates) and replays them without
  adapter work; a resume whose stored decision no longer matches the current
  profile or artifact is refused as stale. Disabled profiles leave runs
  byte-identical; the fixture comparison makes no improvement claim.
- A topological DAG rows view with bounded fork/join lanes, synchronized graph
  selection, live result metadata, and linear/fork-join/diamond snapshots.
- A bounded local planner/generator/evaluator example that runs real Rust builds
  and tests, retains failed-round handoffs, and caps aggregate retries and resources.
- An explicitly selected pinned-reference OMP JSONL candidate profile and typed
  post-start process/capture/teardown observations. The reviewed OMP 18.18.7 build
  passed real gpt-6.1-sol candidate production and local terminal-failure/cancellation
  qualification. Durable continuation remains unsupported by the pinned producer
  contract; independent acceptance and deployment qualification remain separate.
- Opt-in monotonic Octet baseline pruning, checked against complete admitted
  warning occurrences and the baseline identity-count bound so resolved
  findings cannot conceal additional debt.
- Bounded export of exact retained implementation candidates, with owned output
  modes and replay integrity checks; export never dispatches a worker, accepts
  a candidate, or integrates changes.
- An attempt-local model broker over a private Unix socket and network namespace,
  exact endpoint/model admission, host-only credential resolution, bounded calls
  and immutable runtime byte pins. Live OMP qualification remains explicit.
- Material-free configured agent profiles and persisted configuration provenance;
  these do not advertise worker ownership, compute reservations or service readiness.
- Captured workflow-source drafts with durable revision CAS, explicit admission,
  exact source archives and materialized-fork tracking.
- Native graph construction for captured Nickel programs (#20, #21):
  `import --construction` and `authoring draft --construction|--parameterized`
  evaluate captured modules in the companion `lattice-closed-eval-worker`
  (nickel-eval's captured-module profile) and compile them through Choregraph's
  `choregraph.graph-construction.v1` into the existing admission path. Workflows
  retain the observed module closure, template/instance origins (`authoring
  origin`, TUI navigation) and exact v3 runtime bindings; effects stay denied
  until a separate exact v3 grant is selected. Qualification runs on the host with
  `nix run .#construction-runtime-qualification -- NEW_DIR`, because a Nix build
  sandbox cannot present root-owned store executables.
- Worker compute binding for implementation attempts (#24 compute half, #13
  provenance): the `lattice.compute-allocation-observation.v1` consumer DTO for
  Aspen `aac7b5ba` compute-capacity records with bounded pure validation, a
  read-only injected `ComputeAllocationPort` (the standalone CLI uses
  `UnavailableComputePort`; embedders use `run_command_with_ports`), pure
  admission and pre-spawn revalidation, `execution-binding.json` written before
  the intent, `implementation-attempt inspect`, and `Scope::Attempt` usage
  provenance. Producer references stay opaque; no live provider, enforcement or
  serving qualification is claimed.
- Serving-owner request admission for implementation attempts (#24 serving
  half, #13 usage bindings): an optional `lattice.serving-binding.v1`
  declaration frozen into the execution binding, a read-only injected
  `serving::OwnerPort` (standalone `UnavailableOwner`; embedders pass
  `Ports` to `run_command_with_ports`), and Animus #17's pure serving gate run
  per request in the attempt-local model broker. Each request leaves a
  write-once dispatch intent and `lattice.model-request-observation.v1` record,
  summarized into per-service `served` model-request coverage of the attempt
  usage provenance (known versus unknown usage, deduplicated, never reset).
  Animus is re-vendored at `1c370d50` with `animus-frozen-policy`. Record
  evidence labels derive from the port's provenance (`live_endpoint` only for
  an embedder-admitted upstream call, otherwise `fixture` or
  `no_upstream_contact`). The usage summary carries an `overrun` block
  (reserved envelopes, known tokens, per-request overruns, unknown usage,
  refused requests, `is_exact_ceiling` false whenever usage is unknown); an
  optional `stop_on_overrun` refuses later served requests as `overrun-stopped`
  before any port call, and the broker's exhausted request budget is recorded
  once as `budget-exhausted`. `implementation-attempt inspect` reports that
  summary and serving readiness as its actual cause (`no_serving_port`) or the
  injected port's latest snapshot readiness and freshness. Fixture evidence
  only; no live serving owner, endpoint or device was exercised.
- An opt-in pure interaction-net worker profile (`lattice::inet`, #14 and #22) that
  runs the pinned Noble `2677ec88` `ngraph-net-i64/3` worker through the existing
  `bounded_process` operation-pack route. It admits only the canonical acyclic
  two-input wrapping-`i64` subset, re-measures artifacts before dispatch, lowers
  only the producer's fuel/input/retained/output ceilings, binds the report's
  producer revision, contract, limits and input bytes, and publishes results and
  non-authoritative diagnostics through operation data. A semantic interaction
  count is checked only when the receipt names the model theorems and pinned
  proof sources and two fresh admitted evaluations agree on output and count;
  otherwise it stays observed. Not adopted by default; no multicore, device or
  refinement claim.
- A finite, deterministic `workflow-intent` compile/compare boundary over explicit
  host-normalized facts. Plans are inert; typed Cairn/Kiln producer integration
  and execution acceptance remain separate prerequisites.
- An opt-in artifact-bound tiled-render consumer over Aspen's pinned native
  batch/render contracts, with durable unknown-outcome recovery and exact
  accepted-content verification. Remote aggregate-budget enforcement and usage
  telemetry remain unavailable from the selected producer.

### Changed

- Breaking: the implementation-attempt profile revision is now the
  `lattice-worker-profile-revision-v2` identity over profile-level
  configuration only, still spelled as bare hex. The base snapshot, task prompt,
  broker expiry and workspace root no longer change it; the exact request
  bytes bind them. Every revision value changes. Attempts retained under the
  earlier framing are refused with `profile_revision_superseded` and are never
  re-executed or recovered; prepare new work again (#6, Lattice #25).
- Breaking: the implementation-attempt issue source kind is spelled
  `github_issue`, the name Kiln sends, instead of the derived `git_hub_issue`.
  Its identity tag is unchanged.
- Breaking: `implementation-attempt inspect` prints `lattice.worker-inventory.v2`
  with `lattice.worker-profile-facts.v2` and
  `lattice.worker-instance-observation.v2`, and
  `RuntimeOptions::worker_profile_facts` returns profile facts v2. No v1
  inventory is emitted. Readers that deny unknown fields, such as Kiln's
  `worker_inventory` adapter, must move to v2. The later profile-revision
  framing change above replaces every revision value.
- Re-pinned Choregraph to `fcbae0aa` (Choregraph #4, default-off preference
  annotations). Its v1 receipts are byte-identical to `c3577352`; the source
  admission receipt, flake assertions and frozen role-protocol revision name the
  new revision. As with earlier re-pins, the frozen revision is part of every
  role-protocol cohort, so new session-plan identities differ from `c3577352` ones.
- Adopted published Choregraph `c3577352` and nickel-eval `f0ea0d2a` from their
  canonical Radicle seeds. The producer carries the imported-JSON merge repair, so
  the vendored `nickel-lang-core` snapshot and root patch are removed, along with
  52 transitive crates the crates.io `nickel-lang` REPL/formatter pulled in.
  Construction and restricted-lowering callers use the responsibility-qualified
  Choregraph APIs; persisted workflow/source archive keys are unchanged.
- **Breaking:** implementation-attempt requests are `lattice.implementation-attempt-run.v3`.
  Since 0.1.0 they dropped shared-network/namespace fields (scoped broker
  profiles require explicit readmission) and now require `worker_compute` and
  `execution_binding`; v1 and v2 requests are refused before any effect.
- Tests that need the immutable captured worker or v3 runtime are ignored in plain
  Cargo and `nextest` and run only in the host qualification; captured evaluation
  fails closed when no worker is selected.
- Re-keyed the root and `lattice-core` Octet baselines after the root manifest
  changed (the pinned Octet hashes `Cargo.toml` into finding identities). New
  findings were fixed in code; the semantic finding multiset only shrank
  (root 1055 → 1052, core 25 → 25).
- Removed stale repository-verifier, source-byte and marker-placement snapshot
  tests instead of re-pinning formatting, URL literals, counts, byte hashes or
  private-module locations. Actual Cargo/Nix checks and native lifecycle
  behavior remain the validation surfaces.
- Corrected shell-preview activation fixtures to request only their actual
  round/duration accounting; production worker/model admission remains closed
  without a qualified worker profile.

- Orchestration retains reservation, conservative charges, host duration, partial
  provider observations, and unknown usage separately before settlement. Actual
  cancellation and successful completion settle atomically without treating a
  late cancellation request as observed termination.
- Candidate capture includes admitted ordinary agent/build files and rejects
  unsupported file types and modes instead of silently omitting source changes.
- Agent, worker, orchestration and TUI Rust APIs use owner-qualified names without
  deprecated aliases. Receipt constructors borrow named identity bindings, stored
  candidate files use named artifact addresses, and OMP event counts use `u64`.
  Existing persisted schema and artifact-key encodings remain unchanged.
- Private agent, worker, accounting, storage and TUI implementation phases are
  decomposed without changing producer pins or adding accepted Octet warning facts.

### Fixed

- Bounded local processes whose cancellation flag is already set return the
  existing `Cancelled` outcome without spawning. The pinned bounded-exec runner
  checked the flag only after spawn, so a pre-cancelled command could briefly run.
- Live file reads and image loads participate in unresolved-effect conflict
  scheduling; read-only authority is not treated as independence evidence.
- Role-protocol fanout preflights all adapter selections. Ambiguous HTTP responses
  and partial delivery retain unknown outcomes until exact receiver evidence.
- The standalone TUI rejects redirected stdin/stdout before terminal setup.
- Hosted check/release jobs omit dev/test debug symbols and release generated
  local Cargo build trees before isolated Nix checks, preserving the complete
  test rail and Octet JSON evidence rather than exhausting runner disk space.
- Refreshed the explicitly operator-admitted dependency/manifest lint cohort
  without growing complete per-owner warning budgets or weakening the maintained
  identity-only migration guard.
- Native render fault qualification terminates its owned local process sessions,
  including the bounded runner's nested process groups, and verifies SIGKILL
  rather than leaving a child observer holding the journal writer lock.
- Render qualification selects the packaged runtime instead of an oversized debug
  executable; the existing 256 MiB executor identity ceiling remains unchanged.
- Live TUI progress uses a checked whole-run bounded executor port and a bounded,
  backpressured UI mailbox; incomplete observations remain explicit without
  retrying execution or fabricating timing metadata.
- Implementation-attempt recovery verifies retained raw OMP events and their
  normalization before reporting a published result.
- Interrupted candidate workers retain available bounded process-event bytes as
  unknown-outcome evidence rather than discarding them with a post-start error.

## [0.1.0] - 2026-11-12

Initial public release candidate. The runtime, CLI, TUI, durable store, scheduler,
approvals, agent adapters, workflow contract, and local workflow exchange are
present but remain pre-1.0.

### Added

- Public release documentation: security policy, contribution guide, server
  deployment guide, release process, documentation index, and launch material.
- A tagged release workflow that runs the maintained checks, builds the Nix
  package, verifies `lattice --version`, and publishes a binary archive with
  checksums.
- `lattice --version` and public Cargo package metadata.

### Changed

- The optional Tile compose integration now lives in the separate `compose/`
  workspace. The default runtime, public checks, and release binary have no Tile
  dependency and no authenticated source requirement.
- Public build inputs use anonymous HTTPS. The transitive SSH-only sources for
  Artifact and Trellis are patched to their exact public revisions, and the Cairn
  and Octet flake inputs use HTTPS overrides.
- The release package no longer embeds the Cairn lifecycle toolchain; only the
  test rail sets `LATTICE_CAIRN_BIN`.

### Known limitations

- The workflow exchange service is a local Unix-socket service. It has no TLS,
  remote authentication, network listener, or multi-tenant isolation.
- Workflows run with host authority. Lattice is not a sandbox for untrusted
  workflows.
- The compose workspace requires authorized Tile source access and is not part of
  the public build closure.
- Three TUI authoring PTY regressions remain tracked by
  `complete-tui-nickel-authoring-parity` and are ignored by the release gate.
  The server release does not claim TUI authoring-parity completion; see
  [the release audit](.cairn/changes/prepare-public-release/evidence/release-audit.md).
- No hosted service, support SLA, or production-readiness claim is included in
  this release.
