## Context

Basalt's human-authored policy source remains Nickel, and the checked-in generated JSON remains the interchange format. Rust should treat that JSON as untrusted policy input and construct validated in-memory indexes before enforcement. The iddqd adoption should therefore live behind a pure conversion layer rather than leak a new serialized shape into downstream policy artifacts.

## Design

### Internal identity maps

Introduce pure policy-index helpers that convert parsed `Policy` data into iddqd `IdOrdMap` values for:

- contracts keyed by `ContractPolicy.id`;
- backend metadata keyed by `ContractBackendPolicy.id`.

`IdOrdMap` is preferred over `IdHashMap` because policy artifacts are externally supplied and deterministic ordering is already useful for diagnostics and stable tests. The conversion must use explicit unique insertion so duplicate ids fail closed instead of overwriting prior entries.

### Serialization boundary

Do not make iddqd the public policy wire format in this change. Serde still accepts and emits the current object-shaped policy JSON. The conversion layer compares each object key with the value id and returns deterministic diagnostics before enforcement when they differ.

### Enforcement integration

Enforcement should read through the validated identity maps after parse-time validation. Existing public helpers may continue returning `&ContractPolicy` or `&ContractBackendPolicy`; callers should not need to know iddqd is used internally.

### Fixtures

Positive fixtures cover valid contracts and backends with matching keys and ids. Negative fixtures cover duplicate ids, map-key/value-id mismatch, empty ids, unsupported backend id, and malformed policy input that previously could reach enforcement with ambiguous identity.

## Non-goals

- Do not change the Nickel policy source language or generated JSON object shape.
- Do not introduce hash-map semantics for externally supplied policy keys.
- Do not change UCAN capability matching, Steel adapter authority, or enforcement receipt claims.
