# Design: Generic command agent runtimes

## Overview

The production adapter becomes a generic command runner. It receives an `AgentRequest`, looks up a runtime by `agent_target_label(request.target)`, renders configured arguments, runs the command, and decodes output using the existing text/JSON/file-handoff contract.

## Configuration shape

```nickel
agent_runtime = {
  max_output_bytes = 64 * 1024 * 1024,
  runtimes = {
    pi = {
      command = "pi",
      args = ["--no-session", "-p", "{prompt}"],
    },
    clankers = {
      command = "clankers",
      args = ["-p", "{prompt}"],
    },
  },
}
```

The keys remain `pi` and `clankers` because workflow syntax still has those target variants. The production adapter no longer has Pi- or Clankers-specific command branches; adding or changing command-mode behavior is a config change.

## Placeholder rendering

Arguments support literal string replacement for:

- `{prompt}`: rendered prompt with typed input summaries;
- `{model}`: optional model hint or empty string;
- `{session}`: optional session label or empty string;
- `{output_mode}`: `text`, `json`, `image`, or `artifact`.

Empty rendered arguments are omitted. Defaults avoid optional placeholders so no target-specific conditional logic is needed.

## Safety

- Empty commands fail before spawning.
- Missing runtime entries fail before spawning.
- Output size and handoff path safety remain unchanged.
- Text/JSON stdout and image/artifact file handoff behavior remain adapter-neutral.

## Tests

Add/adjust tests for:

- default runtime map contains Pi and Clankers presets;
- command planning uses runtime-map args and placeholder replacement;
- missing runtime entry and empty command are rejected;
- existing production text/JSON/image/artifact command tests pass without target-specific fields;
- headless runtime config uses the generic map.
