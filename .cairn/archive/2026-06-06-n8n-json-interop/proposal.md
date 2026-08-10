# Proposal: n8n JSON interoperability

## Summary

Add import and export support for a documented subset of n8n workflow JSON. The feature should convert supported n8n nodes into n8nclone typed workflows, produce deterministic diagnostics for unsupported features, and avoid importing plaintext credential material.

## Problem

n8nclone uses typed Rust/Nickel workflow data, while many prospective users have existing n8n workflows in JSON. Without an interoperability layer, users must manually recreate graph topology, node settings, and credentials. A safe importer needs to be explicit about supported nodes, degraded mappings, unsupported expressions, and credential placeholders.

## Goals

- Parse n8n JSON into a pure intermediate import plan.
- Map supported n8n node kinds to typed n8nclone nodes or templates.
- Preserve graph topology, node labels, positions, and supported configuration fields.
- Emit deterministic mapping reports for unsupported nodes, expressions, credentials, and options.
- Export supported n8nclone workflows to n8n-compatible JSON where semantics are known.
- Keep credentials as references/placeholders, never plaintext imports.

## Non-Goals

- Claim full n8n compatibility for every community node.
- Execute n8n JavaScript expressions directly.
- Import encrypted n8n credential blobs as usable plaintext secrets.
- Replace typed Nickel as the native workflow format.

## Success Criteria

- Supported n8n JSON fixtures import into runnable n8nclone workflows with expected topology and node config.
- Unsupported fixtures fail or import as disabled placeholders according to explicit policy with stable diagnostics.
- Round-trip export for the supported subset produces deterministic n8n JSON and mapping reports.
- Existing Nickel import/export remains unchanged.
