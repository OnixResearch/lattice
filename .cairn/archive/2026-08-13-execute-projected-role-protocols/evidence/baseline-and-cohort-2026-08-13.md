# Baseline and producer cohort

Date: 2026-08-13

## Pre-change baseline

`cargo test --workspace --all-targets --no-fail-fast` passed before the protocol runtime changed. The workspace result included 4,849 verified-logic tests with no failure. Existing Lattice tests also passed.

## Choregraph producer

Lattice consumes the archived `add-role-choreography-projection` producer at commit `0b2000edaf86d5a7a8601246afbc0d526f4a2a07`.

The canonical source is `https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git`. The Git archive BLAKE3 is `8c35370153e249cc2fe9454746a82950d830e1451b3cd77cece96321acf1ea1e`. The Nix NAR hash is `sha256-0aF9Hhhp/9yrm6tO977JSss0FcqmqPNYKZocqF4PbBA=`.

The archived producer validation reports these results:

- workspace tests passed;
- workspace Clippy passed;
- Octet had zero findings;
- all positive and negative role-choreography checks passed;
- all relevant proof checks passed;
- the complete Nix flake check passed; and
- all 26 producer tasks passed.

The frozen adapter identity is `blake3:0afe63826d52e531da3924bb9a07945b93088a5636d2f35ae4141f297ea3c20c`. It identifies the producer's reviewed source-identity list. The pinned Trellis revision is `d65974b14978e1a44b0f5067beb9ef6d83317dc8`.

## Authority dependency

The accepted Lattice effect-authority boundary is present in `.cairn/specs/choregraph-effect-authority/spec.md`. Protocol dispatch uses that boundary before transport or external-role effects.

The reviewed policy identity is `blake3:56800b3cb2f0819e31cbd32fc3ee41e1a74e7dbf6495540ad8d50db5341f0a79`. The policy adds one separate `lattice-role-protocol` contract. Choregraph graph data does not define this policy.

## Claim boundary

The producer proves bounded source admission, exact Trellis projection, and canonical identity linkage. It does not grant runtime authority or prove Lattice transport, persistence, external-role correctness, delivery, exactly-once behavior, progress, production readiness, or release eligibility.
