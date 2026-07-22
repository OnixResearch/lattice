#![deny(clippy::ptr_arg)]

pub mod coverage;

pub fn lint_boundary(values:&Vec<String>)->usize{values.len()}
