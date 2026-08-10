## Context

The first extraction must create a useful compiler boundary without pulling host authority into the new crate. Workflow format and validation still depend on broad root-owned models.

## Decisions

### Decision: Extract the smallest coherent pure family

**Choice:** Move content-addressed values, expression decisions, control-flow decisions, and schedule identity framing into `lattice-core`.

**Rationale:** These modules share small deterministic dependencies. They already expose typed inputs and return values, decisions, or diagnostics.

### Decision: Defer broad workflow ownership

**Choice:** Keep workflow format and full workflow validation in the root package for this slice.

**Rationale:** Moving them now would import credentials, triggers, runner policy, scheduler models, file-node models, and provider validation. That move would weaken the first boundary.

### Decision: Preserve public paths

**Choice:** Keep `lattice::expressions`, `lattice::control_flow`, `lattice::content_identity`, and `lattice::engine::value` as re-export modules.

**Rationale:** Existing users and tests must compile without source changes.

### Decision: Enforce dependency direction

**Choice:** Add a repository test that admits the selected dependencies and rejects host, persistence, network, process, and TUI dependencies.

**Rationale:** A crate boundary without a checked dependency rule can silently regain host authority.

### Decision: Ratchet each Octet scope separately

**Choice:** Keep the root baseline for `lattice` and add a second baseline for `lattice-core`.

The migration checker compares all old and new warning occurrences. It permits only count-message normalization and one reviewed filename-warning reduction.

**Rationale:** Changing the default Octet scope would rekey unrelated findings. Separate scopes preserve reviewable debt and check the new crate directly.

## Tests

- Positive: the core manifest contains only admitted deterministic dependencies.
- Negative: fixtures with Tokio, Reqwest, Redb, Cap-std, Bounded Exec, Crossterm, Ratatui, or Animus runtime dependencies fail.
- Compatibility: focused pre-move and post-move tests produce the same results through existing public paths.
- Regression: repository, both Octet scopes, Cairn, Clippy, and Nix checks pass.

## Claim Boundaries

The extraction proves dependency direction and tested API compatibility. It does not prove semantic equivalence, whole-workflow correctness, or release readiness.
