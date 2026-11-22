# Design: Operator-authorized OMP and catalog repair ports for the standalone CLI

## Context

Baseline: Lattice `7fb6dc47`. The repair loop (`cli::repair::execute`) takes `Ports { worker, checks, clock }`; `repair::admit` refuses a policy without them as `repair_ports_unavailable` under the per-work lock before the fence record. The single-shot route launches the pinned OMP through `candidate_worker::omp::run` with the attempt's re-admitted workspace boundary and bubblewrap provider. The standalone effect-authority profile (`lattice.standalone-effect-authority.v1`) binds a workflow revision, a machine identity, the Choregraph policy reference and closed grant tuples.

## Goals and non-goals

Goal: let `lattice implementation-attempt run` run repair with the pinned OMP worker and operator-pinned checks, only under an exact operator grant, reusing the loop, the launcher and the boundary. Non-goals: a second launcher, a new loop, served routes, live qualification.

## Decision 1: operator ports beside embedder ports

`cli::Ports` is unchanged. `run_command(args, config)` passes `repair::operator::Standalone` (configuration path, `--repair-checks` catalog, `Launch`, `checks::Execute`, clock) into `execution::run_with`; `run_command_with_ports` and every recovery path pass none. `repair::admission::admit` prefers embedder ports, then operator ports, else `repair_ports_unavailable`. Rejected: changing `Ports.repair` into an enum, which breaks every embedder construction.

## Decision 2: the grant

The profile must name `blake3:<worker profile revision>` (which frames the OMP pins, harness flags and policy, so the catalog's executor and checks too), the machine identity `blake3_reference("lattice.implementation-attempt-operator-repair-worker.v1")`, and hold both `{1, lattice/agent, lattice://agent/1, agent/execute}` and `{2, lattice/shell, lattice://process/2, process/execute}`. The pure `standalone::core::check_subject` checks expiry, identities and exact tuples; `standalone::check_subject` evaluates the profile. The configuration is loaded only for a repair request, so other runs keep their behaviour. Every step rechecks the same path and identity and the admission expiry under the fence lock before its intent; a failure is `authority_withdrawn` with the new cause `effect_authority`.

## Decision 3: the OMP worker port

Each step rebuilds the attempt's worker request (`session::launch::worker_request`), sets its deadline to the loop's remaining time rounded up within the admission bound, sets the OMP profile's base to the workspace's current snapshot (the parent revision) and its prompt to the admitted prompt plus the parent revision and admitted classes, and launches on a scoped thread while the calling thread polls the persisted cancellation request. A launcher panic propagates (a lost controller). Raw events and the launcher observation are written once as `repair/omp-step-N.{jsonl,json}`. Rounding up lets a worker that runs to its deadline return past the elapsed bound, which the loop classifies as `budget_exhausted` (`elapsed`) unchecked.

## Decision 4: outcome mapping

`candidate_worker::omp::stop::of` reads how a `Failed` launch stopped (`Exited { status_code }`, `Deadline`, `OutputLimit`, `WriteBoundary`) from the launcher's own named blocker constants in `candidate_worker::{execution,completion}`, so no caller matches free-form strings and `candidate_worker::Outcome` is unchanged. `Produced`/`Unchanged` are `exited` 0, a non-zero status is `exited` with it, host deadline and output stops are `exited` 137, `Cancelled` is `cancelled`; a stop without status, a write outside the boundary, an unrecognized failure, `Blocked`, `Interrupted` and `Unknown` (including an incomplete OMP terminal) are `unknown`. Blocked launches are `unknown` because the loop has no not-started class and must never retry blindly. The pure modules live at `repair::step::operator` and `repair::checks::catalog`.

## Decision 5: the check catalog

`lattice.implementation-attempt-check-catalog.v1` (at most 64 KiB, 16 sorted entries, 32 arguments of 4 KiB) pins a bubblewrap sandbox and each check's program by absolute `/nix/store` path and BLAKE3, its arguments and failure class. `configuration` is a domain-separated BLAKE3 of the entry; `check_executor` frames the schema, the route, the sandbox pin and the 1 MiB output bound. The shell reads it outside the attempt workspace, verifies every pin's bytes, and requires exactly the policy's executor and configurations. A check runs through `candidate_worker::run` with the copy as root, the worker's read paths read-only, no write path, no broker and no network; `Unchanged` passes, a non-zero exit fails with the entry's class, and anything else is unknown.

## Verification

Shell tests drive the real `run`, `recover`, `inspect` and `export-candidate` with a scripted fake OMP executable through `omp::run_scripted` (the launcher's argv, process, capture, JSONL and completion path without bubblewrap and broker) and catalog checks of a pinned Nix store shell without bubblewrap. Results are in `evidence/verification.json`.
