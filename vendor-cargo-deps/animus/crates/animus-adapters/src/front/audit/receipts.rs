#![allow(unknown_lints)]
#![allow(
    ambiguous_params,
    assertion_density,
    borrowed_argument_types,
    excessive_file_length,
    non_trait_imports,
    path_segment_repetition,
    unchecked_narrowing,
    underscore_in_module_filename
)]

include!("receipts/impl001.rs");
include!("receipts/impl002.rs");
include!("receipts/impl003a.rs");
include!("receipts/impl003b.rs");

#[cfg(test)]
mod tests {
    include!("receipts/tests001.rs");
    include!("receipts/tests002.rs");
}
