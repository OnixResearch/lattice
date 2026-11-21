# Execute one admitted implementation attempt

## Why

Kiln #45 needs an actual candidate-producing consumer of the existing OMP worker, not an unused coordinator envelope.

## What Changes

Add an explicit run/recover CLI over the existing isolated OMP worker, frozen envelope and durable candidate store. Persist dispatch intent before execution and reconcile instead of blindly repeating after controller loss. Keep existing guarded workflows unchanged.

Extend the same envelope with an explicit Cairn artifact source kind without changing existing source-kind identities. Add bounded native candidate export from an already retained produced result for Kiln's independent capture role; export never executes a worker or grants acceptance.

With an optional agent revision store (`manage-agent-revisions`), record each attempt's admission mode and pin in its intent so recovery, inspection and export read the mode back instead of deriving it from the current lifecycle. Without a store the intent bytes are unchanged.

## Impact

Implementation attempt CLI module, main command registration, focused tests and candidate worker documentation. No global work claim, semantic verifier or publisher is added.
