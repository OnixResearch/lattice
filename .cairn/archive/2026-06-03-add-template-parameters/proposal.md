## Why

Node templates are useful, but authors need to fill in values like patterns, URLs, paths, and labels when creating a node. Parameterized templates make reusable templates practical without editing raw commands after insertion.

## What Changes

- Extend Nickel node templates with typed parameter declarations.
- Prompt for required and optional parameters in the add-node flow.
- Render commands and node configuration from validated parameter values.
- Reject missing or invalid parameters before creating a node.
