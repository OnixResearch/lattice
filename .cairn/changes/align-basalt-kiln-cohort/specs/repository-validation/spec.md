# Repository validation

## ADDED Requirements

### Requirement: Basalt Kiln authority cohort
r[lattice.repository_validation.basalt_cohort] Lattice MUST vendor Basalt only at an exact producer revision written by the repository sync script. That revision MUST contain Basalt `main` `5e0f1f4cad925a50f7dd5cdbcae05a04e7b92463`, which provides Kiln implementation v2 framing, source v1 operations, the delegate rule, UCAN `fca62a6e` and the ring RS256 OIDC verifier. It MUST also contain the holder delivery API that Lattice's GitHub CI exchange calls. `SOURCE`, the vendor README and `Cargo.lock` MUST agree with the imported tree.

#### Scenario: Divergent or hand-edited import is rejected
r[lattice.repository_validation.basalt_cohort.scenario.provenance]
- GIVEN a proposed refresh whose tree differs from the `git archive` of the recorded `SOURCE` revision, or whose revision drops an API Lattice calls
- WHEN the refresh is reviewed
- THEN it MUST NOT be accepted until the exact producer revision and tree match.

### Requirement: Holder-bound GitHub CI exchange
r[lattice.repository_validation.basalt_cohort.holder_exchange] Lattice's GitHub CI exchange MUST deliver a Basalt grant only to its stable holder, for the exact typed native Mantle request that the holder will sign under the endpoint's policy reference. The request's subject MUST be the holder, its service, project and operation MUST equal the plan's and policy's, and its canonical resource MUST lie under the gateway and policy prefixes. Lattice MUST check this before any producer effect. Lattice MUST then verify that the issued grant names exactly that holder, resource and operation.

#### Scenario: Exact holder request is issued once
r[lattice.repository_validation.basalt_cohort.holder_exchange.scenario.issued]
- GIVEN a valid RS256 OIDC token, admitted policy, and a native request whose subject is the stable holder
- WHEN Lattice runs the real Basalt exchange
- THEN the verified grant MUST name the holder and carry exactly the request's canonical resource and operation
- AND a second exchange with the same OIDC token MUST be denied.

#### Scenario: Mismatched holder request has no producer effect
r[lattice.repository_validation.basalt_cohort.holder_exchange.scenario.rejected]
- GIVEN a native request whose subject, project, service or operation differs from the holder, plan or policy
- WHEN Lattice runs the exchange
- THEN it MUST refuse with a policy failure before fetching any OIDC document or reserving replay.
