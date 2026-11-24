# Pre-change authoring baseline

Observed against the existing `target/debug/lattice`, before source editing:

- Launched a real 80×24 PTY with an isolated explicit Nickel config and data directory under `/tmp/lattice-parity-baseline-TuiAuthoringParity`.
- The initial workspace rendered Canvas, Properties, and the empty workflow.
- Sent Ctrl-W: no authoring browser appeared.
- Sent Ctrl-N: Add Node appeared with Shell, HTTP request, file operation, image input, Pi agent, Clankers agent, condition, merge, subworkflow, and templates. HTTP/File creation is already prior delivered behavior, not a newly missing feature.
- Sent Escape then Ctrl-Q: process exited 0. No workflow was saved or run.

Read-only native Cairn admission used `/nix/store/kp30jr43dplxzlrgmkq3f3q380mvdgks-cairn-0.1.0/bin/cairn`, explicit `--root /home/brittonr/git/OnixResearch/lattice`, and `--policy /home/brittonr/git/OnixResearch/lattice/cairn-policy/generated/cairn-policy.json`. Proposal, design, and tasks gates each returned PASS with no issues.

Source inspection (not executed regression evidence) found the three checked tasks already have implementations: HTTP/File guided creation, aggregate `remove_node`, and label-based condition output reconciliation. The remaining matrix, typed workflow/graph/runner/trigger editors, comprehensive field editors, stale-plan safeguards, and parity fixtures cannot be inferred from those implementations. Source tests/builds/formatters are intentionally deferred to the integration owner's coordinated verification.

Current producer boundaries discovered before editing:

- Portable `operation_plan` is an accepted content-bound `InvocationPlan`; editing its identity-bound nested values without an accepted catalog/instance/facts replanning operation would forge accepted identity. It must be classified separately rather than represented as completed mutable authoring.
- `NodeRunnerSettings` currently has runtime kind/key and four policy strings, not declared ability/effect arrays, retry counts/backoff, or named limits. Effect policy has an existing validator. Producer-written cache/retry/artifact modes are `disabled`, `none`, and `metadata-only`; unsupported alternatives must not acquire invented runtime semantics through an editor.

These boundaries were reported to the integration owner before implementation; neither is evidence of full package closeout.
