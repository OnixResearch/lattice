# Agent evaluation

Lattice provides a bounded lane for product agent evaluations. The lane separates deterministic decisions from host effects.

## Catalog

`builtins/agent-evaluations/catalog.ncl` is the built-in catalog. It defines unit, skill, hard, and live cases. Each case binds these facts:

- exact repository, workload, fixture, scaffold, and initial-state identities;
- one agent profile and one environment profile;
- one agent or workflow route;
- positive trial, concurrency, token, duration, and deployment limits;
- deterministic gate implementations and inputs;
- optional advisory rubric and judge identities;
- immutable reference identities and freshness rules;
- private artifact roles, media classes, sizes, retention, and redaction;
- explicit live actor, sandbox target, credential references, and cleanup policy;
- required non-claims.

Lattice captures the local catalog through a capability root. It rejects symlinks, non-regular files, path escapes, missing files, extra files, oversized data, uppercase identities, and unsupported fields. Rust revalidates every Nickel value.

Use this command to validate the catalog:

```text
lattice evaluation catalog --root builtins/agent-evaluations --output json
```

The `agent-evaluation-catalog-review` workflow bundle runs this inert catalog check. It does not start a trial.

## Planning

`policy/agent-evaluation-plan-request.ncl` defines the planning-input contract. The built-in local example is `builtins/agent-evaluations/requests/unit-local.ncl`.

The pure planner sorts selected cases and creates stable trial identities. Planning fails if the observed subject differs from the catalog. It also fails if an exact required service is absent.

```text
lattice evaluation plan \
  --root builtins/agent-evaluations \
  --request builtins/agent-evaluations/requests/unit-local.ncl \
  --output json
```

A prior receipt skips work only when all relevant identities match. This set includes the case, subject, agent, environment, gates, rubric, references, artifact policy, and receipt. Started, partial, cleanup-required, and unknown trials block automatic resume.

Use `resume` with the same catalog and request paths. This command rejects caller-supplied prior claims. It loads bounded durable trial and receipt facts, validates each receipt identity, and then runs the pure planner.

```text
lattice evaluation resume \
  --root builtins/agent-evaluations \
  --request builtins/agent-evaluations/requests/unit-local.ncl \
  --output json
```

## Execution boundary

The host runner uses injected ports for these effects:

- workspace preparation and state capture;
- agent and workflow execution;
- deterministic verification;
- advisory judging;
- live deployment and cleanup;
- artifact collection and private persistence;
- explicit clock observations;
- durable trial starts, terminal records, and receipts.

The production entrypoint uses one explicit command driver. Lattice starts it through `bounded-exec`, clears its environment, bounds input, output, and time, and sends metadata-only JSON. Every response must match the driver, trial, case, profile, environment, policy, and operation. A mismatched or malformed response fails closed.

The runner stores `Started` before it calls an agent, workflow, or deployment operation. A service error records `Unknown`. It does not retry automatically. Cancellation records a durable request and does not delete evidence.

Use `run` with an absolute driver path, an absolute working directory, exact profile bindings, and two caller-supplied clock observations. For a live trial, the first call only prints the exact plan. Add `--confirm-live` to grant the separately admitted live invocation.

```text
lattice evaluation run \
  --root <catalog-root> \
  --request <typed-plan-request.ncl> \
  --trial <b3-trial-identity> \
  --driver <absolute-driver-path> \
  --driver-identity <b3-driver-identity> \
  --agent-profile-identity <b3-agent-profile-identity> \
  --environment-identity <b3-environment-identity> \
  --policy-identity <b3-policy-identity> \
  --current-directory <absolute-directory> \
  --started-epoch-ms <value> \
  --completed-epoch-ms <value>
```

Use these commands for durable state and artifact metadata:

```text
lattice evaluation list --limit 20
lattice evaluation status <b3-trial-identity>
lattice evaluation artifacts <b3-trial-identity> --output json
lattice evaluation cancel <b3-trial-identity> --observed-epoch-ms <value>
```

The caller supplies all observation times. Lattice does not use an ambient clock for authority decisions.

## Hard gates and advisory judgment

Deterministic gates decide pass or fail. Missing, stale, malformed, timed-out, contradictory, or failed gate evidence fails closed.

A rubric score is advisory. A high score cannot override a failed hard gate. Agent output cannot define ground truth, reference material, or final status.

## Live cases

A live case needs an exact authority fact. The fact must match the catalog actor, sandbox namespace, credential-reference set, and deployment limit. Production-like targets are rejected.

Lattice stores a durable start before deployment. Unknown or partial deployment outcomes remain unknown or partial. Cleanup failure cannot become success. A later operator can inspect the durable state before any recovery action.

## Artifacts and receipts

Artifacts must match the case policy. Lattice rejects path escapes, symlinks, remote references, secrets, hidden evaluation material, unsupported media, excessive counts, and excessive bytes.

The `artifacts` command shows only the manifest identity, count, byte count, redaction class, receipt identity, trial identity, and status. Detailed private artifacts remain behind the command driver and its separate authority.

Durable trial records contain caller-supplied observation times. Public receipts contain identities, counts, statuses, safe references, redaction state, and non-claims. They do not contain prompts, transcripts, raw model output, credentials, hidden material, or raw deployment responses.

Content identities prove exact bytes only. They do not prove agent correctness, verifier soundness, judge correctness, benchmark superiority, deployment correctness, release eligibility, or effect uniqueness.

## Animus boundary

The current Lattice dependency snapshot does not include the newer Animus evaluation crates. Lattice therefore accepts only the narrow `animus.native-evaluation-adapter.v1` metadata fact at this boundary. It rejects private content and unbounded item or byte counts.

Current sibling Animus revision `205b3d2c01ee310292a6e4ff2af0259a94779611` is a reviewed design reference. It is not an ambient dependency. A later dependency-cohort change can adopt its evaluation crates through a pinned, normalized source update.
