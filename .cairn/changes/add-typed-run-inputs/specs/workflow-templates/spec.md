# Workflow Templates Specification

## ADDED Requirements

### Requirement: Typed run inputs for saved template workflows
r[lattice.workflow_templates.run_inputs] `lattice run NAME` and `lattice authority inspect NAME` MUST accept repeatable `--input NAME=VALUE` and one `--input-file PATH` holding a JSON object. A name repeated within or across both sources MUST be refused. Inputs MUST be typed by the declarations of the exact template the saved workflow was retained from, checked by `params::resolve`, screened by the secret rule, and bound by `workflow_template::bind`, which repeats full bundle admission. Every input failure MUST happen before a run id is reserved, any run record is written, or any node runs, and its diagnostic MUST NOT echo a supplied value or an undeclared name.

#### Scenario: Each declared type binds from either source
r[lattice.workflow_templates.run_inputs.scenario.typed]
- GIVEN a saved workflow retained from a template declaring `bool`, `integer`, `enum` and `relative_path` parameters
- WHEN `--input` text values and `--input-file` JSON values are supplied
- THEN text MUST take the declared type (`true`/`false`, a canonical decimal integer, or text) and JSON values MUST keep their JSON type
- AND the bound workflow MUST hold each value in its declared environment target under a bound workflow revision, with the same effect grants as the saved workflow
- AND equal values from different sources MUST give the same input identity

#### Scenario: Wrong, unknown, missing and duplicate inputs are refused before any effect
r[lattice.workflow_templates.run_inputs.scenario.refused]
- GIVEN a parameterized saved workflow
- WHEN a value has the wrong type or is out of bounds, a name is undeclared, a required value is missing, a name repeats, an argument is not `NAME=VALUE`, or the file is not one JSON object
- THEN the command MUST fail with a safe diagnostic
- AND no node MUST run, no run id MUST be reserved, and no run or input record MUST be written

#### Scenario: Inputless runs are unchanged
r[lattice.workflow_templates.run_inputs.scenario.inputless]
- GIVEN a saved workflow run without `--input` or `--input-file`
- WHEN it runs, is inspected with `runs show`, or is inspected with `authority inspect`
- THEN its run identity, run record, summary bytes, report shape and authority output MUST equal those of the same workflow saved without a template, and no input record MUST exist

### Requirement: Retained template binding
r[lattice.workflow_templates.run_inputs.retained] Saving a bound bundle MUST retain, keyed by the saved workflow name, the bundle id, the template identity, the bound identity and the saved workflow revision. Inputs MUST be refused when no binding is retained, when the template declares no parameters, when the saved workflow revision no longer equals the retained revision, when the catalog no longer holds that exact template identity, or when the workflow has an approval gate.

### Requirement: Run inputs re-verify at the start boundary
r[lattice.workflow_templates.run_inputs.start] A run with inputs is a start path. Before its run id is reserved or its input record is written, the bound bundle MUST pass the same `reverify` as save and start, and the recorded input identity MUST name exactly that bundle id, template, bound identity, workflow revision and value identity. Any difference MUST refuse the run with `bound_content_drift` and MUST leave no run id, run record or input record.

### Requirement: Recorded input identity
r[lattice.workflow_templates.run_inputs.record] A run with inputs MUST reserve its run id and record its input identity in one store transaction after every input check passed. The identity MUST hold the bundle id, template identity, bound identity, bound workflow revision, the `params::Resolved` value identity, and each declared name with its kind and whether it was supplied. It MUST NOT hold a value.

### Requirement: Content-omitting input inspection
r[lattice.workflow_templates.run_inputs.inspection] `runs show` MUST show a recorded input identity as declared names, kinds, supplied flags and BLAKE3 identities only, without a value, workflow name or bundle name. An input record that does not belong to the run MUST make the report invalid with `inputs_identity_mismatch`.

### Requirement: Authority inspection of bound inputs
r[lattice.workflow_templates.run_inputs.authority] `authority inspect` with the same inputs as a run MUST admit them the same way without writing state and MUST report the grants of the bound workflow revision with the content-omitting input identity, so a profile written from it authorizes exactly that bound revision.

### Requirement: Secret-like inputs are refused
r[lattice.workflow_templates.run_inputs.secrets] Run inputs MUST NOT carry secrets; credential references remain the only secret path. A declared parameter whose name or binding key suffix has a secret word or a qualified `key` as an `_`-separated word, a declared binding whose target node binds the same key to a credential reference, and a supplied text value with a credential-prefixed or token-shaped path component MUST be refused naming only the parameter.
