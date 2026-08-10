# Design: finish workflow-example negative evidence

The negative fixture is a normal typed Nickel workflow whose three image paths intentionally resolve to unsupported GIF bytes in the maintained test. The deterministic executor MUST report unsupported-format diagnostics, skip downstream execution, and produce no completed node result. The existing positive example remains unchanged.

Evidence is bound to both example fixtures and the maintained test file. No marker is promoted in this change. This does not claim arbitrary image-format validation or production readiness.
