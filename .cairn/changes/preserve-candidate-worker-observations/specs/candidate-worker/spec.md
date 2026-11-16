## ADDED Requirements

### Requirement: Truthful candidate process observations

r[lattice.candidate_worker.process_observation] The process adapter MUST reserve Spawn for observed spawn failure and distinguish pre-spawn admission rejection, post-start execution/capture failure, and incomplete teardown. It MUST retain bounded safe diagnostics and observed-versus-unknown termination without asserting that unknown work did not start or is still running.

#### Scenario: Post-start failure retains evidence
r[lattice.candidate_worker.process_observation.scenario.interrupted]
- GIVEN a controlled owned child that has written an independently observed candidate marker
- WHEN capture or teardown fails after child start
- THEN the candidate worker MUST retain available file evidence, persist Unknown with the typed failure observation, and reconcile without redispatch or usable candidate receipt.

#### Scenario: Cleanup observation is not erased
r[lattice.candidate_worker.process_observation.scenario.cleanup]
- GIVEN the selected runner's Wait or capture-worker error after successful child cleanup
- WHEN the error crosses the process adapter
- THEN observed child termination MUST remain distinct from a Kill, Reap or child-teardown failure whose termination is unknown.

### Requirement: Explicit bounded OMP JSONL profile

r[lattice.candidate_worker.omp_jsonl] A selected OMP coding profile MUST bind an operator-reviewed executable byte identity, exact protocol reference, admitted base and explicit bounded headless argv. It MUST normalize session-header/JSONL terminal observations without treating model text as a candidate receipt and MUST retain admitted raw events separately from normalized fields. Generic command agents MUST remain usable without selecting this profile.

#### Scenario: Protocol completion is incomplete or failed
r[lattice.candidate_worker.omp_jsonl.scenario.invalid]
- GIVEN bounded OMP events with crossed workspace identity, malformed or truncated records, absent terminal completion, or an error/aborted terminal reason
- WHEN the coding profile normalizes the event stream
- THEN it MUST NOT report a usable produced candidate from exit zero or final-response text alone, and MUST NOT invent complete usage telemetry.

### Requirement: Complete declared file capture

r[lattice.candidate_worker.file_capture] Host candidate capture MUST include ordinary staged, unstaged and untracked files within its byte-only contract, exclude only declared root VCS metadata, reject unsupported file types/modes or excessive output, and never silently omit ordinary agent/build-named paths.
