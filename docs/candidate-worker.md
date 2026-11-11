# Candidate worker boundary

`candidate_worker` runs one candidate-producing command in an admitted local
workspace. It reuses the existing bounded process route; it is not a new coding
runtime or scheduler.

## Request

A `CandidateWorkerRequest` binds:

- work, attempt, fence, and workspace identities;
- an admitted `WorkspaceBoundary` root, read paths, write paths, and network or
  credential policy;
- the executable, arguments, positive deadline, and output limit.

The worker runs with a clear environment and the admitted workspace root as its
current directory. It does not inherit host environment values.

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
identity through the store. Loading revalidates the receipt.

## Non-claims

This boundary does not provide OS-level read/write isolation, prevent a worker
from reading host files outside the admitted root, deny publisher credentials by
kernel policy, publish candidate bytes, or prove candidate correctness. The
workspace boundary is admission data and a clear-environment process route, not
a sandbox. Independent verification and publication remain separate authority.
