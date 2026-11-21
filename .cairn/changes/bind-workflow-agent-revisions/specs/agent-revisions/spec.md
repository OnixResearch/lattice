# Agent Revisions Specification

## ADDED Requirements

### Requirement: An immutable workflow-agent bundle
r[lattice.agent_revisions.workflow] A `lattice.workflow-agent-bundle.v1` bundle MUST be a lifecycle bundle whose `workflow` binding carries a compiled workflow-intent plan's schema, compiler revision, compiler artifact identity, `b3:` plan identity, `blake3:` graph identity, runtime profile identity, Cairn cohort identities and per-obligation identities sorted by identifier, and the worker's definition, definition name, bare profile revision and `lattice-agent-bundle-v1` identity. Its definition MUST be the `lattice-workflow-agent-definition-v1` identity over the pairing name and the worker definition, and its revision the bare-hex `lattice-workflow-agent-revision-v1` digest over the canonical binding. A bundle whose schema, name, definition, revision or worker fields disagree with its binding MUST be refused as `malformed`. A plain agent bundle MUST omit the binding and keep its bytes and identity.

#### Scenario: Any plan or worker change is a new revision of one definition
r[lattice.agent_revisions.workflow.scenario.identity]
- GIVEN a plan and a worker bundle, the same worker with another plan, and the same plan with another worker revision
- WHEN each pairing is composed
- THEN composition MUST be deterministic, the three revisions MUST be distinct, the definition MUST be shared and differ from the worker definition, the worker bundle MUST be rebuilt exactly from the binding, and the fixture identities MUST equal the recorded golden copy

#### Scenario: The existing lifecycle manages workflow-agent revisions
r[lattice.agent_revisions.workflow.scenario.lifecycle]
- GIVEN session A on R1, a changed plan R2 whose preparation fails, and a changed worker R3 that is prepared, qualified and promoted
- WHEN B and C are admitted, a late R2 result arrives and R1 is rolled back to
- THEN R1 MUST stay last usable and selected after R2 fails, B MUST pin R1, C MUST pin R3 while A stays on R1, the late R2 result MUST be recorded as superseded without changing the selection, last usable revision or rollout generation, and rollback MUST change only the selection for new sessions

#### Scenario: A positive prior evaluation does not approve another intent
r[lattice.agent_revisions.workflow.scenario.evaluation]
- GIVEN a promoted R1 and a prepared R2 with the same worker and another plan
- WHEN R2 is promoted with R1's reference, with an attachment produced for R1, or with an attachment produced for the worker revision alone
- THEN promotion MUST be refused as `qualification_mismatch`, `not_qualified` / `subject_mismatch` and `not_qualified` / `subject_mismatch`, and MUST succeed only with an attachment for R2

### Requirement: A proposed adaptation is a new identity with an unknown applicability
r[lattice.agent_revisions.workflow.adaptation] `workflow::adaptation::propose` MUST compare an active and a proposed workflow-agent bundle of one definition and return `lattice.workflow-agent-proposal.v1` with both revisions, the sorted changes and an applicability slot whose decision is `unknown`, owned by `cairn#14` and `kiln#83`. The applicability type MUST admit no other decision. An identical bundle MUST be `unchanged`, another definition `definition_mismatch`, and a plain agent bundle `malformed`. The proposal MUST NOT change any lifecycle.

#### Scenario: An adaptation never mutates an active pin
r[lattice.agent_revisions.workflow.adaptation.scenario.pinned]
- GIVEN session A pinned to the selected R1, and proposals that change the plan, a prompt, a tool, a requirement or a policy
- WHEN each is proposed and desired
- THEN each MUST have a new revision, name its change and carry the `unknown` applicability, and after the desire the selection and every pin MUST be unchanged and A MUST still act under R1

### Requirement: The Kiln projection keeps workflow and worker revisions apart
r[lattice.agent_revisions.workflow.projection] `workflow::publication::slots` MUST project `lattice.workflow-agent-revision-slots.v1`: the definition, the pairing name, and the desired, last usable, selected and pinned revisions, each with its compiled identities (compiler revision and artifact, plan, graph, selection, obligations, policy) and its worker (definition, name, revision, bundle identity) as separate fields, with each pin's status and whether it is blocked. It MUST NOT project guidance, context, source or per-obligation identities. A lifecycle of plain agent bundles MUST have no slots, and a referenced revision that is missing or not a valid workflow-agent bundle MUST be refused.

### Requirement: Inert composition and inspection
r[lattice.agent_revisions.workflow.inspect] `agent-revision resolve-workflow` MUST compose a workflow-agent bundle from an exported implementation-attempt request and a typed workflow intent whose compiler artifact is the running executable, without admitting, preparing or writing anything. `desire-workflow` MUST record it as desired and MUST open no store before composition succeeds. An intent naming another compiler MUST be refused. `agent-revision inspect` MUST add `workflow_slots` for a workflow-agent lifecycle and omit it otherwise. The bundle MUST carry no path, endpoint, credential key or prompt.

### Requirement: The attempt seam binds the exact worker revision
r[lattice.agent_revisions.workflow.attempt] `workflow::check_attempt` MUST accept an attempt for a workflow-agent bundle only when the attempt's profile revision is the worker revision the bundle binds, refusing another revision as `revision_not_selected` and a malformed revision or a plain agent bundle as `malformed`.

### Requirement: A draft-schema OCI package comparison
r[lattice.agent_revisions.package_comparison] The comparison of the agent-package draft at `NVIDIA/agent-package-spec` `697872570744c8cfc1cf6b1d72e04db66e1a2c83` MUST be an inert fixture comparison labelled `draft_schema_fixture_comparison_not_materialization`: it MUST NOT read an archive, contact a registry, install or activate anything. Provider-native `sha256:` digests MUST stay separate from the BLAKE3 identities, and registry, repository, tag and version annotation MUST NOT enter any identity. `docs/agent-package-comparison.md` MUST record the decision.

#### Scenario: Reviewed fixtures map onto identities
r[lattice.agent_revisions.package_comparison.scenario.mapping]
- GIVEN a reviewed skill and a plugin with bundled skills whose supplied facts agree
- WHEN they are compared and mapped
- THEN each MUST report no finding and map its source revision, provider-native digests, inventory, materialized, dependency-closure and evidence-subject identities onto a valid lifecycle bundle, the bundled skill MUST carry the reviewed skill's bytes, and the plugin's evidence subject MUST differ from the skill's

#### Scenario: Every mismatch is reported at the comparison level
r[lattice.agent_revisions.package_comparison.scenario.mismatch]
- GIVEN 31 negative cases for missing, extra, changed and duplicate files, case-folding collisions, unsafe paths, symlinks, hard links and special files, decompression bounds, and manifest and inventory mismatches
- WHEN each is compared
- THEN each MUST report exactly its expected findings

#### Scenario: Payload change, relocation, revocation and rollback use the existing lifecycle
r[lattice.agent_revisions.package_comparison.scenario.lifecycle]
- GIVEN the reviewed skill promoted and pinned, and the skill republished with a changed file under the same name and version
- WHEN the republished revision is promoted with the old approval, the reviewed package is relocated, the republished digest is revoked and the reviewed revision is rolled back to
- THEN the old approval MUST be refused, relocation MUST keep the revision, the revoked digest MUST block its session and refuse admission, desire and rollback, and rollback MUST serve new sessions while the revoked pin stays blocked
