# Operation-pack baseline, architecture, and reference evidence

Date: 2026-08-15

## Dependency and repository baseline

Implementation starts from `183d5fece4f18087d49eb2449999b2be5e2b051c` after the typed-operation and immutable operation-data changes were integrated and archived.

Before core changes, these suites passed:

- `test_operation_contracts`: 16 tests;
- `test_operation_data`: 17 tests;
- `test_doctor`: 5 tests.

The current product already owns strict operation catalogs, data-only instances, exact invocation plans, Bounded Exec, metadata-only run evidence, immutable operation data, passive Animus skill descriptors, manifest-driven agent evaluation, and deterministic doctor facts.

The baseline `octet-precommit-check` status was warning-only with zero new, stale, or error findings. It reported 855 accepted and current identities, plus 1,085 existing warning findings.

## Exact integration baseline

- The closed operation registry admits `cairn.repository` through `cairn-read-only-v1` only.
- Bounded Exec is pinned at `29dac88ecded94457572db3fdfaaaab95fa91525`.
- Nickel Eval is pinned at `bf16cf223ed3dfe75a28f02ac21ed7891a60bac1`.
- Content Identity Core is pinned at `7f55597b5dc879b7601856e8d7fd0dbacaa2a498`.
- Cap Root is pinned at `545e77e5a49e4e1d34037aa9a82d924944ac0a1b`.
- Animus remains pinned at `f1662a8ed290b170c3986d2ddf2909b36d7e5fdc`.
- The sibling Animus revision `205b3d2c01ee310292a6e4ff2af0259a94779611` is reference-only.
- Credential providers already expose references separately from bounded secret resolution.
- Doctor already uses deterministic typed facts and safe remediation text.
- Local pack caches and configured mirrors are optional. Offline exact directories remain valid inputs.
- The Nix baseline uses vendored Cargo sources and supports local offline validation.

## Admission evidence inventory

Pack admission consumes supplied facts only. It can bind Mantle build, Artifact authentication, Valence `RecordedOnly`, Cairn promotion, Basalt authority, and UCAN authority roles.

Radicle identifiers, mirror locators, release channels, deprecation, and supersession remain metadata. They do not replace exact pack, lock, dependency, platform, license, capability, or policy facts.

Valence evidence proves identity and linkage only. Mantle evidence proves bounded build facts only. Cairn evidence proves lifecycle conformance only. Basalt and UCAN evidence proves bounded supplied authority facts only.

## Ownership inventory

Lattice owns manifest meaning, pack identity, dependency and lock admission, capability policy, runtime selection, analyzer policy, skill installation, registry metadata, doctor facts, operation execution, persistence, and operator confirmation.

Bounded Exec owns bounded process mechanics only. Animus supplies passive skill descriptor validation only. Mantle, Artifact, Valence, Cairn, Basalt, and UCAN evidence enters as typed supplied facts. No optional evidence source grants ambient authority.

## Architecture portfolio

Three implementation shapes were reviewed:

1. Add pack fields directly to operation contracts. This was rejected because distribution, registry, analyzer, and skill policy have separate owners and lifecycles.
2. Add an `operation_pack` functional core with a capability-confined shell and visible composition through existing operation, process, data, doctor, and CLI boundaries. This was selected.
3. Add runtime dependencies on every Onix evidence producer. This was rejected because pack admission must validate already loaded facts without invoking producers or transferring their claims.

The selected core owns deterministic validation, identities, admission, lock plans, analyzer views, skill disclosure, channel transitions, registry selection, and doctor decisions. The shell owns Nickel evaluation, bounded file observation, create-only lock and skill publication, Bounded Exec calls, explicit registry reads, and operation-data publication.

## Swamp reference boundary

Reviewed reference: `swamp-club/swamp` at commit `a648c1307fd5cffa029cb8348b2ec20cb727bd66`.

Adopted as product ideas:

- scoped manifests and discoverable operation metadata;
- immutable versioned data and exact local selection;
- explicit trust, channels, deprecation, and supersession;
- passive agent skills and post-run reports;
- explicit local or configured registry access.

Rejected:

- YAML and CEL as pack authority;
- arbitrary TypeScript, JavaScript, Deno, Node, shared-library, callback, or shell-command loading;
- inherited ambient credentials and unrestricted environment;
- automatic pull, push, update, or registry traffic;
- telemetry, popularity, display version, channel, or mirror location as execution authority;
- analyzer success as correctness or release evidence.

Swamp is AGPL-3.0 with its extension and definition exception. This implementation is clean-room and copies no Swamp source. The exception is not treated as permission to reimplement source text.

## camelAI reference boundary

Reviewed reference: `qaml-ai/camelAI` at commit `70dd57a6219d9a19b25840aebbf1105c63c3a5f2`.

The reviewed tree identity is `02b5ed7cd9ad75d9ef86ce6039eb037a5ee460f9`. The repository is MIT licensed.

Lattice adopts only reference ideas for structured manifests and evaluation routing. It rejects Cloudflare coupling, raw transcript defaults, judge-authoritative rollout decisions, and runtime source loading.

No camelAI source is copied or executed. The reference does not become a pack dependency.

## Adversarial search

Closeout must reject any path that lets inspection execute content, mutable labels replace BLAKE3 identities, dependencies resolve ambiguously, optional invalid evidence disappear, capabilities widen, analyzers read ambient secrets or stores, untrusted skill bodies load, channel promotion rebuild bytes, ordinary execution contact a registry, doctor run an ordinary method, or evidence overclaim trust and readiness.

## Non-claims

Pack admission does not prove maintainer trust, source correctness, build correctness, analyzer correctness, skill-routing correctness, remote availability, sandbox completeness, production readiness, or release eligibility.
