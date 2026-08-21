# Explicit Workflow Dependencies Delta

## ADDED Requirements

### Requirement: Dependency behavior has a recorded baseline

r[lattice.workflow_validation.explicit_dependencies.baseline] The change MUST record current expression results, runner dependencies, plan identities, cache behavior, replay behavior, and portable workflow bytes.

#### Scenario: Expression fixture has an incidental order
r[lattice.workflow_validation.explicit_dependencies.baseline.incidental-order]
- GIVEN a valid fixture reads another node output without a normal graph edge
- WHEN the baseline runs through serial execution
- THEN evidence MUST record the output, execution order, and admitted runner dependencies

### Requirement: Expression-bearing fields have an exhaustive inventory

r[lattice.workflow_validation.explicit_dependencies.inventory] Lattice MUST maintain one checked inventory of every workflow field that permits expression syntax. An unclassified expression-bearing field MUST fail validation coverage.

#### Scenario: New field lacks extraction coverage
r[lattice.workflow_validation.explicit_dependencies.inventory.unclassified-field]
- GIVEN a workflow field accepts expression syntax but has no inventory entry
- WHEN the completeness guard runs
- THEN the guard MUST fail before release

### Requirement: Expression references are extracted as typed facts

r[lattice.workflow_validation.explicit_dependencies.extraction] The pure workflow core MUST extract consumer, field path, producer, and producer output-port facts from every supported expression reference.

#### Scenario: Supported reference is extracted
r[lattice.workflow_validation.explicit_dependencies.extraction.supported-reference]
- GIVEN a supported field contains a valid node and port reference
- WHEN pure extraction runs
- THEN it MUST return one typed reference fact with the exact consumer and field path

### Requirement: Expression dependencies fail closed

r[lattice.workflow_validation.explicit_dependencies.admission] Lattice MUST reject missing nodes, missing ports, self-references, unsupported references, and cycles created by the combined dependency set.

#### Scenario: Expression creates a cycle
r[lattice.workflow_validation.explicit_dependencies.admission.cycle]
- GIVEN an expression reference adds a reverse dependency that closes a cycle
- WHEN workflow validation runs
- THEN Lattice MUST report a fatal diagnostic before runtime selection or effects

#### Scenario: Referenced port is missing
r[lattice.workflow_validation.explicit_dependencies.admission.missing-port]
- GIVEN an expression names an unknown output port
- WHEN workflow validation runs
- THEN Lattice MUST reject the workflow with the consumer and field path

### Requirement: Choregraph receives the complete dependency projection

r[lattice.workflow_validation.explicit_dependencies.projection] Before Choregraph admission, Lattice MUST merge normal data-flow and expression-reference dependencies into one canonical set. Machine and runner-plan identities MUST bind that set.

#### Scenario: Expression edge has no normal graph edge
r[lattice.workflow_validation.explicit_dependencies.projection.derived-edge]
- GIVEN a valid expression references an independent producer output
- WHEN Lattice builds the Choregraph input
- THEN the canonical dependency set MUST include the producer-to-consumer edge

#### Scenario: Stored plan has another dependency set
r[lattice.workflow_validation.explicit_dependencies.projection.stale-plan]
- GIVEN a persisted runner plan omits or changes one admitted expression dependency
- WHEN dispatch or recovery validates the plan
- THEN Lattice MUST reject the stale plan

### Requirement: Expression context is confined

r[lattice.workflow_validation.explicit_dependencies.context] Runtime expression context MUST contain only exact admitted references and declared workflow inputs for the consumer. It MUST exclude unrelated prior outputs.

#### Scenario: Unrelated output completed earlier
r[lattice.workflow_validation.explicit_dependencies.context.unrelated-output]
- GIVEN an unrelated node completed before the consumer
- WHEN Lattice builds the consumer expression context
- THEN the unrelated output MUST be absent

#### Scenario: Required value is unavailable
r[lattice.workflow_validation.explicit_dependencies.context.missing-value]
- GIVEN an admitted reference lacks its required runtime value
- WHEN expression preparation runs
- THEN Lattice MUST fail before credential resolution, attempt creation, or effects

### Requirement: Valid workflow compatibility is preserved

r[lattice.workflow_validation.explicit_dependencies.compatibility] Valid supported workflow source MUST remain readable without source mutation. Any derived machine-identity change MUST have an explicit recorded compatibility disposition.

#### Scenario: Existing valid expression workflow is reopened
r[lattice.workflow_validation.explicit_dependencies.compatibility.reopen]
- GIVEN a supported workflow source contains valid expression references
- WHEN Lattice validates and lowers it
- THEN source bytes MUST remain unchanged and execution dependencies MUST become explicit

### Requirement: Hidden dependencies have negative evidence

r[lattice.workflow_validation.explicit_dependencies.negative] The focused rail MUST cover malformed, missing, stale, self, cyclic, unsupported, leaked-context, and pre-effect failure cases.

#### Scenario: Serial order would hide a missing edge
r[lattice.workflow_validation.explicit_dependencies.negative.serial-order]
- GIVEN current topological order happens to run the referenced producer first
- WHEN dependency validation finds no valid producer or port fact
- THEN Lattice MUST reject the workflow instead of relying on serial order

### Requirement: Dependency evidence states its boundary

r[lattice.workflow_validation.explicit_dependencies.evidence] Evidence MUST bind the extractor, inventory, dependency projection, fixtures, identities, and non-claims.

#### Scenario: Data dependency is presented as effect safety
r[lattice.workflow_validation.explicit_dependencies.evidence.non-claim]
- GIVEN the dependency projection passes validation
- WHEN evidence is reported
- THEN it MUST NOT claim that external effects commute or succeed

### Requirement: Dependency validation is maintained

r[lattice.workflow_validation.explicit_dependencies.validation] Repository validation MUST run focused positive and negative fixtures, Choregraph handoff tests, compatibility checks, and lifecycle gates.

#### Scenario: Context confinement test is skipped
r[lattice.workflow_validation.explicit_dependencies.validation.context-rail]
- GIVEN a candidate lacks current context-leak evidence
- WHEN lifecycle validation runs
- THEN the candidate MUST fail closeout
