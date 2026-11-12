# Deployment assets

These files are operator templates, not a supported packaging or orchestration
platform.

- `systemd/lattice-workflow-exchange.service` runs the local Unix-socket
  workflow exchange as a dedicated user.
- [Server deployment](../docs/server-deployment.md) explains the build, profile,
  socket, readiness, upgrade, and non-claim boundaries.

The service is not internet-facing. It has no TLS, remote authentication, HTTP
listener, or multi-tenant isolation. Workflows run with host authority and are
not sandboxed by the service.
