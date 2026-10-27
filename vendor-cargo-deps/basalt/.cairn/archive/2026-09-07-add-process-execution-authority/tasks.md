## Contract

- [x] [serial] Add the exact process action and bounded resource binding without changing existing identities. r[basalt.ucan_token_flow.process_execution]
- [x] [serial] Add explicit opt-in Nickel policy and freshness checks while keeping default policy unchanged. r[basalt.ucan_token_flow.process_policy]
- [x] [serial] Qualify explicit release scratch storage, invalid configuration rejection, unchanged defaults and complete direct checks. r[basalt.ucan_token_flow.process_release_scratch]

## Evidence

- [x] [serial] Verify signed allow, wrong-purpose/default-policy denial, substitution, revocation and replay controls. r[basalt.ucan_token_flow.process_execution]
- [x] [serial] Run lifecycle and release checks, record scoped evidence, and publish only after acceptance. r[basalt.ucan_token_flow.process_policy]

Evidence: `evidence/README.md`. Source `d2a3fe3`, scratch source `8477593`.
Direct release passed: 240 library-configuration and 253 CLI-configuration tests,
zero skipped in those runs. Optional absent Clankers fixture remains deferred.
Full flake check passed on scratch source. No downstream activation or native
execution is claimed. Publication follows the accepted source and archive gates.
