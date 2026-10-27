## Why

Standalone HTTP authority currently dispatches into a no-op client, so valid http and https workflows cannot make real authorized requests. Existing pure request planning and exact authority checks must remain separate from bounded network mechanics.

## What Changes

- Select a real native transport for stock executor and operation adapter construction, preserving explicit embedding clients.
- Admit bounded methods, URLs, bodies, headers, and deadlines before network access; keep typed native diagnostics content-free.
- Disable redirects, retries, proxies and ambient credential forwarding; verify HTTPS certificates and hostnames.
- Propagate admitted cancellation to native in-flight requests and bound response accumulation.
- Cover real loopback success and denied authority, malformed requests, timeout, oversized responses, redirects, cancellation, and TLS rejection.

## Impact

Lattice owns src/http.rs and its native transport, executor selections, operation adapter selection, effect cancellation integration, topic documentation and behavioral tests. Reuse existing reqwest 0.12.28: stable bounded-http admits only HTTPS via CanonicalUrl::parse_https and cannot preserve the existing plaintext HTTP workflow contract. No dependency update or producer changes are needed. This does not grant destination authority, sandbox HTTP, prove arbitrary peer trust, remotely cancel a request, or make effects retry-safe.
