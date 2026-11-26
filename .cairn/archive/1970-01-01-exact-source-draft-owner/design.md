## Context

A catalog revision is a durable CAS token, not a source identity. A stored draft has its own independent input, preview and diagnostic. Publication via `admit_source_draft_inner` must not be repurposed for a draft save.

## Decisions

### Decision: Read explicit owner snapshots

**Choice:** Use a versioned stdin/stdout JSON command. Inspect returns head and retained association metadata. View requires the expected head, returning exact bytes for an existing logical member and the observed draft fingerprint or explicit absence. Read head and draft from a single redb read transaction.

**Rationale:** The caller can target the same edit base it observed. No ambient source paths or inferred source are used. A draft ID belongs to one workflow and cannot be moved to another workflow or silently rebased.

### Decision: Validate first, recheck both tokens at commit

**Choice:** Replace only the selected captured member, preserve all other source inputs, and invoke the existing native source preview in memory. After success, compare head and exact draft fingerprint in one redb write transaction and write only the draft table. Emit a native metadata receipt only after commit.

**Rationale:** Keeping evaluation outside the writer lock avoids blocking other saves. A single atomic recheck closes both races. Reuse native captured compilation and preview identities rather than trusting caller-provided evidence or duplicating evaluation. Reject invalid edits without modifying an older draft. A separate operator action may still admit a successful preview under existing rules.

## Risks / Trade-offs

- Exact requested bytes may contain private content; only view emits them. Native rejection codes omit source text and host paths.
- Draft CAS compares full record contents, including preview and diagnostic; byte-equivalent replacements are the same state, not a distinct historical event.
- A response lost after commit is not proof of failure; inspect and view before making a new decision. No automatic retry, publication, workload execution, credential access or authority is added.
- Exercise both transaction races and a real native preview, and verify failures leave durable state unchanged.
