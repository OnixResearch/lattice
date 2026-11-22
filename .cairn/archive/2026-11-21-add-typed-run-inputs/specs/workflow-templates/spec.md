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
r[lattice.workflow_templates.run_inputs.retained] Saving a bound bundle MUST retain, keyed by the saved workflow name, the bundle id, the template identity and the saved workflow revision. A run with inputs MUST load only the bundle directory the binding names. Inputs MUST be refused with a distinct missing-binding refusal when no binding is retained, and MUST be refused when the template declares no parameters, when the saved workflow revision no longer equals the retained revision, when that bundle fails to load (surfacing its value-free diagnostic) or no longer has the retained template identity, or when the workflow has an approval gate. `--input-file` MUST be opened without blocking and refused unless it is a regular file of at most 64 KiB, and `lattice run` MUST read its inputs before opening the store.

#### Scenario: Only the exact retained template accepts inputs
r[lattice.workflow_templates.run_inputs.retained.scenario.exact]
- GIVEN a saved workflow whose template binding was retained when it was saved
- WHEN inputs are supplied for it
- THEN only the exact retained parameterized template MUST accept them
- AND a template without parameters, a drifted saved workflow or a changed template MUST be refused before any run id is reserved

### Requirement: Run inputs re-verify at the start boundary
r[lattice.workflow_templates.run_inputs.start] A run with inputs is a start path. Before its run id is reserved or its input record is written, the bound bundle MUST pass the same `reverify` as save and start, and the recorded input identity MUST name exactly that bundle id, template, bound identity, workflow revision and value identity. Any difference MUST refuse the run with `bound_content_drift` and MUST leave no run id, run record or input record.

#### Scenario: Drifted bound content never reserves a run
r[lattice.workflow_templates.run_inputs.start.scenario.drift]
- GIVEN an admitted bound bundle whose workflow content was tampered with, or which was swapped for the same template bound to other values
- WHEN the run with inputs starts
- THEN it MUST be refused with `bound_content_drift`
- AND no run id MUST be reserved and no run or input record MUST be written

### Requirement: Recorded input identity
r[lattice.workflow_templates.run_inputs.record] A run with inputs MUST reserve its run id and record its input identity in one store transaction after every input check passed. The identity MUST hold the bundle id, template identity, bound identity, bound workflow revision, the `params::Resolved` value identity, and each declared name with its kind and whether it was supplied. It MUST NOT hold a value.

#### Scenario: A bound run records its input identity without values
r[lattice.workflow_templates.run_inputs.record.scenario.recorded]
- GIVEN a run with admitted inputs
- WHEN it starts
- THEN its input record MUST hold the bundle, template, bound and value identities, the bound workflow revision, and each declared name with its kind and supplied flag
- AND it MUST NOT hold a supplied value

### Requirement: Content-omitting input inspection
r[lattice.workflow_templates.run_inputs.inspection] `runs show` MUST show a recorded input identity as declared names, kinds, supplied flags and BLAKE3 identities only, without a value, workflow name or bundle name, with the report's own identity prefix. An input record that does not belong to the run MUST make the report invalid with `inputs_identity_mismatch`. The identities are unsalted, so the documentation MUST state that values are omitted, not hidden.

#### Scenario: Recorded inputs are shown without values
r[lattice.workflow_templates.run_inputs.inspection.scenario.omitted]
- GIVEN a recorded run with inputs
- WHEN `runs show --output json` reports it
- THEN its `inputs` object MUST show only declared names, kinds, supplied flags and identities, without a value, workflow name or bundle name

### Requirement: Authority inspection of bound inputs
r[lattice.workflow_templates.run_inputs.authority] `authority inspect` with the same inputs as a run MUST admit them the same way without writing state and MUST report the grants of the bound workflow revision with the content-omitting input identity, rendering the bound workflow revision exactly as the report's top-level `workflow_revision`, so a profile written from it authorizes exactly that bound revision.

#### Scenario: Authority inspection reports the bound revision that runs
r[lattice.workflow_templates.run_inputs.authority.scenario.bound]
- GIVEN `authority inspect` with the same inputs as a run
- WHEN an operator profile is written from its output
- THEN that profile MUST admit the run with those inputs
- AND a run with different values MUST be denied

### Requirement: Secret-like inputs are refused
r[lattice.workflow_templates.run_inputs.secrets] Run inputs MUST NOT carry secrets; credential references remain the only secret path. For `relative_path` parameters only, a name or binding key suffix with a secret word or a qualified `key` as an `_`-separated word, a supplied value component (or its stem before `.`, for formats without `.`) in a known provider token format (its prefix and at least its minimum suffix from its own alphabet, with the format's required digits or classes), and a supplied value with a `.`-separated piece that, without `-` and `_`, has at least 20 letters and digits of mixed classes whose lowercase runs average under 3 bytes MUST be refused naming only the parameter. `bool`, `integer` and `enum` parameters MUST NOT be screened. Ordinary refs, repository paths and CamelCase identifier file names whose words are three or more letters SHOULD be accepted; names built from short pieces joined by `-` or `_`, or from one- and two-letter words with digits, MAY be refused, and a token joined behind long lowercase words MAY be accepted. The rule is a pattern check, not a proof.

#### Scenario: Token-shaped values are refused and ordinary refs admit
r[lattice.workflow_templates.run_inputs.secrets.scenario.screened]
- GIVEN a `relative_path` parameter
- WHEN a supplied value matches a listed provider token format or the token-run rule
- THEN the input MUST be refused naming only the parameter
- AND ordinary branch refs and repository paths whose words are three or more letters MUST be admitted
