#![allow(unknown_lints)]
#![allow(
    ambiguous_params,
    non_trait_imports,
    path_segment_repetition,
    platform_dependent_cast,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

include!("parts/command.rs");
include!("parts/response.rs");
include!("parts/framing.rs");
include!("parts/events.rs");
include!("parts/helpers.rs");
