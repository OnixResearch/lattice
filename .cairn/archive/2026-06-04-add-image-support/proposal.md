## Why

Workflow authors need first-class image values to build visual automations like the supplied selfie-to-fine-art example: upload an image, route several inspiration images into an agent request, preview the result, and hand the generated image to downstream media steps. Today images can only be smuggled through string or byte outputs, and prompt/AI/LLM-style work risks becoming provider-specific node sprawl instead of one reusable workflow primitive.

## What Changes

- Add typed image values and image ports so workflows can route images without losing media metadata.
- Add image input node configurations that keep decoding and filesystem access behind explicit interfaces.
- Add one generic agent node for any prompt, AI, or LLM-style workflow step, with adapter targets for Pi and Clankers.
- Preserve image input nodes, generic agent nodes, ports, prompt fields, target settings, and output modes through Nickel import/export and complex examples.
- Update the TUI add-node flow, properties, canvas, and output panels to create image inputs, configure generic agent nodes, and preview image outputs safely.
- Cover positive image-routing and Pi/Clankers agent cases plus negative invalid-file, unsupported-format, oversized-input, adapter-failure, unsupported-target, and malformed-Nickel cases.
