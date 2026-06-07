# Design: Operator doctor command

## Architecture

Add a diagnostic shell around a pure doctor evaluation core.

```text
Doctor shell
        │ config/store/workflow/provider/listener facts
        ▼
Doctor evaluation core
        │ deterministic findings and summary
        ▼
Text or JSON presenter
```

The shell collects facts from the configured filesystem, Nickel parser, workflow store, credential provider registry, trigger store, and listener configuration. The core evaluates those facts into stable findings with identifiers, severities, messages, remediation hints, and redaction guarantees.

## Functional core

The core accepts a `DoctorFacts` value containing already-collected facts such as config presence, parse status, store status, workflow validation summaries, provider reachability summaries, trigger/listener readiness, and example smoke status. It returns a `DoctorReport` with a summary state and ordered findings.

The core performs no filesystem traversal, environment reads, credential resolution, network binding, clock reads, process execution, or stdout/stderr writes. It is covered by plain unit tests for healthy and failing fact combinations.

## Imperative shell

The shell owns CLI parsing, config path resolution, file reads, Nickel evaluation, store opening, credential provider test calls, optional listener bind-readiness probes, and rendering. Provider probes must request only reachability/status facts and must never print resolved secret values. Listener probes must not start a durable listener or accept external requests.

## Output contract

Human output groups findings by severity and prints stable check identifiers. JSON output serializes the same report data with deterministic ordering. Findings include safe messages and optional remediation hints. Fatal findings cause the command to fail after all non-destructive checks complete.

## Checks

Initial checks should cover config discovery, Nickel parse/evaluation, store open and schema compatibility, workflow validation summary, credential provider reachability, trigger/listener readiness, and example/cookbook drift when examples are requested. Checks should be individually testable and independently skippable only through explicit CLI flags or configuration.

## Safety

Doctor output must omit plaintext secrets, authorization headers, raw request payloads, credential provider outputs, and environment variable values. Redaction tests should inject distinctive secret-like values into collected facts and assert they are absent from text and JSON output.
