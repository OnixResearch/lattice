# Workflow exchange consumer compatibility

This record is aimed at consumers such as Kiln that vendor the MIT-licensed
`lattice-workflow-contract` and `lattice-workflow-transport` snapshots.

## Pinned import

Kiln's pinned import is
`70496e67c7fd4a8b05914161a8e09de2759bebc8` (2026-08-20). It matches the
contract request/observation schemas and transport frames. It predates the
`shell_success` exchange rule introduced in
`6339864d3c4e5de72ba4f0899b11b5d5b76704b6` and the observed cancellation
disposition restored in `813076d7e42d0ddfb65042bee615bc5f12a2b4d3`.

## Qualified runtime

Runtime revision `85a38b6a4bceeae4bb2057cf157e21752100f1b5` (2026-10-27)
contains both changes and is the named compatibility point for:

- immutable exchange snapshots requiring every mapped shell command to exit
  zero before reporting successful completion;
- observed local-process cancellation retained as terminal `Cancelled`, with
  pending dependent effects prevented and completion-before-cancel preserved.

Consumers should run a corrected producer executable at or after that revision.
The runtime behavior is not represented by a new request or observation schema;
the contract and transport versions remain v2 and v1 respectively.

## Licensing boundary

The current Lattice contract tree is AGPL-3.0-or-later. It is not a drop-in
replacement for Kiln's MIT import. Kiln's import gate must not be refreshed to an
AGPL revision without a separate licensing decision. The historical MIT notices
remain in `LICENSE-MIT-HISTORICAL`; that file does not relicense new source.

## Non-claims

This record does not prove producer correctness, external effect success,
remote cancellation, publication, deployment, or release readiness. It records
one runtime compatibility point and the contract-import licensing boundary.
