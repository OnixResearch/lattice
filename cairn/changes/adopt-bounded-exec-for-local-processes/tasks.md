## Phase 0: Reproducible dependency gate

- [ ] [serial] Publish the reviewed AGPL `bounded-exec` source with explicit operator authorization, record the immutable revision, and prove the canonical Git source resolves without using a sibling path. r[lattice.repository_validation.bounded_exec_source]
- [ ] [serial] Pin Cargo and Nix to the same exact published revision and add a repository check that rejects source drift, path substitution, or an unresolvable pin. r[lattice.repository_validation.bounded_exec_source]

## Phase 1: Pure process policy and shared shell

- [ ] [serial] Add typed Nickel process-limit defaults and a pure admission core for explicit argv, environment, working directory, stdin, capture, deadline, teardown, termination scope, and truncation policy. r[lattice.workflow_execution.bounded_process_policy]
- [ ] [serial] Implement one thin `bounded-exec` adapter with consumer-owned executable and current-directory resolution, safe error translation, and a Tokio blocking bridge. r[lattice.workflow_execution.bounded_process_mechanics]
- [ ] [parallel] Add positive and negative core tests for valid plans, zero or unsupported limits, stdin overflow, duplicate environment entries, missing executables, relative-path resolution failures, and invalid working directories. r[lattice.workflow_execution.bounded_process_policy] r[lattice.workflow_execution.bounded_process_mechanics]

## Phase 2: Production path cutover

- [ ] [serial] Route shell nodes through the shared adapter while preserving shell `-c`, stdin, environment, working-directory, redaction, typed outputs, cache identity, and node-scoped diagnostics. r[lattice.workflow_execution.bounded_process_mechanics] r[lattice.workflow_execution.bounded_process_compatibility]
- [ ] [serial] Route command-backed agent adapters through the shared boundary with a required deadline, per-stream capture limits, truncation rejection, and no fallback. r[lattice.agent_runtime.bounded_command_process]
- [ ] [serial] Route command credential providers through the shared boundary while preserving opaque references, bounded stdin, fixed safe diagnostics, and secret redaction. r[lattice.credentials.bounded_command_provider]
- [ ] [parallel] Remove duplicate production spawn, pipe, wait, timeout, capture, kill, and reap mechanics only after all three compatibility suites pass. r[lattice.workflow_execution.bounded_process_mechanics]

## Phase 3: Adversarial evidence and release gate

- [ ] [parallel] Add real-process negative tests for stdout and stderr floods, timeout, cancellation, post-timeout side-effect attempts, and Unix descendant-held-pipe cleanup. r[lattice.workflow_execution.bounded_process_mechanics]
- [ ] [parallel] Add command-agent and command-credential tests proving output truncation and deadlines fail before parsing without credential leakage or route fallback. r[lattice.agent_runtime.bounded_command_process] r[lattice.credentials.bounded_command_provider]
- [ ] [serial] Run repeated credential-log capture, focused compatibility tests, `scripts/check.rs`, Octet, Cairn gates and validation, and `nix flake check -L --option secret-key-files ''`; sync and archive only after every task and source identity is current. r[lattice.repository_validation.bounded_exec_source] r[lattice.workflow_execution.bounded_process_compatibility]
