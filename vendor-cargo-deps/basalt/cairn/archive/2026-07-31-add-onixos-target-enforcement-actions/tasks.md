# Tasks: Add exact OnixOS target enforcement actions

## Contract and API

- [x] [serial] Add six closed OnixOS target action mappings without generic action construction. r[basalt.ucan_token_flow.onixos_target_actions.contract]
- [x] [serial] Add one reviewed Nickel policy contract for each exact target action. r[basalt.ucan_token_flow.onixos_target_actions.policy]
- [x] [parallel] Export the checked-in reviewed UCAN snapshot through `lib.ucanSource`. r[basalt.ucan_token_flow.onixos_target_actions.source]

## Verification

- [x] [parallel] Add positive tests for every action, policy mapping, and distinct operation profile. r[basalt.ucan_token_flow.onixos_target_actions.tests]
- [x] [parallel] Add negative contract, ability, action, resource-prefix, and attenuation substitution tests. r[basalt.ucan_token_flow.onixos_target_actions.tests]
- [x] [serial] Document the downstream source output, exact mappings, and bounded claims. r[basalt.ucan_token_flow.onixos_target_actions.docs]

## Closeout

- [x] [serial] Run focused tests, formatting, Clippy, release checks, downstream checks, and canonical Cairn gates. r[basalt.ucan_token_flow.onixos_target_actions.validation]
- [x] [serial] Sync accepted requirements and archive only after all validation passes. r[basalt.ucan_token_flow.onixos_target_actions.archive]
