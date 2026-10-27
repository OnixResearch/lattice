# Add a live Mantle operation-authority boundary

## Why

Mantle's private gateway needs a caller-facing Basalt API that authenticates one
complete operation context rather than accepting persisted evidence or a copied
capability DTO as live authority. Existing document, process and deployment
operations do not imply build, store, project, cancellation or publication grants.

## What changes

Expose the reviewed `basalt::mantle` request, holder invocation, verifier and
opaque session API. Bind every request field, the loaded policy artifact and a
distinct action/ability into the signed resource. Admit only operator-selected
root issuers, verify the actual holder invocation, and consume replay once at
initial admission. Recheck current authority without consuming replay again
before an already-admitted session reaches an effect.

## Scope and non-claims

Basalt owns exact lexical/cryptographic/policy binding and redacted admission
receipts. Mantle owns input normalization, endpoint/account/project selection,
current verification services, persistent replay, cancellation and actual effects.
No default trust discovery, token minting, process execution, store publication,
release eligibility or deployment authority is added. Test keys prove only the
selected local contract, not real operator approval.
