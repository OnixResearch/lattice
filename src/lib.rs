// The reviewed TOON v3 source compiles inside Lattice so Cargo dependency
// changes cannot replace the accepted Octet identity set. The
// `vendor-cargo-deps` path component is the Octet-reviewed vendored-source
// boundary; byte-exact upstream files keep their upstream style.
extern crate self as indexmap;
pub(crate) type IndexMap<Key, Value> = std::collections::BTreeMap<Key, Value>;
// Vendored TOON constants: only dead_code fires on this exact upstream file.
#[rustfmt::skip]
// Vendored TOON v3 bytes are gated by the strict round-trip corpus, not restyled.
#[allow(
    unknown_lints,
    dead_code,
    non_trait_imports,
    reason = "exact reviewed TOON v3 0.5.0 upstream bytes; gated by the strict round-trip corpus"
)]
#[path = "vendor-cargo-deps/toon-format-compile/src/constants.rs"]
mod constants;
// Vendored TOON decoder: naming lints skip the vendor-cargo-deps boundary.
#[rustfmt::skip]
// Vendored TOON v3 bytes are gated by the strict round-trip corpus, not restyled.
#[allow(
    unknown_lints,
    dead_code,
    unexpected_cfgs,
    clippy::collapsible_if,
    ambiguous_params,
    assertion_density,
    bool_naming,
    catch_all_on_enum,
    compound_condition,
    non_trait_imports,
    no_recursion,
    no_unwrap,
    numeric_units,
    path_segment_repetition,
    platform_dependent_cast,
    raw_arithmetic_overflow,
    renamed_imports,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "exact reviewed TOON v3 0.5.0 upstream bytes; gated by the strict round-trip corpus"
)]
#[path = "vendor-cargo-deps/toon-format-compile/src/decode/mod.rs"]
mod decode;
// Vendored TOON encoder: naming lints skip the vendor-cargo-deps boundary.
#[rustfmt::skip]
// Vendored TOON v3 bytes are gated by the strict round-trip corpus, not restyled.
#[allow(
    unknown_lints,
    dead_code,
    unexpected_cfgs,
    clippy::collapsible_if,
    clippy::question_mark,
    ambiguous_params,
    assertion_density,
    bool_naming,
    function_length,
    nested_conditionals,
    non_trait_imports,
    no_recursion,
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    renamed_imports,
    usize_in_public_api,
    reason = "exact reviewed TOON v3 0.5.0 upstream bytes; gated by the strict round-trip corpus"
)]
#[path = "vendor-cargo-deps/toon-format-compile/src/encode/mod.rs"]
mod encode;
// Vendored TOON types: naming lints skip the vendor-cargo-deps boundary.
#[rustfmt::skip]
// Vendored TOON v3 bytes are gated by the strict round-trip corpus, not restyled.
#[allow(
    unknown_lints,
    dead_code,
    unused_imports,
    ambiguous_params,
    non_trait_imports,
    no_panic,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "exact reviewed TOON v3 0.5.0 upstream bytes; gated by the strict round-trip corpus"
)]
#[path = "vendor-cargo-deps/toon-format-compile/src/types/mod.rs"]
mod types;
// Vendored TOON utils: naming lints skip the vendor-cargo-deps boundary.
#[rustfmt::skip]
// Vendored TOON v3 bytes are gated by the strict round-trip corpus, not restyled.
#[allow(
    unknown_lints,
    dead_code,
    unused_imports,
    clippy::let_and_return,
    ambiguous_params,
    assertion_density,
    bool_naming,
    nested_conditionals,
    non_trait_imports,
    no_recursion,
    path_segment_repetition,
    raw_arithmetic_overflow,
    renamed_imports,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "exact reviewed TOON v3 0.5.0 upstream bytes; gated by the strict round-trip corpus"
)]
#[path = "vendor-cargo-deps/toon-format-compile/src/utils/mod.rs"]
mod utils;

// r[impl lattice.architecture.core_crate.compatibility]
pub mod agent_evaluation;

// r[impl lattice.architecture.core_crate.compatibility]
pub mod agent_node;
pub mod animus_agent;
mod capability;
pub mod config;
pub mod content_identity;
pub mod control_flow {
    pub use lattice_core::control_flow::*;
}
pub mod credentials;
pub mod engine;
pub mod error;
#[path = "workflow/expression/dependencies.rs"]
pub mod expression_dependencies;
pub mod expressions;
pub mod file_node;
pub mod guarded;
pub mod headless;
pub mod http;
pub mod image_node;
pub mod n8n_json;
pub mod nickel;
pub mod node_kind_provider;
// r[impl lattice.node_result_cache.boundary]
pub mod node_result_cache;
pub mod openai;
pub mod operation;
pub mod operation_data;
pub mod operation_pack;
pub mod orchestration;
pub mod process;
mod reconciliation;
pub mod protocol_cli {
    pub use crate::engine::protocol::cli::*;
}
pub mod agent_prompt;
pub mod replay;
pub mod run;
pub mod scheduler;
pub mod store;
pub mod triggers;
pub mod tui;
pub mod webhooks;
pub mod workflow;
pub mod workflow_exchange;
pub mod workflow_format;
pub mod workflow_template;
pub mod workflow_validation;
