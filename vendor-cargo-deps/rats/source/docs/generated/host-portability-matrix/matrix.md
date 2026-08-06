# Generated Rats host portability matrix

Only `supported` exact profile/target cells are claims. Missing, failed, stale, blocked, unsupported, partial, cross-only, experimental, and irrelevant states never promote another cell.

Non-claims: `not-all-os;not-other-architectures;not-release;not-downstream;not-security;not-accessibility`.

| Profile | Exact target | Status | Required grades | Reason |
| --- | --- | --- | --- | --- |
| `evidence-only` | `aarch64-apple-darwin` | **irrelevant** |  | `irrelevant` |
| `evidence-only` | `wasm32-unknown-unknown` | **irrelevant** |  | `irrelevant` |
| `evidence-only` | `x86_64-pc-windows-msvc` | **irrelevant** |  | `irrelevant` |
| `evidence-only` | `x86_64-unknown-linux-gnu` | **irrelevant** |  | `irrelevant` |
| `experimental` | `aarch64-apple-darwin` | **irrelevant** |  | `irrelevant` |
| `experimental` | `wasm32-unknown-unknown` | **irrelevant** |  | `irrelevant` |
| `experimental` | `x86_64-pc-windows-msvc` | **irrelevant** |  | `irrelevant` |
| `experimental` | `x86_64-unknown-linux-gnu` | **experimental** |  | `experimental` |
| `meteorite-components-wasm` | `aarch64-apple-darwin` | **irrelevant** |  | `irrelevant` |
| `meteorite-components-wasm` | `wasm32-unknown-unknown` | **supported** | compile | `all-required-grades-passed` |
| `meteorite-components-wasm` | `x86_64-pc-windows-msvc` | **irrelevant** |  | `irrelevant` |
| `meteorite-components-wasm` | `x86_64-unknown-linux-gnu` | **irrelevant** |  | `irrelevant` |
| `meteorite-showcase-web` | `aarch64-apple-darwin` | **irrelevant** |  | `irrelevant` |
| `meteorite-showcase-web` | `wasm32-unknown-unknown` | **supported** | browser-runtime, compile | `all-required-grades-passed` |
| `meteorite-showcase-web` | `x86_64-pc-windows-msvc` | **irrelevant** |  | `irrelevant` |
| `meteorite-showcase-web` | `x86_64-unknown-linux-gnu` | **irrelevant** |  | `irrelevant` |
| `meteorite-web` | `aarch64-apple-darwin` | **irrelevant** |  | `irrelevant` |
| `meteorite-web` | `wasm32-unknown-unknown` | **supported** | browser-runtime, compile | `all-required-grades-passed` |
| `meteorite-web` | `x86_64-pc-windows-msvc` | **irrelevant** |  | `irrelevant` |
| `meteorite-web` | `x86_64-unknown-linux-gnu` | **irrelevant** |  | `irrelevant` |
| `native-terminal` | `aarch64-apple-darwin` | **missing** | compile, native-host-test, native-runtime-smoke | `compile=missing,native-host-test=missing,native-runtime-smoke=missing` |
| `native-terminal` | `wasm32-unknown-unknown` | **irrelevant** |  | `irrelevant` |
| `native-terminal` | `x86_64-pc-windows-msvc` | **missing** | compile, native-host-test, native-runtime-smoke | `compile=missing,native-host-test=missing,native-runtime-smoke=missing` |
| `native-terminal` | `x86_64-unknown-linux-gnu` | **supported** | compile, native-host-test, native-runtime-smoke | `all-required-grades-passed` |
| `pure-core` | `aarch64-apple-darwin` | **missing** | compile, native-host-test | `compile=missing,native-host-test=missing` |
| `pure-core` | `wasm32-unknown-unknown` | **supported** | compile | `all-required-grades-passed` |
| `pure-core` | `x86_64-pc-windows-msvc` | **missing** | compile, native-host-test | `compile=missing,native-host-test=missing` |
| `pure-core` | `x86_64-unknown-linux-gnu` | **supported** | compile, native-host-test | `all-required-grades-passed` |
| `ratzilla-web` | `aarch64-apple-darwin` | **irrelevant** |  | `irrelevant` |
| `ratzilla-web` | `wasm32-unknown-unknown` | **supported** | browser-runtime, compile | `all-required-grades-passed` |
| `ratzilla-web` | `x86_64-pc-windows-msvc` | **irrelevant** |  | `irrelevant` |
| `ratzilla-web` | `x86_64-unknown-linux-gnu` | **irrelevant** |  | `irrelevant` |
| `reusable-subwayrat` | `aarch64-apple-darwin` | **missing** | compile, native-host-test | `compile=missing,native-host-test=missing` |
| `reusable-subwayrat` | `wasm32-unknown-unknown` | **supported** | compile | `all-required-grades-passed` |
| `reusable-subwayrat` | `x86_64-pc-windows-msvc` | **missing** | compile, native-host-test | `compile=missing,native-host-test=missing` |
| `reusable-subwayrat` | `x86_64-unknown-linux-gnu` | **supported** | compile, native-host-test | `all-required-grades-passed` |
