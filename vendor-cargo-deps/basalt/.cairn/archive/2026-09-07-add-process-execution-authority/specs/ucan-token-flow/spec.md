# Process execution authority

## ADDED Requirements

### Requirement: Explicit release scratch storage
r[basalt.ucan_token_flow.process_release_scratch] The release verifier MAY accept `RELEASE_VERIFY_SCRATCH_DIR` as an absolute normalized UTF-8 non-root path. It MUST reject invalid configuration before checks. It SHALL relocate only its five owned scratch paths, preserve distinct target directories, and record actual commands and Cargo targets. The option MUST NOT skip checks or confer filesystem custody. Unset configuration SHALL preserve existing paths.

#### Scenario: Home-backed verification
- GIVEN an explicit valid scratch directory with available storage
- WHEN the full verifier runs
- THEN bootstrap outputs and executions use the same relocated paths and all existing checks remain selected

#### Scenario: Invalid configuration
- GIVEN an empty, relative, root, control-bearing or non-normalized scratch directory
- WHEN the verifier starts
- THEN it rejects before starting checks


### Requirement: Exact process execution grant
r[basalt.ucan_token_flow.process_execution] Basalt MUST bind process execution to a distinct closed action, contract, ability and exact normalized context resource. It SHALL preserve holder-signature, revocation and final replay checks. A document or deployment grant SHALL NOT substitute for execution authority.

#### Scenario: Exact signed invocation
- GIVEN an explicit execution policy and a valid holder-signed execution request
- WHEN current supplied verification services allow that exact request
- THEN Basalt returns private live authority after exactly one replay commit and runs no process

#### Scenario: Wrong purpose or changed context
- GIVEN document authority or a signature for another execution context
- WHEN execution admission runs
- THEN Basalt denies before replay and exposes no live authority

#### Scenario: Malformed execution resource
- GIVEN a missing, uppercase, truncated, oversized or non-hex context reference
- WHEN the pure action binder runs
- THEN it denies without token parsing or effects

#### Scenario: Revoked or repeated invocation
- GIVEN revoked credentials or a replayed holder invocation
- WHEN current verification runs
- THEN it denies without returning a live grant

### Requirement: Explicit policy activation
r[basalt.ucan_token_flow.process_policy] Process execution SHALL require a separately selected policy. The packaged default policy and existing operation identities MUST remain unchanged. Receipts SHALL NOT authenticate native inputs or confer effect, custody, durability or promotion guarantees.

#### Scenario: Default policy remains restrictive
- GIVEN otherwise valid process authority and the packaged default policy
- WHEN admission runs
- THEN it denies before replay because execution policy was not selected

#### Scenario: Receipt is persisted
- GIVEN a successful execution-purpose verification receipt
- WHEN it is stored or read
- THEN it cannot construct live authority or prove that any process ran
