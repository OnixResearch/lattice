# Workflow-execution traceability audit

- selected rows: 83
- exact maintained evidence: 79
- partial evidence only: 4
- missing before/after: 181 / 181
- dangling markers: 0

Partial decisions are bounded to the two Rats source-readiness rows and the subworkflow child-output and parent/child-event rows. The current executor emits a typed admission value for a subworkflow node but does not execute a saved child workflow or correlate child runner events. The vendored Rats `SOURCE` records a revision and path set, but the audit does not prove newest-revision or complete-source admission.
