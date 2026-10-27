# Isolate enforcement dependencies

Animus's pinned Rust 1.95 cannot compile Basalt's unconditional HTTP and durable
state dependencies, which require Rust 1.96. Its adapter needs neither module.

Add explicit OIDC and durable-authority features, enabled by default to preserve
existing default behavior. Explicit no-default consumers retain enforcement without
those dependencies. Qualify the published UCAN isolation snapshot using the
maintained importer. Keep third-party revisions, cryptography and default authority
behavior unchanged. Qualify the minimal profile on Rust 1.95 and full defaults on
the current owner toolchain. No consumer toolchain upgrade or MSRV bypass.
