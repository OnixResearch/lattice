## Context

Lattice owns bounded workflow and orchestration execution. It already records exact workflow revisions, run identities, budgets, retries, recovery, redaction-safe inspection, and explicit unknown outcomes.

Animus owns reusable Harbor-compatible evaluation planning and receipt contracts. Active Animus changes add stricter trajectory interchange, but this Lattice change does not depend on ambient sibling paths.

The missing layer is a concrete catalog and runner for user-level agent tasks. Cases must connect an immutable subject to one agent route, environment, gate set, rubric, reference set, artifact policy, and finite budget.

camelAI commit `70dd57a6219d9a19b25840aebbf1105c63c3a5f2` is a bounded design reference. Its manifest classifies cases and standardizes emitted evaluation artifacts.

Lattice will keep the manifest-driven workflow. It will replace JSON authoring, raw transcript defaults, Cloudflare deployment coupling, and judge-led rollout decisions.

Completion requires production planner and shell evidence for local and live classes. A static catalog, fake-only score reducer, or unbound transcript file is false completion.

## Success Contract

The accepted result is a typed, identity-bound evaluation lane with these properties:

- one Nickel catalog is authoritative for committed case discovery;
- Rust revalidates every generated case and recomputes BLAKE3 identities;
- pure planning binds case, subject, route, environment, budgets, gates, rubrics, references, and artifacts;
- hard gates remain deterministic and final;
- live effects require separate authority and isolated targets;
- public receipts remain metadata-only;
- private artifacts are bounded and separately admitted;
- positive and negative local and live fixtures pass.

## Decisions

### Decision: use one typed Nickel catalog

**Choice:** Add a typed Nickel catalog that exports deterministic runtime data. Each case has one stable case ID and one immutable case revision identity.

The catalog defines the case class, subject descriptor, workload revision, agent profile, environment profile, trial count, concurrency limit, token and duration limits, network posture, secret-handle declarations, deterministic gates, rubric identity, frozen reference identity, artifact allowlist, live posture, retention, and non-claims.

Rust rejects unknown fields, duplicate IDs, duplicate revisions, unsupported classes, missing bounds, unsafe defaults, contradictory policies, and stale generated data.

Committed case discovery compares catalog entries with declared case assets. A missing entry, undeclared asset, or duplicate ownership fails the catalog rail.

**Rationale:** One source prevents runner scripts, test files, dashboards, and documentation from drifting into separate case lists.

### Decision: define four case classes

**Choice:** The initial classes are:

| Class | Purpose | Effect posture |
| --- | --- | --- |
| `unit` | One mechanism or narrow agent behavior | local deterministic fixture by default |
| `skill` | End-to-end user capability over several steps | bounded local or isolated fixture environment |
| `hard` | High-difficulty skill with stronger ground truth and adversarial gates | bounded local or explicitly admitted live environment |
| `live` | Selected real deployment or external integration path | separate live authority, target, retention, and unknown-outcome policy |

A case can have one primary class. A `hard` case that uses live effects must also enable the live posture and satisfy every live gate.

**Rationale:** Difficulty and effect authority are related but distinct. The model avoids treating every hard case as a live deployment.

### Decision: bind immutable case subjects

**Choice:** A case subject includes repository or workload identity, exact revision, fixture and scaffold identities, initial-state identity, hidden-material posture, and expected mutation scope.

The runner creates or opens only the admitted workspace. It records the observed pre-state identity before agent dispatch and the observed post-state identity before validation.

Case instructions, rubrics, frozen references, deterministic gate code, and hidden materials are outside the agent mutation scope. A subject or evaluator drift requires a new case revision.

**Rationale:** Scores and gates are not meaningful when the task or evaluator changes without a recorded identity change.

### Decision: keep planning pure and execution injected

**Choice:** The pure planner validates the catalog, selected matrix, explicit operator request, subject observations, environment availability, authority, budgets, and resume state.

It emits ordered trial plans with exact BLAKE3 identities. It performs no filesystem, process, provider, network, clock, deployment, artifact, or verifier effect.

The shell calls injected services for workspace preparation, agent execution, workflow execution, deterministic validation, optional judge execution, deployment, artifact storage, and cleanup.

Missing or unavailable services fail before dependent effects. No fallback provider, environment, target, verifier, or artifact store is selected automatically.

**Rationale:** Evaluation logic must remain testable without standing up providers, sandboxes, or deployment systems.

### Decision: separate hard gates from semantic judgment

**Choice:** Deterministic hard gates cover objective persisted state, expected files or records, API behavior, arithmetic ground truth, safety boundaries, harness integrity, artifact contracts, redaction, and declared forbidden effects.

Each gate binds one implementation identity, input identity, expected result class, and evidence schema. Gate execution is an injected verifier effect. Pure reduction accepts only exact current gate evidence.

A rubric judge receives only the admitted review view and immutable rubric identity. Its score is advisory and cannot override a failed, missing, stale, timed-out, malformed, or contradictory hard gate.

Agent prose, tool selection, incidental call order, wording, or efficiency cannot become deterministic hard gates unless the case contract names an exact objective invariant.

**Rationale:** Deterministic facts and semantic quality answer different questions. Combining them creates false passing results.

### Decision: freeze reference evidence outside agent authority

**Choice:** Reference evidence is authored or captured before the trial, BLAKE3-identified, immutable for that case revision, and unavailable for mutation by the agent.

Ground-truth records state their source, capture class, freshness rule, allowed verifier use, and non-claims. Current-information cases fail when required reference freshness is stale or unknown.

The agent cannot author, replace, approve, or select the reference set used to evaluate its own result.

**Rationale:** Agent-authored ground truth makes the evaluation circular.

### Decision: make live effects separately explicit

**Choice:** Live posture requires an explicit operator grant, live environment profile, isolated target namespace, admitted credentials by opaque handle, deployment and cleanup services, finite deployment count, retention policy, and unknown-outcome policy.

The runner reserves a live trial identity before effects. It records started state before deployment or external mutation. Recovery never invents success after an uncertain outcome.

A live result distinguishes deployed, validated, partially observed, failed, cancelled, cleanup-required, and unknown-outcome classes. No live result promotes a model, workflow, template, or release automatically.

**Rationale:** Real deployments provide useful evidence and create real cost and state. They need authority and recovery beyond a normal local case.

### Decision: standardize the result envelope

**Choice:** Every trial emits one safe result envelope. It binds catalog, case, case revision, subject, pre-state, post-state, agent profile, environment, policy, budget, hard-gate set, rubric, reference set, artifact manifest, optional deployment, status, redaction, and receipt identities.

The envelope includes bounded counts and outcome classes. It can reference separately stored artifacts by exact safe relative reference and BLAKE3 identity.

The envelope excludes raw prompts, transcripts, model responses, reasoning, tool arguments or results, credentials, environment values, private paths, hidden tests, unrestricted logs, and judge rationale.

**Rationale:** One envelope makes local, batch, restart, and live results comparable without turning private execution content into ordinary telemetry.

### Decision: keep artifacts separate and allowlisted

**Choice:** A case artifact policy lists allowed roles, media classes, byte limits, count limits, retention classes, redaction rules, and destination classes.

The shell can persist admitted source snapshots, result files, bounded logs, normalized trajectories, screenshots, API samples, or deployment observations. Each artifact receives a content identity and manifest entry.

Undeclared, oversized, unsafe, colliding, symlinked, absolute, remote, secret-bearing, or hidden-material artifacts fail before result acceptance. Public receipts contain only manifest identities and bounded role counts.

**Rationale:** Some evaluations need detailed evidence. Separate artifacts preserve that evidence without widening the safe receipt schema.

### Decision: support exact resume and deterministic ordering

**Choice:** Resume skips a trial only when case revision, subject, route, environment, policy, gate set, rubric, references, artifact policy, terminal status, and expected receipt identity match.

A partial, live-unknown, stale, failed, cancelled, cleanup-required, or artifact-mismatched trial does not become complete through resume. Plan-order rendering remains deterministic even when execution order differs.

**Rationale:** A prior score against another task, model, environment, or evaluator cannot satisfy the current matrix.

### Decision: preserve ownership boundaries

**Choice:** Lattice owns concrete catalog loading, trial orchestration, effects, persistence, commands, and operator UX.

Animus Harbor and trajectory contracts can supply validated adapter facts through pinned package boundaries. Cairn owns lifecycle completion and release evidence. Valence can wrap later evidence identities.

No evaluation score authorizes a workflow effect or proves agent correctness. No Lattice receipt claims verifier soundness or benchmark superiority.

**Rationale:** Evaluation evidence is one bounded observation layer in the stack.

## Trial Flow

```text
Nickel catalog -> generated runtime data -> Rust validation
       |
       v
pure matrix plan + authority + subject + environment preflight
       |
       v
workspace preparation -> durable trial start -> one admitted agent run
       |
       v
post-state capture -> deterministic hard gates -> optional rubric judge
       |
       v
artifact admission -> pure result reduction -> safe receipt persistence
       |
       v
optional cleanup and explicit live unknown-outcome handling
```

## Adversarial Cases

- A test asset exists without a catalog entry.
- The catalog names one case twice with different definitions.
- The subject revision changes after planning.
- The agent edits its rubric, reference evidence, or gate code.
- A judge passes while one hard gate fails.
- A hard case silently gains live network or deployment authority.
- A live deployment starts but no terminal result is observed.
- An artifact contains a secret, hidden test, absolute path, or symlink escape.
- Resume finds a passing result for another model or environment.
- A receipt draft contains raw transcript text or benchmark superiority claims.

Each case must fail before the prohibited effect, result acceptance, or claim.

## Alternatives Considered

### Add independent test files and discover them by naming convention

Rejected because file names cannot bind full case, subject, environment, gate, rubric, reference, artifact, and authority policy.

### Reuse only the generic batch trajectory runner

Rejected because batch ordering and resume do not define evaluation subjects, hard gates, live authority, or semantic judgment boundaries.

### Use a judge score as the primary pass signal

Rejected because judge output is advisory and cannot replace objective state, safety, or harness validation.

### Store every raw transcript in the result database

Rejected because transcripts contain private prompts, tool bodies, credentials, paths, and hidden reasoning. Explicit private artifacts remain optional.

### Copy camelAI's Cloudflare evaluation stack

Rejected because Lattice owns local and portable workflow execution. Cloudflare deployment, Miniflare, R2, and Access are not reusable stack requirements.

## Risks and Trade-offs

- Exact identities create catalog maintenance when tasks or evaluators change.
- Live cases cost more and can leave uncertain external state. Explicit retention and unknown outcomes make this visible.
- Hard gates can overfit narrow behavior if authors encode incidental actions. Case review must focus on objective outcomes and safety.
- Advisory judgment can vary across models. It cannot promote or override deterministic evidence.
- This design does not prove evaluator, provider, deployment, or model correctness.

## Non-Claims

This design does not prove agent correctness, model quality, verifier soundness, judge correctness, benchmark superiority, provider billing truth, deployment correctness, remote effect uniqueness, transcript completeness, hidden-test secrecy, production readiness, or release eligibility.
