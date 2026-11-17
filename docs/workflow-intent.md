# Inert workflow-intent compilation

`workflow-intent` compiles a finite **host-normalized Lattice input**, not Cairn prose, into a canonical Choregraph dependency plan. It does not run nodes, open an application store/configuration, resolve credentials, grant effects, accept candidates, or discharge Cairn obligations.

The input schema is `lattice.workflow-intent.v1`; output is `lattice.workflow-intent-plan.v1`. The pure entry point is `workflow::intent::compile(CompileRequest)`. Cairn owns obligation/acceptance meaning; Kiln owns selected work, claims, budgets and execution admission.

## Supported fragment

An input binds producer, selection, obligations, profile, context, source, policy and dependency identities. Each named obligation retains its obligation, guidance, remaining-condition, action and precondition identities plus required observation contracts and prerequisites. Supported operation classes are `cairn_validate`, `named_check`, `review_wait` and `completion_only`. These names are contract data, not commands.

Admission requires complete declared coverage, no unsupported fields, bounded identifiers and exact expected host cohort. Restricted/incomplete input, missing checks, unknown classes/capabilities, ambiguous or unavailable alternatives, invalid dependencies, cycles and exhausted bounds return nonexecuting diagnostics. `complete` is a required input assertion, **not proof of an authenticated complete Cairn snapshot**.

The finite alternative profile selects implementation identities for existing nodes. It cannot remove required nodes, dependencies or evidence. Structural changes and investigation remain proposals; `compare` reports added/removed/changed obligations and requires owner readmission. It never mutates a frozen graph or grants a successor.

The pure core admits at most 128 obligations, 512 dependencies, 16 alternatives and 16 observation contracts/capabilities per obligation. The CLI also bounds input to 1 MiB. Unordered obligations, dependencies, evidence contracts, alternatives and sets normalize before identity calculation. Meaningful source/policy/profile/implementation or compiler changes invalidate the applicable plan binding.

## Local conformance input

Use the admitted release binary: executable identity measurement retains the existing 256 MiB bound. Large debug executables are not exempted.

```sh
lattice workflow-intent compiler-identity
nickel export --format json examples/workflow-intent/verify.ncl > request.json
# Replace request.json's facts.compiler_artifact_identity with the measured identity.
lattice workflow-intent compile request.json
lattice workflow-intent compare request.json proposed.json
```

The checked-in Nickel example is explicitly **synthetic conformance data**, not a producer receipt or live compatibility observation. The CLI checks its actual executable identity against the frozen compiler-artifact field; a different executable is rejected instead of silently rebinding the input. The library takes explicit host facts and remains pure.

The [retained CLI observation](../.cairn/changes/compile-workflow-intent/evidence/cli-observation.json)
exercises the actual named executable: canonical input permutation, owner-readmission
proposals, and restricted/incomplete/unsupported/stale-compiler refusal without
creating application configuration. These are synthetic consumer conformance
observations, not typed Cairn producer or live-worker qualification.

Output carries the selected Choregraph machine descriptor/identity, normalized source contracts and a distinct plan identity. Its disposition is `planning_only_requires_separate_execution_admission`; it has no executable effect declarations. Graph completion and native review/completion predicates remain different decisions.

## Remaining producer and execution boundary

The inspected Cairn generated-workflow descriptor contains prose. [Cairn #12](https://github.com/OnixResearch/cairn/issues/12) owns typed obligation predicates; [Cairn #14](https://github.com/OnixResearch/cairn/issues/14) owns their exact selected-work/context snapshot. Neither is substituted with a locally invented producer DTO.

[Lattice #26](https://github.com/OnixResearch/lattice/issues/26) remains open for that producer cohort, [Kiln #83](https://github.com/OnixResearch/kiln/issues/83), and actual #6/#7 attempt execution, current work/source/attempt/fence/grant binding, counterexample repair and interruption qualification. Presentation-only context cannot become a complete producer snapshot. A passing compile, process exit, task checkbox or generic CI result is not native acceptance.

Maintenance owner: Lattice workflow maintainers. Current consumer/adoption path: Kiln's typed planning boundary, followed by the existing attempt/execution/verification owners. `tests/test_workflow_intent.rs` exercises deterministic bindings, dependency/review/completion contracts, rejected input and proposal non-mutation. Those deterministic cases are not a live worker qualification.
