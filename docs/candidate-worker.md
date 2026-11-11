# Candidate worker boundary

`candidate_worker` runs one candidate-producing command in an admitted local
workspace. It reuses the existing bounded process route; it is not a new coding
runtime or scheduler.

## Request

A `CandidateWorkerRequest` binds:

- work, attempt, fence, and workspace identities;
- an admitted `WorkspaceBoundary` root, read paths, write paths, and network or
  credential policy;
- the executable, arguments, positive deadline, output limit, and whether
  OS-level isolation is required.

The worker runs with a clear environment and the admitted workspace root as its
current directory. It does not inherit host environment values. A boundary that
requests credentials is refused before execution; the worker has no credential
provider or publisher authority. If isolation is required, the worker fails
closed because no qualified isolation provider is admitted.

## Result

The host captures the workspace before and after execution. A successful worker
returns a `CandidateReceipt` derived from the actual file maps:

- base, candidate, and diff BLAKE3 identities;
- sorted changed paths, including additions and deletions;
- exact work, attempt, fence, and workspace bindings.

Exit zero without a changed path is not a candidate. Non-zero exit, timeout, and
output overflow are failures. Observed cancellation is distinct. An interrupted
capture or receipt derivation is `Unknown` and retains the process output.

Receipts can be serialized only after admission and persisted by attempt
identity through the store. Loading revalidates the receipt. `run_and_publish_candidate_worker`
persists a produced receipt and the exact bytes of changed/added files; if
durable publication fails, it returns `Unknown` with the process output rather
than claiming success. Deleted files have no artifact bytes.

`verify_receipt_from_snapshots` independently checks expected work, attempt,
fence, and workspace bindings and recomputes the identities and changed paths
from the actual snapshots. Crossed bindings and stale snapshots are rejected.

`reconcile_candidate_attempt` reads durable records without redispatching. It
reports `Missing` when no result or receipt exists, `Incomplete` when a produced
result lacks its receipt, the stored records disagree, a declared published
artifact is absent, or only a usage observation exists, and `Published` when the
result, receipt, and artifact paths agree.

## Non-claims

This boundary does not provide OS-level read/write isolation, prevent a worker
from reading host files outside the admitted root, deny publisher credentials by
kernel policy, or prove candidate correctness. The workspace boundary is
admission data and a clear-environment process route, not a sandbox. Local
receipt/artifact persistence is not external publication, independent
acceptance, or landing authority.
