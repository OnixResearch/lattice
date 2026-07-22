# Finish webhook authentication boundary evidence

## Summary

Close the final two reviewed credential gaps with direct valid-request persistence and failed-auth rendered/persisted diagnostic observations, without adding selected markers.

## Scope

- Add a domain-separated BLAKE3 credential-reference fingerprint to authentication diagnostics and rejected history.
- Keep the optional field absent from unrelated records for schema compatibility.
- Prove valid authentication persists no plaintext value.
- Prove failed authentication persists only safe trigger and fingerprint context.
- Reclassify only exact rows while preserving 329 missing IDs and zero dangling markers.

## Non-goals

This does not expose raw credential references, change authentication policy, promote markers, prove remote request authenticity, or claim arbitrary confidentiality, complete traceability, release readiness, or production readiness.
