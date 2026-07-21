//! Positive and negative model runtime profile fixtures.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::function_length,
        tigerstyle::non_trait_imports,
        tigerstyle::numeric_units
    )
)]

mod generated;
mod negative;
mod positive;
pub(crate) mod support;
