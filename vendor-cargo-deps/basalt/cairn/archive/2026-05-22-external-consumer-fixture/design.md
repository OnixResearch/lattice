# Design: external consumer fixture

## Approach

Basalt will use `../clankers/` as the external-style downstream consumer fixture. The fixture must compile outside Basalt's own integration-test namespace and use only public Basalt package APIs, with any sibling override or path wiring made explicit and local-development-only.

## Fixture expectations

- Library-only build does not enable CLI-only dependencies.
- `../clankers/` can depend on Basalt through the public package surface or an explicit sibling path override used only for the fixture.
- Packaged policy source/generated JSON are present where consumer docs say they are.
- Public APIs cover policy parsing, request construction, enforcement, receipt inspection, and DTO serialization.
- Negative cases fail closed for malformed policy, unsupported schema, wrong resource, wrong ability, and missing capability.
- Clankers integration stays narrow: it may add a fixture crate, test target, or adapter smoke, but it must not turn Basalt into a runtime/orchestration engine.

## Verification

The release verifier should run the Clankers consumer fixture with a fresh target dir and `RUSTC_WRAPPER=` hygiene, then record a receipt step so fixture regressions are visible in CI artifacts. If `../clankers/` is absent in a sandbox or release environment, the verifier must record an explicit deferral instead of silently claiming downstream coverage.
