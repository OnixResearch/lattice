# Design: Enforce the runtime boundary before effects

## Decisions

- The engine receives complete admitted feature facts and resource limits,
  revalidates them and sets every supported Wasmtime feature explicitly. Raw
  arbitrary Config is no longer an admission bypass.
- The shell measures supplied artifact bytes, validates class and compiled
  import/export names against materialization facts, and bounds compilation input.
  Consumers remain responsible for trusting their supplied materialization facts.
- Module and component adapters share cumulative store limits and stable failure
  classification. Components lift owned byte results under Wasmtime hostcall
  fuel before automatic post-return cleanup; lazy views would observe freed or
  overwritten guest buffers. Error payloads count against the transfer bound.
  No universal product effect ABI is installed.
- Consumer-owned binding closures install each admitted descriptor. The compiled
  artifact must import exactly the admitted descriptor names; an empty descriptor
  set yields an empty linker. Host denial remains distinct from guest traps.
- A host-owned epoch handle is tied to the exact engine. Every invocation requires
  that handle and a positive finite budget. The runtime starts no timer or thread.
- New APIs replace obsolete paths rather than aliasing unsafe constructors.
  Existing live callers and executable regressions migrate together. Historical
  branches are not mainline product adoption.
- Wasmtime 45.0.0's transfer-limit error is private. Recognize its exact pinned
  root diagnostic without allocating; typed traps/host denial take precedence,
  and unknown errors remain `RuntimeFailed`. Behavioral limit regressions gate
  source-cohort changes; wording-only tests are not evidence.


## Verification

Retain the failing oversized-artifact regression, execute positive/negative feature,
shape, import, memory, fuel, deadline, pointer and output scenarios, and invoke two
consumer worlds through actual Wasmtime. Run workspace tests, strict Clippy,
formatting, Octet, Cairn gates and available Nix checks. A gate is source evidence,
not proof of arbitrary guest correctness, isolation or release eligibility.
