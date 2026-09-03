## Why

Lattice must reject workflow delivery that lacks repeatable repository evidence. It did not provide a repository-pinned Cairn runner for this lifecycle profile.

## What Changes

- Add the published Cairn outcome-machine revision as an immutable Nix input.
- Add a repository-owned runner that injects the matching generated policy.
- Run one Lattice change through planned rejection and observed completion.
- Preserve Lattice orchestration, effect, evidence, and release authority.

## Impact

- **Files**: `flake.nix`, generated `flake.lock`, accepted lifecycle requirements, and pilot evidence.
- **Testing**: Cairn gates, blocked sync, fresh-state validation, Octet pre-commit, and Nix evaluation.
