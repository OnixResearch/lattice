# Durable workflow approvals

Lattice workflows can contain typed approval nodes. An approval node stops a run at a durable checkpoint.

The process does not wait for an operator. Lattice writes the checkpoint before it reports `suspended`, then the process can exit.

## Ownership boundary

Choregraph owns the graph suspension and resume-signal transitions. Lattice owns approval requests, authority, expiry, decisions, values, credential references, persistence, and operator actions.

Lattice pins Choregraph revision `0b2000edaf86d5a7a8601246afbc0d526f4a2a07`. The exact lifecycle profile is `choregraph.lifecycle.durable-suspension.v1`.

A Choregraph transition proves only the selected graph-state transition. It does not prove operator identity, approval authority, persistence correctness, or downstream effect success.

## Approval node fields

Use `Workflow::add_approval_node` in Rust. The Nickel workflow format has the same typed fields under `approval`.

Each node declares these facts:

- a stable gate ID and safe summary;
- selected input-port labels and required artifact identities;
- a local or verified Basalt/UCAN authority profile;
- accepted decision abilities and self-approval policy;
- decision and resume expiry windows;
- a bounded reason policy;
- typed resume values and allowed credential-binding names;
- rejection and expiry terminal outcomes;
- the exact required non-claim set.

Workflow validation rejects duplicate gate IDs, unknown fields, invalid bounds, ambiguous policies, unsafe schemas, and missing non-claims.

## Suspension

Run a saved workflow in the normal way:

```text
lattice run <workflow> --output json
```

If execution reaches an approval node, the result status is `suspended`. The store contains the request, workflow revision, run, selected input hashes, artifact identities, event chain, Choregraph events, and prior node outcomes.

No dependent node runs before a separate resume action.

## Inspect and decide

List or inspect safe metadata:

```text
lattice approvals list --output json
lattice approvals show <request-id> --output text
lattice approvals receipt <request-id> --output json
```

These commands omit checkpoint values, node outputs, reasons, resolved credentials, raw tokens, and provider output.

Record a local decision:

```text
lattice approvals decide <request-id> \
  --workflow-revision <revision-id> \
  --actor <actor-id> \
  --decision approve \
  --observed-epoch-ms <epoch-milliseconds>
```

Use `--decision reject` to apply the declared rejection policy. A decision action records facts only. It does not apply a resume signal or execute downstream work.

The library API also accepts already verified Basalt/UCAN facts. Those facts must bind the exact request, revision, node, actor, ability, policy, receipt references, caveat result, replay result, currentness result, and authority-input identity.

Raw tokens are not authority inputs.

## Resume

Resume is a separate command. The caller must supply the observed clock fact. Lattice does not select an ambient clock for decision or resume admission:

```text
lattice workflow resume <run-id> \
  --request <request-id> \
  --workflow-revision <revision-id> \
  --event-head <event-id> \
  --owner <resume-owner-id> \
  --observed-epoch-ms <epoch-milliseconds> \
  --value 'result=string:accepted' \
  --credential 'token=env:TOKEN'
```

Value syntax is `name=type:value`. Supported command-line types are `string`, `json`, hexadecimal `bytes`, and `exitcode`.

Credential arguments contain only an opaque `SecretRef`. Do not put a resolved secret in `--value`.

Resume reloads and validates the active request. It checks the workflow revision, request identity, policy identities, selected input hashes, artifacts, expiry, event chain, Choregraph profile, prior states, and active owner.

Lattice then commits one resume claim and one exact Choregraph resume signal. Only remaining ready nodes can run. Completed nodes stay completed.

A later approval node creates a new request for the same run. Lattice completes the earlier request and changes the active run index atomically.

## Expiry and restart behavior

An expired decision request records the configured terminal outcome. An approved request can also expire before resume.

An exact repeated decision or completed resume is idempotent. A changed actor, decision, reason identity, revision, event head, owner, value, or credential reference fails closed.

If a process stops after it commits a resume claim, the request remains `resuming`. Automatic retry is disabled because downstream outcome can be unknown.

Use inspection evidence to determine the next operator action. Do not edit the database or reset the request state.

## Non-claims

Approval receipts state these limits:

- human identity is not proven;
- independent review is not proven;
- Basalt or UCAN verifier correctness is not proven;
- persistence under arbitrary host loss is not proven;
- downstream effects are not proven exactly once;
- production readiness and release eligibility are not proven.
