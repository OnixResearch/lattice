# Complete package metadata

## Why

Downstream users and registries need stable crate metadata to identify source, docs, license, package purpose, and discoverability. Cargo currently warns that key package metadata is missing.

## What Changes

- Add repository, documentation/homepage where available, README, keywords, and categories to package metadata.
- Ensure the metadata is compatible with the retained AGPL license and actual crate scope.
- Verify metadata through Cargo package/metadata checks.

## Scope

In scope: Cargo package metadata and README alignment. Out of scope: license change, crate rename, or publishing credentials.

## Verification

Validate this planning package with Cairn proposal/design/tasks gates. Implementation must run the focused checks named in tasks plus the repo release rail before archive.
