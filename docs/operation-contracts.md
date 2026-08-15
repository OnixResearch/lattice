# Typed operation contracts

Lattice uses typed operation contracts for reusable, instance-bound actions.

An operation type owns methods, schemas, runtime requirements, capabilities, bounds, and non-claims. An instance supplies target facts, defaults, tags, and credential references.

An instance cannot supply commands, methods, capabilities, executors, or runtime behavior. Rust rejects unsafe target keys and secret-bearing target facts.

## Authority flow

The functional core validates the catalog, instance, schemas, policy, runtime facts, and preflight facts. It then creates one deterministic invocation plan.

The plan binds these facts:

- catalog identity;
- type revision;
- instance identity;
- method and schema identities;
- merged arguments;
- exact executor identity and path;
- target facts and credential references;
- capabilities and bounds;
- required non-claims.

Descriptions, aliases, tags, diagnostics, actor labels, timestamps, and secret values stay outside authority identities. Opaque credential references remain bound, but their resolved values do not.

The plan does not grant execution authority. A self-consistent plan identity does not prove catalog admission or policy approval.

For direct runs, Lattice stores `Started` before it requests authority or starts a process. The explicit `--confirm-read-only` flag grants one local, exact, read-only process request.

The Choregraph effect boundary binds and records that request. Bounded Exec starts the process with a cleared environment and bounded output.

Lattice resolves credential references through its credential providers. It never loads credentials during catalog discovery.

Unknown process or persistence outcomes remain `Unknown`. Lattice does not retry these outcomes automatically.

## Cairn pilot

The built-in catalog is at `builtins/operations/catalog.ncl`. The example instance is at `builtins/operations/instances.example.ncl`.

Copy the example instance. Set `repository_root` to an absolute repository path.

The built-in executor is `builtins/operations/cairn-read-only-fixture`. It is a protocol fixture, not the Cairn product binary.

The fixture supports the read-only `change list` and `validate` command shapes. It lets tests and operators verify admission, authority, persistence, output validation, and promotion.

For a real Cairn binary, create a reviewed catalog revision for its exact bytes. Use this command to calculate the executor and type identities:

```bash
cargo run --example operation_contract_identities -- \
  ./path/to/catalog.ncl \
  /absolute/path/to/cairn
```

Update `runtime.executor_identity` first. Run the command again, then update the type revision.

## Inspect and plan

Catalog inspection is read-only. It does not open the executor or credential provider.

```bash
lattice operation catalog \
  --catalog ./builtins/operations/catalog.ncl \
  --instances ./instances.ncl

lattice operation show \
  --catalog ./builtins/operations/catalog.ncl \
  --instances ./instances.ncl \
  --id local-repository

lattice operation methods \
  --catalog ./builtins/operations/catalog.ncl \
  --instances ./instances.ncl \
  --type-id cairn.repository
```

Planning observes the exact executor bytes and repository-root shape. It performs no operation process effect.

```bash
lattice operation plan \
  --catalog ./builtins/operations/catalog.ncl \
  --instances ./instances.ncl \
  --instance local-repository \
  --method change_list \
  --executable /absolute/path/to/executor
```

## Check readiness

Doctor can check catalog freshness, instance shape, repository roots, and the exact executor identity. Supply all three operation paths:

```bash
lattice doctor \
  --operation-catalog ./builtins/operations/catalog.ncl \
  --operation-instances ./instances.ncl \
  --operation-executable /absolute/path/to/executor
```

This check reads credential references but does not resolve their values.

## Run

A run without confirmation prints the plan and grants no effect authority.

```bash
lattice operation run \
  --catalog ./builtins/operations/catalog.ncl \
  --instances ./instances.ncl \
  --instance local-repository \
  --method change_list \
  --executable /absolute/path/to/executor
```

Add `--confirm-read-only` to grant one exact local request.

A repeated invocation fails because its durable evidence already exists. Inspect the linked standard run with `lattice runs show RUN_ID --output json`.

## Promote to a workflow

Promotion uses the same invocation plan as a direct run. It creates one operation node with the same runtime and authority requirements.

Promotion is preview-first:

```bash
lattice operation promote \
  --catalog ./builtins/operations/catalog.ncl \
  --instances ./instances.ncl \
  --instance local-repository \
  --method change_list \
  --executable /absolute/path/to/executor \
  --workflow-name cairn-change-list \
  --destination /absolute/path/to/cairn-change-list.ncl
```

Add `--confirm-create` to create the file. Publication uses create-only file access and rejects an existing destination.

The generated Nickel source contains the complete typed invocation plan. Import does not grant execution authority.

## Evidence and non-claims

Direct runs persist identities, timestamps, byte counts, state, redacted diagnostics, and non-claims. They do not persist raw stdout, stderr, credentials, or repository content.

A content identity proves exact bytes only. Process success does not prove external truth, production correctness, release eligibility, or release readiness.

Swamp informed the operation and promotion vocabulary. Lattice does not use Swamp code or its runtime model.
