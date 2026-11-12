# Security policy

Lattice executes operator-authored workflows, including shell commands, file
effects, HTTP requests, and agent adapters. A workflow runs with the authority
of its configured host environment. Lattice is not a sandbox for untrusted
workflows, and the workflow exchange service is a local Linux Unix-socket
service, not an internet-facing multi-tenant server.

## Supported versions

Security fixes are applied to `main` and to the latest `0.1.x` release. Older
pre-release snapshots and unreleased branches are not supported. No response or
remediation SLA is offered for this pre-1.0 project.

## Reporting a vulnerability

Use GitHub private vulnerability reporting:

<https://github.com/OnixResearch/lattice/security/advisories/new>

If that route is unavailable, open a minimal public issue that asks the
maintainers for a private channel and contains no vulnerability details. Do not
open a public issue for an exploitable defect. Include:

- the affected commit or release;
- the relevant configuration and handler profile with secrets removed;
- a minimal reproduction;
- the observed impact and any known preconditions;
- whether the issue requires local access, a specific peer user ID, or a
  configured effect authority.

We will acknowledge the report, reproduce it against the affected revision, and
coordinate disclosure after a fix or mitigation is available. Please do not
publish details before that coordination.

## Security boundaries

- **Effect authority is explicit.** Shell, HTTP, file, and agent effects fail
  closed without an operator profile. Granting authority does not deliver
  credentials.
- **Workflow files are trusted input.** Import, validation, and replay checks do
  not make an untrusted workflow safe to execute.
- **The exchange socket is local.** Peer credentials are checked against an
  explicit Unix user-ID allowlist before request decoding. The transport has no
  TLS, remote authentication, or network listener.
- **The runtime is not a sandbox.** Bounded execution, file roots, and adapter
  limits reduce specific failure modes; they do not confine arbitrary shell
  commands or provider behavior.
- **Durable uncertainty is preserved.** A lost observation is not proof of
  failure. Automatic retries can duplicate an unobserved external effect.
- **Imported source keeps its own terms.** Vendored snapshots and exact old
  dependency pins retain their source-specific licenses and non-claims.

## Deployment guidance

Read [server deployment](docs/server-deployment.md) before exposing the workflow
exchange service. Keep the socket under an owner-only runtime directory, run the
service as a dedicated user, and grant only the peer user IDs that must submit
work. Do not place a network reverse proxy in front of the Unix socket and
assume it adds authentication; it does not change the service's trust model.
