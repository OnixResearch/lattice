# Choregraph effect authority

Lattice authorizes only concrete intents returned by the Choregraph V2 handoff.

## Boundary

Choregraph owns machine admission and transition decisions. It returns node, transition, operation, resource, ability, request, and payload facts. These facts are not authority.

Lattice adds machine, workflow revision, run, policy, authority, and optional collaboration-scope identities. It validates the complete binding before product dispatch.

A reviewed Nickel policy defines the Basalt contracts. Workflow graph data cannot change this policy.

## Dispatch order

1. Durably reserve the complete bound intent and attempt.
2. Load current authority and optional scope facts.
3. Validate exact request and scope bindings and enforce the reviewed Basalt policy.
4. Persist any denial before returning; successful authorization alone is not an effect outcome.
5. Resolve only the admitted intent's credential bindings and invoke the selected bounded adapter.
6. Persist an observed success, failure, or unknown-after-dispatch outcome.

A duplicate attempt or persistence error stops before the adapter. Replay always denies fresh dispatch.

## Standalone operator profile

The standalone CLI, TUI, scheduler, orchestration, and approval-resume paths use
the optional absolute `effect_authority` path in application configuration. No
selection preserves unavailable authority. An invalid selected profile denies;
it never falls back to another provider. Explicit embedding providers remain a
separate host-owned choice.

This is local process authority from a trusted operator, not externally verified
UCAN evidence. The profile cannot manufacture token signatures, proof chains,
holders, or verification receipts. Basalt's reviewed policy remains an upper
bound on either provenance.

From the README's isolated daily-digest setup, inspect the saved workflow:

```sh
cargo run --locked -- --config "$demo/config.ncl" authority inspect daily-digest \
  > "$demo/authority-requirements.json"
```

Review the workflow commands and the reported `schema`, `workflow_revision`,
`machine_identity`, `policy_ref`, and `grants`. Inspection is read-only: it does
not create missing stores, grant authority, or execute work. A changed workflow
needs new inspection and an explicitly reviewed replacement profile.

After review, this example grants exactly those requirements for one hour:

```sh
lifetime_seconds=3600
deadline_ms=$(( ($(date -u +%s) + lifetime_seconds) * 1000 ))
cat > "$demo/authority.ncl" <<EOF
let contracts = import "$PWD/contracts/standalone-effect-authority.ncl" in
let required = import "authority-requirements.json" in
{
  schema = required.schema,
  workflow_revision = required.workflow_revision,
  machine_identity = required.machine_identity,
  policy_ref = required.policy_ref,
  grants = required.grants,
  expires_at_unix_ms = $deadline_ms,
  roots = {},
} | contracts.StandaloneEffectAuthorityProfile
EOF

cat > "$demo/config.ncl" <<EOF
{
  data_dir = "$demo/state",
  effect_authority = "$demo/authority.ncl",
  default_timeout_secs = 60,
  shell = "sh",
}
EOF

cargo run --locked -- --config "$demo/config.ncl" run daily-digest --output json
```

The contract is [`contracts/standalone-effect-authority.ncl`](../contracts/standalone-effect-authority.ncl).
Profiles admit 1–64 explicit grants, at most 64 roots, and nonempty root IDs of
at most 128 UTF-8 bytes. Node IDs are unique. Each grant binds one exact node,
operation, resource, and ability; wildcards, unknown fields, and unsupported
tuples deny. Process, network, file, and Agent grants cannot substitute for one another.
The stock executor selects `NativeHttpClient` for HTTP and HTTPS. A current network
grant permits only the existing exact planned effect; transport admission and TLS
verification can still reject it. Authorization alone is not evidence of a sent
request or remote success.

Keep the explicit record form above. The pinned embedded evaluator panicked on
`(import "authority-requirements.json") & { ... }` under this contract even
though external `nickel export` accepted it. Native execution, not external
export alone, is the compatibility check for the selected evaluator.

At run admission, the host evaluates the profile, observes time, checks expiry,
and pins the complete evaluated BLAKE3 identity. Before every effect it repeats
evaluation and checks the same identity, exact workflow/machine/run/authority
binding, and finite expiry. Equality with the deadline denies. Removal, semantic
changes to imported inputs, evaluation failure, and clock failure deny subsequent
dispatch. Formatting that preserves the evaluated value does not revoke it.
Protected collaboration scopes cannot use this unscoped local provider.

For file nodes, replace `roots = {}` with operator-controlled absolute paths,
for example `roots = { workspace = "/srv/lattice/workspace" }`, and select
`"workspace"` in the node's `allowed_roots`. The host opens these directories at
run admission. Workflow root names do not open ambient paths; an unknown root,
escaping relative path, or no-follow violation denies the file effect.

The profile and its imports must remain operator-controlled. A process grant
allows the exact workflow's command to run with its configured host environment;
it is not a sandbox. Rechecks are not atomic with external effects and do not
prove global revocation freshness, filesystem or mount stability, exactly-once
execution, remote caller identity, or release authority. Durable attempt
reservation and the existing adapter limits remain required.

## File effects

The host opens `cap_std::fs::Dir` roots from operator-owned configuration and assigns root identifiers. Graph data can name only one configured identifier and a relative path.

`cap-root` performs lexical admission. Existing capability-relative open behavior enforces the Lattice no-follow boundary. Only the host shell opens ambient roots; pure authority admission does not access the filesystem.

## Process, network, and Agent effects

Process effects use the existing bounded process runner. Network effects use the
native HTTP client after exact authorization and pure method/URL planning.
`cap-std` does not authorize either effect type. Embedders may explicitly select
another `HttpClient`, including `NoopHttpClient` to disable networking.

Agent nodes require `lattice/agent`, `agent/execute`, and the exact
`lattice://agent/<node>` resource under `lattice-agent`. Their material-free
payload binds the request, selected command/OpenAI-compatible/Animus backend,
runtime configuration, credential references, and resolver policy. A target name
alone is not a backend identity. Denied or unavailable authority stops before
provider resolution and transport; a consumed recipe cannot dispatch twice.
Changing the backend or resolver policy invalidates the sealed request, while
rotating a value behind unchanged references does not persist or hash that value.

Adding the Agent contract changes the shared policy identity. Re-inspect every
affected workflow and review a replacement operator profile; profiles bound to
the previous policy are rejected, not silently upgraded.

Custom `AgentAdapter` implementations must supply `execution_identity` without
resolving credentials or performing effects. The explicit compatibility/embedder
path remains trusted host execution, not a product authorization fallback.

### Native HTTP bounds and trust

<!-- r[impl lattice.native_http.bounds] r[impl lattice.native_http.security] r[impl lattice.native_http.cancellation] -->

The native client uses the existing Reqwest 0.12.28 dependency and verified Rustls
TLS, on HTTP/1 only. It disables automatic redirects, protocol retries, proxies,
referer propagation and idle connection pooling. It does not read `.netrc`, infer
credentials from URL userinfo, install cookies, or accept invalid certificates or
hostnames. HTTP status failures and redirects remain ordinary workflow responses;
the client never follows `Location`, including same-origin locations.

| Boundary | Native limit |
| --- | --- |
| URL | 8192 UTF-8 bytes; http/https with a host, no userinfo, fragment, control or whitespace bytes |
| Request body | 1 MiB |
| Response body | 8 MiB, checked before accumulation; UTF-8 required |
| Request/decoded response headers | 64 fields and 16 KiB total name/value bytes plus four framing bytes per field |
| Request deadline | 1–300 seconds, including response body reads |
| Connect deadline | Minimum of request deadline and 10 seconds |
| Cancellation observation | Poll the admitted runtime flag every 10 ms |

Consumers may use `HttpClient::execute_bounded(plan, &HttpBounds, cancellation)`
to narrow response body, header bytes and combined DNS/TCP/TLS connect deadlines.
Each bound must be positive and at most its native ceiling. An embedding client
that does not implement this strict port fails closed instead of ignoring limits.
The ordinary native execute methods use this same path with the default ceilings.
Native-owned accumulated response bodies and decoded headers are zeroized on
failure or cancellation; successful values transfer to the caller. This does not
promise erasure of Reqwest/Rustls internal copies, allocator history, kernel
buffers, or complete process memory. Credential-aware callers must own and clear
their request and successful response buffers.

Request `Host`, framing, proxy authorization and hop-by-hop headers are
transport-owned and rejected, as are case-insensitive duplicate request fields.
Request budgets conservatively reserve three generated fields (`Host`, `Accept`,
`Content-Length`), leaving at most 61 supplied fields even when a method omits
one generated field. The byte budget includes those generated-field reservations.
Repeated response fields use newline-separated values in the workflow header
map, retaining distinctions such as repeated `Set-Cookie`. Header limits include
every repeated field. Native errors contain fixed typed classifications, never
vendor errors, URLs, headers or bodies. Response values are intentional workflow
data and are not metadata-only receipts.

The 16 KiB decoded-header limit is not an exact wire-parser allocation ceiling:
Reqwest has no public knob for the HTTP/1 parser buffer. Pinned Hyper 1.10.1 has
its own finite default read ceiling of 417792 bytes; oversized or malformed wire
heads may fail there as a redacted transport error before decoded admission.
Automatic gzip, Brotli, deflate and Zstd decompression is explicitly disabled,
including when another dependency enables a compression feature.

Stable `bounded-http` checkout `bc4fe8d81f3defc178ed702096452f73d9675c76` was
inspected first. Its `CanonicalUrl::parse_https`
rejects `http:` and the shell always negotiates authenticated TLS. It cannot
implement Lattice's existing plaintext HTTP workflow contract. Using the existing
Reqwest dependency for both schemes avoids two incompatible transport conventions
or a producer-local scheme bypass. This does not import an unaccepted producer
revision or promise bounded-http's destination/peer-revalidation contract.

A network grant is **not** an IP sandbox, SSRF filter, DNS-rebinding defense, or
arbitrary peer trust. Operators own destinations, local services and workflow
content. Plaintext HTTP is intentionally supported and offers no confidentiality;
use HTTPS when sending credentials. HTTPS establishes ordinary certificate and
hostname validation under the client's public roots, not application trust.

The native scoped worker is joined. Pre-cancellation prevents any request;
in-flight cancellation or timeout drops local HTTP futures and closes the client.
OS DNS resolution may continue independently after cancellation; shutdown does
not wait indefinitely for the resolver. Neither cancellation nor timeout proves
the server did not receive or perform the request. There are no implicit retries.

### Deterministic local HTTP and TLS smoke

Run the real production-client and actual-CLI regressions from the Lattice root
after the coordinated source build:

```sh
cargo test --locked --test test_native_http_transport --test test_native_http_authority
```

The transport cases bind isolated loopback ports and exercise request/response
content, malformed plans without connections, exact body/header boundaries,
redirect denial, server errors, truncation, deadlines, cancellation, and rejection
of plaintext on an HTTPS endpoint plus rejection of a fixed public test-only
self-signed localhost certificate by the actual production client. The TLS server
uses an explicit per-server crypto provider, not process-global mutation. The CLI
case launches the actual `lattice`
binary twice against an isolated store: missing authority must not connect; an
exact current operator grant must send `GET /observed` and complete successfully.

For certificate-verification smoke, create a temporary self-signed localhost
certificate with `openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj
'/CN=localhost' -addext 'subjectAltName=DNS:localhost'`, writing its key and cert
only to a temporary directory. Start `openssl s_server -accept 127.0.0.1:PORT
-cert CERT -key KEY -www -naccept 1` on a free local port. Import an HTTP workflow
whose URL is `https://localhost:PORT/`, inspect it, and grant that exact workflow
using the operator-profile procedure above. The actual CLI run must fail with an
adapter rejection, and the server must not receive an HTTP request after its
untrusted handshake. Do not add a test root or disable TLS verification in the
production client. A positive HTTPS smoke requires a locally provisioned endpoint
with a certificate chain already trusted by the production public root store;
self-signing does not supply that prerequisite. Stop the local server and remove
the temporary directory afterward. This is a scoped TLS check, not arbitrary
remote peer trust or a sandbox claim.

## Scope currentness

Protected effects require an explicit current snapshot from a Lattice-owned port. The snapshot binds scope, revision, membership epoch, audience, consent, resource, ability, request, and snapshot identity.

A missing or stale snapshot denies dispatch. `RunOwner`, trigger history, UI audience, and prior authorization are not scope authority.

## Adoption baseline and source admission

r[impl lattice.choregraph_effect_authority.baseline]

The [pre-integration baseline](../.cairn/archive/2026-08-13-authorize-choregraph-effect-intents/evidence/baseline-and-blocker-2026-07-27.md) records the effect adapters, operator-configured roots, outer admission ordering, absent per-node attempts, replay behavior, Basalt/UCAN sources, supplied-fact boundary, and crash fixtures before integration. The [resolution record](../.cairn/archive/2026-08-13-authorize-choregraph-effect-intents/evidence/blocker-resolution-2026-08-13.md) retains the later pre-change checks and selected immutable V2 intent model.

r[impl lattice.choregraph_effect_authority.baseline.scenario.unknown]

The original baseline left implementation blocked while production intent and machine-identity ordering were unresolved; it did not substitute an external allow boolean or a task checkbox for that prerequisite. These are bounded historical adoption records, not fresh execution results or a new automatic readiness gate.

r[impl lattice.choregraph_effect_authority.source]

The admitted build uses the checked-in Basalt snapshot at `bb8fb5f4a763f532df96b6e65407163ac668834e` and its UCAN snapshot at `277ab768eb532ac0c715a3bbff2a7dc37a702f11`. `vendor-cargo-deps/basalt/SOURCE` binds their revisions and the Basalt archive identity. Cargo selects that exact local snapshot; it has no authenticated upstream fetch fallback.

r[impl lattice.choregraph_effect_authority.source.scenario.ssh]

The original baseline explicitly rejected the direct Basalt dependency because its SSH-only UCAN closure could not be reproduced without source authority. It recorded a release blocker rather than skipping Cargo-dependent checks. The resolution selected the reviewed vendored closure instead; it did not infer source access from a sandbox-safe check that deferred Cargo.

## Non-claims

r[impl lattice.choregraph_effect_authority.non_claims.scenario.scope]

`AuthorizationOutcomeEvent` records the bound local attempt, its local outcome state, and any authorization denial. An observed success is an observation of the selected local adapter result, not evidence of external truth or a global guarantee.

Checks do not prove human identity, membership-store correctness, consent-policy sufficiency, key truth, global revocation freshness, replay-store durability, UCAN cryptographic correctness, operating-system correctness, external effect success, exactly-once behavior, or whole-workflow correctness.
