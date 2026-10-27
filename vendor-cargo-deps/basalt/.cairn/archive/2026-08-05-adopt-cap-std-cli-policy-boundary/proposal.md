## Why

Basalt's library core should stay small and deterministic, while the CLI reads operator-supplied policy artifacts and may write or report receipt material. Policy path handling is a shell concern and should be bounded by explicit filesystem authority instead of ambient path access.

## What Changes

- Adopt `cap-std` only for Basalt CLI policy-artifact and receipt/output filesystem boundaries.
- Introduce typed capability roots for policy input, generated policy, and receipt/output directories.
- Keep UCAN enforcement, Nickel/Steel DTO validation, and library APIs independent of `cap-std`.
- Add positive and negative CLI fixtures for valid policy reads, `../`, absolute paths, symlink escapes, and output escape attempts.

## Impact

- Basalt CLI local file authority becomes explicit and root-confined.
- Existing library consumers do not inherit CLI-only filesystem dependencies.
- Invalid policy or output paths deny before reading unrelated host files or writing outside declared outputs.
