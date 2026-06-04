## Context

The workflow engine already has content-addressed values, typed ports, file nodes, HTTP nodes, credential references, Nickel import/export, and a TUI graph editor. Image support should extend those seams instead of routing opaque strings through shell nodes. The reference workflow needs local/user-supplied images, multiple inspiration image inputs, prompt text, agent-runtime target settings, output settings, image previews, and recoverable errors when assets or adapters fail.

Pi provides programmatic agent access through SDK sessions plus process-oriented print/RPC modes, including image attachments. Clankers provides Rust-native agent surfaces such as one-shot prompts, daemon sessions, MCP/ACP control, context image references, and image-generation tooling. n8nclone should model both as adapter targets behind one generic agent node rather than creating separate prompt, AI, LLM, or provider-specific nodes.

## Decisions

### 1. Images are first-class typed values

**Choice:** Add an image value shape and image port type that carry bytes plus media type, content hash, and optional metadata such as dimensions, source label, and adapter metadata.

**Rationale:** Port compatibility, cache identity, summaries, and previews can reason about images without decoding them repeatedly or treating binary data as text.

### 2. Image loading is planned in pure core and performed by interfaces

**Choice:** Convert image input node configuration into deterministic load/validate plans in core; execute filesystem reads, byte sniffing, metadata extraction, and decode checks through shell-owned interfaces.

**Rationale:** The core remains testable with in-memory fixtures while the shell owns operating-system and codec behavior.

### 3. Prompt, AI, and LLM work uses one generic agent node

**Choice:** Model any prompt, AI, or LLM-style step as a generic agent node. Its typed config includes an adapter target, prompt source, ordered text/image/JSON inputs, working directory, context references, model hints, tool policy, output mode, credential references, and optional session references.

**Rationale:** Pi and Clankers can both satisfy agent requests, and future runtimes should not require new graph node kinds for every provider or model family.

### 4. Pi and Clankers stay behind adapter traits

**Choice:** Resolve generic agent node config into adapter-neutral requests, then execute them through shell-owned Pi and Clankers adapter interfaces. Pi adapters may use SDK, print, or RPC mode. Clankers adapters may use one-shot prompts, daemon/session control, MCP, ACP, or local plugin/tool surfaces.

**Rationale:** The core can validate and cache requests deterministically while process spawning, SDK lifecycle, credentials, sessions, and network/provider behavior remain outside the functional core.

### 5. Agent cache keys include content and runtime settings

**Choice:** Derive generic agent node cache identity from upstream value hashes, prompt text, adapter target, model/runtime hints, tool policy, context references, output mode, and adapter-relevant configuration.

**Rationale:** Reusing generated outputs is valuable, but prompt, image, target, model, or tool-policy edits must never return stale results.

### 6. TUI previews are bounded summaries

**Choice:** Render image previews as bounded terminal-safe thumbnails when the terminal backend supports it and as metadata summaries otherwise.

**Rationale:** The UI should communicate visual workflow state without dumping raw bytes, overflowing panels, or requiring graphics support for correctness.

### 7. Nickel remains the portable source format

**Choice:** Extend the typed Nickel workflow contract with image input nodes, generic agent node targets, image/text ports, prompt sources, adapter settings, output settings, and example workflows.

**Rationale:** Image and agent workflows must be importable, exportable, reviewable, and runnable headlessly like existing shell, HTTP, and file workflows.

## Boundaries

- Core owns value shapes, port typing, deterministic request planning, validation diagnostics, cache-key inputs, and summary derivation.
- Shell owns filesystem reads, image decoding/sniffing, terminal preview rendering, Pi/Clankers adapter execution, credential resolution, session/process lifecycle, and persistence I/O.
- Adapter-specific code must stay behind traits so tests can use deterministic fake Pi and fake Clankers adapters.
