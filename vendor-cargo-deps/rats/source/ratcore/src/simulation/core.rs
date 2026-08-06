//! Provider-neutral deterministic-simulation controls, probes, state, and oracles.

#[path = "inputs/core.rs"]
mod inputs;
#[path = "judgment/core.rs"]
mod judgment;
#[path = "nomenclature/core.rs"]
mod nomenclature;
#[path = "observation/core.rs"]
mod observation;
#[path = "snapshot/core.rs"]
mod snapshot;
#[path = "workload/core.rs"]
mod workload;

pub use inputs::*;
pub use judgment::*;
pub use nomenclature::*;
pub use observation::*;
pub use snapshot::*;
pub use workload::*;

#[cfg(test)]
mod tests;
