# Published UCAN isolation adoption

Source: `7238d3bf3d22d8448eeeb455ee8685dcf8becca6`.
UCAN owner: `927dc32641af6cd1b17202aff44c35acef4540fa`, published and archived.

The maintained importer selected that exact Git revision. Offline snapshot and
source parity checks passed for 693 files. Manifest BLAKE3:
`9054f60e30ac4bfa50abd3b5e030f5c83c39f060b1199111eb5cb54a5403b346`.
No vendored source was manually edited. Cargo and Nix locks remained unchanged.

Basalt now disables UCAN defaults and forwards `ucan/durable-authority` from its
own default-on feature. Both default modules remain available. Explicit opt-out
omits both Basalt and UCAN concrete durable adapters and the OIDC HTTP dependency.
Generic signature, holder, revocation, replay and policy checks remain unchanged.

Actual pinned Rust 1.95 passed 109 library and 28 enforcement tests. The normal
library graph was captured. Default controls passed 109 library, 30 enforcement,
two source-pin and five final-profile tests. Strict all-target Clippy passed for
all-features and no-default profiles.

Revision-bound Nickel projections were regenerated. The receipt golden changed
because its canonical digest explicitly hashes UCAN_RUNTIME_REVISION. The first
run correctly rejected the old golden; the new source-bound digest remains covered
by deterministic and redaction assertions. A formatting failure in the edited
source-pin fixture was fixed with the maintained formatter. Failures are retained.

Producer/release source: `e413cbcf7bb0a2a8294b85cacdbd7ca006859cbf`.
A mistaken Nickel identity-field projection omitted computed producer digests and
was rejected by the full tests. The new example generator calls public Rust APIs;
producer tests now check source/schema as well as computed identities. Both default
and minimal producer tests and 12 release-runner controls passed.

The full library, OIDC and documentation selections now explicitly retain defaults;
a regression test prevents accidental opt-out when new features are introduced.
The complete direct release passed 240 default-library and 253 CLI-configuration
tests, zero skipped in each, plus minimal tests/dependency admission, all freshness
checks, Clippy, doctests, inventory and docs. The absent optional Clankers fixture
remains deferred, not a downstream pass. The subsequent flake run caught an old
exact OIDC manifest-line check; its optional field was updated without relaxing the
pinned URL, revision, checksum or fallback controls. The complete flake check then
passed on source `15f7432`, taking 34 seconds.

This supersedes the historical dependency blocker in README.md. It is not a claim
of native execution, live storage custody, global replay protection or downstream
adoption. Raw logs are in the initiating Neural checkout:
`.pi/drains/2026-09-07-basalt-adoption/`.
