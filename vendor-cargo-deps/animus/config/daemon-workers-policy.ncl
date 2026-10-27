# Typed policy for the optional Prime Agent daemon client/supervisor/worker cohort.
let contracts = import "../policy/contracts/primitives.ncl" in
let maximum_attachment_bound = 64 in
let maximum_receipt_bound = 1000 in
let Profile = {
  schema | contracts.exact "DAEMON_WORKERS_SCHEMA" "animus.daemon-workers.v1",
  cohort | contracts.exact "DAEMON_WORKERS_COHORT" "prime-agent-daemon-workers",
  license | contracts.exact "DAEMON_WORKERS_LICENSE" "MIT",
  source_revision | contracts.safe_identifier "DAEMON_WORKERS_SOURCE_REVISION",
  max_attachments | contracts.positive_integer "DAEMON_WORKERS_MAX_ATTACHMENTS" maximum_attachment_bound,
  max_receipt_count | contracts.positive_integer "DAEMON_WORKERS_MAX_RECEIPT_COUNT" maximum_receipt_bound,
  role_authorities | Array { role | String, authorities | Array String },
  receipt_fields | Array String,
  non_claims | Array String,
  live_reattach | Bool,
  current_snapshot_reattach | Bool,
  stale_snapshot_rejected | Bool,
  lifecycle_only_containment | Bool,
} in
let ProfileWithRules = contracts.with_validator Profile (fun value =>
  if std.array.length value.role_authorities != 3 then
    'Error { message = "DAEMON_WORKERS_ROLE_COUNT" }
  else if std.array.length value.receipt_fields < 8 then
    'Error { message = "DAEMON_WORKERS_RECEIPT_FIELDS" }
  else if std.array.length value.non_claims < 4 then
    'Error { message = "DAEMON_WORKERS_NON_CLAIMS" }
  else if !std.array.elem "not-security-sandbox" value.non_claims then
    'Error { message = "DAEMON_WORKERS_SANDBOX_NON_CLAIM" }
  else if !value.live_reattach || !value.current_snapshot_reattach || !value.stale_snapshot_rejected then
    'Error { message = "DAEMON_WORKERS_RECOVERY_RULES" }
  else if !value.lifecycle_only_containment then
    'Error { message = "DAEMON_WORKERS_CONTAINMENT" }
  else
    'Ok
) in
{
  daemon_workers_policy_contract = ProfileWithRules,
  daemon_workers_policy | ProfileWithRules = {
    schema = "animus.daemon-workers.v1",
    cohort = "prime-agent-daemon-workers",
    license = "MIT",
    source_revision = "c5991bc853d27754aed345c13ff4d2e05c40f1f4",
    max_attachments = 4,
    max_receipt_count = 32,
    role_authorities = [
      { role = "client", authorities = ["rendering", "input", "ui_preferences"] },
      { role = "supervisor", authorities = ["discovery", "routing", "attachments", "worker_health", "catalog_scan"] },
      { role = "worker", authorities = ["session_tree", "queue", "schedules", "kernel_handles", "descendants"] },
    ],
    receipt_fields = [
      "policy_identity",
      "role",
      "envelope_identity",
      "attachment_class",
      "recovery_class",
      "health_class",
      "bounded_counts",
      "non_claims",
    ],
    non_claims = [
      "not-security-sandbox",
      "not-crash-freedom",
      "not-arbitrary-host-durability",
      "not-multi-machine-supervision",
    ],
    live_reattach = true,
    current_snapshot_reattach = true,
    stale_snapshot_rejected = true,
    lifecycle_only_containment = true,
  },
}
