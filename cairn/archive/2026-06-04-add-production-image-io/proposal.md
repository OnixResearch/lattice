# Proposal: Add production image I/O runtime wiring

## Summary

Wire image input nodes into real local filesystem image loading for the default TUI and headless executors. The existing typed image model and injected image I/O test seam remain, but production runs should no longer fail with the placeholder `NoopImageIo` when a workflow references a valid local PNG or JPEG.

## Problem

Image input nodes can be modeled, imported, tested, and executed through injected in-memory I/O, but the default runtime path still constructs executors with `NoopImageIo`. A user-created image input workflow therefore cannot load an actual local image from the TUI or headless CLI without test-only wiring.

## Goals

- Add a production `LocalImageIo` implementation that reads image bytes from local files through a thin imperative shell.
- Preserve pure validation/sniffing logic in `image_node` so tests can exercise path, size, and media-type behavior without real filesystem dependencies.
- Make default TUI and headless execution use production image I/O.
- Keep explicit constructor seams for tests and future non-local image backends.
- Cover valid PNG/JPEG, missing file, unsupported format, oversized file, denied/unsafe path, and default runtime wiring.

## Non-Goals

- Implement Pi or Clankers agent adapters.
- Add terminal image thumbnail rendering beyond existing metadata/fallback surfaces.
- Add remote URL, clipboard, drag-and-drop, or camera image sources.
- Change Nickel workflow shape for image nodes.

## Success Criteria

- A workflow containing an image input node pointing at a valid local PNG or JPEG can run through default executor construction and produce typed image and metadata outputs.
- Missing, unsupported, oversized, unreadable, or unsafe image paths fail the node with clear diagnostics and do not invoke downstream work.
- Existing fake/in-memory image tests still pass through injected `ImageIo` implementations.
- `cargo test --workspace`, `cargo fmt --check`, and `cargo check` pass.
