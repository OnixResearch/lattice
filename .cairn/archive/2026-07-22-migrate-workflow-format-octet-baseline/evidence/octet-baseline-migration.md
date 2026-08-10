# Octet baseline migration evidence

- old baseline file BLAKE3: `aa488ddfb77076c19691ec1ddd60aaed52517d6141e83724677e04bc0b8fc1e1`
- new baseline file BLAKE3: `bb68d531713e860de0d5dbf6dd9a6a34fd4ed41df05af4b5a5f5bb0d2d79473a`
- old full finding count: 1162
- candidate full finding count: 1161
- matched candidate findings: 1161
- allowed old-only findings: 1
- candidate-only findings: 0
- package accepted/current: 911 / 911
- package new/stale: 0 / 0
- Tracey missing: 181
- Tracey input BLAKE3: `cfa3a4efd168658f97e6a10ecdc4e022e9307913dc550e860ff0b05c6a6ae367`
- Tracey receipt BLAKE3: `3eb98b14c6d05bdf9577f332daa2984b6ea87eebc84caa5ab8b5bc1412efc337`
- inventory BLAKE3: `0b385660eb11d40d651179b2e4fedd77c5ac003b6affbc5de12a94d067e4b5ea`

The standalone migration self-test passed. Dry-run accepted only the resolved `workflow_diagnostic_from_runner` assertion-density finding after function-scoped dynamic-count normalization. The executed baseline then passed `nix develop -c octet-precommit-check` with zero new and zero stale findings.
