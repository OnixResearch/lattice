# Process-restart trigger-idempotency evidence

Date: 2026-07-21

## Goal

Validate that one durable orchestration-trigger admission is reused after the first store-owning process exits and a separately started process reopens the same redb file.

## Command

```text
nix develop -c cargo test --test test_orchestration_shell restart_
```

## Bounded fixture

- The parent test created one fresh temporary directory.
- Child phase `first` opened the production `Store`, persisted one local shell-node workflow and orchestration definition, called production `orchestration::shell::start_from_trigger`, completed one round, recorded only its process ID and BLAKE3 instance ID, and exited.
- After the first child exit status was observed, child phase `duplicate` reopened the same redb path, submitted the exact definition ID, trigger ID/kind, and idempotency key, compared the returned instance ID with the first result, recorded only its process ID and BLAKE3 instance ID, and exited.
- The parent asserted the two operating-system process IDs differed before reopening the released store for final inspection.
- A separate invalid-phase child exited unsuccessfully before `Store::new`; the parent confirmed no redb or result file was created.

## Durable facts

The positive fixture asserted:

- returned instance identities were equal;
- the exact domain-separated scoped trigger key mapped to that instance;
- one trigger admission existed;
- one orchestration instance existed;
- one run existed and retained the exact orchestration owner plus non-empty dispatch identity;
- status was `Completed`, round was one, and no dispatch remained pending;
- consumed budget was exactly one round with every other dimension zero;
- no reservation remained and the budget was not breached; and
- deterministic orchestration replay was valid.

The focused test command passed both the positive restart fixture and negative invalid-phase fixture. Child result files and the redb store lived only in the temporary test directory and were deleted by the fixture. No workflow output, prompt, credential, authorization value, provider reasoning, arbitrary environment value, or raw store dump was retained.

## Live Animus restart drill

A second drill used the real bounded Animus leaf workflow and reachable VibeThinker OpenAI-compatible endpoint in `/tmp/lattice-live-restart-idempotency`. The first probe process admitted instance `9d0cc2d1ea57558e77afc19cb3bc11848525ff2f199fbcbe896c5d43e85ffb31`, reserved run `0`, attempted one provider dispatch, and terminalized `Failed` after the configured bounded duration. The exact provider error content was not inspected or retained.

After that process exited, a separately launched duplicate probe reopened the same redb store and reported:

```text
phase=duplicate same_instance=true status=failed admissions=1 instances=1 runs=1 consumed_rounds=1 consumed_invocations=1 replay_valid=true
```

The safe `runs show 0 --output json` projection was valid and showed exact owner/dispatch binding, one failed node with no output, valid four-event run replay, valid orchestration event chain, closed recovery, one settled round/invocation upper-bound charge, no budget breach, and no diagnostics. A case-insensitive scan found none of the checked prompt fragments, `<think>` markers, API-key fields, authorization fields, bearer fragments, endpoint strings, model names, or system-prompt fragments.

This live result validates duplicate reuse after a process restart even when the original terminal outcome is failure; it does not convert that provider failure into success or explain its private cause. The temporary probe source, config, identity file, safe report, and redb store were deleted after these bounded facts were recorded.

## Adversarial audit

Four mechanisms were considered: a self-spawned integration test, an external example probe, same-process store reopen, and a product CLI trigger command. The self-spawned test was selected because it crosses a real process and redb ownership boundary without adding product authority. Same-process reopen was rejected as false completion; the example probe and product CLI were rejected as unnecessary durable surfaces.

Secondary review asked for stronger proof that distinct processes executed, the admission key was exact, run ownership remained exact, and all budget dimensions were single-settled. The fixture now asserts those facts directly.

## Non-claims

This evidence covers local persistence across a clean process exit and restart. It does not prove behavior for every kill instruction boundary, remote provider cancellation, remote exactly-once effects, contiguous sequence allocation, external event-ledger anchoring, provider billing truth, or production readiness.
