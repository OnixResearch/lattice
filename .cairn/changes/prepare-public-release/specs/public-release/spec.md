# Public Release Specification

## ADDED Requirements

### Requirement: Private source stays outside the public build
r[lattice.public_release.build.private_source] The default workspace, public checks, and release artifact MUST NOT depend on private Tile source. The optional compose integration MUST remain buildable only through an explicit separate workspace with authorized source access.

#### Scenario: Public lock has no private source
r[lattice.public_release.build.private_source.scenario.lock]
- GIVEN the default workspace lockfile and flake lock are resolved
- WHEN the source closure is inspected
- THEN no private Tile repository or authenticated source MUST be required.

#### Scenario: Compose is explicit
r[lattice.public_release.build.private_source.scenario.compose]
- GIVEN an operator with authorized Tile access
- WHEN they build the compose workspace
- THEN the Tile observation shell and example MUST remain available outside the default runtime.

### Requirement: Anonymous public acquisition
r[lattice.public_release.build.anonymous] Every default build and check input MUST be acquirable over anonymous HTTPS or a vendored local path. Transitive SSH-only references MAY be patched to their exact public revisions.

#### Scenario: Clean checkout builds without SSH
r[lattice.public_release.build.anonymous.scenario.clean]
- GIVEN a clean checkout with no SSH key or GitHub write access
- WHEN the default package and checks resolve their inputs
- THEN acquisition MUST use only committed lock data and public routes.

### Requirement: Public package identity
r[lattice.public_release.package.metadata] The released binary MUST expose its package version and help surface, and the root package MUST declare public description, repository, readme, keywords, and categories.

#### Scenario: Version is observable
r[lattice.public_release.package.metadata.scenario.version]
- GIVEN the built binary
- WHEN `lattice --version` runs
- THEN it MUST print the package version from the manifest.

### Requirement: Trust and contribution documentation
r[lattice.public_release.docs.trust] The repository MUST publish a security policy, contribution guide, changelog, and documentation index that state the runtime, authority, and deployment trust boundaries.

#### Scenario: Security report route is explicit
r[lattice.public_release.docs.trust.scenario.security]
- GIVEN a reporter with a vulnerability
- WHEN they read the security policy
- THEN they MUST find the private reporting route and the supported-version boundary.

### Requirement: Local deployment surface
r[lattice.public_release.deploy.local] The repository MUST provide an operator deployment guide and a systemd template for the local Unix-socket workflow exchange. The material MUST state that the service is not internet-facing, multi-tenant, or a workflow sandbox.

#### Scenario: Operator sees the non-claims
r[lattice.public_release.deploy.local.scenario.nonclaims]
- GIVEN an operator following the deployment guide
- WHEN they review the service boundary
- THEN the guide MUST state the lack of TLS, remote authentication, multi-tenant isolation, and sandboxing.

### Requirement: Repeatable release artifact
r[lattice.public_release.artifact] A tagged release workflow MUST run the maintained checks, build the Nix package, verify the version, and publish a binary archive with checksums. The process document MUST define version, tag, failure, and rollback rules.

#### Scenario: Tag produces verified assets
r[lattice.public_release.artifact.scenario.tag]
- GIVEN a `v*` tag on a checked commit
- WHEN the release workflow runs
- THEN it MUST publish the binary archive and checksum file only after the checks and version smoke pass.

### Requirement: Evidence-bounded launch material
r[lattice.public_release.launch] Launch material MUST link implemented capabilities and MUST state explicit non-claims. It MUST NOT assert production readiness, external effect success, sandboxing, or hosted-service availability.

#### Scenario: Launch claim is traceable
r[lattice.public_release.launch.scenario.claim]
- GIVEN a reader of the go-to-market plan
- WHEN they follow a capability claim
- THEN the claim MUST link the maintained documentation or specification that supports it.

### Requirement: Release evidence
r[lattice.public_release.evidence] Release preparation MUST regenerate policy, cover the new surfaces in the release input manifest, and record observed verification separately from unchecked external release actions.

#### Scenario: Evidence remains truthful
r[lattice.public_release.evidence.scenario.truthful]
- GIVEN a check that was not run or an external action that was not authorized
- WHEN the release audit is recorded
- THEN it MUST remain explicitly unrun or pending rather than reported as passed.
