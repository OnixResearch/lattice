# Typed durable local session-store policy source of truth.
# r[impl agent_harness_runtime.durable_session_store.boundary]
# r[impl agent_harness_runtime.durable_session_store.commit]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.durable-session-store-policy.v1" in
let policy_id_value = "local-sync-each-commit" in
let max_frame_payload_limit_bytes = 1048576 in
let max_transaction_frame_limit = 256 in
let max_file_limit_bytes = 1073741824 in
let private_file_mode = 384 in
let receipt_field_classes = [
  "store_identity",
  "session_identity",
  "branch_identity",
  "head_identity",
  "operation",
  "durability",
  "counts",
  "lock_status",
  "recovery_status",
  "redaction",
  "non_claim",
] in
let PolicyShape = {
  schema | contracts.exact "DURABLE_SESSION_STORE_SCHEMA" schema_identity,
  policy_id | contracts.safe_identifier "DURABLE_SESSION_STORE_POLICY_ID",
  max_frame_payload_bytes | contracts.integer_between "DURABLE_SESSION_STORE_FRAME_BOUND" 1 max_frame_payload_limit_bytes,
  max_transaction_frames | contracts.integer_between "DURABLE_SESSION_STORE_TRANSACTION_BOUND" 1 max_transaction_frame_limit,
  max_file_bytes | contracts.integer_between "DURABLE_SESSION_STORE_FILE_BOUND" 1 max_file_limit_bytes,
  file_mode | contracts.exact "DURABLE_SESSION_STORE_FILE_MODE" private_file_mode,
  durability | contracts.enum "DURABLE_SESSION_STORE_DURABILITY" ["sync_all_each_commit"],
  locking | contracts.enum "DURABLE_SESSION_STORE_LOCKING" ["exclusive_create_new"],
  private_data | contracts.enum "DURABLE_SESSION_STORE_PRIVATE_DATA" ["replay_private"],
  encryption_postures | Array (contracts.enum "DURABLE_SESSION_STORE_ENCRYPTION" ["absent_explicit", "external_required"]),
  tail_recovery | contracts.enum "DURABLE_SESSION_STORE_TAIL_RECOVERY" ["ignore_uncommitted"],
  committed_corruption | contracts.enum "DURABLE_SESSION_STORE_CORRUPTION" ["reject"],
  receipt_fields | Array (contracts.enum "DURABLE_SESSION_STORE_RECEIPT_FIELD" receipt_field_classes),
  policy_hash | contracts.lowercase_blake3 "DURABLE_SESSION_STORE_POLICY_HASH",
} in
let _whole_policy_validator = fun value =>
  if std.array.length value.encryption_postures != 2 then
    'Error { message = "DURABLE_SESSION_STORE_ENCRYPTION_SET" }
  else if !contracts.unique_by (fun posture => posture) value.encryption_postures then
    'Error { message = "DURABLE_SESSION_STORE_ENCRYPTION_SET" }
  else if value.receipt_fields != receipt_field_classes then
    'Error { message = "DURABLE_SESSION_STORE_RECEIPT_FIELDS" }
  else
    'Ok
in
let DurableSessionStorePolicy = contracts.with_validator PolicyShape _whole_policy_validator in

{
  durable_session_store_policy_contract = DurableSessionStorePolicy,
  durable_session_store_policy | DurableSessionStorePolicy = {
    schema = schema_identity,
    policy_id = policy_id_value,
    max_frame_payload_bytes = 65536,
    max_transaction_frames = 64,
    max_file_bytes = 67108864,
    file_mode = private_file_mode,
    durability = "sync_all_each_commit",
    locking = "exclusive_create_new",
    private_data = "replay_private",
    encryption_postures = ["absent_explicit", "external_required"],
    tail_recovery = "ignore_uncommitted",
    committed_corruption = "reject",
    receipt_fields = receipt_field_classes,
    policy_hash = "cb51ed6a1173d80feeac2e39011cc74ad0a5f51b52df2082b8b5af9643b5e68c",
  },
}
