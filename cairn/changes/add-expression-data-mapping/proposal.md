## Why

Workflow authors need to map upstream outputs into downstream node inputs, commands, environment variables, HTTP bodies, and file paths without writing glue shell commands. Expressions provide a deterministic data-binding layer between nodes.

## What Changes

- Add a small expression language for referencing node outputs and simple transforms.
- Evaluate expressions in a pure context built from completed upstream values.
- Report missing references and type mismatches before executing dependent nodes.
- Preserve credential redaction when expressions touch secret-bearing values.
