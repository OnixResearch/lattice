#!/usr/bin/env -S nix shell nixpkgs#rustc nixpkgs#cargo --command rust-script

//! Static guard for the projected role-protocol execution boundary.
//!
//! r[impl lattice.role_protocol.validation]
//! r[impl lattice.role_protocol.fixtures]
//! r[impl lattice.role_protocol.boundary]

use std::path::Path;

const REQUIRED_CORE_MODULES: &[&str] = &[
    "src/engine/protocol/core/mod.rs",
    "src/engine/protocol/core/envelope.rs",
    "src/engine/protocol/core/lookup.rs",
    "src/engine/protocol/core/planning.rs",
    "src/engine/protocol/core/recovery.rs",
    "src/engine/protocol/core/reducer.rs",
    "src/engine/protocol/core/transition.rs",
    "src/engine/protocol/identity.rs",
    "src/engine/protocol/intake.rs",
    "src/engine/protocol/model.rs",
    "src/engine/protocol/model/artifact.rs",
    "src/engine/protocol/model/session.rs",
];
const FORBIDDEN_CORE_MARKERS: &[&str] = &[
    "std::fs",
    "std::net",
    "std::process",
    "std::env",
    "redb::",
    "reqwest::",
    "SystemTime",
    "Instant",
    "println!",
    "eprintln!",
];
const REQUIRED_SHELL_MARKERS: &[&str] = &[
    "SessionRepository",
    "ProtocolAuthority",
    "ProtocolAdapter",
    "DispatchedUnknown",
    "persist_failed_before_dispatch",
];
const REQUIRED_AUTHORITY_MARKERS: &[&str] = &[
    "lattice://role-protocol/",
    "protocol/transport",
    "protocol/external-role",
    "validate_dispatch_envelope",
];
const FORBIDDEN_AUTHORITY_PROMOTIONS: &[&str] = &[
    "exactly-once-delivery-proven",
    "external-role-correctness-proven",
    "deadlock-freedom-under-faults-proven",
    "release-eligibility-proven",
];

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_owned());
    let root = Path::new(&root);
    for module in REQUIRED_CORE_MODULES {
        let text = std::fs::read_to_string(root.join(module)).expect("required protocol core module");
        for marker in FORBIDDEN_CORE_MARKERS {
            assert!(!text.contains(marker), "{module} contains forbidden core effect marker {marker}");
        }
    }
    let shell = std::fs::read_to_string(root.join("src/engine/protocol/shell/mod.rs")).expect("protocol shell module");
    for marker in REQUIRED_SHELL_MARKERS {
        assert!(shell.contains(marker), "protocol shell is missing {marker}");
    }
    let authority = std::fs::read_to_string(root.join("src/engine/protocol/shell/authority.rs"))
        .expect("protocol authority module");
    for marker in REQUIRED_AUTHORITY_MARKERS {
        assert!(authority.contains(marker), "protocol authority is missing {marker}");
    }
    let evidence =
        std::fs::read_to_string(root.join("src/engine/protocol/evidence.rs")).expect("protocol evidence module");
    for marker in FORBIDDEN_AUTHORITY_PROMOTIONS {
        assert!(!evidence.contains(marker), "protocol evidence promotes forbidden claim {marker}");
    }
    println!("projected role-protocol boundary check: PASS");
}
