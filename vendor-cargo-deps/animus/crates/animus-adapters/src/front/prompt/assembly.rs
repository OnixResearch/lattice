#![allow(unknown_lints)]
#![allow(
    assertion_density,
    borrowed_argument_types,
    excessive_file_length,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    underscore_in_module_filename
)]

include!("assembly/impl/impl001.rs");
include!("assembly/impl/impl002.rs");
include!("assembly/impl/impl003.rs");
include!("assembly/impl/impl004.rs");
include!("assembly/impl/impl005.rs");
include!("assembly/impl/impl006.rs");

#[cfg(test)]
mod tests {
    include!("assembly/tests/tests001.rs");
    include!("assembly/tests/tests002.rs");
}
