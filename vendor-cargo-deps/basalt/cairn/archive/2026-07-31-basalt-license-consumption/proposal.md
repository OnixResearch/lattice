## Why

Basalt is AGPL-licensed while some downstream consumers, including Molten/Aspen, use different project licenses. That may be acceptable, but adoption should be explicit and evidence-backed rather than implicit through Cargo resolution.

## What Changes

- Add license-consumption receipts for Basalt downstream use.
- Record consumer repo, consumed Basalt version/revision, license boundary, review action, and non-claims.
- Add positive and negative fixtures for acknowledged AGPL consumption, missing review, wrong package identity, and stale license metadata.

## Impact

- Downstream adoption of Basalt is intentional and reviewable.
- Basalt keeps license metadata as part of its crate consumability boundary.
