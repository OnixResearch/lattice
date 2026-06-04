# Proposal: Generic command agent runtimes

## Summary

Refactor production agent execution so Pi and Clankers are default command-runtime presets, not hard-coded production adapter branches. Generic agent nodes keep typed workflow semantics, while runtime command, arguments, and prompt placement move into configuration.

## Problem

Production agent execution currently has Pi/Clankers-specific command selection and argument planning in Rust. That couples workflow execution to two tools more than necessary even though the desired boundary is command mode: run an external agent command, decode stdout or a file handoff envelope, and map the result to typed workflow ports.

## Goals

- Replace dedicated `pi_command` and `clankers_command` production config fields with a generic runtime map keyed by target label.
- Express Pi and Clankers as default runtime presets with command and argument templates.
- Keep generic agent node typed ports, cache identity, credentials/session fields, and file handoff behavior.
- Preserve default behavior for existing Pi and Clankers targets.
- Add tests proving command planning no longer has target-specific branches.

## Non-Goals

- Remove generic agent nodes or collapse them into shell nodes.
- Add arbitrary custom target variants to persisted workflow syntax.
- Add a template language beyond simple argument placeholder replacement.
- Change image/artifact handoff envelope semantics.

## Success Criteria

- Default config renders Pi and Clankers command runtime presets.
- Production command planning selects runtime entries from the map and expands `{prompt}`, `{model}`, `{session}`, and `{output_mode}` placeholders.
- Missing runtime entries and empty commands fail with node-scoped diagnostics.
- Existing text/JSON/image/artifact adapter tests pass through generic runtime config.
- `./scripts/check.rs` passes.
