//! Immutable identities shared by UCAN semantics replay and release receipts.

/// Schema accepted by the UCAN semantics fixture adapter.
pub const UCAN_SEMANTICS_FIXTURE_SCHEMA: &str = "basalt.ucan-semantics-fixture.v1";
/// Reviewed UCAN revision in Basalt's validated vendored source boundary.
pub const UCAN_RUNTIME_REVISION: &str = "277ab768eb532ac0c715a3bbff2a7dc37a702f11";
/// Reviewed UCAN revision that produced the bound support catalog.
pub const UCAN_SUPPORT_CATALOG_REVISION: &str = "277ab768eb532ac0c715a3bbff2a7dc37a702f11";
/// BLAKE3 identity of the reviewed UCAN integration support catalog.
pub const UCAN_SUPPORT_CATALOG_BLAKE3: &str = "9ffae46c16556feb27614dee3f5945e48321f08a38f0b7c8b1baf7e0e106d095";
/// BLAKE3 identity of Basalt's checked-in generated policy artifact.
pub const BASALT_POLICY_BLAKE3: &str = "5475d5473ec98cbf052a83512836b0739f5dad3e7b9b0ed201eb3094b1cf39bf";
