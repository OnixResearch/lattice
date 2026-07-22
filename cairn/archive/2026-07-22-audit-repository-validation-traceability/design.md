# Design: audit repository-validation traceability

## Success contract

Exactly all 11 repository-validation rows MUST receive reviewed classifications while the 315-item missing set remains unchanged. Exact rows require direct maintained evidence for every accepted clause. Partial rows MUST name the unproved observation and MUST NOT be treated as near-complete correctness.

## Portfolio registry

| Family | Mechanism | Expected discriminator |
|---|---|---|
| executable-config | Compare the documented command and evaluated flake check arguments with each accepted clause | Reject configuration presence when the required hostile condition is not observed. |
| closure-observation | Follow compile-time fixtures and vendored path dependencies through the filtered Nix source | Distinguish successful fixture compilation from untested Git-ignore and nested-path admission. |
| adversarial-scope | Seek direct fixtures that would fail under broad formatting, dependency linting, or embedded signing policy | Keep a row partial until the unsafe alternative is rejected. |

These are correlated serial repository passes. Deterministic repository checks remain authoritative.

## Classification boundary

The Nix Nextest target directly compiles a repository-owned test that embeds a tracked JSON fixture from the filtered `tests/` source, so the two test-source rows are exact. The other nine rows are partial: their intended command, source-filter, formatter, and Clippy configuration exists, but no maintained adversarial fixture directly proves missing-key override scope, external signing policy, nested output-name admission, vendor formatting exclusion, or vendor lint exclusion.

## Non-claims

Inventory rows create no Tracey evidence. Successful Nix evaluation or a matching configuration string alone does not prove hostile host policy handling, arbitrary source closure completeness, arbitrary upstream formatting/lint compatibility, release readiness, or production readiness.
