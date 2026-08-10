## Why

Shell nodes can call HTTP tools indirectly, but workflow authors need a first-class HTTP node with typed inputs and outputs. This makes common API automation visible, testable, and portable.

## What Changes

- Add an HTTP request node configuration and execution path.
- Output status, headers, and body as graph values.
- Use an HTTP client interface so network effects stay outside pure planning logic.
- Include request configuration in cache keys and validation diagnostics.
