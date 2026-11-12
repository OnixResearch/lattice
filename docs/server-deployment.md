# Workflow exchange server deployment

Lattice can serve the published workflow exchange contract as a local Linux
Unix-socket service. This guide covers a single-operator deployment under
systemd. It does not describe a hosted, multi-tenant, or internet-facing service.

## What the service is

`/usr/local/bin/lattice workflow-exchange serve` composes three layers:

1. the versioned `lattice-workflow-transport` envelope and peer admission;
2. the application handler that maps exact consumer graphs to saved workflow
   revisions;
3. the normal durable store, effect authority, and headless execution runtime.

The service accepts a finite number of connections, checks the peer Unix user ID
before decoding a request, stores acceptance before starting work, and retains
bounded terminal diagnostics. See
[Durable workflow exchange application](workflow-exchange-application.md) and
[Workflow exchange transport](workflow-exchange-transport.md) for the exact
protocol and lifecycle rules.

The service is **not** a sandbox, a TLS endpoint, an HTTP server, a remote
authentication boundary, or a multi-tenant scheduler. Run it only for local
consumers that you trust as much as the service user.

## Build and install

Build the release package from the repository root:

```sh
nix build .#lattice
./result/bin/lattice --version
sudo install -m 0755 ./result/bin/lattice /usr/local/bin/lattice
```

The default package has no Tile dependency and no authenticated source
requirement. The optional compose workspace is separate and is not needed for
the server.

Create the service account and directories:

```sh
sudo useradd --system --home /var/lib/lattice --shell /usr/sbin/nologin lattice
sudo install -d -o root -g lattice -m 0750 /etc/lattice
sudo install -d -o lattice -g lattice -m 0700 /var/lib/lattice
sudo install -d -o lattice -g lattice -m 0750 /run/lattice
```

## Configuration

Create `/etc/lattice/config.ncl`:

```nickel
{
  data_dir = "/var/lib/lattice",
  default_timeout_secs = 60,
  shell = "sh",
  # Select only an absolute, operator-owned authority profile. Omitting this
  # field keeps effectful nodes fail-closed.
  effect_authority = "/etc/lattice/effect-authority.ncl",
}
```

The authority profile binds exact workflow revisions, machines, policies, and
grants with a finite expiry. Inspect a saved workflow before authoring it:

```sh
sudo -u lattice /usr/local/bin/lattice --config /etc/lattice/config.ncl \
  authority inspect <workflow>
```

The profile cannot grant path roots by itself. File effects also require
operator-configured roots and a matching node selection. See
[Choregraph effect authority](choregraph-effect-authority.md).

## Handler profile

Install the typed contracts beside the profile:

```sh
sudo install -m 0644 \
  contracts/workflow-exchange-handler-profile.ncl \
  contracts/workflow-transport-profile.ncl \
  /etc/lattice/
```

Create `/etc/lattice/workflow-exchange-handler.ncl` from the typed contract
[`contracts/workflow-exchange-handler-profile.ncl`](../contracts/workflow-exchange-handler-profile.ncl).
Replace every `b3:<...>` placeholder with the exact identity supplied by the
consumer and saved workflow; the profile intentionally fails typechecking until
then.

```nickel
let contracts = import "/etc/lattice/workflow-exchange-handler-profile.ncl" in
{
  schema = "lattice.workflow-exchange-handler-profile.v1",
  socket_path = "/run/lattice/workflow-exchange.sock",
  transport = {
    schema = "lattice.workflow-transport-profile.v1",
    limits = {
      maximum_frame_bytes = 1048576,
      maximum_log_bytes = 262144,
      maximum_connections = 1024,
      read_deadline_milliseconds = 5000,
      write_deadline_milliseconds = 5000,
    },
    peers = {
      # Replace with the exact consumer user ID. Do not use a broad range.
      allowed_user_ids = [1001],
    },
    non_claims = [
      "transport admission does not prove workflow correctness",
      "transport peer identity does not prove peer intent",
      "transport acceptance does not prove an operation executed",
      "transport response does not prove an external effect succeeded",
      "transport checks do not establish production deployment",
    ],
  },
  maximum_active_executions = 16,
  controls = { cancel = true },
  routes = [
    {
      profile_identity = "b3:<consumer profile identity>",
      graph_identity = "b3:<consumer graph identity>",
      workflow_name = "daily-digest",
      workflow_revision = "b3:<saved workflow revision>",
      effect_authority = "route_exact",
      source_binding = "unavailable",
      nodes = [
        {
          request_node_id = "collect",
          operation_identity = "b3:<consumer operation identity>",
          workflow_node_id = 0,
        },
      ],
    },
  ],
  non_claims = [
    "handler admission does not prove workflow correctness",
    "route authority does not prove host confinement",
    "execution observation does not prove an external effect succeeded",
    "peer identity does not prove peer intent",
    "handler checks do not establish release readiness",
  ],
} | contracts.HandlerProfile
```

Keep the copied contracts in `/etc/lattice/` so the relative import path is
stable. Routes must be sorted by profile and graph identity, and route nodes by
consumer node ID. A changed workflow or consumer graph requires a new route.

Validate the profile before starting the service:

```sh
nickel typecheck /etc/lattice/workflow-exchange-handler.ncl
```

`pause` and `resume` controls are rejected. A cancel-only profile, including one
that omits `controls`, is valid.

## Install the unit

```sh
sudo install -m 0644 deploy/systemd/lattice-workflow-exchange.service \
  /etc/systemd/system/lattice-workflow-exchange.service
sudo systemctl daemon-reload
sudo systemctl enable --now lattice-workflow-exchange.service
```

The unit sets `RuntimeDirectory=lattice`, so systemd creates `/run/lattice` with
mode `0750`. The socket is created with group access, and the store directory is
mode `0700`, so group access reaches the socket but not durable state. Add only
the exact consumer users to the `lattice` group:

```sh
sudo usermod -aG lattice <consumer-user>
```

The transport still checks each peer user ID against `allowed_user_ids`; group
membership is not authorization by itself.

If workflows must write files, add the exact configured roots to
`ReadWritePaths=` in a drop-in. Do not grant broad paths. The service also needs
network access for HTTP or agent effects; remove `AF_INET`/`AF_INET6` from
`RestrictAddressFamilies=` if the deployment must be network-free.

## Readiness and logs

There is no HTTP health endpoint. Check readiness with:

```sh
systemctl status lattice-workflow-exchange.service
test -S /run/lattice/workflow-exchange.sock
journalctl -u lattice-workflow-exchange.service --since today
```

The service records bounded terminal diagnostic chunks only. It does not retain
workflow outputs, commands, credentials, environment values, or provider
payloads. Runtime file logs are written to
`/var/lib/lattice/lattice.log` with owner-only permissions on Unix. Use
`--verbose` for debug-level records during diagnosis.

`lattice doctor --config /etc/lattice/config.ncl` checks local configuration and
store readiness. It does not probe the socket or prove a consumer can complete an
exchange. Run it with read access to the selected data directory, for example
`sudo -u lattice /usr/local/bin/lattice doctor --config /etc/lattice/config.ncl`.

## Upgrade and rollback

1. Stop the service and back up `/var/lib/lattice`.
2. Install the new binary atomically (`install` to a temporary name, then rename).
3. Run `lattice --version` and `lattice doctor --config ...`.
4. Re-run `authority inspect` for every routed workflow and review changed
   identities before replacing authority or handler profiles.
5. Start the service and verify the socket plus one admitted consumer exchange.

A new binary does not automatically migrate durable state, and a changed
workflow revision does not reuse an old route. If the new revision cannot read
the store or the profiles, roll back the binary and restore the matching data
directory backup. Do not infer success from a process restart alone.

## Non-claims

This deployment proves checked local route, durability, lifecycle, peer, and
bounded observation facts only. It does not prove workflow correctness, host
confinement, external effect success, credential freshness, peer intent,
multi-tenant isolation, remote cancellation, or release readiness.
