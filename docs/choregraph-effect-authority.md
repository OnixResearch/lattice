# Choregraph effect authority

Lattice authorizes only concrete intents returned by the Choregraph V2 handoff.

## Boundary

Choregraph owns machine admission and transition decisions. It returns node, transition, operation, resource, ability, request, and payload facts. These facts are not authority.

Lattice adds machine, workflow revision, run, policy, authority, and optional collaboration-scope identities. It validates the complete binding before product dispatch.

A reviewed Nickel policy defines the Basalt contracts. Workflow graph data cannot change this policy.

## Dispatch order

1. Durably reserve the complete bound intent and attempt.
2. Load current authority and optional scope facts.
3. Validate exact request and scope bindings and enforce the reviewed Basalt policy.
4. Persist any denial before returning; successful authorization alone is not an effect outcome.
5. Invoke only the selected bounded adapter.
6. Persist an observed success, failure, or unknown-after-dispatch outcome.

A duplicate attempt or persistence error stops before the adapter. Replay always denies fresh dispatch.

## Standalone operator profile

The standalone CLI, TUI, scheduler, orchestration, and approval-resume paths use
the optional absolute `effect_authority` path in application configuration. No
selection preserves unavailable authority. An invalid selected profile denies;
it never falls back to another provider. Explicit embedding providers remain a
separate host-owned choice.

This is local process authority from a trusted operator, not externally verified
UCAN evidence. The profile cannot manufacture token signatures, proof chains,
holders, or verification receipts. Basalt's reviewed policy remains an upper
bound on either provenance.

From the README's isolated daily-digest setup, inspect the saved workflow:

```sh
cargo run --locked -- --config "$demo/config.ncl" authority inspect daily-digest \
  > "$demo/authority-requirements.json"
```

Review the workflow commands and the reported `schema`, `workflow_revision`,
`machine_identity`, `policy_ref`, and `grants`. Inspection is read-only: it does
not create missing stores, grant authority, or execute work. A changed workflow
needs new inspection and an explicitly reviewed replacement profile.

After review, this example grants exactly those requirements for one hour:

```sh
lifetime_seconds=3600
deadline_ms=$(( ($(date -u +%s) + lifetime_seconds) * 1000 ))
cat > "$demo/authority.ncl" <<EOF
let contracts = import "$PWD/contracts/standalone-effect-authority.ncl" in
let required = import "authority-requirements.json" in
{
  schema = required.schema,
  workflow_revision = required.workflow_revision,
  machine_identity = required.machine_identity,
  policy_ref = required.policy_ref,
  grants = required.grants,
  expires_at_unix_ms = $deadline_ms,
  roots = {},
} | contracts.StandaloneEffectAuthorityProfile
EOF

cat > "$demo/config.ncl" <<EOF
{
  data_dir = "$demo/state",
  effect_authority = "$demo/authority.ncl",
  default_timeout_secs = 60,
  shell = "sh",
}
EOF

cargo run --locked -- --config "$demo/config.ncl" run daily-digest --output json
```

The contract is [`contracts/standalone-effect-authority.ncl`](../contracts/standalone-effect-authority.ncl).
Profiles admit 1–64 explicit grants, at most 64 roots, and nonempty root IDs of
at most 128 UTF-8 bytes. Node IDs are unique. Each grant binds one exact node,
operation, resource, and ability; wildcards, unknown fields, and unsupported
tuples deny. Process, network, and file grants cannot substitute for one another.
The stock executor still selects `NoopHttpClient`; a network grant does not
install a transport or prove an HTTP request was sent.

Keep the explicit record form above. The pinned embedded evaluator panicked on
`(import "authority-requirements.json") & { ... }` under this contract even
though external `nickel export` accepted it. Native execution, not external
export alone, is the compatibility check for the selected evaluator.

At run admission, the host evaluates the profile, observes time, checks expiry,
and pins the complete evaluated BLAKE3 identity. Before every effect it repeats
evaluation and checks the same identity, exact workflow/machine/run/authority
binding, and finite expiry. Equality with the deadline denies. Removal, semantic
changes to imported inputs, evaluation failure, and clock failure deny subsequent
dispatch. Formatting that preserves the evaluated value does not revoke it.
Protected collaboration scopes cannot use this unscoped local provider.

For file nodes, replace `roots = {}` with operator-controlled absolute paths,
for example `roots = { workspace = "/srv/lattice/workspace" }`, and select
`"workspace"` in the node's `allowed_roots`. The host opens these directories at
run admission. Workflow root names do not open ambient paths; an unknown root,
escaping relative path, or no-follow violation denies the file effect.

The profile and its imports must remain operator-controlled. A process grant
allows the exact workflow's command to run with its configured host environment;
it is not a sandbox. Rechecks are not atomic with external effects and do not
prove global revocation freshness, filesystem or mount stability, exactly-once
execution, remote caller identity, or release authority. Durable attempt
reservation and the existing adapter limits remain required.

## File effects

The host opens `cap_std::fs::Dir` roots from operator-owned configuration and assigns root identifiers. Graph data can name only one configured identifier and a relative path.

`cap-root` performs lexical admission. Existing capability-relative open behavior enforces the Lattice no-follow boundary. Only the host shell opens ambient roots; pure authority admission does not access the filesystem.

## Process and network effects

Process effects use the existing bounded process runner. Network effects use the selected HTTP client after method and URL validation. `cap-std` does not authorize either effect type.

## Scope currentness

Protected effects require an explicit current snapshot from a Lattice-owned port. The snapshot binds scope, revision, membership epoch, audience, consent, resource, ability, request, and snapshot identity.

A missing or stale snapshot denies dispatch. `RunOwner`, trigger history, UI audience, and prior authorization are not scope authority.

## Adoption baseline and source admission

r[impl lattice.choregraph_effect_authority.baseline]

The [pre-integration baseline](../.cairn/archive/2026-08-13-authorize-choregraph-effect-intents/evidence/baseline-and-blocker-2026-07-27.md) records the effect adapters, operator-configured roots, outer admission ordering, absent per-node attempts, replay behavior, Basalt/UCAN sources, supplied-fact boundary, and crash fixtures before integration. The [resolution record](../.cairn/archive/2026-08-13-authorize-choregraph-effect-intents/evidence/blocker-resolution-2026-08-13.md) retains the later pre-change checks and selected immutable V2 intent model.

r[impl lattice.choregraph_effect_authority.baseline.scenario.unknown]

The original baseline left implementation blocked while production intent and machine-identity ordering were unresolved; it did not substitute an external allow boolean or a task checkbox for that prerequisite. These are bounded historical adoption records, not fresh execution results or a new automatic readiness gate.

r[impl lattice.choregraph_effect_authority.source]

The admitted build uses the checked-in Basalt snapshot at `bb8fb5f4a763f532df96b6e65407163ac668834e` and its UCAN snapshot at `277ab768eb532ac0c715a3bbff2a7dc37a702f11`. `vendor-cargo-deps/basalt/SOURCE` binds their revisions and the Basalt archive identity. Cargo selects that exact local snapshot; it has no authenticated upstream fetch fallback.

r[impl lattice.choregraph_effect_authority.source.scenario.ssh]

The original baseline explicitly rejected the direct Basalt dependency because its SSH-only UCAN closure could not be reproduced without source authority. It recorded a release blocker rather than skipping Cargo-dependent checks. The resolution selected the reviewed vendored closure instead; it did not infer source access from a sandbox-safe check that deferred Cargo.

## Non-claims

r[impl lattice.choregraph_effect_authority.non_claims.scenario.scope]

`AuthorizationOutcomeEvent` records the bound local attempt, its local outcome state, and any authorization denial. An observed success is an observation of the selected local adapter result, not evidence of external truth or a global guarantee.

Checks do not prove human identity, membership-store correctness, consent-policy sufficiency, key truth, global revocation freshness, replay-store durability, UCAN cryptographic correctness, operating-system correctness, external effect success, exactly-once behavior, or whole-workflow correctness.
