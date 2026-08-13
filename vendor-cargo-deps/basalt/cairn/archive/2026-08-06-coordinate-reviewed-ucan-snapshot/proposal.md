# Proposal: Admit exact reviewed UCAN snapshot revisions

## Why

Basalt's importer uses one revision compiled into the tool. A workspace coordinator cannot bind an apply plan to an exact reviewed revision argument.

The importer must accept a full Git object ID. It must reject branch names, abbreviated revisions, and malformed values before import.

## What Changes

- Add exact-revision forms for source parity and import operations.
- Keep the existing commands as compatibility forms for the current reviewed revision.
- Read the recorded revision during snapshot checks.
- Add positive and negative importer tests.

## Impact

- **Files**: Change the vendored UCAN importer, tests, and operator documentation.
- **Snapshot**: Keep the reviewed UCAN bytes and revision unchanged during this pilot.
- **Testing**: Run importer tests, source parity, snapshot checks, focused tests, and the release verifier.
- **Non-claims**: Exact source selection does not prove UCAN correctness, compatibility, release readiness, or authority.
