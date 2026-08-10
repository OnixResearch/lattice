# Design

## Functional cores

- Active replay start records are produced deterministically from run identity and workflow nodes; the App shell emits them before execution and stores the resulting timeline while the run remains active.
- Canvas image fallback selection derives a bounded metadata line from typed run-output records without exposing raw bytes.
- Run-detail load classification maps loaded, missing, and storage-error observations into deterministic TUI transitions before the App shell mutates state.
- Trigger status rendering joins declared triggers to bounded safe run-history summaries without rendering raw messages, payloads, credentials, or authorization data.

## Imperative shells

The existing App event loop, Store, panel renderer, and webhook listener remain the effect boundaries. Tests drive public or test-local shell seams and inspect bounded state or rendered buffers; no terminal device, network authority beyond the existing loopback fixture, ambient credential resolution, or unrestricted dump is added.

## Evidence completion

The completion validator starts from the immutable 81-exact/17-partial audit and requires all 98 rows to become exact against current evidence hashes. Tracey must remain exactly 98 missing, uniquely sorted, invalid, and free of dangling markers. Completion does not attach selected markers.
