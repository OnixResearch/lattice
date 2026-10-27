# Inspect bounded content artifacts without executing guest metadata

## Why

The existing runtime extraction owns general profile admission, restricted linking, limits, and normalized outcomes. The proposed Site content-transform pilot needs a narrower artifact profile with metadata that the host can inspect before guest execution.

## What Changes

- Add an optional versioned content profile with statically inspectable ABI metadata.
- Separate core Wasm validity, structural profile admission, optional static findings, and runtime observations.
- Validate output ranges even when a checker reports a bounded output count.
- Keep all resource bounds cumulative and explicit through the existing extraction contracts.

## Impact and ownership

The runtime maintainers own artifact-profile admission and the Wasmtime adapter. Site owns transform semantics, output trust, and publication authority. The first consumer is Site's bounded content-transform pilot. This profile does not replace the extraction's required Aspen and independent-consumer evidence.

## Dependencies

Implementation follows `extract-hardened-wasmtime-runtime`. The new package adds no duplicate runtime extraction. Native lifecycle metadata records that dependency through `cairn change depend add`.

`site/.cairn/changes/pilot-bounded-content-transforms` is a cross-repository evidence input. Consumer adoption requires published immutable contracts and real integration receipts.

## Durable capability

Consumers can admit one small content profile without executing arbitrary getters or importing a universal component world. Repeatability requires final-artifact fixtures and runtime controls, not compiler flags alone.

## Non-goals

No blanket WASI/WIT replacement, Qip CLI dependency, general theorem prover, universal loop checker, production deployment, or claimed runtime implementation from this proposal.

## Reference and readiness

Reference: https://github.com/royalicing/qip, `docs/content-component.md` and `docs/hard-limits.md`. The reviewed mutable `main` is not an immutable source pin. Source pinning, license review, and checker audit precede reuse. Qip's documented bounded-output checker proves only the returned count, not the output pointer or content.

This package is a proposal, not implementation evidence. Supplemental validation uses the current Cairn source policy explicitly. Repository-default policy compatibility remains a separate check and no policy files change in this package.
