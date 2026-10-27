## Phase 1: Upstream cohort

- [x] [serial] Verify and pin archived UCAN final-profile wire and authorization evidence, support catalog, source revision, file manifest, and licenses. r[basalt.ucan_token_flow.v1_profile.upstream]
- [x] [depends:basalt.ucan_token_flow.v1_profile.upstream] Refresh the reviewed vendored UCAN snapshot through the existing exact-revision importer and release checks. r[basalt.ucan_token_flow.v1_profile.upstream]
- [x] [parallel] Add negative missing, stale, failed, unsupported, untested, profile-mismatch, manifest-mismatch, and license-mismatch evidence cases. r[basalt.ucan_token_flow.v1_profile.upstream]

## Phase 2: Final-profile application adapter

- [x] [depends:basalt.ucan_token_flow.v1_profile.upstream] Add a separate adapter from UCAN's live verified final-profile Invocation into the Basalt-owned authority input. r[basalt.ucan_token_flow.v1_profile.adapter]
- [x] [depends:basalt.ucan_token_flow.v1_profile.adapter] Add typed Nickel command-to-action and subject/resource mappings with exact cohort binding. r[basalt.ucan_token_flow.v1_profile.mapping]
- [x] [parallel] Add positive exact mapping cases and negative unknown, duplicate, ambiguous, missing-fact, lossy, caller-assertion, and adapter-substitution cases. r[basalt.ucan_token_flow.v1_profile.tests]

## Phase 3: Authority and evidence

- [x] [serial] Compose final-profile mapping with the existing pure authority core and final replay boundary. r[basalt.ucan_token_flow.v1_profile.authority]
- [x] [parallel] Assert every UCAN or Basalt preflight denial makes zero final replay calls and no second replay occurs. r[basalt.ucan_token_flow.v1_profile.authority]
- [x] [serial] Extend redacted receipts with exact final profile, UCAN cohort, mapping, and policy identities. r[basalt.ucan_token_flow.v1_profile.receipt]
- [x] [parallel] Add receipt redaction, determinism, cross-profile substitution, and persisted-evidence non-authority tests. r[basalt.ucan_token_flow.v1_profile.receipt]

## Phase 4: Validation

- [x] [serial] Document the final-profile application mapping, retained compact path, upgrade boundary, and non-claims. r[basalt.ucan_token_flow.v1_profile.docs]
- [x] [serial] Run policy freshness, vendored snapshot checks, focused positive and negative tests, formatting, Clippy, release verification, Cairn gates, and `nix flake check -L`. r[basalt.ucan_token_flow.v1_profile.validation]
