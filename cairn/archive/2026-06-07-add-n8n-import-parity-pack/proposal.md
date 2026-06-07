# Proposal: n8n import parity pack

## Summary

Add a curated n8n JSON parity fixture pack that covers common supported workflow shapes, unsupported edge cases, deterministic diagnostics, and stable mapping reports.

## Problem

n8n JSON import/export support exists, but confidence depends on narrow fixtures. Users evaluating n8nclone need evidence that common n8n-style workflows import predictably and that unsupported features fail with actionable, stable diagnostics. Without a fixture pack, compatibility can regress silently and the project has less concrete proof of import parity.

## Goals

- Add supported n8n JSON fixtures for common workflow shapes such as webhook intake, scheduled HTTP polling, expression-backed mapping, credential placeholders, and simple branching where supported.
- Add unsupported fixtures for common n8n constructs that n8nclone intentionally rejects.
- Keep fixture assertions deterministic and independent of external services.
- Extend mapping-report checks so imported node ids, connection mappings, credential placeholders, and unsupported features are visible without leaking secrets.
- Document the supported subset and predictable rejection behavior.

## Non-Goals

- Claim complete n8n compatibility.
- Execute arbitrary third-party n8n node packages.
- Import plaintext credential material.
- Add network-dependent fixture tests.

## Success Criteria

- The parity pack imports all supported fixtures and rejects unsupported fixtures with stable diagnostics.
- Mapping reports are golden-tested for supported and unsupported fixture classes.
- Export behavior for the supported subset remains stable where export is implemented.
- Secret placeholders remain opaque in fixtures, reports, diagnostics, and test output.
