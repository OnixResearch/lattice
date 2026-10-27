## 1. Admission

- [x] [serial] Inspect stable bounded-http and existing Reqwest patterns; record HTTPS-only mismatch and explicit non-claims. r[lattice.native_http.security]
- [x] [serial] Admit proposal, design and tasks gates before implementation. r[lattice.native_http.authority]

## 2. Implementation

- [x] [serial] Implement native request admission, finite decoded bounds, redacted typed failures and cancellation-aware transport. r[lattice.native_http.bounds] r[lattice.native_http.cancellation]
- [x] [serial] Replace production no-op selections and propagate existing runtime cancellation without changing authority binding. r[lattice.native_http.authority]
- [x] [serial] Update topic documentation with concrete bounds, producer mismatch, TLS and smoke instructions. r[lattice.native_http.security]

## 3. Verification

- [x] [serial] Run focused behavior regressions for real request/response, malformed input, body/header bounds, redirects, timeout, cancellation and TLS rejection. r[lattice.native_http.bounds] r[lattice.native_http.security] r[lattice.native_http.cancellation]
- [ ] [serial] Exercise actual standalone authorized and denied local HTTP execution; record observed output and no-request evidence. r[lattice.native_http.authority]
- [ ] [serial] Parent runs coordinated checks and lifecycle closeout using observed evidence, never marking runtime tasks from structural gates alone. r[lattice.native_http.authority]

## Observed verification boundary

All nine unchanged transport tests passed against the actual native module in a
temporary consumer using the declared Lattice nightly/clang/mold tools and the
unchanged repository Cargo configuration. All 143 resolved registry package
identities matched the existing product lock. A real TCP API smoke observed
`GET /smoke`, HTTP 201, and exact body `native` (six bytes). See
`evidence/native-transport-observations.json` for source BLAKE3 identities,
commands, setup failures, and scope. This does not exercise the full library,
standalone authority path, or coordinated product gates; those tasks stay open.

