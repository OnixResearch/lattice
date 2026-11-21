# Implementation Attempt Consumer

## MODIFIED Requirements

### Requirement: Explicit one-attempt execution consumer

r[lattice.implementation_attempt.consumer] An explicitly selected CLI MUST bind the admitted immutable envelope, source snapshot, workspace, worker profile and provider before invoking the existing bounded isolated OMP worker. It MUST persist execution intent before effects. Re-entry after possible dispatch MUST reconcile retained results instead of redispatching. Worker text and process exit MUST NOT establish acceptance, integration or project completion. Within one state root, the fence check, the fence record and the creation of the execution intent of one work item MUST be one critical section under an exclusive lock that the operating system releases when its holder exits and that is never treated as stale by timeout.

#### Scenario: Crash after execution intent

r[lattice.implementation_attempt.consumer.scenario.recovery]
- GIVEN an intent persisted before worker execution and a controller crash
- WHEN the same request is submitted again
- THEN it MUST recover the existing candidate or report incomplete/unknown
- AND it MUST NOT blindly execute another worker
- AND recovery that finds no intent MUST close the attempt by exclusively creating its intent path, so no later run of that attempt can dispatch

#### Scenario: Stale, crossed and substituted requests

r[lattice.implementation_attempt.consumer.scenario.refusal]
- GIVEN a state root that records a higher fence of the same work, another attempt at the same fence, or the intent of different request bytes
- WHEN a run or recovery is requested
- THEN it MUST print one typed `lattice.implementation-attempt-refusal.v1` record and exit non-zero
- AND it MUST NOT create an execution binding, intent or worker process for that request
- AND the record's `dispatched` MUST state whether a worker effect of the refused attempt may have started, not whether this request dispatched: for `stale_fence` and `crossed_attempt`, `true` exactly when the refused attempt's directory holds an execution intent other than recovery's closure marker, read under the work's fence lock; `false` for `revision_not_admitted` and `revision_blocked`; `true` for `substituted_request`, whose attempt directory holds another request's intent, and for `export_revoked`, whose attempt executed; and for `profile_revision_superseded`, `true` exactly when the attempt directory holds an intent other than recovery's closure marker

#### Scenario: Fence check, record and intent are one critical section

r[lattice.implementation_attempt.consumer.scenario.fence_lock]
- GIVEN one state root, a run of a work item at fence 7 that has passed its fence check, and a higher fence 8 of that work recorded before the fence-7 run creates its intent
- WHEN the fence-7 run, a later fence-7 invocation, a repair step of the fence-7 attempt or a recovery of it proceeds
- THEN every fence record, execution intent, repair step intent and recovery closure marker of that work MUST be created while holding the work's exclusive lock in that state root, and an execution intent or step intent MUST be created only after a recheck under that lock finds the attempt recorded at its fence and no higher fence recorded
- AND the fence-7 run MUST be refused `stale_fence` before any intent or worker, and a `stale_fence` refusal with `dispatched: false` MUST mean that no execution intent exists for that attempt and fence and that no run can create one in that state root
- AND a repair loop whose recheck fails MUST NOT spawn its next step and MUST end as the retained post-dispatch disposition `fence_superseded`, never as a refusal or `dispatched: false`, because the attempt's intent exists and earlier steps may have had effects
- AND the lock MUST be released when its holder closes it or exits, and this guarantee MUST NOT be claimed across state roots

#### Scenario: Profile revision frames profile-level configuration only

r[lattice.implementation_attempt.consumer.scenario.profile_revision]
- GIVEN two requests of one worker profile that differ only in base snapshot, task prompt, broker expiry, workspace root or coordinator bindings
- WHEN their profile revisions are computed
- THEN they MUST share one bare-hex `lattice-worker-profile-revision-v2` revision, while a changed executable, model, route, tools, compute, capability, reviewed harness flag or request schema MUST change it
- AND an attempt retained under an earlier framing MUST be refused before any effect and never re-executed or recovered

#### Scenario: Distinct Cairn artifact source

r[lattice.implementation_attempt.consumer.scenario.artifact_source]
- GIVEN an envelope whose source kind is `cairn_artifact`
- WHEN it is admitted
- THEN its source MUST name an exact `change/stage/artifact` and it MUST use its own identity tag
- AND existing issue and task identity bytes MUST remain unchanged

#### Scenario: The recorded admission mode survives recovery

r[lattice.implementation_attempt.consumer.scenario.admission_mode]
- GIVEN attempts run without a revision store, with a store for a definition without a lifecycle, and with a store for a lifecycle-managed definition
- WHEN each writes its intent and is later recovered, inspected or exported, including after the definition becomes managed
- THEN the intent without a store MUST be the bare request identity, unchanged, and the others MUST be one canonical `lattice.implementation-attempt-intent.v1` record of the request identity, the session and the admission (`unmanaged`, or `managed` with its pin), created in one exclusive write
- AND every reader MUST take the admission from the intent and never re-derive it, an intent without an admission field MUST read as `unmanaged`, a record for another request or session, with an inconsistent pin, unknown fields or non-canonical bytes MUST be refused (by recovery as a substituted request), and a `managed` attempt MUST NOT be recovered, inspected or exported without its revision store
