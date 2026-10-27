# Process execution verification

Implementation source: `d2a3fe322b3e0c335e5737290d46c63bfaa7cb21`.
Parent: `89675cd4f585f837323c049e4a25f7b94c903038`.

The baseline passed 24 compact enforcement tests. The candidate passed 30 tests,
including six new signed execution-purpose controls. Strict all-target/all-feature
Clippy passed after a private-test-helper visibility correction. Lifecycle commands
initially exited zero, but the task receipt reported missing concurrency markers.
Sync correctly blocked. Explicit serial markers repaired the tasks; the subsequent
sync plan admitted all three requirements with no reasons. Exit zero alone was
not gate acceptance. No new suppression was added.

The complete `nix flake check -L` passed. This includes the opt-in policy freshness
check. The sandboxed release check defers Cargo-dependent acceptance by design.
It is not a substitute for the direct release verifier.

The direct verifier passed policy, source, license, and format stages, then failed
at library-test linking. Mold reported `failed to write to an output file. Disk full?`
and SIGBUS. A pinned coreutils check observed `/tmp` at 100% of its 61 GiB tmpfs,
with 144 KiB available. The home filesystem had 421 GiB available.

A development-only private mount namespace successfully mapped home-backed storage
to `/tmp`. The unchanged release verifier then failed to spawn `rustc` with
`Permission denied (os error 13)`. A shell in that namespace could run `rustc --version`.
The cause of that process-launch difference is unresolved. No release success is
inferred from the namespace probe. No host mount or unrelated file was changed.

## Storage repair and direct acceptance

Scratch implementation source: `8477593094663e6e43a129ce73d9cd7892375790`.
`RELEASE_VERIFY_SCRATCH_DIR` now relocates the five owned paths and child temporary
storage without changing check selection. Seven baseline runner tests passed.
Ten runner tests passed after the change. An actual invalid-root invocation failed
before any check. Standalone compilation has seven existing dead-code warnings;
a trial with additional `-D warnings` rejected those unchanged warnings. No source
suppression was added. The normal baseline command passed before and after.

The complete direct verifier passed in 2 minutes 13 seconds on fresh home-backed
scratch storage: 240 library-configuration tests and 253 CLI-configuration tests,
zero skipped in each run. Doctests, library/CLI Clippy, package inventory, docs,
policy and fixture freshness passed. The optional adjacent Clankers fixture remains
explicitly deferred because it is absent. This is not Clankers compatibility proof.
There were no prevalidated freshness overrides or Cargo-check skips in this run.

Animus and Neural Stream have not adopted this source. No live native admission
or trainer execution is claimed.
Default policy bytes, existing operation fields, dependency pins, vendor snapshot,
Cargo lock, and Nix lock remain unchanged. No process, trainer, or promotion ran.

The earlier storage and namespace failures remain retained, not relabeled as passes.
The successful run used ordinary host execution, not the failed namespace workaround.
Do not treat these test keys as operator authority or this scratch option as custody.

Logs and failed receipts are retained in the initiating Neural Stream checkout:
`.pi/drains/2026-09-07-execution-authority/`.
