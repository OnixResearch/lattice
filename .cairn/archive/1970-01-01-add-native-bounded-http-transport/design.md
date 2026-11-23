## Context

HttpRequestPlan is pure workflow data, and Adapters::network validates its exact authorized payload before invoking HttpClient. Stock constructors select NoopHttpClient. Existing OpenAI transport isolates reqwest runtime lifecycle on an OS worker. The stable bounded-http producer only accepts HTTPS (bounded-http-core/src/target.rs: CanonicalUrl::parse_https rejects other schemes) and always negotiates authenticated TLS in orchestration. Routing only https there would establish two conventions and still require another client for the existing http contract.

## Decisions

### Decision: Reuse the existing Reqwest dependency

Use existing reqwest 0.12.28 with Rustls and HTTP/1 only for both schemes. Keep pure request admission in the HTTP module and native I/O in its subordinate module. Disable proxy discovery, redirects, protocol retries, cookies, referer propagation and idle pooling. Use the default verified public root store; never accept invalid certificates or hostnames. URL userinfo and fragments deny; the client never infers Authorization from a URL.

### Decision: Bound inputs and decoded outputs

Named ceilings: URL 8192 bytes, request body 1 MiB, response body 8 MiB, each header map 64 fields and 16 KiB aggregate names/values/framing, request timeout 1–300 seconds and connect timeout at most 10 seconds. Reject framing/hop-by-hop/Host overrides and case-insensitive duplicate request headers. Bound decoded response headers before exposing values and check each body chunk before accumulation; invalid UTF-8 fails without printing content. Reqwest does not expose a configurable HTTP/1 parser buffer bound: the pinned Hyper parser has its own finite default ceiling, separate from the stricter decoded-header boundary. Do not claim that 16 KiB is a wire-parser allocation ceiling.

Actual OIDC/GitHub consumers need lower limits. A narrow HttpBounds value supplies response_body_bytes, header_bytes and connect_timeout_secs per call, each positive and no greater than native ceilings. HttpClient::execute_bounded rejects unsupported embeddings by default; native execute methods use the same implementation with default ceilings. The connect deadline bounds DNS, TCP and TLS establishment together; it is not independent resolver-stage admission. Request header budgets reserve Host/Accept/Content-Length before supplied fields. Accumulated body and decoded headers are cleared on failure/cancellation until ownership passes to the caller; Reqwest/Rustls/kernel buffers and complete RAM erasure are explicitly outside this guarantee.

### Decision: Cancellable native worker lifecycle

A scoped named OS worker owns a current-thread Tokio runtime and async reqwest client, so construction/destruction is safe inside and outside Tokio. Poll the admitted AtomicBool every 10 ms using Tokio select; cancellation or deadline drops the request future and client. Join the worker, never detach ongoing HTTP work. Cancellation cannot undo bytes already sent or prove remote cancellation. The HttpClient default cancellation method checks pre-cancellation for explicit embeddings; native overrides provide in-flight support.

### Decision: Preserve authorization before dispatch

Keep all request/payload authority checks and durable attempt ordering unchanged. Replace production no-op selections only, preserve NoopHttpClient for deliberately disabled embeddings. Pass existing runtime cancellation into HttpClient after authorization. No transport-level retries or redirects may expand the admitted destination or duplicate uncertain effects.

## Risks / Trade-offs

Authorized plaintext HTTP is intentionally unencrypted; operators own endpoint selection and must use HTTPS for credentials. Explicit network grants are not an IP sandbox, DNS-rebinding protection, arbitrary peer trust, or SSRF filtering. Response headers/body are workflow data, not diagnostics. A scoped worker per request trades setup overhead for bounded lifecycle and safe synchronous embedding. OS DNS resolution may continue in a resolver worker after a dropped request; runtime shutdown must not wait indefinitely for DNS. A finite runtime shutdown timeout closes the network runtime without promising OS resolver cancellation.

## Verification

Paired deterministic loopback regressions exercise success versus malformed input and redacted failure, response body/header boundaries, redirects without forwarding, timeout and pre/in-flight cancellation, and denied authority with no accepted connection. Existing verified TLS remains mandatory; a local untrusted TLS server must be rejected. Parent owns focused/integrated builds and actual CLI verification after concurrent edits. No runtime task is complete on structural gate evidence alone.
