# Live Trigger Idempotency Dogfood

## Success contract

A fresh typed orchestration and imported `animus-bounded-agent` workflow used the reachable `VibeThinker-3B` OpenAI-compatible endpoint without bearer credentials. The first `start_from_trigger` call was left in flight after its dispatch became durably acknowledged; the same definition, trigger identity, and idempotency key were then submitted again.

Validated completion required one trigger admission, one orchestration instance, one run, one consumed budget reservation, exact owner binding, valid replay, and a duplicate response before the original provider request completed. Run records are local observations and do not prove provider-side invocation counts or remote exactly-once effects.

## Before-fix observation

- First live execution: 28,100 ms.
- Duplicate response: 0 ms while the first execution remained in flight.
- Duplicate and original instance: `8af1b4a07d699d6aae32016738cf38166b4facce1d06e4d299ef320ed35dda37`.
- One admission, one instance, one run (`run:0`), one agent invocation/round reservation consumed, no remaining reservation, zero failures.
- Exact orchestration owner and dispatch identities were present on the run.
- Three orchestration events replayed validly.
- The bounded JSON evidence contained no prompt, API-key, authorization, or bearer fragments.
- `load_run_details(run:0)` returned `None`, exposing the headless persistence gap.

Machine-readable evidence is retained in `live-trigger-idempotency-before.json`. The temporary probe invokes production `start_from_trigger`, store, replay, and agent-runtime paths directly because no network/CLI adapter currently exposes durable orchestration trigger admission; this validates the product shell beneath an external adapter, not webhook transport integration.

## First post-persistence rerun

The next fresh-store run persisted one node detail record, but the probe intentionally failed its safe-detail assertion. The provider ignored `enable_thinking = false` and placed a leading `<think>...</think>` block in `message.content`; that block repeated the raw user prompt and planning text before the final answer. This falsified the assumption that request hints or the separate `reasoning_content` field alone keep hidden reasoning out of typed output. Raw captured detail text was retained only under `/tmp/lattice-animus-idempotency/details-debug.json` during diagnosis, was never checked into the repository, and was deleted with the ephemeral dogfood stores after bounded evidence was recorded.

## Final post-fix validation

A second fresh-store run exercised the shared detail core and provider-boundary reasoning parser together:

- First live execution: 34,577 ms.
- Duplicate response: 1 ms while the first execution remained in flight.
- One admission, one instance, one owner-bound run, one consumed reservation, zero failures, and three replay-valid orchestration events.
- One durable node detail record with 13 bounded output/metadata lines.
- The durable detail serialization contained no raw prompt fragment, `<think>`/`</think>` marker, API-key field, authorization field, or bearer fragment.
- No budget reservation remained and no duplicate run or instance appeared.

Machine-readable final evidence is retained in `live-trigger-idempotency-after.json`. The safe scan proves absence only for the declared markers in the persisted local detail artifact; it does not prove arbitrary semantic non-disclosure or provider-side deletion of reasoning.
