#![allow(unknown_lints)]
#![allow(non_trait_imports)]

//! UI-neutral host-control DTOs and pure validation.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

include!("core.rs");
include!("topology.rs");
include!("topologystate.rs");
include!("output.rs");
include!("layout.rs");
include!("event/mod.rs");
include!("transport.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    const HOST_ID: &str = "host-main";
    const OTHER_HOST_ID: &str = "host-other";
    const PANE_ID: &str = "pane-1";
    const AGENT_ID: &str = "agent-1";
    const LABEL_MAIN: &str = "main";
    const CORRELATION_ID: &str = "corr-1";
    const BAD_VALUE: &str = "bad";
    const OUTPUT_BODY: &str = "line one\nline two\nline three";
    const SECRET_OUTPUT: &str = "API_KEY=secret";
    const SAFE_REFERENCE: &str = "logs/pane-1";
    const UNSAFE_REFERENCE: &str = "/home/user/logs";
    const LIVE_REVISION: u64 = 7;
    const STALE_REVISION: u64 = 6;
    const OUTPUT_LINE_BUDGET: u32 = 2;
    const OUTPUT_LINE_COUNT: u64 = 3;
    const OUTPUT_VISIBLE_TEXT: &str = "line one\nline two";
    const CWD_SUMMARY: &str = "repo-root";
    const PROVENANCE_SUMMARY: &str = "spawned-by-test";
    const COMMAND_LABEL: &str = "animus-agent";
    const ERROR_CODE: &str = "host-denied";
    const ERROR_MESSAGE: &str = "host denied command";
    const UNKNOWN_FIELD_NAME: &str = "future_field";

    include!("tests/topology.rs");
    include!("tests/outputlayout.rs");
    include!("tests/eventtransport.rs");
    include!("tests/support.rs");
}
