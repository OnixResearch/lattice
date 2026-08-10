# Finish operator-diagnostics evidence

The prior bounded audit found two exact, four partial, and two unsupported operator-diagnostics evidence rows. The pure evaluation and renderer logic exists, but it remains embedded in the command shell, mixed-severity output is not asserted, redaction inputs are not constructible through a deliberate API, and the CLI redaction sentinel is never injected.

This change will extract the deterministic doctor core behind a binary-local module boundary, add warning and explicit redaction fact construction, test complete healthy/fatal/mixed-output/redaction behavior, and prove the eight reviewed requirements can be promoted without changing unrelated Tracey debt.

The change does not make doctor output confidential against arbitrary host observation, does not resolve credential values, does not probe remote providers, does not establish release readiness, and does not claim that marker coverage proves semantic correctness.
