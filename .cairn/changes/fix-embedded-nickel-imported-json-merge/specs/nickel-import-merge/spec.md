# Embedded Nickel Imported JSON Merge

## ADDED Requirements

### Requirement: Imported closed values preserve recursive merge invariants
r[lattice.nickel_import_merge.values] The native embedded evaluator MUST evaluate valid imported JSON record merges with nested records and arrays without rewriting the input or intercepting panics, preserving JSON value semantics and existing recursive override behavior.

#### Scenario: Nested imported JSON participates in merge
r[lattice.nickel_import_merge.values.scenario.nested]
- GIVEN a JSON file containing nested records and arrays and a Nickel source importing it by relative path
- WHEN the native evaluator merges it with a record in either operand order, including nested overlap
- THEN evaluation MUST complete with the expected complete JSON value and agree with the pinned Nickel CLI on the selected fixture

#### Scenario: Recursive overrides survive imported merge
r[lattice.nickel_import_merge.values.scenario.recursive]
- GIVEN a recursive Nickel record merged with imported JSON and another overriding record
- WHEN the native evaluator exports it
- THEN dependent fields MUST observe the final merged values without extra-indirection dependency loss

### Requirement: Merge rejection remains bounded and source-exact
r[lattice.nickel_import_merge.boundary] The repair MUST preserve invalid-import and contract rejection with bounded content-omitting Lattice failure categories, one native evaluator path, and explicit immutable admission for patched evaluator source.

#### Scenario: Import or contract fails
r[lattice.nickel_import_merge.boundary.scenario.invalid]
- GIVEN a missing or malformed import or an imported value violating a contract
- WHEN native evaluation is requested
- THEN it MUST return the appropriate existing safe error category without disclosing supplied source, paths, or secret sentinels

#### Scenario: Patched source is not admitted
r[lattice.nickel_import_merge.boundary.scenario.source]
- GIVEN the corrected evaluator source lacks accepted immutable publication or aligned Cargo and Nix identity
- WHEN completion is evaluated
- THEN production runtime acceptance MUST remain blocked and no sibling, cache, runtime CLI, or input-rewriting fallback may substitute
