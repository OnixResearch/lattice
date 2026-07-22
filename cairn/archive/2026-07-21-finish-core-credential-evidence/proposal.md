## Why

The bounded credential audit classified eight core scenarios as exact and four as partial. The remaining gaps are concrete: save/export is not exercised after real secret resolution, inline plaintext rejection lacks a stable redaction-safe credential error and no-store proof, output redaction does not jointly cover stdout/stderr/log/history/JSON/persistence, and validation rendering does not assert reference identity with secret absence.

## What Changes

- Add a typed plaintext-credential import rejection that emits fixed safe diagnostics before store access.
- Make credential redaction deterministic, overlap-safe, and reusable at execution-output boundaries.
- Add exact positive and negative tests for post-resolution save/export safety, import rejection/no-store behavior, output/log/history/JSON/persistence redaction, and validation rendering.
- Reclassify the selected 12 credential scenarios as exact maintained evidence without adding credential Tracey markers.
- Preserve the exact 353-item missing set and zero dangling markers until a separate promotion change.

## Impact

- **Files**: credential/workflow-format cores, focused credential tests, inventory policy/generated report, audit fixture/test expectations, and lifecycle evidence.
- **Compatibility**: valid reference-only workflows keep their existing format; imports with a `secret_env.value` field fail with a stable typed error.
- **Security**: no new secret persistence or authority; failures remain before store access and resolved values remain execution-boundary inputs.
