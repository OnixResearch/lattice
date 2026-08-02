# Design: Enforce Lattice core and shell crate boundaries

## Context

Lattice has useful local FCIS seams. Examples include scheduler admission and shell modules, orchestration core and shell modules, replay helpers, and several adapter traits. One Cargo package still grants every module access to the package dependency set. `WorkflowMeta::new` and `Workflow::touch` also obtain time from `chrono::Utc::now()`.

## Goals

- Make dependency direction a Cargo and Octet property.
- Keep deterministic decisions testable without stores, runtimes, network services, processes, environment variables, clocks, or UI state.
- Preserve accepted wire, persistence, replay, and command behavior.
- Use a staged move that can coexist with active scheduler and orchestration changes.

## Decisions

### 1. Add one bounded Lattice product core

Create `lattice-core` as the first compiler-enforced product core. It owns only Lattice decisions and DTOs that need shared deterministic use. Choregraph remains the authority for Choregraph-owned graph and machine behavior.

The first extraction cohort is:

- workflow metadata and workflow mutation rules;
- schedule validation, due-run evaluation, occurrence admission, settlement, retry, abandonment, and recovery classification;
- replay timeline, cursor, event, bundle, and diagnostic decisions;
- selected orchestration state transitions, retry admission, and replay decisions after active ownership conflicts are resolved.

A module moves only when its public inputs and outputs are owned data or reviewed core dependency types.

### 2. Supply temporal facts explicitly

Core APIs accept a typed timestamp or epoch value. They do not call a wall or monotonic clock. The existing shell package can retain convenience constructors that obtain time and call explicit core constructors such as `new_at` or `touch_at`.

Persisted timestamp values and ordering semantics remain unchanged.

### 3. Keep effects in existing shells

The current package remains the imperative shell. It owns:

- redb transactions and recovery;
- filesystem and capability operations;
- environment and credential lookup;
- HTTP, webhook, and provider dispatch;
- bounded process execution;
- Tokio tasks, cancellation, and timeouts;
- Nickel evaluation, CLI rendering, and TUI mutation.

Shell code maps observations into core inputs, executes accepted effects, and maps results back to existing public forms.

### 4. Enforce topology instead of relying on naming

Declare `lattice-core` as a core crate and the existing package as shell and adapter authority in Octet architecture policy. Core dependency checks reject runtime, storage, host I/O, CLI, UI, and ambient-authority crates.

The architecture rail includes positive fixtures and negative fixtures that inject a clock, environment, filesystem, process, network, store, or Tokio dependency into core-shaped source.

New core violations cannot enter the accepted Octet baseline.

### 5. Preserve compatibility through adapters

Before each move, freeze representative serialized bytes, BLAKE3 identities, schedule occurrence outcomes, replay diagnostics, and orchestration transitions. Compatibility adapters preserve current public field names and shell-facing function behavior where callers still need it.

The migration proceeds by domain. It does not move scheduler, replay, and orchestration code in one commit.

## Alternatives

### Keep module-only boundaries

Rejected. Module layout cannot prevent a later core module from importing a host dependency already available to the package.

### Create one crate for every module

Rejected. It increases package and adapter overhead before independent ownership is clear.

### Require immediate `no_std`

Deferred. Compiler-enforced dependency direction and explicit observations provide the first required boundary. A later change can tighten suitable modules to `no_std`.

## Risks and Controls

- **Active changes touch the same modules**: inventory ownership first and move source only after conflicts are resolved.
- **Adapters duplicate decisions**: add source and parity checks that permit conversion only, not a second policy implementation.
- **Timestamp migration changes bytes**: freeze canonical fixtures before changing constructors.
- **Core becomes a product dumping ground**: enforce an API and dependency budget and require domain ownership for each export.
- **Static success is overclaimed**: evidence states that architecture checks prove source shape and dependency direction only.

## Verification

Run baseline and post-change tests for workflow, scheduler occurrence, replay, orchestration, serialization, and shell integration. Run compile-pass and compile-fail architecture fixtures, Octet on all relevant targets, `scripts/check.rs`, Cairn validation and gates, and the relevant Nix checks.
