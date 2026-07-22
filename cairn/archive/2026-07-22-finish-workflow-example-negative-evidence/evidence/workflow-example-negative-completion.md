# Workflow-example negative completion

The new typed Nickel fixture `examples/workflows/invalid-unsupported-image-format.ncl` intentionally resolves all three image inputs to unsupported GIF bytes. Maintained execution assertions observe unsupported-format diagnostics, skipped downstream execution, and zero completed nodes. Both workflow-example rows are exact and remain unpromoted.

Coverage remains 317 missing with zero dangling and `valid: false`. Focused image-example, historical audit, completion, inventory, repository, and Cairn checks pass. This does not claim arbitrary image validation or production readiness.
