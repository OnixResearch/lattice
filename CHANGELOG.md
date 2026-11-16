# Changelog

All notable changes to Lattice are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
Semantic Versioning. This project is pre-1.0, so minor releases may change
interfaces.

## [Unreleased]

### Added

- A topological DAG rows view with bounded fork/join lanes, synchronized graph
  selection, live result metadata, and linear/fork-join/diamond snapshots.
- A bounded local planner/generator/evaluator example that runs real Rust builds
  and tests, retains failed-round handoffs, and caps aggregate retries and resources.
- An explicitly selected pinned-reference OMP JSONL candidate profile and typed
  post-start process/capture/teardown observations. Live worker and deployment
  qualification remain separate prerequisites.
- Opt-in monotonic Octet baseline pruning, checked against complete admitted
  warning occurrences and the baseline identity-count bound so resolved
  findings cannot conceal additional debt.
- Bounded export of exact retained implementation candidates, with owned output
  modes and replay integrity checks; export never dispatches a worker, accepts
  a candidate, or integrates changes.

### Changed

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

- Live file reads and image loads participate in unresolved-effect conflict
  scheduling; read-only authority is not treated as independence evidence.
- Role-protocol fanout preflights all adapter selections. Ambiguous HTTP responses
  and partial delivery retain unknown outcomes until exact receiver evidence.
- The standalone TUI rejects redirected stdin/stdout before terminal setup.
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
