# GitHub asynchronous Check Runs transport

The adapter in `src/github_ci/github.rs` connects the provider-independent
`github_ci::core::CheckProjection` to GitHub's HTTPS APIs. It is not an OIDC
verifier, Basalt issuer, Mantle build executor, or release decision. Boundary
fixtures in `tests/test_github_ci_transport.rs` are deterministic controlled-port
evidence, **not** a claim that a live GitHub repository or Basalt deployment has
been exercised.

## Private/report-only prerequisites

Before enabling publication, provision a private test repository, a reviewed
Basalt audience and exact Actions identity endpoint host allowlist, a trusted
Mantle producer and authorized artifact/result links. Enable GitHub Actions
`id-token: write` only on the identity-acquiring job. That permission exposes the
documented `ACTIONS_ID_TOKEN_REQUEST_URL` and
`ACTIONS_ID_TOKEN_REQUEST_TOKEN`; the adapter reads no alternative identity or
static Mantle token. The request URL may carry GitHub's `api-version=2.0` query;
Lattice adds exactly one explicit Basalt `audience`. Other queries, userinfo,
non-HTTPS endpoints, non-443 ports, fragments and unapproved hosts are rejected.
The allowlist must enumerate exact deployment-reviewed hosts, not a suffix or
wildcard. Do not assume an illustrative hostname is a valid deployment endpoint.

For REST publication, grant the selected GitHub App installation access to the
exact repository and Checks write permission. GitHub's Check Runs write API is
intended for GitHub Apps; do not assume an ordinary PAT or job token has this
capability. Use an explicit credential reference in the accepted Lattice
credential registry, backed by the reviewed Animus SecretSpec provider. Supply
the actual current admitted node's `CredentialResolutionContext`, with
`HttpHeader` destination, `Authorization` binding, matching reference and the
configured exact public resolution reason. The adapter never invents workflow
or node identity and never tries an ambient credential when resolution fails.
`api.github.com` must appear in the explicit API host policy. Enterprise/custom
API origins are not accepted by this adapter.

Use `NativeHttpClient` only under the host's admitted effect/dispatch authority.
The bounded transport port itself supplies no grant. Pin producer/dependency
revisions through the repository-owned acquisition path before claiming fresh
integration evidence; credentials do not establish producer compatibility.

Start with private repository scenarios: identity permission denied, credential
revocation, duplicate external IDs, lost create/update responses, pagination,
rate limiting, cancellation and recovery. Then enable report-only publication
on the selected real workflow, without required status. Required status needs
the core rollout admission evidence and repository Cairn gates; this document
and offline boundary fixtures are not a substitute for those receipts.

## Transport/resource policy

The native shared HTTP implementation validates public-root TLS, disables
redirect following, proxy inheritance, retries and automatic decompression, and
fails closed when the bounded port is unavailable. GitHub requests use the
following named ceilings from `github.rs`:

| Resource | Bound |
| --- | --- |
| Whole request | 20 seconds |
| DNS + TCP connect + TLS, combined | 5 seconds |
| URL | 8,192 bytes |
| Headers | 32 adapter-observed fields, 8,192 bytes; native framing/global ceilings also apply |
| OIDC response body | 16,384 bytes, enforced while reading |
| Transient bearer | 12,288 bytes; request header ceiling may reject a smaller effective token |
| REST body | 1,048,576 bytes, enforced while reading |
| List pagination | 10 pages, 100 records per page, at most 1,000 records |
| Annotations | 50 per create/update |
| Title | 255 bytes |
| Public summary | 32,768 bytes, plus a fixed bounded public reconciliation marker |
| Rendered text including evidence links | 32,768 bytes |
| Annotation message | 4,096 bytes |

OIDC acquisition does not verify token claims: send the returned opaque
`OidcToken` immediately to Basalt for verification/exchange. Both it and the
documented request credential's `ActionsRequestToken` have no Debug, Clone or
Serialize implementation. Explicit acquisition takes `&ActionsRequestToken`,
constructed by moving the documented value through `from_documented_value`.
Owned environment/token/request/response
buffers are zeroizing transient containers; API credentials retain the registry's
redacted, zeroizing `SecretValue` until immediate use. The adapter clears its
own secret header/body copies on every return path. This is not a guarantee
that third-party TLS/HTTP libraries erase every internal in-flight buffer.
Never serialize/debug a secret-bearing generic HTTP plan, log HTTP bodies,
retain an OIDC token for restart, or include credentials/value-derived hashes
in durable state or evidence.

REST requests pin `Accept: application/vnd.github+json` and
`X-GitHub-Api-Version: 2022-11-28`. Repository segments and lowercase 40-hex head
SHA are validated before URL construction. Pagination is reconstructed under
the fixed API origin; response `Link` destinations are never followed. The
adapter lists **all** check runs for the exact commit (`filter=all`), identifies
the exact job external ID, and verifies check name/head SHA. Duplicate matching
external IDs, duplicate record IDs, changing/inconsistent counts, oversized
pages and ambiguous bindings fail closed. Concurrent writers must be serialized
by the owning durable host; GitHub provides no create idempotency key.

Only bounded, already-public/redacted reducer content may be projected. Logs
stay in Mantle and are represented by admitted HTTPS evidence links. Core
`PublicUrl` policy rejects query strings, userinfo, fragments and off-policy
origins/path prefixes; the adapter rechecks HTTPS/query rejection. Do not place
signed query credentials in evidence URLs or bypass this typed URL admission.

## Durable reservation and recovery

1. Create `CheckRunBinding::new(CheckRunTarget { owner, repository, head_sha,
   name, external_id })`, using the core job identity for `external_id`.
2. Call `prepare_check(&binding, &projection)`. **Commit the returned
   `plan.reserved_binding()` durably before any network effect.** Retain the
   public projection or reconstruct the exact same projection from durable core
   state on restart. The pending intent binds version, payload update identity
   and terminal status; it contains no bearer material.
3. Pass the one-use plan by value to `GitHubChecksClient::execute`, together with
   the same projection and the current contextual credential grant. It lists
   before creating/updating. Persist the acknowledged binding and apply the
   returned check ID to the owning orchestration state before another effect.
4. On a lost response or malformed success, keep the reservation. Replanning a
   pending intent is **recovery only**: list by exact public job identity and
   acknowledge only the exact public version/update marker and remote status /
   conclusion. Never infer that an empty list proves an uncertain POST failed.
5. A missing uncertain create, an unmatched uncertain update, duplicate records,
   or a contradictory terminal record requires operator reconciliation. The
   adapter does not automatically resend an uncertain mutation: GitHub can
   append duplicate annotations, and listing is not an exactly-once guarantee.
   Preserve the reservation and investigate the exact repo/SHA/external ID.
6. Only a definitive observed rejection permits `binding.after_rejection(error)`
   to clear the reservation for a policy-controlled retry. Timeouts,
   disconnects, malformed successes, 409/422 conflicts and 5xx responses never
   grant that reset. A rejected recovery read also cannot clear the earlier
   uncertain mutation.
   Persist the cleared public state before preparing a later attempt.

A stale projection version is a no-op, including read-back of a newer remote
version marker. An exact applied terminal duplicate is a
no-op; equal-version payload conflict or mutation after a terminal state is
rejected. A public HTML comment appended to the output summary records the
version/update identity for read-back recovery. Do not edit this marker by hand
or adopt a check run based only on its display name.

## Failure policy and operations

Errors expose only closed categories and public timing facts, never a raw
response body, bearer, request URL or credential-provider error. A 401 is a
revoked credential, 404 a missing/inaccessible repository, ordinary 403 a
permission denial. Exhausted primary limits use `X-RateLimit-Reset`; secondary
rate-limit/abuse responses and 429 use `Retry-After` when present. Secondary
limits without that header wait the named 60,000 ms minimum. Invalid,
overflowing or unsupported date-form retry headers stop rather than authorize
an early retry. Server errors/disconnects are transient but their mutation
outcome remains uncertain.

Call `Error::retry_decision(retries, started_ms, now_ms, &core_policy)` with
injected Unix milliseconds. The core owns the reviewed retry count, exponential
delay, maximum delay and elapsed-time bounds. Server minimum delays are never
capped downward; a delay outside policy stops. Retry decisions do not clear a
pending mutation and do not authorize a second uncertain POST/PATCH.

For an incident, disable new publication, revoke/rotate the referenced GitHub
App credential through the configured provider, preserve public bindings and
producer/evidence identities, and reconcile exact pending jobs before resuming.
Do not dump environment variables or HTTP captures into the incident report.
Cancellation remains the Mantle/core cancellation path; changing a GitHub check
is not proof that a remote build stopped. Roll back by disabling required-check
status and then publication while retaining durable reservations; deleting
reservations can cause duplicate creates.

A successful Check Run records observed orchestration/Mantle status only. It
does not prove build correctness, hermeticity, host isolation, identity beyond
Basalt's checked claims, deployment, or release eligibility.

## Native private Mantle API boundary

`src/github_ci/mantle.rs` consumes the admitted producer's
`mantle-build-contract::async_api` contract. It is **not** an HTTP Build API and
does not provision a public Nix gateway. GitHub's identity and Check Runs edges
are HTTPS; the Mantle edge is a private Unix-domain socket, with a four-byte
big-endian frame length followed by a bounded JSON frame. The composition root
must supply the reviewed absolute socket path, exact peer UID, service DID,
project, holder/owner DID, producer-revision identity and exact signing-key
name plus 32-byte public key. Neither an event-supplied key nor a socket path
alone establishes trust. Connection establishment is bounded to five seconds,
the complete operation to twenty seconds, and a frame to 1,048,576 bytes.
Producer event pages are at most 32 entries and log pages at most 32,768 bytes.

The signed request/response and observed-event contract comes from the exact
native Mantle snapshot recorded in `vendor-cargo-deps/mantle/SOURCE`; Basalt's
holder delivery and matching canonical-request contract come from its own
`SOURCE`. Canonicalization does not grant an operation: the reviewed Nickel
policy still admits each requested capability explicitly. Session persistence
propagates a failed immediate-durability selection instead of continuing with
an unobserved durability state. These local imports do not establish a live
GitHub, OIDC issuer, private build deployment, or published producer cohort.

Outgoing bearer envelopes first undergo a bounded byte-counting serialization
without retaining output. Actual encoding then writes into one fixed-size
zeroizing frame; the encoded JSON size, not only the raw derivation field size,
must fit before socket I/O. The bearer-containing frame never grows through
reallocation. This is not an allocator-wide or third-party decoder-scratch
erasure guarantee.

Current native APIs are:

- `MantleClient::new(Endpoint)` validates explicit transport/trust configuration.
- `PlannedBuild::new(endpoint, job, label, drv_path, derivation_json,
  build_time_limit_secs)` binds an already-admitted concrete public job to the
  producer request. The approved derivation's **exact serialized bytes** must
  hash to the semantic request's derivation identity. Lattice does not replace
  these bytes with a re-evaluated expression, an ambient command or reformatted
  JSON. Identical parsed JSON is not permission to substitute different bytes.
- `PlannedBuild::request()` supplies the producer-native request;
  `attempt_request` constructs the explicit status/events/log/result/cancel
  operation. Resource and invocation digest come from the shared producer
  functions, not a consumer-invented parallel identity convention.
- `MantleClient::execute` takes the exact request, transient `ExchangedUcan`,
  explicit recipient-holder signer and a fresh nonzero nonce from the host's
  entropy port. The signed invocation binds resource, ability, canonical request
  digest and nonce. The receiver still owns authoritative UCAN verification,
  replay state, current revocation facts and dispatch policy. Socket peer-UID
  equality is an additional check, not a replacement for that authority.
- `bind_submission` authenticates the signed latest event before creating a
  `NativeAttempt`. `verify_event` authenticates event facts for that exact
  attempt; `signed_status` requires the response status to equal its signed
  latest-event status. The core, not the wire decoder, owns monotonic sequence,
  duplicate/gap decisions and terminal immutability.
- `verify_result` authenticates the result's independently signed bytes and
  exact digest named by the admitted successful event; the result's public
  schema/service/project/owner/attempt/fence/request-hash binding must match the
  configured endpoint and native attempt. It does not render private result
  payloads into Check Runs or diagnostics.

`NativeAttempt` preserves **two different request identities**: the semantic
Lattice `binding.request` and the producer-native canonical
`native_request_hash`. The former identifies admitted source/derivation/input
facts. The latter also binds native service/project/operation and concrete
submission fields, including label, path, exact derivation bytes, idempotency
key and execution-time limit. Do not substitute one hash for the other during
restart. Persist the producer-issued attempt/fence, both identities and public
binding; rebuild the concrete request only from the same reviewed public inputs.
Never persist its bearer authorization or invocation nonce.

The producer signs `async_api::event_digest` with Ed25519. Its event/result
public-key and signature fields use the real Nix named-key encoding
`name:STANDARD-base64`, not an invented hexadecimal signature envelope. Pin the
configured name and decoded bytes. Fresh observations reject zero sequence or
fence, wrong schema/service/project/owner/attempt/request hash, unexpected
producer, future timestamps beyond 30 seconds and age beyond 900,000 ms. Signed
historical status recovery may authenticate an older event without claiming it
is a fresh observation. Success must name a result digest; non-success phases
must not attach a successful result.

The private service must already be deployed/configured by its owning operator
with a real producer, authorized concrete source inputs, replay/signing state,
accepted output keys and store/import authority. Lattice does not acquire these
prerequisites by selecting a socket or possessing an OIDC token. Publication
authorization to existing canonical source remotes does not grant permission
to deploy services, open network listeners, add remotes, change visibility or
delegate new authority. A private GitHub repository and report-only rollout
remain externally provisioned stages; offline tests do not establish they ran.

`tests/test_github_ci_mantle.rs` uses the shared producer digest functions and
actual `basalt::ucan::Ed25519SigningKey` signing for positive/negative binding,
signature, key-name, sequence, timestamp and result fixtures. It does not stand
up a fake production endpoint, execute a derivation, import an output or claim
live private-repository integration. Its coordinated verification command is
`nix develop -c cargo test --test test_github_ci_mantle`.

## Native session composition and recovery

The explicit host composition is `RecoveryStore::new(&database, max_records)`
with the actual owner's private redb `Database`,
`InProcessAuthority { oidc, basalt, oidc_audience }` with an `ActionsOidcClient`, a
mutable real `BasaltIssuerHost` and the exact OIDC audience, and
`Session { store, client, policy, rollout }` with the actual admitted
`MantleClient`. `Session::{submit,poll,recover,cancel,discover_result,logs,update_check}`
are the native public operations. This composition does not invent a remote
exchange service or install a production gateway.

Before every native effect, the session durably records only bounded fixed
`ExchangeObservation` values from the actual successful Basalt receipt: public
receipt, policy, subject, requested-capability and granted-capability identities.
None is a digest of an OIDC token, UCAN bearer, credential or provider secret.
No bearer is persisted or reconstructed for recovery; renewed effects acquire
and exchange a current identity again. Whole-second UCAN planning floors the
session's current time; standalone exchange callers must supply bounds
compatible with the producer's whole-second contract.

Pending submission retransmits the exact canonical request and idempotency key.
Recovery independently checks semantic and producer-native request hashes and
authenticates signed status. An expired cursor is reacquired from a bounded
event origin, not trusted as a fresh status. Polling reads at most four pages of
32 events. Logs are at most 32,768 transient bytes and are never automatically
copied into Check Runs. Result discovery checks the real named-key signature,
payload scope and successful event's result digest. `update_check` recomputes
the exact configured core projection and commits a durable reservation before
the mutation; an uncertain mutation remains pending for reconciliation.

## Actual Valence build-service evidence composition

`github_ci::evidence::project(record, producer_bundle,
accepted_producer_policy, policy)` is a pure in-memory function using the
accepted `valence-core::build_service` types, canonical identity functions and
`validate_build_service_bundle` validator. It does not use a local replacement
profile, shell out to a validator, acquire credentials, verify signatures or
authorize effects. The caller supplies actual public producer request, attempt
and authority-decision records; Lattice never creates missing producer truth.
The producer-policy argument admits the exact bundle profile policy, not the
Basalt exchange policy or a signature key.

The function matches the native canonical request hash and idempotency key,
the exact producer attempt and canonical request reference, and every persisted
exchange receipt's exact authority reference, public subject and policy. It
rejects missing or ambiguous links and role elevation on selected request,
attempt and authority records: these may be `Linked` or `RecordedOnly`, not an
inferred `Verified`/`Signed` verdict. Authority operation names must be exact
closed Mantle abilities. A linked operation remains explicitly
`declared_operation`; the composition validator does not turn a producer's
claim into cryptographic proof or grant current authority.

Digest spelling is explicit. Canonical Valence identities, the profile's
`producer_policy`, native request/idempotency identities and authority
`policy_identity` are bare lowercase 64-hex. Basalt subject and verifier receipt
references use exactly `blake3:` followed by that spelling. These boundaries
reject prefixes, uppercase or foreign digests where not admitted; no heuristic
normalization aliases distinct contracts. Workflow generation and the
producer's nonzero attempt generation are retained separately, never inferred
from each other or the fencing token.

Only an acknowledged Check Run at the exact durable state version, with no
pending mutation, contributes an external status. Lattice appends its own
`Linked` orchestration record and `RecordedOnly` GitHub observation and runs
the actual Valence validator over the complete composed bundle. The supplied
header may claim `external-status` supported by that appended observation;
any other claimed class still needs its actual producer records. A successful
event does not cause Lattice to fabricate build outputs, CAS observations,
signature references, transition timestamps, machine choices, usage, billing,
reuse, exports or release readiness. Unsupported classes fail closed.

The separately domain-hashed `lattice.github-ci.valence-linkage.v1` companion
records only fixed public identities, exact observed event sequences/phases,
the successful event's optional native result digest, canonical Valence record
identities and the acknowledged check update. Events and native result digests
have no equivalent field in this accepted profile, so the companion does not
pretend to be another Valence record or a verified `BuildResultRecord`.
It states its recorded-only role and preserves the profile's required
non-claims, including no build correctness, hermeticity, isolation, current
authority, external-status truth or release eligibility.

Each supplied/composed bundle and companion observation document is bounded to
1,048,576 serialized bytes before allocation of a full serialized document;
per-kind producer records are capped at 64. Session events are capped at the
lesser of the admitted policy and 256, and exchanges at the lesser of policy
and 64. These limits supplement, not replace, the real profile's own bounds
and privacy exclusions. Callers must supply reviewed public observations, not
expect the validator to scrub arbitrary secrets. The focused coordinated
verification command is
`nix develop -c cargo test --lib github_ci::evidence::tests`.
Offline fixtures exercise actual profile validation; they do not prove a live
GitHub/Mantle deployment or private/report-only rollout.
