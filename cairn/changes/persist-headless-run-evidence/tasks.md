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
- [ ] [serial] Run repository, Octet, Cairn, and Nix gates; sync the accepted spec and archive current evidence. r[lattice.workflow_execution.headless_run_details]

## Baseline evidence

- `nix develop -c cargo test --test test_headless --test test_run_history_details` passed 8 headless and 6 run-history-detail tests.
- The live probe recorded one 28.1-second provider execution; the duplicate returned the same instance in 0 ms while that execution remained in flight.
- The terminal store contained one trigger admission, one orchestration instance, one owned run, three replayable events, zero failures, no pending budget reservation, and no run-detail record.
- `nix develop -c cargo test --test test_openai_agent` passed all 12 pre-change OpenAI-compatible tests.
- The first post-persistence live rerun found a leading `<think>` block in `message.content` that repeated the raw prompt, proving `enable_thinking = false` is only a provider hint and not an output safety boundary.
