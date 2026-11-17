# Tiled render consumer fixture

This opt-in fixture is driven by `scripts/qualify-render.py`; see
[render workflow admission and evidence](../../docs/render-workflow.md). It uses
the hand-authored frozen scene from exact public Aspen revision
`afc8308bff9a8c50b2315cb4a17c9454252d7c45`, its canonical manifest, actual
import-free renderer component, and existing controller/worker CLI/iroh route.
No scene assets or code from Hatchet, Graphics Lab or private sources are needed.

Run from the Lattice root with a built absolute Lattice binary and an absent
absolute private retention directory:

```sh
nix-shell --pure examples/render-workflow/shell.nix --run \
  'python3 scripts/qualify-render.py --retain /ABSOLUTE/NEW/RENDER-EVIDENCE --lattice /ABSOLUTE/lattice/target/debug/lattice'
```

The driver retains inert admitted graphs/plans before execution, installs only
exact local operator grants for those isolated test graphs, starts loopback
processes and exercises same-backend repetition and selected interruption controls.
The Nix shell supplies driver tools from Lattice's lock; the producer component
build consumes the producer's own retained locks/toolchain. No sibling working
tree, private credentials, hidden source fetch or ambient renderer fallback is used.

The driver assigns each managed process a private POSIX session. Its pinned
`procps` session selector terminates the observer's nested local process groups,
including bounded-exec children, without signalling the separate producer
sessions. This is controlled fixture teardown, not an arbitrary sandbox or a
claim that killing an observer stops remote work.

`qualification.json` is a measured result only after execution. Merely having
this fixture or passing native contract tests is not qualified Lattice rendering.
No completed qualification is claimed by these authored sources.
