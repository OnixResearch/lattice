## Context

The initialization shell creates the parent directory, writes `config.ncl` with `std::fs::write`, and then writes built-in templates. `std::fs::write` truncates an existing target, so a repeated or misdirected `--init` destroys reviewed Nickel configuration. Initialization is operator-triggered filesystem mutation and must fail closed when ownership of the target path is ambiguous.

## Decisions

### Decision: Claim the config path with create-new semantics

**Choice:** Open the target config with write-only `create_new(true)` semantics and write defaults only through the newly created handle. Map `AlreadyExists` to an explicit configuration error naming the target and refusal to overwrite. Keep parent-directory creation and built-in template generation in the imperative shell.

**Rationale:** The operating-system create-new primitive combines the existence check and file creation, so a competing creator cannot be truncated through a check-then-write race. A rejected repeated initialization exits before any template write. The shell remains small and contains only path creation, exclusive file creation, bounded static writes, and error plumbing.

### Decision: Do not add a force-overwrite mode

**Choice:** Keep `--init` create-only with no bypass flag.

**Rationale:** Operators can edit or explicitly remove a config using ordinary filesystem tools. Adding an overwrite switch would recreate the destructive path and complicate the safety contract without serving first-time initialization.

## Risks / Trade-offs

- A write failure after exclusive config creation can leave a partial new file; this change prevents destruction of pre-existing configuration but does not claim transactional creation of an entire config/template directory.
- Existing template files are unchanged when the config already exists because rejection occurs before template generation. Initialization into a missing config with independently pre-existing templates retains the existing template behavior and is outside this narrow fix.
