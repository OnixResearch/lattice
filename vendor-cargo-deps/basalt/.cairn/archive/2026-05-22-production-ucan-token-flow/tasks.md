# Tasks

- [x] [serial] Add deterministic UCAN test identities and token/delegation fixtures with no production secrets. r[ucan_token_flow.production_ucan_token_flow.secret_free]
- [x] [serial] Add a narrow test helper or adapter that translates verified UCAN token capabilities into Basalt `CapabilityGrant` values. r[ucan_token_flow.production_ucan_token_flow.deterministic_fixture]
- [x] [serial] Cover valid issuance/delegation/attenuation allow paths through Basalt enforcement. r[ucan_token_flow.production_ucan_token_flow.deterministic_fixture.verified_grants]
- [x] [serial] Cover invalid token, wrong audience/issuer, attenuation mismatch, temporal/caveat constraints where supported, and resource/ability mismatch fail-closed paths. r[ucan_token_flow.production_ucan_token_flow.fail_closed_cases]
- [x] [serial] Bind token-flow fixture coverage into release-verifier receipts or record explicit sandbox deferral without overclaiming. r[ucan_token_flow.production_ucan_token_flow.release_coverage]
- [x] [serial] Run direct Cargo fixture tests, release verifier, Nix/Cairn gates, secret search, and `git diff --check` before archive. r[ucan_token_flow.production_ucan_token_flow.release_coverage.receipt]
