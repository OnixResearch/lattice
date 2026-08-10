# Change: Confine runtime file reads

## Why

Image inputs, Markdown prompt hydration, and generic-agent handoffs still validate a path and then reopen it through ambient `std::fs`. A concurrent rename or symlink replacement can therefore invalidate the checked containment boundary. These readers also duplicate containment and bounded-read logic.

## What Changes

- Add one repository-owned capability filesystem core for deterministic relative-path admission, root selection, descriptor-relative opens, and bounded reads.
- Route `LocalFs`, production image inputs, Markdown prompt files, and agent image/artifact handoffs through that core.
- Require explicit roots for production image reads; treat the workflow directory and agent working directory as explicit capabilities for their bounded readers.
- Preserve image media sniffing, prompt UTF-8/non-empty checks, handoff media checks, byte limits, and typed diagnostics.
- Add positive and negative tests for each reader, including deterministic intermediate-path replacement after root acquisition.

## Impact

- Affected specs: `workflow-execution`, `workflow-format`
- Affected code: capability filesystem module, file/image/agent/workflow-format shells and focused tests
- Compatibility: production image inputs without `allowed_roots` now fail closed; prompt paths remain relative; handoff paths remain confined to command `cwd`
