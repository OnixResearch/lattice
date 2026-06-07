# Design: Webhook trigger cookbook

## Architecture

The cookbook is a documentation and example layer over the existing webhook listener implementation. It should not introduce new runtime behavior unless tests reveal a missing shell seam needed for deterministic smoke coverage.

```text
Nickel cookbook workflow
        │ import/export validation
        ▼
Temporary cookbook store
        │ listener dry-run and in-process request smoke
        ▼
Webhook listener shell
        │ existing planner/admission/response cores
        ▼
Run history and safe diagnostics
```

The workflow example remains declarative Nickel. It defines a small webhook trigger, a minimal workflow body, safe output, and a credential reference for authenticated cases. The example avoids external network dependencies so cookbook smoke tests can run offline.

## Functional core and shell boundary

Reusable assertions for cookbook validation live in pure helper functions that accept parsed workflow metadata, trigger metadata, listener outcomes, and redaction candidates. The shell owns temporary directories, CLI execution, listener startup, request sending, and reading persisted run history.

## Documentation flow

The cookbook documentation should show the shortest operator path:

- initialize or choose a config directory;
- import the Nickel workflow;
- run listener dry-run and inspect loaded trigger status;
- start the listener;
- send one valid request and one invalid request;
- inspect run history and safe rejection output.

The docs should call out credential-reference setup without printing real secrets. Example commands should prefer local loopback and deterministic payloads.

## Smoke coverage

Smoke tests should import the cookbook workflow into a temporary store, run validation, exercise listener dry-run, and route synthetic valid and invalid requests through the listener or its in-process shell. Positive assertions prove a request starts exactly one workflow run and records a safe accepted history entry. Negative assertions prove missing path, invalid auth, malformed body, or duplicate idempotency does not start a run and produces stable safe diagnostics.

## Redaction

Cookbook tests include explicit redaction checks. Authorization headers, bearer values, signature values, raw sensitive JSON fields, and credential provider values must not appear in responses, diagnostics, run history, or documentation snippets.
