# Harden test harness and verification receipts Design

## Context

The repository already packages UCAN/Nickel policy artifacts and has a Rust release verifier. The remaining gap is that some release-readiness behavior is documented but not directly asserted by the default flake check or durable receipts, and several consumer-facing failure modes are not pinned by tests.

## Decisions

### 1. Keep the Rust verifier as the canonical harness

**Choice:** Extend `tools/release-verify.rs` rather than replacing it with shell or Python.

**Rationale:** The repo already uses a Rust-only harness; extending it preserves typed command orchestration and avoids ad hoc scripting.

### 2. Flake check delegates to the canonical harness

**Choice:** Add a `checks.release` Nix check that compiles and runs the Rust release verifier in a sandbox-aware mode. The check depends on the policy export check and records Cargo-dependent rails as deferred because this crate currently depends on a private Git UCAN source that the Nix sandbox cannot fetch.

**Rationale:** `nix flake check` should expose the canonical verifier and produce receipts without pretending sandboxed Nix can run private-network Cargo rails. The direct `nix develop -c` verifier remains the full release gate.

### 3. Receipts are local evidence, not committed release artifacts

**Choice:** Emit `summary.json` and `summary.md` under `target/release-verify` for direct runs, with an env override for Nix checks.

**Rationale:** Operators get durable machine-readable evidence without committing volatile target artifacts.

### 4. Nextest owns integration-test execution, Cargo owns doctests

**Choice:** Run `cargo nextest run` for no-default and CLI-feature test rails, then run `cargo test --doc` separately.

**Rationale:** Nextest improves per-test reporting but Cargo remains the right doctest runner.

## Risks / Trade-offs

- A flake check that runs Cargo against a private SSH dependency can be more environment-sensitive than the existing pure policy check; the verifier still remains directly runnable through `nix develop` for operator evidence.
- Additional CLI negative tests pin current exit/status behavior and should be updated deliberately if operator semantics change.
