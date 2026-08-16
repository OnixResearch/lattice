# Content-addressed operation packs

Operation packs distribute exact Lattice operation components as data. A pack can contain operation catalogs, external runtimes, analyzers, passive skills, and references.

The pack core validates data and makes deterministic decisions. The shell reads files, runs admitted processes, publishes data, and performs confirmed installation.

## Safety boundary

A pack does not grant authority. Admission does not replace operation authority, credential policy, runtime limits, or output validation.

Lattice admits only canonical identities with this form:

```text
b3:<64 lowercase hexadecimal characters>
```

Lattice rejects absolute content paths, parent traversal, symlinks, unknown file kinds, duplicate paths, undeclared files, and changed bytes.

The directory must contain `manifest.ncl`. Every other regular file must have one exact manifest content entry.

## Manifest authoring

Use the typed contracts in `policy/operation-packs.ncl` for authoring. Rust validates the decoded value again and remains authoritative.

The manifest declares these items:

- pack identifier and safe description
- exact content paths, roles, byte lengths, and BLAKE3 identities
- operation type revisions and allowed methods
- reviewed runtime artifacts, launch labels, platforms, capabilities, and bounds
- analyzer scopes, views, output contracts, retention, sensitivity, and non-claims
- passive skill bodies, references, supported hosts, trust state, and routing cases
- exact dependencies, licenses, admission policy, release metadata, and non-claims

Authoritative text is ASCII. This rule rejects ambiguous Unicode normalization at the admission boundary.

Runtime commands are not manifest strings. Each runtime names one declared executable artifact and one closed launch label.

Lattice rejects Node, Deno, JavaScript, TypeScript, shared libraries, arbitrary commands, and undeclared host callbacks.

## Identity model

The manifest identity covers normalized behavioral fields. The pack identity also covers every declared content identity and byte length.

Runtime, analyzer, skill, dependency, and lock identities use separate BLAKE3 domains.

Descriptions, display versions, publication times, channels, deprecation state, and source locators do not change authoritative pack identity.

A lock binds these facts:

- pack and manifest identities
- exact dependency closure
- selected platform
- runtime, analyzer, and skill identities
- admission profile identity

The lock records a source locator for display. The locator does not affect lock identity or relocated installation validity.

## Admission profiles

Admission is a pure decision over supplied facts. It does not read credentials, contact a registry, install files, or run executors.

A profile can require these evidence roles:

- Mantle build evidence
- Artifact authentication evidence
- Valence `RecordedOnly` evidence
- Cairn promotion evidence
- Basalt authority evidence
- UCAN authority evidence

Evidence must be current, valid, exact, and no stronger than its declared role. Optional but undeclared evidence fails closed.

A lock also requires exact trust, platform, license, capability, and dependency facts.

## External runtimes

Lattice executes reviewed regular executable files through Bounded Exec. It uses explicit arguments, a declared working directory, bounded input, bounded output, deadlines, cancellation, and owned teardown.

The process environment is clear. Pack runtimes cannot inherit ambient credentials or environment variables.

The operation runtime uses the existing typed operation planner and authority adapter. It persists `Started` before dispatch and does not retry unknown effects.

Run reports omit operation output, credentials, secret values, and private paths.

## Analyzers

Analyzers receive only the views in their manifest. Supported views contain safe run metadata and exact operation-data references.

Analyzers do not receive unrestricted store access. They run as external bounded processes with a clear environment.

Lattice validates structured output and optional Markdown before publication. It rejects schema drift, oversized Markdown, private paths, secret fragments, authority claims, and missing non-claims.

A required analyzer blocks parent success when it is missing, failed, timed out, truncated, or invalid. Optional failures remain typed and visible.

Validated analyzer output becomes immutable operation data. The published metadata binds retention and sensitivity policy.

## Passive gateway skill

The fixture pack contains the `lattice-operation-gateway` skill. Discovery returns only skill metadata and identities.

Full skill content appears only after exact pack admission and explicit skill selection. Skill content never executes and grants no authority.

The gateway routes these admitted tasks:

- Cairn lifecycle inspection
- OnixOS configuration inspection
- Mantle build inspection
- Valence evidence inspection

It rejects generic shell tasks and unavailable operation types. Structural positive and negative routing cases are deterministic.

Skill installation and removal are explicit create-only host actions. The removal adapter checks the exact installed skill and body identities.

## Registry and release channels

A registry is a bounded Nickel catalog selected by the operator. Ordinary operation execution never contacts it.

Registry search, exact information, freshness, and restore planning are metadata-only. Restore plans omit destination paths and perform no install or network action.

Promotion changes only registry metadata. The allowed channel sequence is:

```text
beta -> rc -> stable
```

Reverse, missing, ambiguous, and rebuilt promotion requests fail. Deprecation and supersession remain visible during search.

Lattice does not auto-install, auto-update, change a lock, or resolve a content locator during registry inspection.

## Doctor

Pack doctor reports separate facts for manifest, identity, lock, dependencies, platform, runtimes, capabilities, evidence, analyzers, skills, cache, and registry.

Healthy, warning, fatal, and skipped states include safe remediation text. Reports omit content, credentials, and private paths.

A declared no-effect probe is optional. It must use a reviewed runtime, a clear environment, no production credentials, and no mutation or network authority.

Doctor does not run the probe unless the operator supplies `--run-probe` with exact admission inputs.

## CLI workflow

Inspect metadata without loading skill bodies or running content:

```sh
lattice pack inspect --pack /absolute/path/to/pack
```

Preview a lock:

```sh
lattice pack lock \
  --pack /absolute/path/to/pack \
  --confirm-trust-local \
  --allow-license MIT \
  --destination /absolute/path/to/pack.lock.json
```

Add `--confirm-create` to create the lock. Creation is create-only.

Preview a read-only run:

```sh
lattice pack run \
  --pack /absolute/path/to/pack \
  --confirm-trust-local \
  --allow-license MIT \
  --lock /absolute/path/to/pack.lock.json \
  --instances /absolute/path/to/instances.ncl \
  --instance local-repository \
  --method change_list
```

Add `--confirm-read-only` to execute. The command uses the same typed invocation plan as direct operation execution.

Disclose one selected skill:

```sh
lattice pack skill-show \
  --pack /absolute/path/to/pack \
  --confirm-trust-local \
  --allow-license MIT \
  --lock /absolute/path/to/pack.lock.json \
  --name lattice-operation-gateway
```

Use `registry-search`, `registry-info`, and `registry-restore` for inert registry inspection. Use `registry-promote` for confirmed create-only promotion output.

## Offline use

An admitted local directory and exact lock are sufficient for offline inspection, installation, execution, analysis, and skill disclosure.

Offline restore uses an explicitly selected local or mirror locator. It never treats registry availability as execution authority.

## Reference and license boundary

Swamp is a design reference at revision `a648c1307fd5cffa029cb8348b2ec20cb727bd66`. It is licensed under `AGPL-3.0`.

Lattice does not copy Swamp source. The implementation uses clean-room concepts only, and it does not depend on Swamp at runtime.

`qaml-ai/camelAI` is a reference at commit `70dd57a6219d9a19b25840aebbf1105c63c3a5f2`. Its tree is `02b5ed7cd9ad75d9ef86ce6039eb037a5ee460f9`.

The camelAI reference is MIT licensed and reference-only. Lattice does not load or execute its source.

## Non-claims

Operation packs retain these non-claims:

- Pack identity proves exact admitted content only.
- Pack admission does not prove maintainer or source trust.
- Bounded process execution does not prove sandbox completeness.
- Pack evidence does not prove production or release readiness.
- Analyzer output does not strengthen the original operation observation.
- Passive skill content grants no execution authority.
