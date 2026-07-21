# Node Templates Specification Delta

## ADDED Requirements

### Requirement: Non-destructive configuration initialization
r[lattice.config.init_no_clobber] Lattice MUST create a requested configuration target without replacing an existing path and MUST fail visibly before modifying the existing config or its template catalog when the target config already exists.

#### Scenario: Fresh target initializes defaults
r[lattice.config.init_no_clobber.scenario.fresh]
- GIVEN the requested config path does not exist
- WHEN the operator runs `lattice --init` for that path
- THEN Lattice MUST create the default Nickel config and built-in template catalog and exit successfully

#### Scenario: Existing target fails without mutation
r[lattice.config.init_no_clobber.scenario.existing]
- GIVEN the requested config path and operator-authored template content already exist
- WHEN the operator runs `lattice --init` for that path
- THEN Lattice MUST exit unsuccessfully with an explicit refusal and MUST preserve the existing config and template bytes

#### Scenario: Competing creator cannot be truncated
r[lattice.config.init_no_clobber.scenario.race]
- GIVEN another process creates the requested config path before initialization claims it
- WHEN Lattice attempts to create the default config
- THEN exclusive creation MUST reject initialization without truncating the competing creator's file
