## Context

Lattice's thin Nickel adapter delegates unchanged source to nickel-eval f23be09e41337de8fa729274f50023f861ae0e8a. That producer calls nickel-lang 2.2.0 Context::eval_deep_for_export, backed by nickel-lang-core 0.18.0. A minimal imported JSON object containing a nested record and array merged with a disjoint Nickel record reaches eval/fixpoint.rs rec_env with a nonconstant non-thunk value. The debug assertion fails. The release CLI exports the same source successfully; that does not justify disabling debug assertions.

Upstream issue https://github.com/nickel-lang/nickel/issues/2653 identifies this exact invariant violation. PR https://github.com/nickel-lang/nickel/pull/2677 fixes it at import deserialization, immutable commit aa94d9b1f7a0981b4f0d6e8dff79f0a74ff1bb1a, merged as 4cb2323661598e48fc0a6cfc29350573e5177d0b. The fixed upstream tree is core 0.19.0/parser 0.4.0/CLI 1.18.0, not the accepted consumer cohort.

## Decisions

### Decision: backport the canonical producer repair

**Choice:** Backport the exact upstream two-source-file fix into a producer-owned copy of the published core 0.18.0 package. The upstream fix wraps plain records created from recursive-record ASTs in the existing compat closurization operation, and adds the same boundary to generic serde array/record deserialization. No fixpoint assertion is removed. No input is rewritten. Do not maintain a second merge-level workaround.

**Rationale:** The AST translator implicitly closurizes recursive records during evaluation. Import conversion changes them to plain records, losing that implicit step; the upstream repair explicitly restores it at the conversion boundary. This fixes the source of the invariant violation for imports, record contracts, and deserialization without changing recursive merge mechanics.

### Decision: require explicit immutable package admission

**Choice:** Keep the 1.17.0/2.2.0/0.18.0 language cohort fixed and record the upstream repair identity separately. The producer worktree starts at exact f23be09, not its unrelated newer HEAD. Its repo-owned import script verifies the published crate and patch identities and reproduces the source snapshot. Integration owns immutable producer publication, aligned Cargo/Nix overrides, and exact public readback before runtime acceptance.

**Rationale:** The panic is below Lattice's adapter. Cache edits and unreviewed sibling source would be irreproducible and violate the accepted exact-source contract. Cargo ignores transitive patch tables, so the consumer must select the same corrected core explicitly.

### Decision: test values and rejection boundaries

**Choice:** Native adapter regressions evaluate an unchanged file with relative JSON import and merge in both operand orders, nested overlap, captured-source absolute import, and recursive overriding. They assert complete JSON values. Negative malformed/missing imports and contract rejection assert bounded SafeFailure categories without path or secret disclosure. A pinned CLI comparison is a separate smoke command, not a test requiring an ambient external executable.

**Rationale:** These cases exercise the invariant and consumer-visible semantics rather than source text or forwarding wiring.

## Risks / Trade-offs

- A new immutable patched producer publication is a prerequisite for production source cutover; local patch evidence cannot replace it.
- Upstream's repair belongs to a newer cohort; the source-only backport must remain narrowly reviewed and pass native regressions against core 0.18.0.
- Existing recursive values and pending contracts are sensitive to extra thunk indirections; regression cases cover repeated overrides and contracts.
- Focused agreement does not prove arbitrary evaluator equivalence, Nickel soundness, import closure completeness, or workflow safety.
