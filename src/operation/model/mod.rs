#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "operation DTOs use their local serialization and ordered-map vocabulary"
)]
#![allow(
    path_segment_repetition,
    reason = "public operation DTO names retain their domain meaning at external call sites"
)]
use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

pub const CATALOG_SCHEMA: &str = "lattice.operation-catalog.v2";
pub const TYPE_SCHEMA_VERSION: u32 = 2;
pub const INSTANCE_SCHEMA: &str = "lattice.operation-instances.v1";
pub const PLAN_SCHEMA: &str = "lattice.operation-invocation-plan.v2";
pub const PROMOTION_SCHEMA: &str = "lattice.operation-promotion-plan.v1";
pub const BLAKE3_PREFIX: &str = "b3:";
pub const BLAKE3_HEX_LENGTH: usize = 64;
pub const MAXIMUM_CATALOG_TYPES: usize = 64;
pub const MAXIMUM_TYPE_METHODS: usize = 32;
pub const MAXIMUM_SCHEMA_FIELDS: usize = 64;
pub const MAXIMUM_INSTANCES: usize = 128;
pub const MAXIMUM_TARGET_FACTS: usize = 32;
pub const MAXIMUM_TAGS: usize = 32;
pub const MAXIMUM_CREDENTIAL_BINDINGS: usize = 16;
pub const MAXIMUM_CAPABILITY_CLASSES: usize = 3;
pub const MAXIMUM_PREFLIGHT_CHECKS: usize = 4;
pub const MAXIMUM_DEFAULT_METHODS: usize = 32;
pub const MAXIMUM_ID_BYTES: usize = 128;
pub const MAXIMUM_DESCRIPTION_BYTES: usize = 512;
pub const MAXIMUM_VALUE_BYTES: usize = 65_536;
pub const MAXIMUM_OPERATION_BYTES: usize = 16_777_216;
pub const MAXIMUM_TIMEOUT_SECONDS: u64 = 3_600;
pub const MINIMUM_POSITIVE_BOUND: u64 = 1;
pub const REQUIRED_NON_CLAIMS: &[&str] = &[
    "content identities prove exact bytes only",
    "invocation plans do not grant execution authority",
    "process success does not prove external-system truth",
    "operation results do not prove production or release readiness",
];

include!("types.rs");
