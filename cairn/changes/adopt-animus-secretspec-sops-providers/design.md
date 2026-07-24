## Context

Lattice credentials are opaque `SecretRef` values resolved by a configured registry immediately before side-effecting runtimes. The runner hashes reference identity rather than plaintext, and redaction protects output/history surfaces. Animus is the reusable owner for the new authority-free secret-resolution request contract and optional SecretSpec/SOPS std adapters; Lattice is the product owner for workflow configuration, provider selection, effects, persistence, triggers, operator UX, and runtime evidence.

The upstream mechanisms are deliberately distinct. SecretSpec maps a project/profile/key declaration to a configured provider. SOPS decrypts an encrypted file through host-owned key services. The accepted Animus cohort must preserve this distinction and release-gate any future native SecretSpec SOPS provider.

## Dependency Gate

Implementation MUST NOT start from a workspace-relative Animus checkout or an unarchived change. The dependency is satisfied only when:

1. `add-secretspec-sops-host-resolution` is archived and its requirements are present in accepted Animus specs;
2. focused Animus positive/negative conformance and feature-isolation evidence passes;
3. one immutable revision exposes the documented adapter and host APIs;
4. Lattice's sync script imports that exact revision and verifies provenance;
5. license and optional-feature checks show no new default or no-std coupling.

A blocked dependency is a valid status for this follow-up Cairn; copying the proposed APIs locally is not.

## Architecture

```text
workflow SecretRef + Lattice provider registry/config
                         |
                         v
Lattice pure credential request adapter
  -> Animus SecretResolutionRequest + explicit product facts
                         |
                         v
Lattice provider shell
  -> injected Animus SecretSpecHost or SopsHost
                         |
                         v
ephemeral value -> existing Lattice node/trigger boundary
                         |
                         v
Lattice redaction + safe provider/run evidence
```

The pure adapter maps only identities and explicit bounds. The shell selects a configured provider and invokes the pinned Animus host service. Plaintext remains in the shortest existing preparation/execution lifetime and does not enter runner plans or persisted state.

## Decisions

### Decision: add two explicit provider kinds

**Choice:** Nickel configuration gains closed `animus-secretspec` and `animus-sops` variants. SecretSpec fields bind manifest root/path, project, profile, provider alias, reason policy, and bounds. SOPS fields bind encrypted root/path, documented format, key-path mapping policy, exact executable/runtime binding, explicit credential environment-key names, and process/file/value bounds.

**Rationale:** Distinct kinds keep declaration/provider resolution separate from encrypted-file storage and make capability diagnostics honest.

### Decision: preserve Lattice `SecretRef` as the workflow contract

**Choice:** Workflows continue to carry provider id, key, optional field, and binding identities. A pure adapter validates the configured provider and maps the reference to an Animus request. No Animus-specific plaintext or machine config enters workflow files.

**Rationale:** Existing workflows, cache keys, imports, trigger auth, and UI editors already depend on the opaque reference contract. Replacing it would transfer product semantics to a dependency and break compatibility.

### Decision: retain product admission around the reusable host

**Choice:** Lattice validates workflow/provider capability, effect policy, runtime binding, trigger context, reason, and cache/redaction facts before invoking Animus. Animus validates its own request/policy/host boundary. A success from Animus is one bounded value observation, not permission to run a node or trigger.

**Rationale:** Secret resolution cannot grant workflow, network, process, filesystem, trigger, or orchestration authority.

### Decision: deliver per binding and persist nothing

**Choice:** The executor requests only the references required by the current node or trigger, installs each ephemeral value into the existing adapter-specific environment/header/signature boundary, executes under current limits, then drops it. Resolved values, decrypted documents, and value-derived hashes never enter redb, workflow snapshots, runner events, node details, replay, mapping reports, or cache identities.

**Rationale:** Narrow delivery minimizes blast radius and preserves current serialization guarantees.

### Decision: use fixed diagnostic classes

**Choice:** Lattice maps Animus failures to stable local credential codes with safe provider/reference identity. It discards arbitrary provider, SDK, process, stdout, and stderr text before rendering or persistence. Reachability distinguishes configured/unsupported/unavailable/denied/missing/timed-out only when the underlying typed observation supports that distinction.

**Rationale:** Secret-bearing backend errors cannot be made safe by guessing substrings.

### Decision: preserve references across every workflow node kind

**Choice:** Nickel export/import obtains `secret_env` from the workflow credential map independently of `NodeConfig` variant. TUI mutation and deletion use one workflow-level lifecycle operation so credentials follow their node. Cross-kind parity tests cover shell, HTTP, file, image, agent, condition, merge, and subworkflow nodes where references are admitted.

**Rationale:** Provider adoption is unsafe if references survive only shell serialization or become orphaned after UI edits.

### Decision: provenance-pin Animus

**Choice:** Lattice updates the vendored Animus snapshot only through `scripts/sync_animus.rs`, records the immutable source revision and BLAKE3 identities, and checks that the selected optional secret features match the reviewed cohort. Workspace sibling paths remain development inputs only.

**Rationale:** Secret resolution is security-sensitive and must be reproducible from reviewed source.

## Adversarial Audit

| Family | Mechanism | Result |
|---|---|---|
| Extend command provider | Configure `secretspec export` or `sops` as arbitrary commands | Rejected: weak typed capabilities, broad output, and duplicated parser/redaction policy. |
| Embed providers directly | Add SecretSpec and SOPS implementation only in Lattice | Rejected: duplicates reusable Animus boundary and complicates downstream consistency. |
| Replace `SecretRef` | Store Animus request objects in workflows | Rejected: transfers product semantics and harms compatibility. |
| Adapt at registry boundary | Keep Lattice refs/admission and invoke pinned Animus hosts through explicit provider variants | Selected. |

The selected route must survive stale/unsupported Animus revisions, provider-id collisions, malformed profile/key/field mappings, missing reason, agent and webhook paths, reference round trips, node deletion, cache reuse, provider errors with sentinels, encrypted path/symlink escape, executable substitution, timeout/truncation/cleanup uncertainty, redb reopen, replay export, and disabled-feature builds.

## Compatibility and Non-claims

Existing provider configs and serialized workflows remain valid. New variants are opt-in. A build or runtime that does not configure them performs no Animus secret-host, SecretSpec, SOPS, manifest, encrypted-file, credential, or network operation.

Passing tests proves only the reviewed Lattice/Animus adapter cohort and fixtures. It does not prove upstream cryptography, access control, provider truth, secret freshness, audit completeness, host-memory confidentiality, arbitrary external effect safety, or production readiness.
