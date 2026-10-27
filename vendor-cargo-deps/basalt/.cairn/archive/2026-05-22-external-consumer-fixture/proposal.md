# Proposal: external consumer fixture

## Summary

Add a checked consumer fixture that builds `../clankers/` against Basalt's public package surface so crate-consumability regressions are caught before release in the intended downstream agent runtime.

## Problem

The current integration tests import Basalt like a downstream app, but they still live inside the same Cargo package. `../clankers/` is the real downstream consumer for Steel/UCAN agent-runtime work, so using it as the fixture catches package metadata, feature, artifact, and public API issues that in-package tests can miss.

## Scope

- Add or update a narrow `../clankers/` fixture/gate that consumes Basalt as a downstream dependency.
- Depend on Basalt through the public package path or an explicit local override suitable for sibling-repo development, using the default library feature set.
- Exercise packaged default policy, supplied policy JSON, enforcement receipts, and optional CLI feature smoke where appropriate.
- Keep the fixture deterministic and runnable through the release verifier.

## Non-goals

- No publishing to a registry.
- No remote deployment or network dependency beyond the existing pinned UCAN source requirements.
- No broad Clankers architecture rewrite; this is a narrow consumer-compatibility fixture.
