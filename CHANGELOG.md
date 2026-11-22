# Changelog

All notable changes to Lattice are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
Semantic Versioning. This project is pre-1.0, so minor releases may change
interfaces.

## [Unreleased]

### Added

- Export and inventory of repaired candidates (Lattice #6).
  `implementation-attempt export-candidate` exports the exact final candidate
  revision of a retained `correct_candidate` or `repaired_candidate` repair
  disposition after the single-shot export's request, intent, output and
  revision-seam checks (a revoked revision is still `export_revoked`). Export
  and `inspect` act only on the complete, typed and request-bound record set:
  the disposition (now `lattice.implementation-attempt-repair-observation.v2`)
  and the intent and record (`...repair-step.v2`) of every step. Export
  replays the run in the pure core: repairs within the admitted policy and the
  global bound, every step exited with the loop's parent and failure classes,
  every revision re-derived (`...candidate-revision.v2`, which also binds the
  execute bits) with a receipt bound to this work, attempt, fence, workspace
  and base, and every verdict re-derived from the check observations and the
  retained integrity of each check's copy. The workspace must keep the
  checked execute bits, and the whole receipt, changed paths included, must
  re-derive from it and the retained base digests of the changed paths. New
  `dispatched:true` refusals: `repair_outcome_unknown` (no retained
  disposition, never guessed), `repair_not_candidate` and
  `repair_disposition_invalid`. `inspect` reports a repair disposition as
  `terminal_observed` with its kind, within `lattice.worker-inventory.v3`,
  only when the records agree with the kind and show every started worker
  exited or cancelled; `uncertain` (`worker_outcome`) and unrecorded step
  intents stay `dispatch_recorded_outcome_unknown`, and contradicted records
  are an error. Recovery still prints a first-schema disposition of this
  attempt, which export and inventory never act on; recovery and inventory
  refuse any other first-schema record alike as malformed or crossed.
  Single-shot attempts behave as before.
- Typed `params` and a required `mode` for workflow-template bundle
  manifests (Lattice #27). Schema 2 manifests declare `mode = "read_only"`
  or `"mutating"` and a closed list of parameters typed `bool`, `integer`,
  `enum` or `relative_path`, each with bounds, a default or `required`, and
  1 to 8 `LATTICE_PARAM_*` environment targets on shell nodes. The pure
  `workflow_template::params::resolve` rejects unknown names, wrong types,
  out-of-bounds values and missing required values without echoing a value,
  and returns the values with a BLAKE3 identity. `workflow_template::bind`
  writes them only as environment values and re-runs bundle admission. It
  rebinds an optional orchestration and returns a bundle whose identity
  covers the template, the values and the bound workflow revision. Commands
  may read a bound variable
  only as `"${NAME}"`, and binding never changes a node kind, command or
  `authority inspect` grant. Modes are classified conservatively: only file
  reads and image inputs are read-only, so any shell, operation, HTTP, agent,
  subworkflow or file write is mutating. A `read_only` bundle with such a
  node fails with `mode_violation`. Save and start require a bound bundle
  (`ParamsUnbound`) and an admitted mode (`ModeViolation`), and the TUI
  browser marks each bundle's mode. The four built-in bundles move to schema
  2 as `mutating`. The catalog review's `--root` becomes the typed
  `catalog_root` parameter, and the guarded bundles keep their pinned
  `--root .`. Schema 1 manifests still load as `mutating` without
  parameters; see `docs/workflow-template-bundles.md`. A shell node that
  receives a value must be one simple command of safe literals and whole-word
  `"${NAME}"` references, so a value never takes the command-name position
  and the shell never parses it as syntax. As defense in depth
  (case-insensitive, shells also with a version suffix such as `bash5`), the
  first word may not name a shell, an argument-executing or evaluating
  builtin such as `trap`, `jobs`, `coproc`, `mapfile`, `enable`, `let` or
  `declare`, or a common wrapper such as `tmux` or `hyperfine`; later words
  may not name a shell; and `-exec`, `-execdir`, `-ok`, `-okdir` and `-c` are
  refused. `git` and `find` stay usable. Its `cwd` and `env` may hold no
  `{{…}}` expression (`param_target_expression`), and its static env may not
  set `BASH_FUNC_*`, `%%`, `BASH_ENV`, `ENV`, `SHELLOPTS`, `BASHOPTS` or
  `PS4` keys or hold `$` or a backtick (`param_target_env`). A
  `LATTICE_PARAM_*` credential key is refused. `ValidatedBundle` content is
  read through accessors only, and save, activation planning and start
  re-verify bound values, content, revisions, template and identity, refusing
  a mode failure with `mode_violation` and anything else with
  `bound_content_drift`. A schema 1 manifest that names `mode` or
  `params`, even as `null`, is refused.
- Git-backed runbook catalogs admitted through content-addressed operation
  packs (Lattice #29). A pack can carry workflow-template bundles as
  `workflow_bundle` content under `workflow-templates/<bundle-id>/`, pinned by
  per-file identities and a declared loader `bundle_identity`; the pack
  identity covers every bundle byte, and packs without bundles keep their
  identity. `lattice pack catalog-fetch` is the only networked step: it
  fetches one full SHA-1 or SHA-256 commit from a `file://`, `https://` or
  `ssh://` repository through bounded `git` with a cleared environment and
  lazy fetching disabled. The commit's trees are fetched with
  `--filter=blob:none`, then only the selected blobs. Every file git writes
  is capped at the remaining `--maximum-transfer-bytes` budget, and blob sizes
  and their total are checked before any content is read. It refuses symlink
  and submodule members and an unexpected `--pack-identity`, and publishes a
  verified tree create-only from a private, handle-pinned staging directory.
  Only `file://` is exercised by tests. `lattice pack catalog-install` reads
  local bytes only: pack byte drift, before admission or while capturing the
  bytes to install, refuses the whole pack while naming every drifted bundle;
  after admission each bundle is decided independently (loader diagnostics
  such as compatibility, `identity_mismatch`, `pack_content_mismatch`,
  `catalog_conflict`, `catalog_capacity` at the loader's 64-bundle limit) and
  the good ones are renamed one at a time from a private, handle-pinned
  staging directory. Installation grants no effect authority.
  `lattice pack catalog-list` lists a catalog through the workflow-template
  loader. A seed catalog fixture is retained in
  `tests/fixtures/runbook-catalog`. See `docs/operation-packs.md`.
- Typed `--input` values for `lattice run` and `lattice authority inspect`
  (Lattice #28). `--input NAME=VALUE` is repeatable and `--input-file PATH`
  holds one JSON object of at most 64 KiB; a name may appear only once across
  both. The file is opened without blocking and must be a regular file, and
  `lattice run` reads inputs before it opens the store. Saving a bound
  workflow-template bundle now retains its template binding by identity, and
  a run with inputs loads only that bundle's directory and rebinds through
  the exact template with `workflow_template::bind`, then passes the same
  bound-content re-verification as start (`bound_content_drift`). Inputs are
  refused without a retained binding (save again from the bundle), for a
  template without parameters, after the saved workflow or the template
  drifted, or for a workflow with an approval gate. Wrong, unknown, missing,
  duplicate and secret-like inputs fail before a run id is reserved, a record
  is written or a node runs, and diagnostics never echo a value. The secret
  rule applies to `relative_path` parameters only: a secret word in the name
  or binding key, a value component in a known provider token format, or a
  separator-free run of 20 or more mixed-class letters and digits.
  Credential references remain the only secret path. The run records its
  input identity (template, bound identity, bound workflow revision, value
  identity, names and kinds, no values) in the same transaction that reserves
  its run id. `runs show` displays it in an `inputs` object, and `authority
  inspect` with the same inputs reports the grants of the bound revision. The
  identities are unsalted, so values are omitted from output, not hidden.
  Runs without inputs keep their identity, records and output bytes; see
  `docs/workflow-template-bundles.md#run-inputs`.
- Bounded same-attempt repair and the Kiln #61 budget lineage for
  implementation attempts (Lattice #6). The v3 request accepts two optional
  fields; without them a request keeps its bytes, identity and profile
  revision and runs as before. `repair`
  (`lattice.implementation-attempt-repair-policy.v1`) admits failure classes,
  1 to 8 repair iterations within the envelope's turns, an elapsed bound,
  aggregate model and tool budgets, one check executor and the required
  checks with their configuration identities, and is framed into the profile
  revision. A pure loop decides every spawn and ends in `correct_candidate`,
  `repaired_candidate`, `budget_exhausted`, `cancelled`, `uncertain`,
  `not_repairable`, `checks_rejected`, `lineage_blocked`, `kiln_owned`,
  `fence_superseded` or `authority_withdrawn`. Every candidate is captured on
  the host as a new revision of the same attempt and fence and every declared
  check is rebound to it and run against its own read-only copy of that
  revision; exit status, worker claims and skipped, missing, reconfigured,
  crossed or self-approved checks cannot pass it, and a check whose copy
  changed bytes, file type or permission bits is rejected as
  `candidate_drift`. Each repair step after the first rechecks its fence,
  execution binding and managed revision under the fence lock, and a lost
  binding or revision ends the loop as the post-dispatch
  `authority_withdrawn`. The worker and check ports are given the time left
  before the elapsed bound, read immediately before each call (after the
  step lock and after the check's copy is written), and the clock and the
  charged totals are checked again after each step and before a passing
  disposition, so a step or check
  that overruns the elapsed bound, a policy budget or the lineage remainder
  ends as `budget_exhausted` and an unreadable clock as `uncertain`. A
  cancelled or unknown worker outcome outranks any proposal of changed work.
  Embedders supply `Ports::repair` (worker, check and clock ports); the
  standalone CLI refuses a repair request as `repair_ports_unavailable`. Step
  intents, step records and the disposition are retained write-once under
  `repair/`, and recovery only reports the retained disposition. `lineage`
  (`lattice.kiln-budget-lineage.v1`) maps Kiln `339116f6`'s lineage
  explanation, account remainder and serving gate, and accepts every
  dimension name Kiln's `Alias` admits, reporting the ones Lattice cannot
  charge as unaccounted; a blocker, backpressure or exhausted remainder is
  refused before the fence record, writing only the attempt-fence's closure
  marker (`lineage_blocked`, `lineage_exhausted`), every step draws from the
  one account, and a child is
  admitted only within the remainder. Usage is attributed per phase with the
  existing provenance and unknown counts stay unknown. `cli::Ports` gains the
  `repair` field. See `docs/candidate-worker.md`.
- Workflow-agent revisions (Lattice #25, fixture level). A
  `lattice.workflow-agent-bundle.v1` bundle binds a compiled workflow-intent
  plan (compiler revision and artifact, plan, graph, runtime profile, Cairn
  cohort and per-obligation identities) to one worker bundle. Its definition
  is `lattice-workflow-agent-definition-v1` over the pairing name and the
  worker definition, and its revision the bare-hex
  `lattice-workflow-agent-revision-v1` digest of the binding, so the existing
  lifecycle prepares, qualifies, promotes, pins, revokes and rolls it back
  unchanged, and an evaluation of another plan or of the worker alone is
  `subject_mismatch`. `lattice agent-revision resolve-workflow` and
  `desire-workflow` compose it from an implementation-attempt request and a
  typed intent naming this compiler executable; `inspect` adds
  `workflow_slots` (`lattice.workflow-agent-revision-slots.v1`), the
  projection Kiln #84 would read with workflow and worker revisions apart.
  `workflow::adaptation::propose` describes a changed plan, prompt, tool,
  requirement or policy as a new revision with an applicability decision
  fixed to `unknown` (owners Cairn #14 and Kiln #83) and changes no
  lifecycle. Plain agent bundles keep their bytes and identities. An
  implementation-attempt request may name one in the optional
  `workflow_agent` field (`{definition, revision}`), which is per-attempt
  and not part of the profile revision; absent, a request keeps its bytes,
  identity and profile revision. `seam::subject` is the one subject
  selection: with the field, `seam::admit`, `effect`, `retained` and `close`
  key on the workflow-agent definition and revision, and admission requires
  `--revision-store`, a stored bundle and `workflow::check_attempt` against
  the request's profile revision. An unknown, mismatched or revoked pair, or
  a plain agent bundle named as one, is `revision_not_admitted` with
  `dispatched:false` after the fence record and before any binding or intent.
  `seam::effect`, `retained` and `close` also compare the revision, so a
  request naming another revision of the same definition never acts on a
  pinned session (`session_conflict`). `inspect` reads the attempt's pin
  from the workflow-agent lifecycle. See `docs/agent-revisions.md`.
- An inert draft-schema comparison of the NVIDIA agent-package proposal
  (`697872570744c8cfc1cf6b1d72e04db66e1a2c83`) with Lattice bundle
  identities: a reviewed skill, a plugin that bundles that skill's whole file
  tree, 34 negative cases and the payload-change, relocation, revocation and
  rollback cases through the existing lifecycle. The harness recomputes each
  file's SHA-256 from the supplied bytes and checks the observed layer size
  against its descriptor before bounding the expansion ratio. The version
  annotation is not a direct identity input, but changing it changes the
  manifest digest and so the evidence subject and revision. Decision: useful
  mapping, adoption deferred. No archive, registry, installer or activation
  controller is involved, and no Cargo dependency is added: SHA-256 comes
  from the *ring* provider of the existing `rustls` dev-dependency. See
  `docs/agent-package-comparison.md`.
- Qualification attachments for agent revisions (Animus #18, Lattice #25).
  `lattice agent-revision promote` and `rollback` accept a repeatable
  `--qualification-attachment PATH`. Each file must be a regular,
  non-symlinked file of at most 16 KiB. The attachment port answers through
  the vendored `animus_runtime::qualification::qualify` for the exact
  definition, bare revision, task family and required class. `Qualified`
  keeps its reference, class, protocol, verifier, observation time and
  `fixture`/`live` provenance label (recorded, never gating), and
  each producer refusal maps to the same `not_qualified` reason. The named
  `--qualification-ref` must equal the answer's reference. A qualified answer
  now carries the subject it was produced for, taken from the decoded
  attachment, and promotion and rollback refuse an answer for another
  definition or revision (`subject_mismatch`) or task family
  (`scope_mismatch`) even when it names the requested reference. Without
  attachments the CLI still refuses as `qualification_unavailable` (owner
  `animus#18`). The vendored Animus snapshot moves to
  `e6f27948b19ea2e1bcb67428ca2e12f7acb6e558` (the qualification module with
  its review fixes and workspace-Octet fixes that change no behavior), adding
  only the qualification module. A favorable attachment is evidence, not
  promotion authority, and live efficacy is not claimed. See
  `docs/agent-revisions.md`.
- Immutable agent revision lifecycle (#25, sections 1 to 5). `lattice
  agent-revision` resolves an inert bundle from an exported
  implementation-attempt request, records the desired revision, prepares it
  through an injected preparation port, promotes or rolls back the revision
  selected for new sessions with an exact `qualification_ref`, an expected
  rollout generation and an idempotent request identifier, revokes revisions,
  attachments and sessions, pins sessions and cleans up unretained artifacts.
  The revision is the existing profile revision, used verbatim; the pure core
  is `lattice_core::agent_revision`, and lifecycles persist in a
  version-checked `agent_revision_lifecycles_v1` table. Bounds count only
  what is still referenced (live sessions, a sliding window of receipts,
  revisions nothing references are forgotten), so admission, desire,
  promotion and rollback never exhaust for good, and revoking a referenced
  target is never refused. Every refusal, including a refused
  `admit-session`, exits non-zero. The session-admission seam takes the
  admission mode its caller records, so sessions admitted before a
  definition became managed stay unmanaged. The standalone CLI
  reports preparation as `unknown` and refuses promotion as
  `qualification_unavailable` (owner `animus#18`). `implementation-attempt
  run`, `recover`, `export-candidate` and `inspect` take an optional
  `--revision-store`: a run admits the attempt's session (its envelope
  identity) before writing anything, records the admission mode and pin in
  `intent.json`, checks the effect right before the worker spawns and closes
  the session after the terminal result, where `unknown_session` counts as
  already closed. Without a store an attempt is unmanaged and unchanged. See
  `docs/agent-revisions.md`.
- Coordinator transport for Kiln #45's `kiln-lattice-dispatch --worker-request`
  (#6). Implementation-attempt requests may declare `network:false` and the
  coordinator's `dispatch_plan` identity; `network:true` is refused. A run
  records its attempt per work and fence before any binding or intent, and
  stale fences, crossed attempts and substituted request bytes are refused with
  a typed `lattice.implementation-attempt-refusal.v1` record before any effect
  of that request. Its `dispatched` field states whether a worker effect of
  the attempt may have started: `true` for a substituted request, whose
  attempt directory holds another request's intent, and for a superseded
  request whose directory holds an intent.
  Recovery that finds no intent closes the attempt, so a late or duplicate run
  can never dispatch. Every observation, including binding refusals and the new
  `not_started_closed_by_recovery` status, carries the exact envelope and
  request identity. A `cairn_artifact` source must name its exact
  `change/stage/artifact`; issue and task identity bytes are unchanged. See
  `docs/candidate-worker.md`.
- Worker inventory facts for Kiln #60 (#24, #25). Each profile carries a
  definition identity that is stable across revisions
  (`lattice-worker-definition-v1`). It also carries declared `task_families`
  and `tools`, which stay `unknown` when undeclared and are never empty.
  Lattice #25's desired, prepared, qualified and selected revisions are
  projected from the definition's lifecycle when `inspect` is given
  `--revision-store`, and each instance carries its session's pinned
  revision. Every slot is `unmanaged` (no store or no lifecycle), `unknown`
  with the owner that can supply it, or `recorded`. `task_families` is
  `qualified` only when every declared family has a current qualification of
  the profile's revision, listed in `capability_qualifications` with its
  Animus #18 attachment identity, evidence class and `fixture`/`live`
  provenance. Profiles and non-terminal instances carry bounded
  closed-vocabulary wait reasons that name only published routes, including
  `revision_not_admitted` for a managed definition that would refuse a new
  session. Instances carry their definition identity and single-shot liveness
  (`not_started`, `exited` or `unknown` with `unobserved`/`stale`), never
  idle or live. An optional typed `lattice.worker-capability.v1` declaration
  in `contracts/worker-execution-binding.ncl` and the implementation-attempt
  request supplies the definition name and capability sets. It is part of the
  profile revision only when present, and malformed declarations are refused
  before any effect. See `docs/worker-inventory.md`.
- Default-off declared preference factors on projected role-protocol choices
  (#16). `admit_annotated_projection` admits a published Choregraph
  preference-annotation cohort, verifies its receipt and annotated local set, and
  records factors and world variables only under `record-declared-factors`.
  Exact-label admission, cursors, ownership and authority are unchanged. The
  default profile reports `factors-ignored-profile-off` and leaves the cohort and
  session bytes identical to the unannotated intake. The game mapping is reported
  as not formalized.
- An optional typed policy operation (#23): a versioned profile binds Animus's
  frozen-policy leaf (vendored at `1c370d50`) to one declared Condition node,
  validates receipts independently, resolves typed outcomes through a declared
  fallback, persists bounded decision records in run details (including runs
  that suspend at and resume from approval gates) and replays them without
  adapter work; a resume whose stored decision no longer matches the current
  profile or artifact is refused as stale. Disabled profiles leave runs
  byte-identical; the fixture comparison makes no improvement claim.
- A topological DAG rows view with bounded fork/join lanes, synchronized graph
  selection, live result metadata, and linear/fork-join/diamond snapshots.
- A bounded local planner/generator/evaluator example that runs real Rust builds
  and tests, retains failed-round handoffs, and caps aggregate retries and resources.
- An explicitly selected pinned-reference OMP JSONL candidate profile and typed
  post-start process/capture/teardown observations. The reviewed OMP 18.18.7 build
  passed real gpt-6.1-sol candidate production and local terminal-failure/cancellation
  qualification. Durable continuation remains unsupported by the pinned producer
  contract; independent acceptance and deployment qualification remain separate.
- Opt-in monotonic Octet baseline pruning, checked against complete admitted
  warning occurrences and the baseline identity-count bound so resolved
  findings cannot conceal additional debt.
- Bounded export of exact retained implementation candidates, with owned output
  modes and replay integrity checks; export never dispatches a worker, accepts
  a candidate, or integrates changes.
- An attempt-local model broker over a private Unix socket and network namespace,
  exact endpoint/model admission, host-only credential resolution, bounded calls
  and immutable runtime byte pins. Live OMP qualification remains explicit.
- Material-free configured agent profiles and persisted configuration provenance;
  these do not advertise worker ownership, compute reservations or service readiness.
- Captured workflow-source drafts with durable revision CAS, explicit admission,
  exact source archives and materialized-fork tracking.
- Native graph construction for captured Nickel programs (#20, #21):
  `import --construction` and `authoring draft --construction|--parameterized`
  evaluate captured modules in the companion `lattice-closed-eval-worker`
  (nickel-eval's captured-module profile) and compile them through Choregraph's
  `choregraph.graph-construction.v1` into the existing admission path. Workflows
  retain the observed module closure, template/instance origins (`authoring
  origin`, TUI navigation) and exact v3 runtime bindings; effects stay denied
  until a separate exact v3 grant is selected. Qualification runs on the host with
  `nix run .#construction-runtime-qualification -- NEW_DIR`, because a Nix build
  sandbox cannot present root-owned store executables.
- Worker compute binding for implementation attempts (#24 compute half, #13
  provenance): the `lattice.compute-allocation-observation.v1` consumer DTO for
  Aspen `aac7b5ba` compute-capacity records with bounded pure validation, a
  read-only injected `ComputeAllocationPort` (the standalone CLI uses
  `UnavailableComputePort`; embedders use `run_command_with_ports`), pure
  admission and pre-spawn revalidation, `execution-binding.json` written before
  the intent, `implementation-attempt inspect`, and `Scope::Attempt` usage
  provenance. Producer references stay opaque; no live provider, enforcement or
  serving qualification is claimed.
- Serving-owner request admission for implementation attempts (#24 serving
  half, #13 usage bindings): an optional `lattice.serving-binding.v1`
  declaration frozen into the execution binding, a read-only injected
  `serving::OwnerPort` (standalone `UnavailableOwner`; embedders pass
  `Ports` to `run_command_with_ports`), and Animus #17's pure serving gate run
  per request in the attempt-local model broker. Each request leaves a
  write-once dispatch intent and `lattice.model-request-observation.v1` record,
  summarized into per-service `served` model-request coverage of the attempt
  usage provenance (known versus unknown usage, deduplicated, never reset).
  Animus is re-vendored at `1c370d50` with `animus-frozen-policy`. Record
  evidence labels derive from the port's provenance (`live_endpoint` only for
  an embedder-admitted upstream call, otherwise `fixture` or
  `no_upstream_contact`). The usage summary carries an `overrun` block
  (reserved envelopes, known tokens, per-request overruns, unknown usage,
  refused requests, `is_exact_ceiling` false whenever usage is unknown); an
  optional `stop_on_overrun` refuses later served requests as `overrun-stopped`
  before any port call, and the broker's exhausted request budget is recorded
  once as `budget-exhausted`. `implementation-attempt inspect` reports that
  summary and serving readiness as its actual cause (`no_serving_port`) or the
  injected port's latest snapshot readiness and freshness. Fixture evidence
  only; no live serving owner, endpoint or device was exercised.
- An opt-in pure interaction-net worker profile (`lattice::inet`, #14 and #22) that
  runs the pinned Noble `2677ec88` `ngraph-net-i64/3` worker through the existing
  `bounded_process` operation-pack route. It admits only the canonical acyclic
  two-input wrapping-`i64` subset, re-measures artifacts before dispatch, lowers
  only the producer's fuel/input/retained/output ceilings, binds the report's
  producer revision, contract, limits and input bytes, and publishes results and
  non-authoritative diagnostics through operation data. A semantic interaction
  count is checked only when the receipt names the model theorems and pinned
  proof sources and two fresh admitted evaluations agree on output and count;
  otherwise it stays observed. Not adopted by default; no multicore, device or
  refinement claim.
- A finite, deterministic `workflow-intent` compile/compare boundary over explicit
  host-normalized facts. Plans are inert; typed Cairn/Kiln producer integration
  and execution acceptance remain separate prerequisites.
- An opt-in artifact-bound tiled-render consumer over Aspen's pinned native
  batch/render contracts, with durable unknown-outcome recovery and exact
  accepted-content verification. Remote aggregate-budget enforcement and usage
  telemetry remain unavailable from the selected producer.

### Changed

- Breaking: `agent-revision admit-session` refuses a request's own attempt
  session (its envelope identity) before any state change, with
  `attempt_session: … admitted only by lattice implementation-attempt run`,
  a narrowing relative to `db64cd65`, where it pinned that session outside
  the attempt's fence. Only `implementation-attempt run` admits an attempt's
  session, under the attempt's fence lock and behind its write-ahead record.
  Any other session is still pinned, after `run`'s request validation and
  from the same single read of the request (#6, Lattice #25).
- Breaking: the implementation-attempt profile revision is now the
  `lattice-worker-profile-revision-v2` identity, still spelled as bare hex,
  over profile-level configuration and the reviewed harness template: the OMP
  launch prefix `launch --cwd /workspace` and print options in argv order,
  read from the constants the shell passes to the worker, and the request
  schema. Changing any reviewed flag (tools, mode, `--no-session`, approval
  mode) is a new revision. The base snapshot, task prompt, broker expiry and
  workspace root no longer change it; the exact request bytes bind them.
  Every revision value changes, in requests, inventory and agent revision
  lifecycles. Attempts retained under the earlier framing are refused with
  `profile_revision_superseded` and are never re-executed or recovered;
  prepare new work again (#6, Lattice #25).
- Breaking: the implementation-attempt issue source kind is spelled
  `github_issue`, the name Kiln sends, instead of the derived `git_hub_issue`.
  Its identity tag is unchanged.
- Breaking: `implementation-attempt inspect` prints `lattice.worker-inventory.v3`
  with `lattice.worker-profile-facts.v3` and
  `lattice.worker-instance-observation.v3`, and
  `RuntimeOptions::worker_profile_facts` returns profile facts v3. No v1 or v2
  inventory is emitted or decoded. Readers that deny unknown fields, such as
  Kiln's `worker_inventory` adapter, must move to v3. The profile's
  `qualification_ref` is replaced by `capability_qualifications`, the five
  owned-unknown revision slots by four profile slots and the instance's
  `pinned` slot, and the `revision_selection_unpublished` wait reason by
  `revision_not_admitted`. v2 revisions used the earlier per-attempt framing,
  which the revision change above replaces.
- Breaking: with `--revision-store`, an implementation attempt of a
  lifecycle-managed definition is refused before any write unless its
  revision is selected for new sessions (`revision_not_admitted`), and is
  blocked right before spawn once its session, pinned revision or
  qualification is revoked (`revision_blocked`; recovery reports
  `not_dispatched_revision_blocked` and inventory
  `revision_blocked_before_dispatch`). Its `intent.json` becomes a
  `lattice.implementation-attempt-intent.v1` record of the admission mode and
  pin instead of the bare request identity. Recovering, inspecting or
  exporting a managed attempt requires its `--revision-store`, and exporting
  one is refused as `export_revoked` (`dispatched: true`, since the worker
  ran) once its session, revision or qualification is revoked.
  Attempts run without a store keep today's behavior and intent bytes
  (Lattice #25).
- Re-pinned Choregraph to `fcbae0aa` (Choregraph #4, default-off preference
  annotations). Its v1 receipts are byte-identical to `c3577352`; the source
  admission receipt, flake assertions and frozen role-protocol revision name the
  new revision. As with earlier re-pins, the frozen revision is part of every
  role-protocol cohort, so new session-plan identities differ from `c3577352` ones.
- Adopted published Choregraph `c3577352` and nickel-eval `f0ea0d2a` from their
  canonical Radicle seeds. The producer carries the imported-JSON merge repair, so
  the vendored `nickel-lang-core` snapshot and root patch are removed, along with
  52 transitive crates the crates.io `nickel-lang` REPL/formatter pulled in.
  Construction and restricted-lowering callers use the responsibility-qualified
  Choregraph APIs; persisted workflow/source archive keys are unchanged.
- **Breaking:** implementation-attempt requests are `lattice.implementation-attempt-run.v3`.
  Since 0.1.0 they dropped shared-network/namespace fields (scoped broker
  profiles require explicit readmission) and now require `worker_compute` and
  `execution_binding`; v1 and v2 requests are refused before any effect.
- Tests that need the immutable captured worker or v3 runtime are ignored in plain
  Cargo and `nextest` and run only in the host qualification; captured evaluation
  fails closed when no worker is selected.
- Re-keyed the root and `lattice-core` Octet baselines after the root manifest
  changed (the pinned Octet hashes `Cargo.toml` into finding identities). New
  findings were fixed in code; the semantic finding multiset only shrank
  (root 1055 → 1052, core 25 → 25).
- Removed stale repository-verifier, source-byte and marker-placement snapshot
  tests instead of re-pinning formatting, URL literals, counts, byte hashes or
  private-module locations. Actual Cargo/Nix checks and native lifecycle
  behavior remain the validation surfaces.
- Corrected shell-preview activation fixtures to request only their actual
  round/duration accounting; production worker/model admission remains closed
  without a qualified worker profile.

- Orchestration retains reservation, conservative charges, host duration, partial
  provider observations, and unknown usage separately before settlement. Actual
  cancellation and successful completion settle atomically without treating a
  late cancellation request as observed termination.
- Candidate capture includes admitted ordinary agent/build files and rejects
  unsupported file types and modes instead of silently omitting source changes.
- Agent, worker, orchestration and TUI Rust APIs use owner-qualified names without
  deprecated aliases. Receipt constructors borrow named identity bindings, stored
  candidate files use named artifact addresses, and OMP event counts use `u64`.
  Existing persisted schema and artifact-key encodings remain unchanged.
- Private agent, worker, accounting, storage and TUI implementation phases are
  decomposed without changing producer pins or adding accepted Octet warning facts.

### Fixed

- `implementation-attempt run` no longer lets a superseded fence start a worker
  (Lattice #6, #7). Previously a run checked and recorded its fence, then wrote its
  execution intent without rechecking. A run that had passed its check could
  therefore start after a higher fence was recorded, while a later invocation
  at the lower fence was told `stale_fence` with `dispatched:false`. Now one
  exclusive per-work `flock` lock inside the state root covers the fence check,
  the fence record and the intent, which is created only after the fence is
  rechecked under that lock. The same lock covers each repair step intent,
  where a superseded fence ends the loop as `fence_superseded`, and recovery's
  closure marker. `stale_fence` and `crossed_attempt` refusals now read
  `dispatched` from the attempt's intent store under that lock.
  A first run now does everything up to its intent under that lock: it reads
  the intent store first (another request's intent is `substituted_request`
  with `dispatched:true` and nothing is admitted), then the lineage and
  repair-port gates, the fence, the agent revision session, the binding and
  the intent. Every refusal decided before this request's intent
  (`lineage_blocked`, `lineage_exhausted`, `repair_ports_unavailable`,
  `stale_fence`, `crossed_attempt`, `revision_not_admitted`,
  `profile_revision_superseded`) settles `dispatched` from the intent store
  under the lock and, when it is `false`, first writes the attempt-fence's
  closure marker, so no request for that attempt-fence can create an intent
  afterwards; such a refusal therefore writes the state root, the lock and
  attempt directories and the marker (and, for `revision_not_admitted`, the
  fence record it follows) instead of leaving the state root untouched. A
  request of other bytes for a closed attempt-fence is refused as the new
  `attempt_closed` (`dispatched:false`), and `inspect` reports a closed
  attempt as `instance: null`. Closure markers now name their work
  (`closed-before-execution:<request identity>:<blake3(work_id)>`; markers
  without it stay readable), and another work's marker, intent or pending
  admission at the same attempt id and fence is `crossed_attempt`, with
  `dispatched:true` exactly when the attempt directory holds an execution
  intent of any request: attempt ids
  must be unique across the work items of one state root. Before it pins, an
  admission with a revision store writes `admission-pending.json` naming its
  work, request and the subject it is about to pin. A run, recovery or
  pre-intent refusal of the same work, including a superseded request's,
  that later finds it without the record's own intent closes exactly that pin
  under the work's lock before anything else, and only then removes the
  record, so a process lost between pin and intent no longer leaves a live
  pin. A request of another work leaves the record untouched and is
  `crossed_attempt`, creating no intent, marker or pin.

- Bounded local processes whose cancellation flag is already set return the
  existing `Cancelled` outcome without spawning. The pinned bounded-exec runner
  checked the flag only after spawn, so a pre-cancelled command could briefly run.
- Live file reads and image loads participate in unresolved-effect conflict
  scheduling; read-only authority is not treated as independence evidence.
- Role-protocol fanout preflights all adapter selections. Ambiguous HTTP responses
  and partial delivery retain unknown outcomes until exact receiver evidence.
- The standalone TUI rejects redirected stdin/stdout before terminal setup.
- Hosted check/release jobs omit dev/test debug symbols and release generated
  local Cargo build trees before isolated Nix checks, preserving the complete
  test rail and Octet JSON evidence rather than exhausting runner disk space.
- Refreshed the explicitly operator-admitted dependency/manifest lint cohort
  without growing complete per-owner warning budgets or weakening the maintained
  identity-only migration guard.
- Native render fault qualification terminates its owned local process sessions,
  including the bounded runner's nested process groups, and verifies SIGKILL
  rather than leaving a child observer holding the journal writer lock.
- Render qualification selects the packaged runtime instead of an oversized debug
  executable; the existing 256 MiB executor identity ceiling remains unchanged.
- Live TUI progress uses a checked whole-run bounded executor port and a bounded,
  backpressured UI mailbox; incomplete observations remain explicit without
  retrying execution or fabricating timing metadata.
- Implementation-attempt recovery verifies retained raw OMP events and their
  normalization before reporting a published result.
- Interrupted candidate workers retain available bounded process-event bytes as
  unknown-outcome evidence rather than discarding them with a post-start error.

## [0.1.0] - 2026-11-12

Initial public release candidate. The runtime, CLI, TUI, durable store, scheduler,
approvals, agent adapters, workflow contract, and local workflow exchange are
present but remain pre-1.0.

### Added

- Public release documentation: security policy, contribution guide, server
  deployment guide, release process, documentation index, and launch material.
- A tagged release workflow that runs the maintained checks, builds the Nix
  package, verifies `lattice --version`, and publishes a binary archive with
  checksums.
- `lattice --version` and public Cargo package metadata.

### Changed

- The optional Tile compose integration now lives in the separate `compose/`
  workspace. The default runtime, public checks, and release binary have no Tile
  dependency and no authenticated source requirement.
- Public build inputs use anonymous HTTPS. The transitive SSH-only sources for
  Artifact and Trellis are patched to their exact public revisions, and the Cairn
  and Octet flake inputs use HTTPS overrides.
- The release package no longer embeds the Cairn lifecycle toolchain; only the
  test rail sets `LATTICE_CAIRN_BIN`.

### Known limitations

- The workflow exchange service is a local Unix-socket service. It has no TLS,
  remote authentication, network listener, or multi-tenant isolation.
- Workflows run with host authority. Lattice is not a sandbox for untrusted
  workflows.
- The compose workspace requires authorized Tile source access and is not part of
  the public build closure.
- Three TUI authoring PTY regressions remain tracked by
  `complete-tui-nickel-authoring-parity` and are ignored by the release gate.
  The server release does not claim TUI authoring-parity completion; see
  [the release audit](.cairn/changes/prepare-public-release/evidence/release-audit.md).
- No hosted service, support SLA, or production-readiness claim is included in
  this release.
