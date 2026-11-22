# Design: Typed `--input` values on `lattice run`

## Context

Base: Lattice integration `99e65721`: published `0862c017` plus Lattice #6, #27 with its review fix, and #29 with its review fix.

- `lattice run NAME` loads the saved workflow, reserves a run id with `next_run_id`, runs, and writes run, detail and replay records. `authority inspect NAME` loads the saved workflow read-only and prints `inspect_workflow`.
- #27 added `params::resolve(declared, supplied)` and `workflow_template::bind(template, supplied)`. `save_draft` stores only the bound workflow. The saved workflow keeps no declarations, and the same template bound to other values conflicts on save.
- The run record (`RunRecord`) has no optional fields and is built at many sites. `runs show` reports a content-omitting `Report` with a hashed workflow identity instead of the name.

## Decisions

### Decision: Retain the template binding by identity, rebind through the catalog

**Choice:** `save_draft` writes a `RetainedBinding` keyed by the saved workflow name: bundle id, template identity, bound identity and saved workflow revision. A run with inputs loads the saved workflow and binding, requires the saved revision to equal the retained one, loads the catalog, and requires the bundle with the retained template identity. It then calls `bind(template, supplied)`.

**Rationale:** The acceptance requires binding through #27's `resolve` and `bind`, and `bind` takes a catalog template. Equal template identity means equal manifest and workflow bytes, so the declarations are exactly the reviewed ones. The revision check makes sure the saved workflow is still that template's output, not an operator edit. Copying declarations into the store would create a second, unreviewed source for them.

### Decision: Inputs are a full binding of the template, not an overlay

**Choice:** Omitted optional parameters take their declared defaults, and omitted required parameters fail, even when the saved workflow holds a value. Any `--input` or `--input-file` selects this path. An empty input file binds the defaults.

**Rationale:** `resolve` already defines total resolution, and the resolved identity then describes every value of the run. Overlaying would make a run depend on how its saved workflow was produced.

### Decision: Text inputs take the declared type, JSON keeps its type

**Choice:** A `--input` text for a `bool` must be exactly `true` or `false`, and for an `integer` a canonical decimal (`value.to_string() == text`). Enum and path parameters take the text. JSON values map `bool`, integer and string to the matching value, and anything else is a wrong type. Names are typed only by declarations, so an undeclared name fails as unknown without echo.

**Rationale:** Untyped guessing would turn a numeric enum token or path into an integer. Canonical integers keep one spelling per value.

### Decision: Duplicate detection keeps JSON keys

**Choice:** The input file is parsed by a map visitor that keeps every key in order. Entries from both sources are merged, and any repeated name fails.

**Rationale:** `serde_json` objects keep the last duplicate key silently.

### Decision: Secret rule

**Choice:** Every declared parameter is refused if its name or binding key suffix has a secret word or a qualified `key` as an `_`-separated word, or if a target node binds that key to a credential reference. A supplied text value is refused if a `/`-component starts with a known credential prefix followed by 8 or more bytes, or is a 32-byte-or-longer `[A-Za-z0-9_-]` component mixing uppercase, lowercase and digits.

**Rationale:** Types exclude free text, but a relative path can still hold a token. Name and target checks cover every declaration because `bind` writes every target. Defaults are reviewed manifest bytes, so values are checked only when supplied. Lowercase hex, such as a commit id, stays allowed.

### Decision: Record identity in a separate table, reserved with the run id

**Choice:** `Store::reserve_run_with_inputs` reserves the run id and inserts a `RunInputRecord` in `run_inputs_v1` in one transaction. It holds the bundle id, template, bound identity, bound workflow revision, resolved value identity, and each name with kind and supplied flag. It holds no value. Inputless runs keep `next_run_id` and never touch the table.

**Rationale:** Adding a field to `RunRecord` would change every construction site. A separate table, created on first write, keeps inputless records and store bytes unchanged. Writing the record with the id reservation means every run record of a bound run has its inputs. It also means a crash before the run record leaves an input record that `runs show` never reaches.

### Decision: `runs show` shape

**Choice:** An optional `inputs` object, skipped when absent: `template`, `bound`, `workflow_revision`, `values` as `b3:` identities and `params` as `{name, kind, supplied}`. A record whose run id or workflow name differs from the run adds `inputs_identity_mismatch` and omits it.

**Rationale:** The issue requires names with a value identity or per-value digests. One value identity is #27's `Resolved` identity, which a caller holding the values reproduces with `resolve`. Per-value digests would add a second hashing convention, and over small domains (`bool`, enum, bounded integer) they would not hide a value anyway. Workflow and bundle names are omitted as the report already omits the workflow name.

### Decision: Approval gates refuse inputs

**Choice:** A template with an approval node refuses inputs.

**Rationale:** Resume reloads the saved workflow and requires the suspended revision, so a bound run could not resume.

### Decision: A run with inputs is a start boundary

**Choice:** Before the run id is reserved, `reverify_admitted` calls #27's `reverify` on the bound bundle and requires the recorded input identity to name exactly its bundle id, template, bound identity, workflow revision and value identity. A difference refuses the run with `bound_content_drift`. `save_draft` keeps #27's flow, where `reverify` runs before any store write, and retains the binding only in the `CreateDraft` and `ExactAlreadyPresent` arms after that check. All bundle reads use the `ValidatedBundle` accessors.

**Rationale:** `Admitted` is a public value that a caller holds between admission and run, so the run must not trust it. Start and save already re-verify bound content; a run with inputs is the same kind of boundary.

### Decision: Authority inspection with inputs

**Choice:** `authority inspect` admits the same inputs from a read-only store and prints the bound workflow's inspection with a flattened `inputs` object. Without inputs, its output is unchanged.

**Rationale:** The standalone authority profile pins the workflow revision, and bound values change it. A profile written from this output authorizes exactly the inspected values.

## Tests

- `tests/test_run_inputs.rs`: every type from both sources and the bound revision and grants; wrong type, bounds, unknown, missing, duplicate and malformed inputs without echo; secret values, names, binding keys and credential targets; drift, removed template, unparameterized and hand-saved workflows; missing, oversized and non-regular input files; approval detection.
- `tests/test_run_inputs_records.rs`: a bound run reaching its effect with its input record and a value-free `runs show`, with the mismatch diagnostic; CLI refusals leaving no effect, run id or record; `authority inspect` with inputs producing a profile that runs exactly those values and denies others; inputless runs matching a hand-saved workflow in summary bytes, report and authority output.
- Unit tests in `inputs/parsing.rs` and `inputs/secrets.rs`; `src/headless/shell/bound/tests.rs` refuses tampered bound content and a swapped bundle with `bound_content_drift` before any run id, run record or input record.

## Risks / Trade-offs

- The secret rule is a pattern check. It does not prove an accepted value is not sensitive.
- A catalog upgrade invalidates the retained binding until the workflow is saved again from the new bundle.
- An input record can exist without a run record after a crash or an authority-context failure before the run record is written.
