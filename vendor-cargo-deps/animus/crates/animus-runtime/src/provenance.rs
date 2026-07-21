//! Pure provenance ledger, receipt, artifact, and interchange contracts.
//!
//! Hosts own persistence, provider payloads, tool bodies, filesystem paths,
//! clocks, network clients, and concrete Toolpath integration. This module owns
//! deterministic in-memory DTOs plus validation helpers for safe replay and
//! redacted evidence projection.

#![allow(unknown_lints)]
#![allow(
    ambiguous_params,
    assertion_density,
    function_length,
    no_recursion,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    platform_dependent_cast,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

include!("provenance/basis.rs");
include!("provenance/ledger.rs");
include!("provenance/artifacttoolpath.rs");
include!("provenance/fact.rs");
include!("provenance/branchgraph.rs");
include!("provenance/receipteffect.rs");
include!("provenancegitai/mod.rs");
include!("provenance/queryvalidate.rs");

#[cfg(test)]
mod tests {
    include!("provenancetests/support.rs");
    include!("provenancetests/replayfacts.rs");
    include!("provenancetests/graphreceiptseffects.rs");
    include!("provenancetests/toolpath.rs");
    include!("provenancegitaitests/support.rs");
    include!("provenancegitaitests/core.rs");
    include!("provenancegitaitests/range.rs");
    include!("provenancegitaitests/compat.rs");
    include!("provenancegitaitests/receipts.rs");
}
