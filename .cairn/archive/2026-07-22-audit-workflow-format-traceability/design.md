# Design

The audit partitions the exact 41-ID workflow-format subset. Direct maintained assertions are exact; image-format import admission, malformed control-flow import, credential-reference import round trip, and cookbook drift remain partial. Immutable source snapshots preserve byte identity.

Positive and adversarial validation rejects missing or unexpected decisions, classification/rationale/path drift, stale evidence hashes, selected markers, or any coverage drift.
