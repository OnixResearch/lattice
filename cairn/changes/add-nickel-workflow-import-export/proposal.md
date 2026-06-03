## Why

Workflows stored only in redb are hard to review, share, version, and seed as examples. Nickel workflow import/export provides typed, reviewable workflow definitions that fit the repository's configuration direction.

## What Changes

- Export saved workflows to typed Nickel files.
- Import Nickel workflow files after schema and graph validation.
- Preserve graph topology and node configuration across round trips.
- Add positive and negative fixtures for malformed workflow files.
