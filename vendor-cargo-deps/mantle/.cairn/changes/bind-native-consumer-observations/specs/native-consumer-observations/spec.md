# Native consumer observations

## ADDED Requirements

### Requirement: Exact source-tree remeasurement

r[mantle.native_consumer_observations.source_tree]

The consumer verifier SHALL preserve the existing source-tree identity algorithm, distinguish files from source trees, reject capability-root escapes and symlinks, and report a bounded exact regular-file inventory without absolute host paths or payloads.

#### Scenario: Complete directory closure

- WHEN an admitted materialization bundle names a directory source closure
- GIVEN an explicit consumer capability root and a structurally valid bundle
- THEN verification remeasures its sorted breadth-first file/directory records using the producer's existing bounds and identity encoding
- AND each regular file has an exact source-relative path, content digest and length observation

#### Scenario: Missing or drifted source

- WHEN a required source is unavailable, escapes its capability root or contains an unsupported node
- GIVEN a structurally valid bundle and its declared source identity
- THEN it cannot produce a verified verdict
- AND readable content/shape/length drift is rejected

### Requirement: Signed native asynchronous observations

r[mantle.native_consumer_observations.signed_gateway]

The native gateway SHALL sign complete typed response snapshots only after its existing holder-bound authorization and actual journal/host operations, binding the verified owner, project, audience, canonical operation request digest, producer key and current observation time.

#### Scenario: Authenticated status and event page

- WHEN a current verified holder requests status or events for its own admitted attempt
- GIVEN a native private gateway with admitted holder authority and configured completion key
- THEN the signed response binds exactly the native journal observations and existing fence/result evidence
- AND no unsigned consumer fallback is introduced

#### Scenario: Existing execution and terminal proof

- WHEN submission, recovery, cancellation or result discovery is requested
- GIVEN an existing native gateway execution and durable API journal
- THEN existing durable idempotency, quota, replay, host execution, teardown and result-verification mechanisms remain authoritative
- AND terminal completion signatures and bundle identities retain their existing encodings
