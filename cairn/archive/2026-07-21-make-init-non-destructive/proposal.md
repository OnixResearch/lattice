## Why

`lattice --init --config <path>` currently uses an unconditional filesystem write. Re-running initialization against an operator-authored Nickel config silently replaces that config with defaults before exiting successfully. The live Animus dogfood reproduced this data-loss behavior.

## What Changes

- Make configuration initialization create-only at the target config path.
- Return a visible non-zero error when the target config already exists.
- Preserve the existing config and template catalog unchanged on rejection.
- Document the create-only contract and cover fresh and repeated initialization.

## Impact

- **Files**: `src/main.rs`, `tests/test_template_parameters.rs`, `README.md`, and the `node-templates` accepted specification.
- **Testing**: focused initialization tests before and after the change, full repository checks, Octet baseline comparison, Cairn gates/validation, and bounded Nix validation.
