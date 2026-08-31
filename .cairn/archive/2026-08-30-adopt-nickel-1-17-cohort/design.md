# Design: Adopt the Nickel 1.17 workflow evaluator cohort

## Context

Lattice consumes `nickel-eval` through an immutable Radicle object. Development fixtures also use `nickel-lang-core` directly.

The current consumer and fixture paths belong to the Nickel `1.16` release cohort.

## Decisions

### Decision: Consume the published evaluator first

**Choice:** Wait for the archived `nickel-eval` cohort change and pin its new immutable Radicle object.

Do not use a sibling path or GitHub fallback during the migration.

**Rationale:** Lattice must consume the same artifact that release checks review.

### Decision: Align direct fixture dependencies

**Choice:** Update the direct development dependency to exact `nickel-lang-core 0.18.0`.

Pin command-line Nickel `1.17.0` to the exact upstream source when the current Nix package set lacks it.

**Rationale:** Product, fixture, and operator paths must share one reviewed cohort.

### Decision: Keep workflow meaning in Lattice

**Choice:** `nickel-eval` returns bounded evaluator results. Lattice retains workflow decoding, Choregraph admission, effects, persistence, and diagnostics.

**Rationale:** The evaluator dependency does not own product semantics.

### Decision: Test the import boundary in both directions

**Choice:** Test valid workflows through evaluation, decoding, and pure Choregraph admission.

Reject malformed sources, missing imports, contract failures, unknown fields, excessive bounds, and mismatched evaluator cohorts.

**Rationale:** The migration must preserve accepted and rejected behavior.

## Flow

```text
published nickel-eval object
  -> Lattice evaluator adapter
  -> Lattice workflow decoding
  -> Choregraph pure admission
  -> no effect until admission passes
```

## Risks and trade-offs

- Diagnostic wording can change while stable Lattice categories remain correct.
- The new evaluator can reject sources that previously passed by accident.
- Consumer publication must remain blocked until the exact producer object exists.
