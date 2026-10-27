# SecretSpec credential delivery

Lattice's optional `animus-secretspec` provider adapts opaque workflow references
to the accepted Animus SecretSpec host. It does not implement a SOPS provider,
run an arbitrary credential command, discover manifests, select ambient profiles,
or grant effect authority.

## Reviewed producer

Animus archived `add-secretspec-sops-host-resolution` in commit
`b1ce46d4533c9d80b629ca932d65d947a8720c89`. Its accepted
`credentials.secretspec.provider_conformance` requirement and archived evidence
cover paired environment, systemd credential, guarded SOPS, bounded-worker,
redaction, and feature-isolation fixtures. This archive is an ancestor of the
immutable `origin/main` revision
`51d2683776ab73d4fd36d3d9e6ced2beadf9793d` inspected for this adapter.
Lattice imports this exact revision through the repository-owned script. The
generated `vendor-cargo-deps/animus/source.ncl` records its reviewed archive
BLAKE3 identity, tar format and `0000` umask, exact dependency revisions,
separate SDK/host licenses, and the disabled-defaults/`sops` feature selection.
This local immutable import is not fresh anonymous acquisition or integrated
runtime acceptance; a sibling checkout is not a runtime dependency. Reproduce
the import only through:

```sh
scripts/sync_animus.rs /absolute/path/to/animus vendor-cargo-deps/animus
```

The sync script's reviewed exact revision and required-crate set must include
`animus-std-secretspec`. Do not edit a vendored snapshot directly. The producer
pins SecretSpec `v0.17.0`, commit
`a8794e46ec9664a0e1a3869cc3105d0853937e48`, Apache-2.0, with defaults disabled
and only `sops` enabled. Environment and systemd credential providers are
unconditional SDK library providers. Its bounded-exec cohort is
`29dac88ecded94457572db3fdfaaaab95fa91525`, AGPL-3.0-or-later.
The optional host itself is AGPL-3.0-or-later. These dependencies must not be
added to pure `no_std` cores or enabled implicitly for an unconfigured provider.

The reviewed Animus archive BLAKE3 is
`5f59a2b900095aec757bad5b5542d844e92eac2a0bd818e110d70927611ddb46`,
computed with `git -c tar.umask=0000 archive --format=tar` at the revision above.
When changing that revision, review and update its archive identity together.
The integrated lock uses `proc-macro-crate 2.0.0` within Bolero's `^2`
requirement: `2.0.2` fixes `toml_datetime` to `0.6.3`, which conflicts with the
imported session crate's `toml 0.8.23` requirement for `toml_datetime ^0.6.11`.
This resolver change does not change the Animus or SecretSpec revisions.

Native dependency resolution excludes the optional Animus host with the feature
off and selects only `sops` on its exact Git SDK when enabled. A separate registry
SDK is present through Basalt with no optional SDK features; this is not an
SDK-free default build. The four reusable workspace cores have no normal
dependency path to either SDK or the host. These graph observations do not prove
provider execution, cancellation cleanup or whole-consumer compilation.

## Public configuration and private deployment

All provider/host DTOs and pure validation have one always-compiled canonical
home, `lattice::credentials::config`, including `SecretSpecProviderConfig`,
`SecretSpecHostConfig`, `DeclarationMapping`, `ProviderScheme`, `RequestBounds`,
`HostInputConfig`, `SopsCredentialVariable`, `SopsCredentialKind`, and
`SecretSpecDiagnostic`. Only adaptation, host execution and `build_provider`
live behind the optional `credentials::secretspec` module. A build without the
feature still parses these exact closed DTOs and rejects nonempty selected
configuration explicitly; it does not ignore it or substitute an untyped schema.


One provider id identifies one reviewed manifest/profile/scope/policy tuple.
`SecretSpecProviderConfig` is a closed, value-free public record:

- `id`, `cohort` (exactly `0.17.0`), `manifest_identity`, and `policy_identity`;
- explicit `profile`, `scope`, and safe public `reason`;
- `declarations`, each mapping an opaque workflow `key` and optional `field` to
  one exact SecretSpec `declaration` and provider `scheme`;
- nonempty unique `allowed_schemes`: `environment`, `systemd-credential`, `sops`;
- positive `bounds`: `input_max_bytes`, `value_max_bytes`, `stdout_max_bytes`,
  `stderr_max_bytes`, `timeout_ms`, `poll_interval_ms`, `teardown_timeout_ms`.

Field selection is an explicit declaration mapping, not JSON extraction from a
resolved value. Duplicate mappings, ambiguous provider ids, empty identities,
unknown fields, fallback settings, plaintext values, provider credentials,
unsupported cohorts, unsafe metadata and zero/excessive bounds are rejected.
The one-value count is fixed. Labels are bounded ASCII identifiers; BLAKE3
identities are exactly 64 lowercase hexadecimal characters. Bound ceilings match
the producer: 64 KiB input/stderr, 1 MiB value, 60 s resolution, 1 s polling,
10 s teardown; stdout must equal the value bound plus the 32-byte private frame.

Do not place a manifest path, encrypted payload, worker path, key, provider URI,
decrypted-file setting or private credential in this public record or workflow.
A host injects `BoundedSecretSpecHost` separately, with:

1. the exact bytes of a checked Animus generated SecretSpec policy;
2. absolute `ResolverDeployment` manifest, worker and working-directory paths;
3. one explicit `HostInputSource`.

The host re-admits the request against the exact checked policy. Manifest and
policy BLAKE3 identities, profile, scope, declaration, scheme, reason, destination
and bounds must match that policy. It does not synthesize a permissive policy.
A successful resolution still does not admit a shell/network/filesystem effect.

The standalone operator `credential_hosts` collection uses the closed
`SecretSpecHostConfig` record: `provider_id`, `policy_path`, `manifest_path`,
`worker_executable`, `working_directory`, and tagged `input`. An environment
input has `kind = "environment"` and `variable`; a service input has
`kind = "systemd-credential"` and `directory`; a SOPS input has `kind = "sops"`,
`guard_directory`, `executable` and `credential_variables` records with
`credential` and `variable`. Credential classes are `age-key`,
`aws-secret-access-key`, `azure-client-secret`, `google-oauth-access-token` and
`vault-token`. These records contain names/coordinates only, never their values.
Provider ids must match one explicit host; duplicate or orphan hosts are invalid.
`build_provider(config, &host, cancellation)` validates shape, bounds policy and
manifest reads to 1 MiB each, verifies their identities and admits the exact
policy tuple without reading provider inputs. Deployment paths must be absolute
without parent components. Use trusted operator-owned immutable manifest/policy
files; this does not claim safety against concurrent malicious host mutation.
Resolution caps explicit environment values at the declared value bound and
named provider-credential inputs at 64 KiB, clearing rejected temporary strings.


## Provider setup

Use the producer's typed policy authoring/export procedure. Declare one required
value in an explicit SecretSpec profile and scope, select exactly one provider,
and bind the exact manifest bytes. Do not configure cache/fallback providers.
Install the reviewed `animus-secretspec-worker` as an absolute executable.

- **Environment:** inject `HostInputSource::Environment { variable }`. Lattice
  reads only that explicitly named host variable and supplies it to the private
  worker; there is no whole-environment inheritance or ambient fallback.
- **systemd credentials:** inject an absolute service credential directory.
  Grant the service access using systemd `LoadCredential` or
  `LoadCredentialEncrypted` and pass its actual directory explicitly. Lattice
  does not infer `CREDENTIALS_DIRECTORY`. This does not manage service keys.
- **SOPS:** install `animus-secretspec-sops-guard` as `sops` in a private guard
  directory. Inject that absolute directory, the absolute real SOPS executable,
  and explicitly named provider-credential environment variables. The accepted
  host supports age, AWS secret access key, Azure client secret, Google OAuth
  token, and Vault token input classes. It never inherits ambient `PATH` or
  unrelated provider credentials. The operator owns encrypted inputs, filesystem
  access, provider login, KMS/age/PGP authority and key rotation.

Animus disables SecretSpec audit persistence. Lattice retains only fixed local
outcome classes and public reference metadata, not raw SDK/provider/worker
messages, stdout/stderr, full host paths, decrypted documents or value-derived
hashes. A reachability display says **configured; reachability not probed**;
it never loads a value merely to render provider selection.

## Execution and TUI contract

The executor supplies descriptive `CredentialResolutionContext` facts from its
actual current execution context: exact reference, binding, workflow/node or
workflow/trigger identity, public reason and allowed destination. This record
does not issue authority or bypass Choregraph dispatch authorization. Webhook
credential resolution precedes authentication and does not assert that the
dependent workflow effect is authorized. Context-free resolution is denied.
Node contexts admit shell environment, HTTP header and agent environment delivery;
separate trigger-authentication and trigger-invocation subjects admit only their
matching destination. Wrong
references, subjects, reasons, destinations or unavailable bounded capabilities
fail before host dispatch. `resolve_secret_env_in_context` derives each exact
current-node binding independently; it does not resolve the rest of the workflow.

Agent planning seals the exact request, selected backend/configuration, opaque
references and resolver policy without obtaining material. Exact and product
execution resolve only after current `lattice-agent` authority succeeds. Denial,
unavailable authority, stale request/policy/backend identity and one-shot reuse
stop before provider or transport calls. The explicit trusted compatibility
path is separate; descriptive contexts do not make it product-authorized.

The injected host returns ephemeral material only. `SecretValue` has redacted
Debug output and zeroizes its owned string when dropped. `ResolvedSecretEnv`
unconditionally clears both its environment values and redaction copies on scope
exit, including partial resolution, cancellation, errors and unwinding. Transfers
into process requests, HTTP headers or Agent `ExecutionContext` move cleanup
responsibility into another drop guard; displaced environment/header values are
cleared before replacement. These owned buffers must never be serialized into
plans, caches, events, history, replay or stores. This does not establish erasure
of copies in transport libraries, child processes, allocators or the OS.
TUI edits must retain opaque references and preserve prior values on rejection.
Deleting a node must use the workflow lifecycle mutation, removing its refs.

## Verification and production smoke

Run from the Lattice repository with the admitted optional dependencies:

```sh
cargo test --locked --lib credentials::config
cargo test --locked --lib credentials::execution
cargo test --locked --lib engine::effect::runtime::credential
cargo test --locked --lib agent_node::authority
cargo test --locked --features animus-secretspec --test test_secretspec_credentials
cargo test --locked --test test_credentials --test test_provider_credential_boundaries --test test_webhook_credential_boundaries
```

The focused fixtures exercise all three schemes through an injected finite host,
optional field mapping, current node/trigger bindings, capability and policy
rejection, no context-free fallback, cancellation, oversized/invalid values,
fixed outcomes, redacted material and probes that never resolve secrets.
They do not replace real producer conformance or live operator evidence.

For real production execution, install the exact admitted producer worker (and
SOPS guard when selected), provision the actual provider input, and prepare
public `CONFIG_JSON` matching the exact checked `POLICY_JSON` and `MANIFEST`.
Supply an explicit smoke-only expected value via a secure process environment,
not an argv value, workflow, history entry or checked-in file. The example
compares material without printing or hashing it:

```sh
cargo run --locked --features animus-secretspec --example secretspec_delivery -- \
  "$CONFIG_JSON" "$POLICY_JSON" "$MANIFEST" "$WORKER" "$WORKING_DIRECTORY" "$SOURCE"
```

Here `SOURCE` is the explicitly named environment variable or an absolute systemd
credential directory. For SOPS it is the absolute real SOPS executable; append
the absolute private guard directory. The SOPS smoke explicitly reads the age
provider input from `LATTICE_SMOKE_SOPS_AGE_KEY`. All cases require the explicit
smoke-only `LATTICE_SMOKE_EXPECTED_VALUE` variable. Do not print either variable
or store their values in lifecycle evidence. Success prints only
`credential_secretspec_delivered; material omitted`; failures print fixed codes.
Repeat with a missing declaration, denied scheme, missing provider input or
executable, wrong manifest identity and exceeded bounds; no dependent effect
should execute. Run the producer's exact
`scripts/cairn/check-secretspec-host-resolution --root .` from its owning repo
for worker timeout/cancellation/overflow/cleanup fixtures.

These commands are instructions, not a claim that current integration checks or
live provider smoke have run. Required operator prerequisites are the admitted
worker/guard, a matching manifest and checked policy, explicit deployment
coordinates, provider-owned credentials/files/keys, and effect authority for any
actual dependent workflow effect.

## Migration, rollback and non-claims

Existing environment, local-vault and command providers remain valid. Opt in
explicitly; do not rewrite stored references to plaintext. Migrate a binding by
adding its reviewed declaration mapping, checking the provider policy and reason,
then selecting its new provider id. Keep the prior references until validation
succeeds. A disabled build or absent provider must not access SecretSpec inputs.

To rotate, change only the provider-owned value where possible; retain the
reviewed declaration identity and restart the one-shot host flow. For rollback,
stop new deliveries, restore the previous manifest and checked policy together,
restore references and the admitted binary cohort, and re-run positive/negative
smoke without retaining decrypted files or old values as evidence.

Fixture success does not establish upstream cryptographic correctness, global
authorization, value correctness/freshness, arbitrary confidentiality, audit
completeness, provider truth, external exactly-once effects, release eligibility,
or production readiness. Host-memory confidentiality and hostile-host resistance
are not claimed.
