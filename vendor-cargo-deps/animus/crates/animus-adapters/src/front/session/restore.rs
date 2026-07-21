#![allow(unknown_lints)]
#![allow(
    ambiguous_params,
    assertion_density,
    borrowed_argument_types,
    excessive_file_length,
    function_length,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    underscore_in_module_filename
)]

include!("restore/impl/impl001.rs");
include!("restore/impl/impl002.rs");
include!("restore/impl/impl003a.rs");
include!("restore/impl/impl003b.rs");
include!("restore/impl/impl004.rs");
include!("restore/impl/impl005.rs");

#[cfg(test)]
mod tests {
    include!("restore/tests/tests001.rs");
    include!("restore/tests/tests002.rs");
    include!("restore/tests/tests003.rs");
}
