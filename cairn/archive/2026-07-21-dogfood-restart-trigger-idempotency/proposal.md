## Why

Lattice already proves duplicate orchestration-trigger admission through one in-process store handle, and a prior live drill proved concurrent duplicate reuse while the original process remained alive. Those checks do not establish that the redb-backed admission survives executor shutdown and is reused by a separately started process. A bounded process-restart drill is needed before retry or broader runtime work can safely expand the effect surface.

## What Changes

- Add a deterministic integration test that invokes two separate operating-system processes against one fresh durable store.
- Have the first process admit and complete one trigger-backed orchestration, then exit and release the database lock.
- Have the second process submit the exact duplicate trigger and prove it receives the original instance without another run, admission, or budget charge.
- Record bounded lifecycle evidence and explicit non-claims; do not retain workflow output, prompts, credentials, provider reasoning, or unrestricted store dumps.

## Impact

- **Files**: `tests/test_orchestration_shell.rs`, focused Cairn lifecycle/evidence artifacts, and the accepted agent-orchestration specification.
- **Testing**: focused orchestration-shell baseline and post-change tests, positive and negative process-harness paths, repository checks, Octet comparison, Cairn gates/validation, and Nix checks.
