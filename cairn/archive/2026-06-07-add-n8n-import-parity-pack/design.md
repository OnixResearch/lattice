# Design: n8n import parity pack

## Architecture

The parity pack is a fixture and verification layer over the existing n8n JSON import/export core.

```text
n8n JSON fixtures
        │ parse and classify
        ▼
Import compatibility core
        │ workflow IR + mapping report or deterministic diagnostic
        ▼
Golden assertions
        │ supported imports, unsupported rejections, export subset checks
        ▼
Documentation summary
```

Fixtures should live under a dedicated examples or tests fixture tree with clear supported and unsupported group names. Each fixture includes a small README or metadata comment describing the compatibility intent. Fixtures avoid live credentials and external network calls.

## Fixture classes

Supported fixtures cover workflows that should import and validate today. Candidate shapes include webhook intake, schedule-driven HTTP polling, expression-backed field mapping, credential placeholder references, file or CSV-style processing, and simple control-flow when the accepted model supports the equivalent shape.

Unsupported fixtures cover common n8n features that should fail predictably. Candidate shapes include unsupported community nodes, unsupported expression syntax, malformed connection objects, plaintext credential material, unsupported binary payload assumptions, and trigger types without an accepted n8nclone equivalent.

## Mapping reports

Mapping report assertions should be pure comparisons over deterministic report data. They should prove imported node ids, node-kind mappings, connection mappings, credential placeholder handling, unsupported feature diagnostics, and warning categories remain stable. Reports must redact or omit secret material.

## Functional core and shell boundary

Fixture classification, expected outcome selection, and report normalization should be pure helper logic. The shell owns file discovery, reading fixture JSON, invoking import/export commands, and writing golden updates only through explicit developer action.

## Documentation

The docs should describe the tested compatibility subset without overselling full n8n parity. They should show how to run fixture checks locally and how to add new supported or unsupported fixtures with both positive and negative assertions.
