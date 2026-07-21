#![allow(unknown_lints)]
#![allow(
    ambiguous_params,
    assertion_density,
    bool_naming,
    excessive_file_length,
    function_length,
    no_panic,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth,
    underscore_in_module_filename,
    usize_in_public_api
)]

include!("routing/impl/impl001.rs");
include!("routing/impl/impl002.rs");
include!("routing/impl/impl003.rs");
include!("routing/impl/impl004.rs");
include!("routing/impl/impl005.rs");
include!("routing/impl/impl006.rs");

#[cfg(test)]
mod tests {
    include!("routing/tests/tests001.rs");
    include!("routing/tests/tests002.rs");
    include!("routing/tests/tests003.rs");
    include!("routing/tests/tests004.rs");
}
