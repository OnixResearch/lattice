# Finish workflow-execution evidence

## Why

The workflow-execution audit found four partial rows covering the pinned Rats source closure, focused consumer compilation, saved child-workflow execution, real output-port mapping, and parent/child event correlation.

## What changes

Make the Rats snapshot reproducible and source-complete, add a focused consumer compile rail, and replace subworkflow admission placeholders with a fail-closed runner-shell boundary that executes the named saved child, maps declared child outputs onto real parent output ports, and carries bounded parent correlation metadata into child events.

## Non-goals

This does not promote Tracey markers, grant child workflows ambient authority, claim arbitrary n8n parity, guarantee remote exactly-once effects, or treat an unavailable upstream Rats workspace fetch as validated evidence.
