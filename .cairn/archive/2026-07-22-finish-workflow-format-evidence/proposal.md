# Finish workflow-format evidence

## Why

The workflow-format audit found five partial rows covering image-format import admission, malformed control-flow import, opaque credential-reference import, and cookbook drift.

## What changes

Reject statically declared unsupported image formats during validation before store mutation and add direct positive and negative maintained assertions for every identified gap.

## Non-goals

This does not promote Tracey markers, claim arbitrary n8n parity, inspect image bytes during import, or resolve credential values during parsing.
