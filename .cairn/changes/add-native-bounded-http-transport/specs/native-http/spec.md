# Native HTTP Transport Specification

## ADDED Requirements

### Requirement: Authorized native HTTP
r[lattice.native_http.authority] Stock standalone and operation adapter selections MUST provide a real bounded native HTTP client supporting existing http and https request plans. Existing exact request and payload binding and durable reserve-before-effect behavior MUST remain mandatory.

#### Scenario: Authorized request reaches local server
r[lattice.native_http.authority.scenario.allowed]
- GIVEN an exact current operator network grant and an admitted local HTTP workflow
- WHEN standalone execution dispatches the request
- THEN the server MUST receive the planned request and the workflow MUST observe its response.

#### Scenario: Missing authority cannot send
r[lattice.native_http.authority.scenario.denied]
- GIVEN the same workflow without admitted authority
- WHEN standalone execution attempts dispatch
- THEN no connection or request MUST reach the server.

### Requirement: Finite content-free transport boundaries
r[lattice.native_http.bounds] Native transport MUST enforce named URL, request body, response body, request and decoded response header count and byte limits, finite request and connect deadlines, and content-free typed failures. Malformed or oversized request input MUST deny before network access. The transport MUST NOT print request URLs, headers, bodies or underlying vendor diagnostics in native errors. Per-call bounds MAY narrow native response, header and combined DNS/TCP/TLS connect ceilings, but MUST NOT enlarge or disable them; embedding clients without bounded support MUST reject explicitly.

#### Scenario: Oversized response is rejected
r[lattice.native_http.bounds.scenario.response]
- GIVEN a response exceeding its decoded header or body bound
- WHEN native transport reads it
- THEN it MUST return a typed limit error without exposing response content.

#### Scenario: Malformed request is rejected
r[lattice.native_http.bounds.scenario.request]
- GIVEN malformed input or a zero or excessive deadline
- WHEN native transport admits the plan
- THEN it MUST reject without making a request.

### Requirement: No implicit destination or credential expansion
r[lattice.native_http.security] Native transport MUST disable automatic redirects, retries, proxy discovery and ambient URL credentials. HTTPS MUST verify certificates and hostnames with safe TLS. HTTP grants MUST NOT be described as sandboxing, DNS peer admission, or arbitrary peer trust.

#### Scenario: Redirect is returned unchanged
r[lattice.native_http.security.scenario.redirect]
- GIVEN an admitted request returning a redirect
- WHEN native transport observes the response
- THEN it MUST return that response without following its location or forwarding credentials.

#### Scenario: Untrusted TLS fails closed
r[lattice.native_http.security.scenario.tls]
- GIVEN an HTTPS server with an untrusted certificate
- WHEN native transport connects
- THEN the TLS handshake MUST fail without disabling certificate verification.

### Requirement: Bounded cancellation semantics
r[lattice.native_http.cancellation] Existing runtime cancellation MUST reach the native client after authorization. Pre-cancellation MUST prevent a request; in-flight cancellation and timeout MUST drop local request processing without a retry or detached HTTP worker. Cancellation MUST NOT claim that a peer undid an already received effect.

#### Scenario: In-flight cancellation interrupts waiting
r[lattice.native_http.cancellation.scenario.inflight]
- GIVEN a native request waiting for a delayed server response
- WHEN admitted cancellation is observed
- THEN local execution MUST return a typed cancelled error without waiting for the server response.

#### Scenario: Timeout is classified
r[lattice.native_http.cancellation.scenario.timeout]
- GIVEN a server delaying past the admitted deadline
- WHEN the finite deadline expires
- THEN native transport MUST return a typed timeout without retrying.
