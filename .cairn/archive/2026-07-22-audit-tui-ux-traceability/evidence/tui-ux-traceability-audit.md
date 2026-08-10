# TUI/UX traceability audit

- selected IDs: 98
- exact maintained evidence: 81
- partial evidence only: 17
- missing before/after: 98 / 98
- dangling markers: 0
- coverage remains unsuccessful: yes
- audit receipt file BLAKE3: `b7581f8920364bf77a0b5f4dd1a377586ad19e0a16804263f527ae4906e81320`
- reviewed inventory file BLAKE3: `e76d3e2f52d01df04edf31f5c5a50a9c63478ce6a55c60df6988e3117b34e08a`
- selected-ID file BLAKE3: `24e35ab595761ba10f58242a8c7f5f9afefcf950ce9292a5b6e9fe730adaeb5a`
- audit validator file BLAKE3: `2a8eb70a121a3e5da31d4404801d9b593b1592ccdae325c65ddbfd9b6b7f09c9`
- Tracey input BLAKE3: `76d38819f1644b1df084a66c4bbeb681dba3cd13441b279acbf29ee9f0e82549`
- Tracey receipt BLAKE3: `94fd61af90f12f05c83cd3cf7e9fa60da17ae0020c7f76ccf1eb4d89bfa58cb7`

The 17 partial rows are bounded to missing direct assertions or behavior for terminal startup/layout, exhaustive keyboard authoring, active input/rendering during a long run, canvas image fallback, invalid-target diagnostic display, live replay streaming, historical replay opening, rendered run timeline/output details, empty node outputs, unavailable rerun, run-detail storage failure, recent trigger errors, and listener startup status output. In particular, persisted replay events after terminal state are not treated as active streaming evidence, and a live listener test with discarded stdout is not treated as status-output evidence.

The remaining 81 rows have direct positive or negative assertions in immutable snapshots of the maintained TUI app, input, panel, control-flow, credential, node-provider, replay, run-history, trigger, and webhook evidence. The validator rejects missing or duplicate rows, classification or rationale drift, unsafe or unknown paths, byte-hash drift, changed failing coverage, dangling markers, successful coverage, and selected markers.

A local VibeThinker adversarial review agreed that none of the 17 partial rows should be promoted from current evidence; deterministic Cargo, Cairn, Tracey, and repository checks remain authoritative.

This audit adds no TUI behavior or marker and does not claim complete terminal compatibility, complete traceability, release readiness, or production readiness.
