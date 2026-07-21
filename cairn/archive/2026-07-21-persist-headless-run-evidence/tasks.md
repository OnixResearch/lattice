## Phase 1: Dogfood and baseline

- [x] [serial] Run a fresh live duplicate-trigger drill and record one admitted instance/run plus the missing-detail observation. r[lattice.workflow_execution.headless_run_details]
- [x] [serial] Run focused headless and run-history tests before changing shared conversion or persistence behavior. r[lattice.workflow_execution.headless_run_details]
- [x] [serial] Run the focused OpenAI-compatible suite before changing provider response decoding. r[lattice.agent_runtime.inline_reasoning_boundary]

## Phase 2: Shared detail core

- [x] [serial] Extract deterministic `StepResult` conversion from TUI code into a shared pure run-history core with fail-closed sensitive-value rendering. r[lattice.workflow_execution.headless_run_details]
- [x] [serial] Route TUI detail construction through the shared core without changing public compatibility behavior. r[lattice.workflow_execution.headless_run_details]

## Phase 3: Provider and headless persistence boundaries

- [x] [serial] Normalize one exact leading inline-reasoning block before typed OpenAI-compatible output and fail closed on ambiguous markers. r[lattice.agent_runtime.inline_reasoning_boundary]
- [x] [serial] Add positive text/JSON normalization and negative malformed-marker tests that exclude prompt/reasoning content from diagnostics. r[lattice.agent_runtime.inline_reasoning_boundary]
- [x] [serial] Persist shared node details for terminal headless runs through the existing `RunHistoryStore` boundary. r[lattice.workflow_execution.headless_run_details]
- [x] [serial] Add positive output/detail persistence tests and negative sensitive-value redaction tests. r[lattice.workflow_execution.headless_run_details]

## Phase 4: Verification and closeout

- [x] [serial] Rerun the live trigger drill and verify one effect, durable details, safe metadata, and valid replay. r[lattice.workflow_execution.headless_run_details]
- [x] [serial] Run repository, Octet, Cairn, and Nix gates; sync the accepted spec and archive current evidence. r[lattice.workflow_execution.headless_run_details]

## Verification evidence

- Post-change focused OpenAI-compatible, headless, orchestration-shell, and run-history-detail suites passed, including plain text/JSON, exact reasoning suppression, malformed-marker rejection, successful/failed detail persistence, sensitive-value redaction, owner binding, and TUI compatibility.
- Final live dogfood completed one 34,577 ms provider call while the duplicate returned the same in-flight instance in 1 ms; one admission, instance, run, reservation, detail record, and replay chain remained.
- Durable live details contained one node and 13 bounded output/metadata lines and passed explicit scans for the raw prompt fragment, `<think>` markers, API-key fields, authorization fields, and bearer fragments.
- `scripts/check.rs` passed the full default-member suite, strict project Clippy, and accepted-spec references; `nix flake check -L --option secret-key-files ''` passed all bounded `x86_64-linux` checks.
- `octet-precommit-check` passed with 916 accepted/current findings, zero new, zero stale, 1,166 total findings, and baseline hash `b3:1aeb6f42e5bbb4bf92fbb1b16fbc2a05fe03e997ac30a3b7ba2b00b63ffab8f0`.
- Cairn validation plus proposal, design, and tasks gates passed through the canonical sibling Cairn CLI.
- No provider-side invocation count, remote exactly-once effect, billing truth, model truth, arbitrary semantic non-disclosure, externally anchored completeness, or production-readiness claim is made.

## Baseline evidence

- `nix develop -c cargo test --test test_headless --test test_run_history_details` passed 8 headless and 6 run-history-detail tests.
- The live probe recorded one 28.1-second provider execution; the duplicate returned the same instance in 0 ms while that execution remained in flight.
- The terminal store contained one trigger admission, one orchestration instance, one owned run, three replayable events, zero failures, no pending budget reservation, and no run-detail record.
- `nix develop -c cargo test --test test_openai_agent` passed all 12 pre-change OpenAI-compatible tests.
- The first post-persistence live rerun found a leading `<think>` block in `message.content` that repeated the raw prompt, proving `enable_thinking = false` is only a provider hint and not an output safety boundary.
