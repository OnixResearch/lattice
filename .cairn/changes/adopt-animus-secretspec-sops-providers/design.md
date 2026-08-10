## Context

Lattice credentials are opaque `SecretRef` values resolved by a configured registry immediately before side-effecting runtimes. The runner identifies the reference rather than plaintext, and redaction protects output and history surfaces.

Animus owns the reusable authority-free SecretSpec host contract. Lattice owns workflow configuration, provider selection, effects, persistence, triggers, operator UX, and runtime evidence.

The assessed upstream cohort is SecretSpec `v0.17.0`, tag commit `a8794e46ec9664a0e1a3869cc3105d0853937e48`. This release includes a native SOPS provider. Animus contains its provider access in an imperative shell and runs providers that can launch processes inside an owned bounded worker. Lattice must consume that boundary rather than add a direct SOPS host.

## Dependency Gate

Implementation MUST NOT start from a workspace-relative Animus checkout or an unarchived change. The dependency is satisfied only when:

1. `add-secretspec-sops-host-resolution` is archived and its requirements are present in accepted Animus specs;
2. focused Animus positive and negative conformance, redaction, bounded-worker, and feature-isolation evidence passes;
3. one immutable revision exposes the documented SecretSpec host API and pins the reviewed upstream cohort;
4. Lattice's sync script imports that exact revision and verifies provenance;
5. license and optional-feature checks show no new default or `no_std` coupling.

A blocked dependency is a valid status. Lattice does not copy proposed APIs locally.

## Architecture

```text
workflow SecretRef + Lattice provider registry/config
                         |
                         v
Lattice pure credential request adapter
  -> Animus SecretSpec resolution request + explicit product facts
                         |
                         v
Lattice provider shell
  -> injected Animus SecretSpec host
  -> SecretSpec Rust SDK and explicit provider profile
  -> bounded Animus worker when provider execution is possible
                         |
                         v
ephemeral value -> existing Lattice node/trigger boundary
                         |
                         v
Lattice redaction + safe provider/run evidence
```

The pure adapter maps identities, profile, scope, provider allowlist, reason, destination, and explicit bounds. The shell selects the configured Lattice provider entry and invokes the pinned Animus host. Plaintext remains in the shortest existing preparation and execution lifetime.

## Decisions

### Decision: add one explicit provider kind

**Choice:** Nickel configuration gains a closed `animus-secretspec` variant. It binds manifest identity, profile, scope, declaration mapping, allowed provider schemes, reason policy, and named positive bounds.

SOPS is selected by the SecretSpec profile and provider policy inside Animus. Lattice does not configure encrypted roots, SOPS executables, key services, or provider credentials in workflow-facing configuration.

**Rationale:** One SDK boundary avoids duplicate SOPS parsing and process policy. Explicit allowed schemes still let Lattice deny a provider class for a workflow or deployment.

### Decision: preserve Lattice `SecretRef` as the workflow contract

**Choice:** Workflows continue to carry provider id, key, optional field, and binding identities. A pure adapter validates the configured provider and maps the reference to an Animus request. No Animus-specific plaintext, provider credential, machine path, or encrypted-file setting enters workflow files.

**Rationale:** Existing workflows, cache keys, imports, trigger auth, and UI editors already depend on the opaque reference contract.

### Decision: retain product admission around the reusable host

**Choice:** Lattice validates workflow and provider capability, effect policy, runtime binding, trigger context, reason, destination, and cache or redaction facts before it invokes Animus. Animus validates its own request, SecretSpec profile and scope, provider policy, and host boundary.

**Rationale:** Successful secret resolution is one bounded value observation. It does not grant workflow, network, process, filesystem, trigger, or orchestration authority.

### Decision: deliver per binding and persist nothing

**Choice:** The executor requests only references required by the current node or trigger. It installs each ephemeral value into the existing adapter-specific environment, header, or signature boundary and then drops it. Resolved values, decrypted documents, provider credentials, and value-derived hashes never enter durable Lattice state.

**Rationale:** Narrow delivery minimizes exposure and preserves serialization guarantees.

### Decision: use fixed diagnostic classes

**Choice:** Lattice maps typed Animus failures to stable local credential codes with safe provider and reference identity. It discards arbitrary provider, SDK, worker, stdout, and stderr text before rendering or persistence.

**Rationale:** Secret-bearing backend errors cannot be made safe by guessing substrings.

### Decision: preserve references across every workflow node kind

**Choice:** Nickel export and import obtains `secret_env` from the workflow credential map independently of `NodeConfig` variant. TUI mutation and deletion use one workflow-level lifecycle operation so credentials follow their node.

**Rationale:** Provider adoption is unsafe if references survive only shell serialization or become orphaned after UI edits.

### Decision: provenance-pin Animus and SecretSpec

**Choice:** Lattice updates the vendored Animus snapshot only through `scripts/sync_animus.rs`. It records the immutable Animus revision, selected secret feature, upstream SecretSpec version, source identities, licenses, and BLAKE3 identities. Workspace sibling paths remain development inputs only.

**Rationale:** Secret resolution is security-sensitive and must be reproducible from reviewed source.

## Adversarial Audit

| Family | Mechanism | Result |
|---|---|---|
| Extend command provider | Configure SecretSpec or SOPS as arbitrary commands | Rejected: weak typed capabilities, broad output, and duplicated parser or redaction policy. |
| Add direct Lattice SOPS | Keep SecretSpec and encrypted-file providers separate in Lattice | Rejected: SecretSpec now owns the SOPS provider, and Animus owns the bounded host boundary. |
| Replace `SecretRef` | Store Animus request objects in workflows | Rejected: transfers product semantics and harms compatibility. |
| Adapt at registry boundary | Keep Lattice references and admission, then invoke one pinned Animus SecretSpec host | Selected. |

The selected route must survive stale or unsupported Animus revisions, provider-id collisions, malformed profile, scope, declaration, and field mappings, disallowed schemes, missing reason, agent and webhook paths, reference round trips, node deletion, cache reuse, provider errors with sentinels, timeout or cancellation, oversized values, worker cleanup uncertainty, redb reopen, replay export, and disabled-feature builds.

## Compatibility and Non-claims

Existing provider configs and serialized workflows remain valid. The new variant is opt-in. A build or runtime without `animus-secretspec` performs no Animus secret-host, SecretSpec, manifest, provider-credential, encrypted-file, executable, or new network operation.

Passing tests proves only the reviewed Lattice, Animus, and SecretSpec adapter cohort under the fixtures. It does not prove upstream cryptography, access control, provider truth, secret freshness, audit completeness, host-memory confidentiality, arbitrary external effect safety, or production readiness.
