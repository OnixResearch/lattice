# Serving Binding Specification

## ADDED Requirements

### Requirement: Brokered requests leave unattributed evidence
r[lattice.serving_binding.brokered_evidence] For an implementation attempt whose execution binding declares no serving route, the attempt-local model broker MUST retain evidence for every request it relays, with the same write-once discipline as served evidence. Before its worker starts it MUST create the attempt's `model-requests/` directory and make it durable. For each request its own checks admit it MUST durably write one `lattice.brokered-model-request-dispatch.v1` intent before any upstream contact, and MUST refuse the request with nothing sent when the intent cannot be retained. After the call ends it MUST write one `lattice.brokered-model-request-observation.v1` terminal record, and MUST fail when the record cannot be retained. Both MUST bind the binding identity, attempt, fence, route name, the broker's request index and a request id derived from the binding identity and that index. The record MUST carry a `completed` or `uncertain` outcome and only provider-reported token counters, and MUST NOT name any service, endpoint, deployment, grant, replica or resource consumer. The worker MUST receive each response exactly as without the evidence. A binding with a serving declaration MUST keep its served-route evidence unchanged.

#### Scenario: Measured without attribution
r[lattice.serving_binding.brokered_evidence.scenario.unattributed]
- GIVEN an attempt without a serving declaration whose broker relayed requests
- WHEN its usage is summarized
- THEN its model-request coverage MUST be `brokered_unattributed`, counting every dispatched, unconfirmed, uncertain and unknown-usage request and the known provider-reported tokens with every request in index order, and naming no service
- AND an attempt without a serving declaration and without a retained `model-requests/` directory MUST stay `unknown`, never zero

#### Scenario: Only complete evidence is an exact count
r[lattice.serving_binding.brokered_evidence.scenario.charged]
- GIVEN retained brokered or served request evidence
- WHEN the number of dispatched requests is read for the whole attempt, or for one broker run's request-index range
- THEN it MUST be exact only when every intent in scope has a terminal record and none ended `uncertain`; otherwise, and for `unknown` or `no_model` coverage, there MUST be no exact count
- AND served evidence MUST keep its per-service attribution and MUST give no count for an index range; only the `model-requests/` directory the broker pre-created and synced before it ran MAY prove a count, so a retained empty directory MUST count zero requests while a missing one MUST be `unknown`, never zero, for served and brokered attempts alike

#### Scenario: Conflicting evidence fails closed
r[lattice.serving_binding.brokered_evidence.scenario.conflict]
- GIVEN a second write of a brokered intent or record with different content, a record without its intent, a record whose request id, index, binding, attempt, fence or route crosses its intent or binding, brokered evidence under a served or no-model binding, or an unexpected file
- WHEN the evidence is written or summarized
- THEN the write MUST be refused and the summary MUST be refused, so no attempt usage is persisted and no draw is charged from it; identical duplicate files read back on replay MUST count once
- AND a broker admission for a request index at or below one its route already admitted, or whose brokered intent already exists, identical bytes included, MUST be refused before any ticket or upstream contact, so each index reaches upstream at most once and distinct indices count separately
- AND a served dispatch whose intent already exists, identical bytes included, MUST be refused before any upstream contact, and a serving session MUST number each route's requests after the highest ordinal the retained intents and records hold, so a session restarted for the same binding and worker incarnation never reuses an ordinal and distinct contacts count separately
