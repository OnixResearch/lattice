# Typed TUI workflow authoring

Launch `lattice --config /absolute/path/to/config.ncl` in a terminal. **Ctrl-W** opens the typed authoring browser. The command palette (**Ctrl-P**) exposes separate workflow, selected-node configuration, graph, credential, runner, trigger, and admitted-operation authoring actions. Select a node before editing node-owned fields. The canvas and properties footers keep the direct `e`/`l`/`t`/`k` edit shortcuts visible alongside the authoring and diagnostics actions.

Use arrows, Page Up/Down, Home/End to browse; Enter opens a draft. Enter validates and atomically applies a draft; Escape cancels it without changing the workflow. Ctrl-U clears the draft and Ctrl-J inserts a newline. Blank removes optional values or existing collection rows; required fields need a valid replacement. Collection `+` rows create members. Explicit move rows reorder collections. Workflow save remains **Ctrl-S**. Closing an editor does not save it or execute a workflow.

At narrow dimensions the footer starts with **Esc back**. Escape first cancels a draft, then closes its browser. Resize back to a larger terminal to inspect the full field help. Validation errors keep the draft open. A workflow change while a draft is open makes that draft stale: cancel it and reopen rather than overwriting the newer workflow.

## Monitoring DAGs as rows

Open a saved workflow with **Ctrl-O**, choose it with **Up/Down**, and press **Enter**. With canvas focus, press **v** to switch between the spatial graph and **Rows (v: graph)**. Both surfaces use the same selected node identity: selecting a row updates the properties/output panels, and switching back preserves that selection without moving graph nodes. **Up/Down**, **Page Up/Down** (ten rows), **Home/End**, a left click, or the mouse wheel select rows. The viewport follows selection, including on small terminals. Left/right arrows and drags do not edit the hidden spatial graph.

Rows follow dependency depth and stable node identity, never stored graph coordinates. Lane connectors show pending branches, forks, and joins; unequal-length branches retain their reserved lanes. Visible lane columns grow with available width, up to six. A **+N** header/footer counts additional collapsed lanes, and **◆** marks a node sharing the final visible lane. Every node remains a separate selectable row; collapse does not omit workflow steps.

Each row shows current state, locally observed start time (`HH:MM:SS`), elapsed or final observed duration, dispatch-attempt count, and a bounded key-output summary. At wider canvas sizes, additional columns show node kind and provider-reported tokens/microunit cost. Missing observations remain `—`; attempts count observed dispatch starts, not inferred retries. The footer sums observed node time (not run wall time), tokens, cost, and attempts. Output summaries prefer meaningful non-metadata output in declared port order, so a shell's stdout is not replaced by its exit-code port. They are one-line excerpts, not an output export.

**Ctrl-R** starts a run through the configured authority provider; it is not required to inspect either layout. Dispatch and final admitted per-node results update the rows while other nodes remain active, including failures, skips, usage, and output. A failed or unobserved effect is not presented as successful. Layout switching does not grant authority, execute effects, retry work, or alter cancellation semantics. Source-aware Nickel editing is a separate authoring capability, not part of the rows layout.

The run-event mailbox holds at most 32 events, and a UI tick applies at most 32.
Async forwarding waits for mailbox capacity instead of dropping observations or
blocking a runtime thread. The executor's separate observation port reserves the
whole run's event budget: at most two observations per admitted graph node, with
checked multiplication and Tokio's semaphore limit; an empty graph uses one
unused slot. This bounds queued event count, not the byte size of node outputs.

That budget follows the actual producers: the ready frontier marks a node running
before dispatch and terminal before it can become ready again; each dispatch
emits one start and at most one finish. Cache hits and preparation failures emit
only a finish. The final skipped-node pass excludes nodes already having results;
frontier-wide failure records final results without extra progress events. The
approval path visits each topological node once, skipping checkpoint nodes, and
may emit fewer observations. Current runner policy does not introduce dispatch
retries; any retries inside an adapter remain within its single dispatch. The
repository subworkflow runner creates a fresh child executor without the parent's
progress sender, so child steps do not consume parent slots. Each UI run gets a
new port; the port is not an accumulated history across runs.

The synchronous producer never waits for the UI, including on a single-threaded
async runtime. A retained completeness observer does not retain a channel sender,
so the forwarder drains and closes before the final run result is sent. Queue
overflow or disconnection records an explicit observation fault; a failed
forwarder is also diagnosed before the final result. The UI then reports live
metadata as incomplete and clears unfinished elapsed-time observations rather
than presenting a growing duration as final evidence. Final execution results
remain separate and are retained. UI lag does not retry or cancel execution, and
there is no overflow queue.

Rows-specific verification after integration:

```sh
cargo test --lib tui::rows::
cargo test --lib tui::app::rows_tests::
cargo test --lib tui::input::tests::rows_
```

The linear, fork-join, and diamond screen-column snapshots live in `tests/fixtures/tui-rows/`. Reducer regressions cover live observations before run completion, failure/skipped truthfulness, and final-event output deduplication; they do not establish external effect success or terminal-emulator pixel identity.

## Capability matrix

The checked field-by-field matrix is [`src/tui/authoring/catalog.rs`](../src/tui/authoring/catalog.rs), with provider-owned node field identities in [`src/node_kind_provider/authoring.rs`](../src/node_kind_provider/authoring.rs). Editor descriptors and matrix rows share owner, value-kind, required/optional/repeated shape, parser, validator, and mutation-target metadata. Each matrix row identifies its portable path, authoring action, reset/removal rule, and regression family. The shape checker rejects missing, duplicate, stale, display-only, and newly uncovered portable fields. Preserving imported values alone is not authoring evidence.

| Owner | Create/edit/reset or remove path |
| --- | --- |
| Workflow | Name, description, explicit created/modified RFC3339 timestamps. Rename updates owned trigger workflow references atomically. |
| Shell | Command, working directory, environment map, timeout, stdin piping. |
| HTTP | Method, URL, header map, optional body, timeout. |
| File | Read/write/append operation, path, optional content, overwrite, allowed-root list. |
| Image input | Path, allowed-root list, positive byte limit. |
| Agent | Target, inline prompt, bundle-relative Markdown prompt reference, output mode, model hint, tool policy, context-reference list, working directory, session. |
| Condition | Branch add/edit/remove/reorder; typed predicates. Exact unchanged labels retain port IDs and edges. Rename is remove plus add, not silent edge retargeting. |
| Merge | Policy and required-input list, with reconciled graph ports. |
| Subworkflow | Child workflow reference and parent/child input/output mappings, with reconciled ports. |
| Approval | Gate identity, summary, selected inputs, required artifacts, local authority, expiry/clock, reason, resume values/credential bindings, terminal outcomes. Producer-required non-claims remain mandatory. |
| Operation | Select exact admitted catalog/instance/runtime observation, method, and typed arguments, then admit a new or replacement plan. Nested object/array arguments have typed member rows, not raw JSON replacement. |
| Graph | Node label and signed coordinates; port IDs/directions/labels/type tags; compatible directed edges. Port removal or identity/direction changes clean up incident edges. |
| Credentials | Per-node environment name and opaque provider/key/optional-field reference. Add/edit/remove references, never secret values. |
| Runner | Runtime binding and effect/cache/retry/artifact policies supported by the current producer. Remove restores inherited settings. |
| Triggers | Manual, bounded schedule, and webhook definitions; identity, workflow, enabled flag, schedule fields, webhook method/path/content types/authentication, typed input bindings, overlap and idempotency. Add/edit/reset/remove without activating a scheduler or listener. |

Node identities are allocated through checked graph creation and removed with their owner, not forged through a text field. Invocation-plan identities, contracts, runtime/backend route, credentials, bounds, and non-claims are derived through admission; replace the accepted plan to change them. Plaintext secret payloads are intentionally not an authoring surface.

Current runner producers support `disabled` cache, `none` retry, and `metadata-only` artifacts. This change does not invent retry counts/backoff, named budgets, or unsupported execution-policy semantics. Unsupported policy text is rejected. Credential authoring validates opaque references but never resolves providers. File/image/prompt references are lexical during normal field editing; authoring does not read their payloads, execute commands, dispatch HTTP, or start external agents. Explicit operation-source loading reads the selected definition files and observes the selected executable identity without executing the operation.

## Real-terminal smoke scenario

Use an isolated data directory and do not press Ctrl-R:

1. Open Ctrl-W, select Workflow name, Enter, Ctrl-U, Enter. Observe the blank-name error; resize to 32 columns × 6 rows and verify `Esc back` remains visible. Escape cancels; restore terminal size and confirm the old name remains.
2. Reopen the name draft, Ctrl-U, type `operator-authoring-pty`, Enter, then Escape to close the browser.
3. Ctrl-N, `h`, Enter opens an HTTP request draft. Clear its initial request and enter `GET https://example.invalid/authored`. Enter creates the definition, without sending a request.
4. Ctrl-S must report `Saved 'operator-authoring-pty'`; Ctrl-Q exits without an unsaved-change prompt.
5. Reload the saved workflow. Verify the exact URL and that no run was recorded. Export/import it through the typed Nickel format and compare portable meaning.

The automated real-PTY scenario is `operator_e2e_tui_process_authors_http_and_recovers_invalid_constrained_draft`. Its persisted-workflow assertions and Nickel round trip accompany the terminal interaction; terminal transcript text alone is not parity proof.

## Focused verification

Run from the Lattice repository in its pinned development environment, after coordinated source integration:

```sh
cargo test --lib tui::authoring:: -- --nocapture
cargo test --test tui_aggregate_lifecycle
cargo test --test test_tui_ux_completion operator_e2e_tui_process_authors_http_and_recovers_invalid_constrained_draft -- --nocapture
cargo test --test test_tui_ux_completion
cargo test --test test_workflow_format
```

`all_current_node_kinds_and_field_families_author_store_and_nickel_round_trip` drives actual TUI key handlers for all ten current node variants and every field family, then checks aggregate validation, store load/save, the portable capability matrix, and Nickel import/export. Owner-specific regressions exercise field boundaries and collection mutation. Negative fixtures cover stale/cancelled drafts, orphan-producing mutations, unsupported policies, malformed references and Nickel, failed saves, and constrained layouts. Full lifecycle gates and repository-wide verification remain required before package closeout; these commands are instructions, not a claim that they have already passed.
