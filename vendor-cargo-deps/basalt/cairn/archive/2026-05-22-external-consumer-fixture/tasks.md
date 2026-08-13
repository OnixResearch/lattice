# Tasks

- [x] [serial] Add or update a narrow `../clankers/` downstream fixture/gate that depends on Basalt through the public package path or explicit sibling override. r[crate_consumability.external_consumer_fixture.separate_consumer]
- [x] [serial] Prove the Clankers fixture builds with Basalt default library features and without Basalt CLI-only dependencies. r[crate_consumability.external_consumer_fixture.separate_consumer.build]
- [x] [serial] Exercise packaged policy, supplied policy JSON, public request construction, enforcement receipts, and fail-closed consumer mistakes from the Clankers fixture. r[crate_consumability.external_consumer_fixture.public_api_behavior]
- [x] [serial] Add release-verifier receipt coverage for the Clankers consumer fixture step including bounded output and explicit `../clankers/`/private-dependency deferral. r[crate_consumability.external_consumer_fixture.release_evidence]
- [x] [serial] Run Basalt Cargo tests, the Clankers consumer fixture, clippy, Nix release checks, Cairn validate/gates, and `git diff --check` before archive. r[crate_consumability.external_consumer_fixture.release_evidence.receipt_step]
