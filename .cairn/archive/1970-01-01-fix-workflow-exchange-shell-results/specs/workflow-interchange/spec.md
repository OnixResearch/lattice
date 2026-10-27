## ADDED Requirements

### Requirement: Successful command semantics for exchange shells

r[lattice.workflow_interchange.shell_success] Immutable workflow snapshots executed by the workflow-exchange application MUST require every mapped shell command to exit zero before completing that node successfully. A nonzero or unavailable successful exit status MUST persist node failure, suppress dependent effects through normal runner failure propagation, and prevent a successful terminal workflow observation. General executor workflows outside this exchange snapshot path MUST retain their existing exit-as-data semantics.

#### Scenario: Shell exits zero
r[lattice.workflow_interchange.shell_success.scenario.zero]
- GIVEN an admitted mapped shell command that exits zero
- WHEN immutable exchange execution completes and its retained run is observed
- THEN the node MUST be retained as completed and the workflow MAY expose a successful terminal observation when every required node succeeds.

#### Scenario: Shell exits nonzero
r[lattice.workflow_interchange.shell_success.scenario.nonzero]
- GIVEN an admitted mapped shell command that exits 23 and a dependent shell with an observable effect
- WHEN immutable exchange execution completes and its retained run is observed
- THEN the first node MUST be retained as failed, the dependent effect MUST NOT occur, and the workflow MUST expose failed terminal state with `successful` false.
