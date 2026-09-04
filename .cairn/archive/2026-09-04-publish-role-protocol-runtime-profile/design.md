## Context

Lattice already owns the durable projected-role core and shell. Its public `protocol` CLI supports create, inspect, recover, retry, abandon, and replay. The CLI intentionally constructs unavailable authority and empty adapter registries, so it cannot dispatch protocol effects.

OnixOS needs an exact producer profile before it can bind native roles to machine services. Deriving that profile from Rust source or free-form documentation would create a second policy owner.

## Success Contract

The accepted result is one closed, canonical, source-linked profile for the existing operator-invoked runtime surface. It is sufficient for consumer admission and dry-run service planning. It does not claim an effect-dispatching daemon.

## Approach Registry

| Family | Mechanism | State |
| --- | --- | --- |
| Typed canonical profile | Closed Nickel source, canonical JSON, BLAKE3 identity, source manifest | selected |
| Consumer inference | OnixOS scans Lattice source and guesses commands | rejected |
| Pretend daemon | Describe unsupported automatic runtime behavior | rejected |
| New service implementation | Add a long-running protocol daemon | deferred |

## Decisions

### Decision: publish observed capability, not desired capability

**Choice:** The profile records the exact current CLI commands and states `automatic_dispatch = false`, `start_at_boot = false`, and `effect_dispatch = "library_only"`.

**Rationale:** The CLI has unavailable authority and empty adapter registries. A stronger profile would be false.

### Decision: keep runtime and profile revisions distinct

**Choice:** The profile binds runtime revision `cbb5db0e4844b466dcd1877218eb1f00a2a7caf2`. The final profile publication commit is a separate consumer pin.

**Rationale:** A tracked file cannot safely contain its own future Git revision. The runtime revision and canonical profile identity together provide immutable linkage.

### Decision: use canonical JSON identity

**Choice:** Nickel remains the human-authored source. The maintained check exports JSON, sorts object keys, compares the committed JSON, and verifies its BLAKE3 sidecar.

**Rationale:** Consumers need deterministic bytes without treating JSON as runtime authority.

### Decision: bind selected runtime sources

**Choice:** A sorted BLAKE3 manifest covers the projected-role protocol core, shell, model, evidence, CLI adapter, policy, and operator documentation at the selected runtime revision.

**Rationale:** The Git revision binds the complete tree. The selected manifest makes the consumed runtime boundary reviewable.

### Decision: keep service integration consumer-controlled

**Choice:** The profile supplies package, binary, command prefix, state backend, service mode, supported commands, and required placeholders. The consumer still selects machine, user, paths, package derivation, policy references, and activation.

**Rationale:** Lattice owns runtime behavior. OnixOS owns target placement and machine assembly.

## Functional Core and Shell

The Nickel contract and profile are pure data. Canonical export and digest verification are build-time checks. No profile evaluation reads files, starts processes, opens sockets, loads credentials, or grants authority.

## Validation Strategy

Positive evaluation exports the exact profile. Negative fixtures reject an unsupported runtime revision, automatic boot start, claimed CLI effect dispatch, widened commands, and unknown fields.

The source-manifest check runs in an exact checkout of the selected runtime revision. The canonical profile check verifies Nickel-to-JSON equality and the BLAKE3 sidecar.

## Adversarial Audit

The audit must try to show that the profile:

- enables automatic startup;
- claims CLI effect dispatch;
- adds an undeclared command;
- accepts a stale runtime revision;
- embeds a host path, endpoint, credential, or secret;
- grants deployment or adapter authority; or
- claims behavior beyond the selected source manifest.

Any successful counterexample blocks archive.

## Risks and Trade-offs

- The operator-oneshot profile cannot run autonomous protocol traffic.
- A future daemon requires a new profile identity and separate review.
- Consumers must retain blockers when requested behavior exceeds this profile.

## Non-Claims

This design does not prove runtime correctness, transport security, endpoint identity, external-role correctness, exactly-once delivery, global progress, deployment success, production readiness, or release eligibility.
