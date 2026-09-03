# Nickel evaluator source

Lattice uses `nickel-eval` from canonical Radicle RID `rad:z21miuQLqLwe1HkaMu2mEZy5fdG9F`.

Cargo and Nix pin reviewed object `f23be09e41337de8fa729274f50023f861ae0e8a` through `https://seed.radicle.garden/z21miuQLqLwe1HkaMu2mEZy5fdG9F.git`.

The supported cohort is Nickel CLI `1.17.0` at upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`, `nickel-lang 2.2.0`, and `nickel-lang-core 0.18.0`.

Lattice retains configuration discovery, defaults, product decoding, warning policy, store-opening order, and rollback authority.

A source outage fails dependency acquisition. There is no GitHub, sibling-path, branch, or runtime fallback.

Rollback reverts the dependency, Cargo and Nix locks, adapter, and call-site changes in one VCS revert.

Passing checks prove selected fixture compatibility. They do not prove Nickel soundness, import completeness, contract correctness, configuration safety, or release readiness.
