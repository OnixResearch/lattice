# Deployment assets

These files are operator templates, not a supported packaging or orchestration
platform.

- `systemd/lattice-workflow-exchange.service` runs the local Unix-socket
  workflow exchange as a dedicated user.
- [Server deployment](../docs/server-deployment.md) explains the build, profile,
  socket, readiness, upgrade, and non-claim boundaries.
- [Guest workflow exchange over vsock](../docs/workflow-exchange-guest-gateway.md)
  covers running the exchange inside a no-NIC microVM guest. Its tested profile
  template is `tests/fixtures/workflow-exchange-handler/profile-v2-vsock.ncl`.
  The guest has no systemd unit here: the VM launcher (Kiln) starts it, and the
  host unit above keeps `RestrictAddressFamilies` without `AF_VSOCK`.

The service is not internet-facing. It has no TLS, remote authentication, HTTP
listener, or multi-tenant isolation. Workflows run with host authority and are
not sandboxed by the service.
