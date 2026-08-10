# Dangling Tracey marker reconciliation

## Scope and non-claims

This evidence proves that the eight marker identities exposed by corrected Lattice source discovery were classified and reconciled without changing the exact pre-existing missing-requirement set. It does not prove the 369 missing requirements, semantic correctness of every retained marker, complete test adequacy, or release readiness.

## Baseline

The standard maintained command exited unsuccessfully and reported:

- accepted requirements: 696
- referenced requirements: 327
- missing requirements: 369
- dangling markers: 8
- `valid: false`
- input BLAKE3: `9421b315927253e0f868e9cbdaeafc59bf6dfd5241188becf668dbd12f9d96f7`
- receipt BLAKE3: `2160e982d7019220178a37bdf389db31d7ac2dfff3e9f029babd97e7cf2ff0cf`

The dangling set was:

1. `n8nclone.credentials.webhook_auth.provider`
2. `n8nclone.credentials.webhook_auth.redacted`
3. `n8nclone.node_templates.parameters.invalid.bounds`
4. `n8nclone.scheduling.triggers.admission.enabled`
5. `n8nclone.scheduling.triggers.model.persist`
6. `n8nclone.scheduling.triggers.webhook.content_type`
7. `n8nclone.scheduling.triggers.webhook.headless_run`
8. `n8nclone.scheduling.triggers.webhook.method`

## Classification and disposition

| Baseline marker | Disposition | Evidence boundary |
|---|---|---|
| `node_templates.parameters.invalid.bounds` | Added the exact accepted above-maximum scenario | Existing test asserts `NumberAboveMaximum` before command rendering. |
| `triggers.model.persist` | Replaced with accepted `triggers.model.save` | Persistence test now compares the complete saved and loaded `TriggerDefinition`, not only its ID. |
| `webhook_auth.provider` | Replaced with accepted parent `webhook_auth` | Test proves credential-reference resolution and missing-reference rejection, not the complete valid-request scenario. |
| `webhook_auth.redacted` | Replaced with accepted parent `webhook_auth` | Test proves reference-backed authentication and secret omission, but not every narrower redaction-scenario clause. |
| `triggers.webhook.headless_run` | Replaced with accepted `webhook_listener.admission.valid` on the implementation | The shell starts the configured workflow from an admitted plan and records accepted trigger history; that accepted ID was already referenced. |
| `triggers.admission.enabled` | Removed | The same test retains exact idempotency and overlap markers; no accepted enabled scenario exists. |
| `triggers.webhook.method` | Removed | Renaming it to the broader valid/invalid scenarios would promote partial evidence. |
| `triggers.webhook.content_type` | Removed | Renaming it to the broader valid/invalid scenarios would promote partial evidence. |

## Bounded alternatives

- **Mechanical nearest-name renaming** was rejected because it would reduce missing coverage without complete scenario evidence.
- **Deleting every dangling marker** was rejected because it would discard exact bounds evidence and avoid strengthening persistence evidence.
- **Classified reconciliation** was selected because each marker receives an explicit accepted identity or an explicit removal decision.

An adversarial review flagged the partial persistence assertion and approximate webhook/admission mappings. The final change compares complete trigger state, maps the headless implementation only to an already accepted exact scenario, and removes partial test aliases rather than manufacturing coverage.

## Final Tracey result

After accepted-spec synchronization, the maintained command still exited unsuccessfully and reported:

- accepted requirements: 698
- referenced requirements: 329
- missing requirements: 369
- dangling markers: 0
- requirement files: 12
- evidence files: 124
- `valid: false`
- requirement-manifest BLAKE3: `9d1b602aad8155bcc1fe50d1fa0211f1cfee17b7fac4aba9205e31b5401caaf6`
- evidence-manifest BLAKE3: `7ba90dcb545c29631b3cb19e3210130a7f7a889f6f154359120da17a0e51e5a7`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `c42291640f88856ca44d066ca68f2b602d0410f51e1e9a0c258455c62b02448e`
- receipt BLAKE3: `9ce2c5cc15a12242fe1f5393440b8c75c83d70fce7ddf49acb0d1ad0ddd3f967`

A Steel comparison of the ordered baseline and final `missing` arrays returned `missing_exactly_equal: true`. The two-count increase in requirements and references comes only from the exact bounds scenario and the marker-reconciliation scenario.

## Validation

- `cargo test --test test_template_parameters`: 12 passed.
- `cargo test --test test_triggers`: 10 passed.
- `cargo test --test test_traceability_marker_alignment`: 2 passed, with positive accepted-linkage and negative retired-marker cases.
- `cairn policy export --check`: generated policy remained fresh.
- `scripts/check.rs`: passed formatting, complete repository tests, strict project Clippy, Octet admission, and accepted Cairn spec validation.
- Cairn proposal, design, tasks, synchronization, and repository-validation gates passed.
- `nix flake check -L --option secret-key-files ''`: all configured `x86_64-linux` checks passed; incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems were omitted.
