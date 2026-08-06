#[path = "tests/control.rs"]
mod inputs;
#[path = "tests/identity.rs"]
mod naming;
#[path = "tests/probe.rs"]
mod observation;
#[path = "tests/semantics.rs"]
mod semantics;
#[path = "tests/pilot.rs"]
mod workload;

use super::*;

fn registry() -> SimulationIdentityRegistry {
    headless_pilot_registry().expect("built-in simulation registry is valid")
}
