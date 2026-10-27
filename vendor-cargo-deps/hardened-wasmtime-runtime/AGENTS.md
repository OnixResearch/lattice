# Agent Notes

- Keep the component product-neutral. Product DTOs, policy, effects, storage, and runtime adapters stay in consumer repositories.
- Preserve a pure deterministic core and a thin imperative shell.
- Make every authority, compatibility fact, and resource limit explicit.
- Add positive and negative tests together.
- Use named constants for every nontrivial numeric value.
- Do not add durable sibling-path dependencies. Pin external repositories to immutable Git revisions.
- Use native `.cairn/` lifecycle packages and validate every changed package.
- Do not push unless the user explicitly requests it.
