# Process execution contract design

## Mapping

`ProcessExecute` maps to contract `process-execute`, ability `process/execute`,
and resource `onix://process-execution/` followed by exactly 64 lowercase hex digits.
The resource identifies one complete consumer-admitted context, not just a binary.
The consumer must bind invocation, executable/argv/environment, inputs, services,
resource ceilings and policy context. Basalt cannot authenticate that preimage.

A new operation profile uses the existing domain and framing with distinct closed
fields. Existing profiles remain unchanged. The holder signature retains exact
resource, operation and request binding. No generic string action constructor exists.

## Policy

The packaged default policy remains byte-identical and denies the new contract.
A separate Nickel-authored opt-in policy declares only the new contract. Source
and generated JSON freshness are checked. It grants nothing without verified UCAN
and holder authority. No default final-profile command mapping is added.

## Execution and evidence

The existing raw verifier obtains current supplied key/proof/revocation/caveat
facts, evaluates explicit policy, builds its receipt, and consumes replay once.
No fallible work follows that final replay commit. Private live authority remains
non-cloneable and non-serializable. Receipts never become live grants.

Tests cover genuine test-key holder signatures, new-purpose/default-policy denial,
wrong document authority, changed resource/holder binding, revocation and replay.
Test keys do not authorize any consumer deployment or process execution.

## Release scratch storage

Full qualification encountered an exhausted host `/tmp`. An explicit
`RELEASE_VERIFY_SCRATCH_DIR` selects normalized absolute UTF-8 storage. Pure layout
validation and exact five-path translation stay separate from environment reading,
directory creation and process execution. The shell creates the directory, maps
bootstrap program/output paths and isolated Cargo targets, and records actual
commands and target paths. Child temporary storage uses the selected directory.
The option does not skip checks, alter authority, clean files, isolate processes,
or authenticate filesystem custody. Unset configuration preserves existing paths.

## Compatibility

No dependency or vendored source upgrade. No imported host type enters the new
pure mapping logic. Existing document/OnixOS behavior, packaged policy bytes and
operation identities remain unchanged. Consumers must qualify a published immutable
owner revision and their own native mapping before activation.
