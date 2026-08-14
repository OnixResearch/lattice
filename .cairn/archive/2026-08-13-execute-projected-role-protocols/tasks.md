## Phase 1: Dependencies and baseline

- [x] [depends:choregraph-add-role-choreography-projection] D1 Pin the published Choregraph protocol schemas, global artifact, local-set artifact, projection receipt, adapter cohort, and source revision. r[lattice.role_protocol.intake]
- [x] [depends:authorize-choregraph-effect-intents] D2 Use the accepted Choregraph effect-intent authority boundary before protocol transport or external effects. r[lattice.role_protocol.authorization]
- [x] [serial] B1 Record current workflow persistence, attempt, crash, replay, adapter, credential, authorization, Cairn, Octet, and Nix baselines. r[lattice.role_protocol.validation]

## Phase 2: Pure intake and session planning

- [x] [serial] I1 Define strict frozen adapter DTOs for global, complete local-set, projection-receipt, schema, and proof-reference identities. r[lattice.role_protocol.intake]
- [x] [depends:protocol-intake-dtos] I2 Implement pure canonical intake that rejects stale, mismatched, partial, duplicate, reordered, or unsupported artifacts. r[lattice.role_protocol.intake]
- [x] [serial] I3 Define native and external role placement contracts with exact runtime, contract, adapter, and profile identities and no fallback. r[lattice.role_protocol.placement]
- [x] [depends:role-placement-contract] I4 Implement pure complete-placement admission and deterministic session-plan identity. r[lattice.role_protocol.placement] r[lattice.role_protocol.session_identity]

## Phase 3: Durable protocol core

- [x] [serial] C1 Define pure role cursor, expected action, message envelope, attempt, observation, outcome, and session transition types. r[lattice.role_protocol.state]
- [x] [depends:role-protocol-state] C2 Implement exact envelope admission for protocol, session, roles, local artifacts, step, label, payload tag, value, payload reference, attempt, schema, and adapter cohort. r[lattice.role_protocol.envelopes]
- [x] [depends:role-protocol-state] C3 Implement pure transfer and choice cursor decisions without transport, persistence, clocks, credentials, or rendering. r[lattice.role_protocol.transitions]
- [x] [depends:role-protocol-transitions] C4 Enforce former-owner exclusion after committed transfer and deny stale, duplicate, wrong-role, wrong-step, and reordered input without cursor advancement. r[lattice.role_protocol.ownership] r[lattice.role_protocol.replay]
- [x] [depends:role-protocol-transitions] C5 Implement replay over persisted records with no dispatchable transport or external effect. r[lattice.role_protocol.replay]

## Phase 4: Persistence, dispatch, and uncertainty

- [x] [serial] P1 Add versioned session, role cursor, prepared message attempt, observation, outcome, and terminal record schemas. r[lattice.role_protocol.persistence]
- [x] [depends:protocol-persistence-schema] P2 Persist exact cursor and attempt facts before any transport or external-role dispatch. r[lattice.role_protocol.persistence]
- [x] [depends:prepared-protocol-attempt] P3 Route current authorized actions through selected bounded transport and external-role adapters. r[lattice.role_protocol.authorization] r[lattice.role_protocol.external_roles]
- [x] [depends:protocol-dispatch] P4 Admit receiver observations against the exact expected envelope and commit linked semantic transfer only after accepted observation. r[lattice.role_protocol.transitions] r[lattice.role_protocol.ownership]
- [x] [depends:protocol-dispatch] P5 Preserve crash windows as explicit unknown outcomes and deny automatic resend or branch replay. r[lattice.role_protocol.uncertainty]
- [x] [depends:protocol-uncertainty] P6 Add explicit settle, abandon, and duplicate-risk-acknowledged fresh-attempt recovery with new identities. r[lattice.role_protocol.uncertainty] r[lattice.role_protocol.replay]

## Phase 5: Operator and evidence surfaces

- [x] [serial] O1 Add safe create, inspect, recover, retry, abandon, and replay commands with structured output and bounded diagnostics. r[lattice.role_protocol.diagnostics]
- [x] [serial] O2 Add domain-separated BLAKE3 session, role, step, message, attempt, observation, outcome, and receipt identities. r[lattice.role_protocol.session_identity] r[lattice.role_protocol.evidence]
- [x] [serial] O3 Bind protocol artifacts, placement, authorization, adapters, persistence records, outcomes, recovery actions, and non-claims into bounded evidence. r[lattice.role_protocol.evidence] r[lattice.role_protocol.boundary]

## Phase 6: Positive, negative, and crash evidence

- [x] [parallel] V1 Add positive native-role, transfer, labeled-choice, shared-prefix, chained-ownership, external-role, terminal-failure, replay, and recovery fixtures. r[lattice.role_protocol.fixtures]
- [x] [parallel] V2 Add negative stale-artifact, incomplete-placement, wrong-role, wrong-peer, wrong-step, wrong-local-artifact, wrong-label, wrong-payload, wrong-value, duplicate, and reordered fixtures. r[lattice.role_protocol.fixtures]
- [x] [parallel] V3 Add negative stale-authorization, wrong-contract, missing-credential, replay-dispatch, automatic-retry, unsafe-diagnostic, and authority-overclaim fixtures. r[lattice.role_protocol.authorization] r[lattice.role_protocol.fixtures] r[lattice.role_protocol.boundary]
- [x] [parallel] V4 Add crash fixtures before persistence, after prepared persistence, after dispatch start, after receiver acceptance, and before commit publication. r[lattice.role_protocol.uncertainty] r[lattice.role_protocol.fixtures]

## Phase 7: Documentation and closeout

- [x] [serial] E1 Document protocol intake, placement, envelopes, persistence order, ownership commit, branch labels, recovery, adapter authority, and non-claims. r[lattice.role_protocol.boundary]
- [x] [serial] X1 Run focused protocol, persistence, crash, replay, adapter, authorization, credential, `scripts/check.rs`, Octet, Cairn, and bounded Nix checks. r[lattice.role_protocol.validation]
- [x] [serial] X2 Run the adversarial audit and block archive for duplicate advancement, cross-session acceptance, stale branch choice, stale ownership, replay dispatch, hidden retry, or authority promotion. r[lattice.role_protocol.validation] r[lattice.role_protocol.boundary]
- [x] [serial] X3 Review sync and archive plans and archive only with current Choregraph cohort and non-contradictory runtime evidence. r[lattice.role_protocol.validation]
