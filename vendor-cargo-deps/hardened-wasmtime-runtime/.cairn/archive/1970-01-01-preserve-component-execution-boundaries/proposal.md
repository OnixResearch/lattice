## Why

Molten's executed frozen corpus at `ce58357f2` found two incompatibilities with the immutable `6381322a68407da71325c4c33113d11ada9af67f` producer. Sliced instantiation work is exposed as callback yields, and one memory bound supplies both a per-memory ceiling and an aggregate budget. Equal final output/fuel does not establish scheduler or resource-policy parity.

The current consumer is Molten's test-only adapter. The target outcome is an explicit instantiation checkpoint and independently admitted memory bounds, enabling an immutable consumer parity run without changing Molten's profile or retaining another engine. Hardened-runtime maintainers own these product-neutral mechanisms; Molten maintainers own product translation, scheduling, authority and adoption acceptance.

## What Changes

- Publish `hwr.shell.v4` with typed yielded phases and a distinct successful instantiation checkpoint before the selected export can execute.
- Preserve one future/store, cumulative fuel and limits, owned output, cancellation and the existing single terminal result.
- Separate per-memory pages from total invocation memory pages in explicit runtime bounds, enforcing both.
- Keep the frozen profile reader's previous aggregate behavior explicit; direct consumers select their own complete resource facts.
- Add actual positive and negative execution regressions and record producer-local evidence. Consumer pins, policy, hostcalls, canonical receipts and backend selection remain outside this repository.

## Impact

- **Files**: resource admission/profile translation, shell engine/store/slicing, actual execution fixtures, public contract documentation and native Cairn artifacts.
- **Testing**: workspace all-target tests, strict Clippy, formatting, pinned deny-all Octet, strict native Cairn gates and host Nix checks; independently pinned Molten parity after publication.
- **Non-claims**: producer publication is not Molten adoption, host isolation, source trust, effect authority, arbitrary guest correctness or release readiness.
