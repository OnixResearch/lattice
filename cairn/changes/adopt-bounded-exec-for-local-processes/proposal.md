## Why

Lattice has three production local-process paths with incompatible and incomplete mechanics. Shell nodes wrap `wait_with_output` in a timeout that does not own child or descendant teardown and retain stdout and stderr without byte bounds. Command-backed agent and credential providers buffer complete output before enforcing a post-hoc limit, and their production runner does not enforce the timeout carried by `ProcessRequest`. These gaps can leave work running after a reported timeout or allow output floods to consume unbounded memory.

The product-neutral `bounded-exec` crate already separates deterministic limit and outcome policy from explicit argv, environment, working-directory, bounded-I/O, deadline, cancellation, and owned-teardown mechanics. Lattice should consume that mechanism rather than maintain another process runner, while retaining executable discovery and authorization, shell semantics, redaction, parsing, workflow policy, evidence, and lifecycle claims.

## What Changes

- Pin one reviewed immutable published `bounded-exec` Git revision and align Cargo and Nix source identity without a sibling path dependency.
- Add a pure Lattice process-admission core that converts consumer policy into explicit executable, argv, environment, working-directory, stdin, capture, deadline, teardown, and outcome decisions.
- Replace shell-node, command-agent, and command-credential process mechanics with one thin `bounded-exec` shell adapter.
- Add typed Nickel process limits for stdin, stdout, stderr, polling, and teardown while preserving each path's existing timeout ownership and product diagnostics.
- Select process-group teardown on supported Unix hosts and fail closed on timeout, cancellation, output truncation, malformed bounds, and infrastructure errors.
- Preserve command authorization, shell interpretation, credential resolution, output redaction, adapter parsing, cache identity, graph typing, and all existing non-claims in Lattice.

## Success Criteria

- No production Lattice source directly owns child spawning, pipe draining, timeout polling, killing, reaping, or output-retention mechanics outside the reviewed `bounded-exec` adapter.
- Shell-node, command-agent, and command-credential requests all carry explicit positive stdin, stdout, stderr, deadline, polling, and teardown bounds into the shared runner.
- Timeout and cancellation tests prove the owned process group is torn down, including a descendant-held-pipe negative fixture on Unix.
- Output-flood tests prove retained bytes never exceed configured stream bounds and truncation fails the affected node or provider before output parsing.
- Existing success, non-zero exit, stdin, environment, working-directory, credential-redaction, typed-output, cache, and no-fallback behavior remains compatible.
- Focused tests, repeated credential-log capture, `scripts/check.rs`, Octet, Cairn gates and validation, and `nix flake check -L --option secret-key-files ''` pass.

## Publication Gate

The reviewed local `bounded-exec` checkout is currently detached at `29dac88ecded94457572db3fdfaaaab95fa91525`, has no configured remote, and `https://github.com/OnixResearch/bounded-exec` returns 404. Its own adoption contract forbids consumer cutover before publication at an immutable exact Git revision. Lattice implementation MUST remain blocked until that source is published with explicit authorization; an ambient `../bounded-exec` path, copied unreviewed mechanics, or a non-resolving Git pin is not acceptable completion.

## Non-Goals

- Claiming that process bounds provide sandboxing, hermeticity, executable authorization, filesystem confinement, network isolation, child correctness, or remote exactly-once execution.
- Moving credential, workflow, effect, output-parsing, evidence, or release authority into `bounded-exec`.
- Adding new workflow node kinds, agent features, provider fallback, remote workers, or orchestration behavior.
- Publishing or pushing the sibling repository without explicit operator authorization.

## Impact

- **Files**: Cargo/Nix dependency inputs, typed Nickel config and contracts, process admission/adapter modules, executor and agent/credential shells, tests, README, and Cairn specifications.
- **Compatibility**: Existing commands and outputs remain product-compatible; output floods, ignored command-provider deadlines, and post-timeout survivors become explicit failures.
- **Testing**: Positive execution compatibility plus negative malformed-limit, stdin overflow, stdout/stderr flood, timeout, cancellation, descendant cleanup, missing executable, invalid working directory, and secret-redaction tests.
