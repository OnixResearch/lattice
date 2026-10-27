# Holder Observation Specification

## Purpose

Defines the `holder-observation` capability.

## Requirements

### Requirement: Use the reviewed published cohort
r[basalt.holder_observation.cohort] Basalt MUST import UCAN object `3a2aa043068a82663cf72ec05ed5d56a7d65ecd5` through its reviewed snapshot mechanism. Runtime and generated cohort identities MUST agree with the imported source and BLAKE3 manifest.

#### Scenario: Exact published import
GIVEN the published UCAN commit
WHEN the importer selects its complete required source closure
THEN source parity and retained license checks MUST pass without an ambient worktree dependency.

#### Scenario: Snapshot substitution
GIVEN the reviewed snapshot manifest
WHEN an imported file or source identity changes outside the importer
THEN the snapshot checker MUST reject it.

### Requirement: Observe only the retained holder admission
r[basalt.holder_observation.boundary] The shell operation MUST require a live Basalt enforcement value and MUST delegate to UCAN using its retained holder authorization. It MUST accept no replay port and MUST NOT create another Basalt or holder authorization.

#### Scenario: Current supplied services
GIVEN a live Basalt enforcement value and the same compact token
WHEN current supplied UCAN services allow the original operation
THEN the method MUST return a borrowed observation with no additional replay admission.

#### Scenario: Denial after initial admission
GIVEN a previously admitted holder
WHEN token binding, time, revocation, key, proof, profile, or caveat checks fail
THEN observation MUST fail without another replay admission or authority receipt.

### Requirement: Preserve bounded evidence
r[basalt.holder_observation.evidence] Documentation and tests MUST distinguish holder observations from current Basalt policy, backend freshness, custody, and release authority. Default and minimal profiles MUST retain the same denial behavior.

#### Scenario: A consumer needs release authority
GIVEN a successful holder observation
WHEN the consumer considers a protected release
THEN it MUST still obtain current Basalt and application authority at its atomic effect boundary.
