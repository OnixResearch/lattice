#!/usr/bin/env -S nix shell nixpkgs#rustc nixpkgs#cargo --command rust-script

//! Static guard for the Choregraph effect authority boundary.

use std::path::Path;

const FORBIDDEN_RUN_OWNER_FIELDS: &[&str] = &["scope_id", "audience", "membership_epoch", "consent"];
const REQUIRED_MODULES: &[&str] = &[
    "src/engine/effect/choregraph.rs",
    "src/engine/effect/authority.rs",
    "src/engine/effect/dispatch.rs",
    "src/engine/effect/runtime.rs",
];
const FORBIDDEN_AUTHORITY_SOURCE: &str = "ambient_authority()";

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_owned());
    let root = Path::new(&root);
    for module in REQUIRED_MODULES {
        let text = std::fs::read_to_string(root.join(module)).expect("required authority module");
        assert!(!text.contains(FORBIDDEN_AUTHORITY_SOURCE), "{module} must not acquire ambient root authority");
    }
    let store = std::fs::read_to_string(root.join("src/store/mod.rs")).expect("store source");
    let run_owner = store
        .split("pub enum RunOwner")
        .nth(1)
        .and_then(|tail| tail.split("}\n\n").next())
        .expect("RunOwner body");
    for forbidden in FORBIDDEN_RUN_OWNER_FIELDS {
        assert!(!run_owner.contains(forbidden), "RunOwner must not become collaboration authority: {forbidden}");
    }
    let policy =
        std::fs::read_to_string(root.join("policy/choregraph-effect-authority.ncl")).expect("authority policy");
    assert!(!policy.contains("machine"), "graph machine data must not define Basalt policy");
    assert!(!policy.contains("workflow"), "workflow data must not define Basalt policy");
    println!("choregraph effect authority boundary check: PASS");
}
