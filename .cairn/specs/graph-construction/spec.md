# Graph Construction Specification

## Purpose

Defines the `graph-construction` capability.

## Requirements

### Requirement: Selected captured construction profile

r[lattice.graph_construction.profile] Lattice MUST construct graph-construction candidates only through the selected captured-module evaluator profile (nickel-eval `2d1fb2b59f373967cc69e13ec289759d9cd4df46`) and the Choregraph construction/v3 projection (`choregraph.graph-construction.v1`, Choregraph `c3577352ee1e39391f5204ee234d991c7dbeb151`), bound to the host-selected local shell registry. Successful evaluation MUST produce only a candidate; Lattice validation, Choregraph admission, the selected runtime binding and a current exact effect grant MUST still decide execution. Nickel metadata MUST NOT relabel an effectful operation or select a registry implementation.

#### Scenario: Registered binary imports and runs the example

r[lattice.graph_construction.profile.cli]
- GIVEN the captured example `examples/construction/local-shell-construction.ncl` and the registered `lattice` binary
- WHEN it is imported with `import --construction`, inspected, navigated and run with a grant derived from `authority inspect`
- THEN the admitted workflow MUST retain a captured-modules association and run every constructed node through the production run route
- AND running it without that grant MUST fail.

### Requirement: Independent generated instances

r[lattice.graph_construction.instances] Instantiating one template twice and generating a worker collection MUST give each instance an independent node and port identity. Nested exposed boundaries and cross-instance connections MUST resolve to the intended instance ports, and reusing one output handle MUST NOT add an operation instance.

#### Scenario: Shared handle fans out to four instances

r[lattice.graph_construction.instances.remap]
- GIVEN the example with two template instances, two generated workers, one shared source handle and a join
- WHEN it is compiled and admitted
- THEN it MUST contain exactly six operations with one data edge from the shared handle to each instance input
- AND each node's configuration, effect slot binding and implementation reference MUST stay attached to its own admitted instance after remapping.

#### Scenario: Fanout and discard follow the selected target

r[lattice.graph_construction.instances.sharing]
- GIVEN the admitted example under the selected registry
- WHEN its string values become non-copyable or its exit codes become non-droppable
- THEN native elaboration and projection admission MUST both reject the candidate.

### Requirement: Independent reference agreement

r[lattice.graph_construction.reference] A generated graph MUST agree with an independently authored materialized reference on ordered boundary values and required dependencies. The comparison MUST NOT require incidental event order, numeric identifiers or labels.

#### Scenario: Generated and hand-authored graphs agree

r[lattice.graph_construction.reference.values]
- GIVEN the generated example and a hand-authored workflow with the same six commands and join dependencies
- WHEN both run with their exact local grants
- THEN their per-role stdout values MUST be equal
- AND their join dependencies, compared by admitted instance role, MUST be equal.

### Requirement: Exact local authority

r[lattice.graph_construction.authority] A constructed graph MUST run its local shell effects only with an exact run-scoped grant for that admitted revision. Absent or foreign authority MUST fail before the shell adapter executes or any credential is delivered; credential references MUST NOT be attached to an admitted constructed graph.

#### Scenario: Same graph with and without authority

r[lattice.graph_construction.authority.same_graph]
- GIVEN one admitted constructed graph whose first command creates a marker
- WHEN it runs with no grant, with a grant for another graph, and with its exact grant
- THEN only the exact grant MUST create the marker and complete every node.

### Requirement: Zero-data control order

r[lattice.graph_construction.control_order] Unit/zero-data control dependencies MUST survive portable interchange, projection, runner planning and readiness, so a second effect cannot start before its prerequisite completes even without a payload connection.

#### Scenario: Barrier-controlled second effect

r[lattice.graph_construction.control_order.barrier]
- GIVEN two shell effects ordered only by a control dependency, where the first blocks on an external gate
- WHEN the first effect has started and the gate is still closed
- THEN the second effect MUST NOT have started.

### Requirement: Pre-dispatch rejection

r[lattice.graph_construction.rejection] Unknown runtime bindings, wrong types, invalid expression dependencies, identity collisions, wrong-scope handles, cycles, source generation-cap overflow and native expansion overflow MUST reject before any dispatch.

#### Scenario: Expansion beyond selected bounds

r[lattice.graph_construction.rejection.overflow]
- GIVEN the example candidate
- WHEN its generation cap is lowered below its items, or native node or instance limits are below its expansion
- THEN compilation MUST fail
- AND the unchanged candidate MUST compile under limits that fit it.

### Requirement: Binding drift invalidation

r[lattice.graph_construction.drift] Changing an imported member, a parameter, the construction contract or library, the evaluator, the selector policy, a same-named registry implementation, the elaborator or the final graph MUST invalidate the affected compilation binding before runtime admission or authority inspection. Source hashes alone MUST NOT be treated as a cache policy or semantic proof.

#### Scenario: Foreign registry implementation

r[lattice.graph_construction.drift.registry]
- GIVEN a self-consistent compilation for a registry that differs only in implementation identity
- WHEN its binding replaces the retained one
- THEN runtime admission and effect-authority inspection MUST reject it.

#### Scenario: Same output, distinct provenance

r[lattice.graph_construction.drift.provenance]
- GIVEN two captured sources that differ only in a comment
- WHEN both are compiled
- THEN their evaluated output MAY be equal
- AND their source revisions and observed closures MUST remain distinct.

### Requirement: Existing route compatibility

r[lattice.graph_construction.compatibility] Existing materialized workflow import/export, legacy v1/v2 execution and named child-workflow semantics MUST remain compatible; compile-time instantiation MUST remain distinct from runtime child invocation.

#### Scenario: Legacy and named-child routes

r[lattice.graph_construction.compatibility.legacy]
- GIVEN a legacy workflow without construction facts and a saved named child workflow
- WHEN it is exported or run, and the parent invokes the child
- THEN the export MUST keep the committed contract
- AND the child MUST route its declared output to the parent port.

### Requirement: Existing runtime controls

r[lattice.graph_construction.runtime_controls] Constructed runs MUST use the existing snapshot cancellation, shell-result validation, orchestration budget and effect-check routes; no construction-specific runtime path may bypass them.

#### Scenario: Rejected results and cancellation stop dependents

r[lattice.graph_construction.runtime_controls.result]
- GIVEN a constructed two-effect sequence run through the production snapshot route
- WHEN the first effect exits non-zero, or cancellation is requested while it runs
- THEN the run MUST fail
- AND the dependent effect MUST NOT run, while a successful first effect releases it.

#### Scenario: Budget gates constructed rounds

r[lattice.graph_construction.runtime_controls.budget]
- GIVEN an orchestration definition over a constructed workflow
- WHEN its enforced round budget is one or zero
- THEN one funded round MUST run and consume its budget
- AND an unfunded definition MUST dispatch no run.

### Requirement: Complete captured module closure

r[lattice.graph_construction.closure] An admitted construction association MUST retain the evaluator's observed module closure, including the exact library and contract modules and every selected captured member, and MUST re-verify it on runtime admission. Reopening the store MUST preserve source and parameter identities.

#### Scenario: Reopen preserves closure and projection

r[lattice.graph_construction.closure.reopen]
- GIVEN an admitted construction program
- WHEN the store is reopened
- THEN its source revision and native projection MUST be unchanged
- AND its observed closure MUST include the library and contract modules.

### Requirement: Construction origins and navigation

r[lattice.graph_construction.origins] Selecting a constructed node or port MUST show its template and instance from the producer's origin map and a logical source location in the exact captured revision only where the producer supplied a valid span. Otherwise attribution MUST report unavailable rather than guessing.

#### Scenario: Navigate to template, instance and location

r[lattice.graph_construction.origins.navigation]
- GIVEN an admitted constructed graph
- WHEN its `left` worker node or port is selected in the TUI or the `authoring origin` command
- THEN the result MUST name template `worker`, an instance path containing `left` and a span in the captured module
- AND an unrelated port or a retained snapshot without origins MUST be refused.

### Requirement: Template and instance edits

r[lattice.graph_construction.editing] Editing a template MUST affect all of its instances after recompilation; changing one instance parameter MUST affect only that instance as the source defines. Differences MUST be inspectable in a preview before admission, and admission MUST create a new revision.

#### Scenario: Template versus instance parameter

r[lattice.graph_construction.editing.scope]
- GIVEN the parameterized example
- WHEN its template body changes, then one instance parameter changes
- THEN every instance MUST change after the first readmission
- AND only that instance MUST change after the second.

### Requirement: In-flight snapshot

r[lattice.graph_construction.snapshot] An in-flight run MUST continue against the exact admitted snapshot it loaded; admitting a newer source revision MUST NOT mutate it, and the older exact grant MUST NOT authorize the newer revision.

#### Scenario: Readmission while a run is blocked

r[lattice.graph_construction.snapshot.readmit]
- GIVEN a run of an admitted revision blocked on its first effect
- WHEN a newer source revision is admitted and the block is released
- THEN the run MUST complete the old commands only
- AND the old grant MUST NOT run the newer revision.

### Requirement: Inert authoring

r[lattice.graph_construction.inert_authoring] Preview, compilation, admission, navigation, export and authority inspection of a constructed graph MUST NOT start workloads or deliver credentials.

#### Scenario: Authoring an effectful graph

r[lattice.graph_construction.inert_authoring.marker]
- GIVEN a constructed graph whose command would create a marker
- WHEN it is previewed, admitted, navigated, exported and inspected
- THEN no marker or run record MUST exist.

### Requirement: Retained host qualification

r[lattice.graph_construction.qualification] The supported qualification route MUST run on the host from immutable store artifacts, refuse an existing output directory, and retain source, expanded graph, exact commands, observed results and negative controls. It MUST NOT be presented as a sandboxed flake check, and it MUST NOT weaken the immutable-executable policy.

#### Scenario: Host-run app output

r[lattice.graph_construction.qualification.output]
- GIVEN the `construction-runtime-qualification` flake app and a new output directory
- WHEN it runs on a Linux host with a root-owned Nix store
- THEN the directory MUST contain the source, expanded graph, `commands.tsv`, logs and `summary.json`
- AND every negative control MUST be recorded with its observed failure.
