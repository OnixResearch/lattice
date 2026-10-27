# Peer-auth policy evidence

## Baseline

Basalt `main` at `dd510f9f9839e1f138cfc83ae1a39459ae96783c` passed its direct release verifier and every Nix check before implementation.

The baseline included 196 no-default tests, 209 CLI-feature tests, policy freshness, formatting, Clippy, package checks, and documentation checks.

## Bounded reference

- Repository: `https://github.com/devinautotomic226/celld`
- Revision: `c978a2590e875b003ecd860a86cb7551287eba67`
- Selected source blob: `40d6ac4613d293a148e7222e192153eaeaeb29db`
- License: `Apache-2.0`
- Claim: bounded comparison only

The reference supplies version, body-binding, HMAC, clock, and replay comparison constraints. Basalt does not claim parity or equivalence.

## Implementation

`ucan-core` verifies the authenticated message and returns the opaque `VerifiedAuthenticatedMessage` value.

The pure Basalt core evaluates that value against a checked Nickel policy and caller-supplied action expectations.

The core reads no clock, key store, replay store, transport, environment, process, or filesystem.

An allow binds one authenticated peer message to `dispatch-control-message`. It does not authenticate an end user or grant general authority.

## Positive and negative coverage

The checked matrix includes one positive Steel DTO case and five negative cases.

Rust tests cover exact admission, wrong version, wrong action, missing authority, metadata promotion, stale messages, replay denial, empty messages, digest-role substitution, end-user promotion, secret fields, stale source identity, and reference overclaim.

Eight invalid Nickel policies reject missing facts, unsafe defaults, wrong protocol facts, secret fields, weakened boundaries, and parity claims.

## Receipts

Receipts bind message, body-authentication, implementation, policy, action, authority, and reference identities with BLAKE3.

They exclude message bodies, routing contexts, keys, authentication tags, and end-user identity facts.

## Validation

- The exact Nickel policy and Steel fixture exports passed freshness checks.
- Eight invalid Nickel policies failed closed.
- Five focused Rust peer-auth tests passed.
- Strict Clippy passed for every target and feature with denied warnings.
- The direct release verifier passed 201 no-default tests and 214 CLI-feature tests.
- Formatting, two doctests, package checks, consumer checks, and documentation checks passed.
- Every repository Nix check passed with local builders.
- Cairn repository, proposal, and design validation passed.

## Non-claims

This evidence does not prove transport security, TLS, key custody, clock truth, replay durability, membership, routing, end-user authentication, authorization, or effect execution.
