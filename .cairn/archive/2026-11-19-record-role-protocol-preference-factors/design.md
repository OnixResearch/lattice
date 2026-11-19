## Design

The functional core is `src/engine/protocol/preference.rs`, with Lattice-owned
DTOs in `src/engine/protocol/model/annotation.rs`. Neither module performs I/O.

`admit_annotated_projection(AnnotatedProjection, PreferenceProfile)` runs these
steps in order:

1. `admit_projection` admits `bundle.projected()` and `bundle.base()`. This is the
   unchanged v1 intake, so every v1 identity, schema, proof and revision check
   still applies.
2. `preference::linkage::verify_receipt` compares the transported receipt body
   and packed bytes with the bundle. `verify_local_set` compares the transported
   annotated complete local set with it.
3. Lattice checks the receipt contract: the annotated profile selector, the
   `identity-linkage-only-no-claim-promotion` claim, a `not-formalized` game
   mapping, and the producer's nine non-claims, which are frozen in Lattice. It
   also checks that the receipt names the admitted v1 receipt, global artifact,
   ordered local artifacts and complete local set, and that the annotated local
   set names that complete local set.
4. The verified local set becomes a `PreferenceRecord`. The record holds the
   producer profile, the annotated receipt, the v1 receipt, the complete local
   set, the annotation and annotated-local-set identities, `not-formalized`, the
   non-claims, and one `RoleChoices` entry per role. Each `DeclaredChoice` keeps
   its global and local paths, its labels, its factors (id, owning role, name)
   and its optional world (id, name). Spans are dropped.

The profile then selects the result:

- `ExactLabelBaseline` is the default. It returns the base cohort unchanged and
  reports `factors-ignored-profile-off`.
- `RecordDeclaredFactors` sets `Cohort::preference` and reports
  `factors-recorded`.

## Identity and byte stability

`Cohort::preference` is an `Option` with `#[serde(skip_serializing_if = "Option::is_none")]`;
serde decodes a missing field as `None`, so earlier persisted cohorts still decode.
An unannotated or baseline cohort serializes to the same bytes as before. Its
plan, session, step, action, message, attempt, observation and evidence
identities are therefore unchanged. A recorded cohort serializes its record, so
the record is bound into the plan identity. `create_session` already recomputes
the plan and compares it with the supplied one.

`plan_session` calls `preference::validate_record` before it derives any
identity. For a record, it rejects:

- another producer profile, or missing non-claims (`unsupported-cohort`);
- another v1 receipt or complete local set (`identity-mismatch`);
- a missing or reordered role, or another local artifact;
- a local path that does not reach that role's `InternalChoice`, or labels that
  are not exactly that node's branch labels in order;
- a choice with no factors, or a factor owned by another role;
- a record with no decisions at all.

Path resolution is an iterative walk over the role's Lattice `RoleProgram`.
`Next` moves through a send or receive; `Branch { label }` moves through an
`InternalChoice` or `Offer` branch. This matches the producer's local-path
definition.

## Boundary

Nothing in `core/`, `shell/` or the authority binding reads
`Cohort::preference`. A choice is still planned only from the cursor's exact
`InternalChoice` labels, and its envelopes bind the same fields. Ownership
commits and authority requests are unchanged. The factor names, roles and worlds
never enter an envelope or an authority request. Unknown record fields fail
deserialization, so a record cannot carry a selected label, weight, utility or
authority.

Session inspection adds a `preference` block, skipped when absent. It is not
`safe_diagnostic_text` output: names are the producer's validated lowercase
kebab identifiers of at most 64 bytes, and spans are omitted.

The standalone CLI is not extended. Its `SessionCreateInput` keeps the v1
`RoleSource` with `deny_unknown_fields`, so an annotated source fails before any
session exists. Its unavailable authority and its lack of dispatch are unchanged.

## Producer re-pin

Choregraph `fcbae0aa17bc0eab1345529adb9ff104a46540ef` adds only the preference
module, its schemas, Nickel contract, fixtures and regenerated evidence. The new
compile-time includes are four `schemas/preserves/*.preserves` files, which the
existing `cargoVendorDir` restore already copies. The preference `tests.rs`
fixture include is `#[cfg(test)]` only. The source archive BLAKE3 is
`715d11b6e73ae618f778aaf0a951fb003b5745f54356b6d60e393f0cb20627fc`
(`git archive --format=tar fcbae0aa | b3sum`). The flake narHash is
`sha256-ZByP+RlUb2Gj2M6K28mNXUDdk/ukmjEBSCs8boJn9i0=`, and `nix hash path` over
the extracted archive gives the same value. The historical publication bundle
and candidate identities in the admission receipt are unchanged, with
`publication_receipt_matches_revision = false`.

## Fixtures

`tests/test_role_protocol_preference.rs` compiles the existing two-branch
`InternalChoice`/`Offer` fixture (`common::protocol::choice()`) through
`preference::compile_evaluated`. It also compiles a variant that makes an owned
transfer before the choice, which gives a non-empty local path.

- Positive: the selected profile records and exposes the factors, and the record
  survives persistence and CLI inspection. The default profile ignores the
  factors and leaves the cohort, session and inspection bytes identical to
  `admit_projection`. The baseline and selected sessions make byte-identical
  label, cursor, ownership and denied-authority decisions.
- Negative: the producer and CLI refuse annotated input without opt-in. A
  crossed or tampered receipt or local set is rejected. A forged record fails
  when it narrows or reorders labels, moves a decision, reassigns a factor,
  misaddresses a choice, drops factors, drops a non-claim, names another profile
  or receipt, or carries a selected-label, weight, utility or authority field.
