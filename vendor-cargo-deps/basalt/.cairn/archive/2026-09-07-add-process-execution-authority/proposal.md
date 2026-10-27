# Explicit process execution authority

## Why

Consumers need a holder-signed execution grant for one complete admitted process
context. Document write and OnixOS deployment permissions cannot substitute for
execution purpose. Runtime-effect policy facts alone do not authenticate a token.

## What changes

Add one closed `ProcessExecute` mapping to the existing compact live-authority
adapter. Its resource names a complete consumer-owned invocation-context digest.
Use an explicit opt-in policy artifact. Keep the packaged default policy and
existing operation identities unchanged. Preserve signature, revocation, and
single final replay semantics. No process runs inside Basalt.

## Scope

Basalt owns purpose, lexical resource admission, policy and exact signed bindings.
Consumers own authenticated context resolution, immutable inputs, native limits,
currentness, durable replay, invocation identity, execution and output meaning.
Neural Stream pilot arithmetic and promotion policy do not enter this component.
No UCAN snapshot, crypto dependency or existing protocol profile changes.
