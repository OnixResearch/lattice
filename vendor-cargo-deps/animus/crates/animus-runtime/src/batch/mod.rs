//! Pure batch trajectory planning, resume, output, and metadata contracts.

#![allow(unknown_lints)]
#![allow(
    ambiguous_params,
    assertion_density,
    non_trait_imports,
    path_segment_repetition,
    usize_in_public_api
)]

use core::fmt;

const RUN_LABEL_FIELD: &str = "run_label";
const SOURCE_LOCATION_FIELD: &str = "source_location";
const OUTPUT_LOCATION_FIELD: &str = "output_location";
const OUTPUT_FORMAT_FIELD: &str = "output_format";
const CONCURRENCY_LIMIT_FIELD: &str = "concurrency_limit";
const EXECUTION_MODE_FIELD: &str = "execution_mode";
const REDACTION_POLICY_FIELD: &str = "redaction_policy";
const JOB_ID_FIELD: &str = "job_id";
const PROMPT_FIELD: &str = "prompt";
const OBJECTIVE_LABEL_FIELD: &str = "objective_label";
const OUTPUT_LABEL_FIELD: &str = "output_label";
const MODEL_HANDLE_FIELD: &str = "model_handle";
const SESSION_HANDLE_FIELD: &str = "session_handle";
const STATUS_FIELD: &str = "status";
const COMPLETION_SEQUENCE_FIELD: &str = "completion_sequence";
const OBJECTIVE_SUMMARY_FIELD: &str = "objective_summary";
const OUTPUT_EVIDENCE_HASH_FIELD: &str = "output_evidence_hash";
const REDACTION_MARKER_FIELD: &str = "redaction_marker";

const FORMAT_JSONL: &str = "jsonl";
const FORMAT_EVAL_JSONL: &str = "eval_jsonl";
const FORMAT_SHARE_GPT: &str = "sharegpt";
const MODE_HEADLESS: &str = "headless";
const MODE_SESSION: &str = "session";
const REDACTION_HASH_ONLY: &str = "hash-only";
const REDACTION_METADATA_ONLY: &str = "metadata-only";
const STATUS_SUCCEEDED: &str = "succeeded";
const STATUS_FAILED: &str = "failed";
const STATUS_SKIPPED: &str = "skipped";
const RESUME_COMPLETED: &str = "completed";
const RESUME_FAILED: &str = "failed";
const RESUME_PARTIAL: &str = "partial";
const HTTP_SCHEME: &str = "http://";
const HTTPS_SCHEME: &str = "https://";
const S3_SCHEME: &str = "s3://";
const SCHEME_SEPARATOR: &str = "://";
const UNC_PREFIX: &str = "\\\\";
const EMPTY_JOBS_MARKER: &str = "jobs";
const BATCH_PLAN_SCHEMA: &str = "animus.batch.plan.v1";
const BATCH_RUN_SCHEMA: &str = "animus.batch.run.v1";
const BATCH_JOB_SCHEMA: &str = "animus.batch.job.v1";
const BATCH_OUTPUT_SCHEMA: &str = "animus.batch.output.v1";
const BATCH_RECEIPT_SCHEMA: &str = "animus.batch.receipt.v1";
const MISSING_SERVICE_REASON: &str = "execution_service_unavailable";
const MIN_CONCURRENCY_LIMIT: u16 = 1;
const FIRST_COMPLETION_SEQUENCE: u64 = 1;

type Text = String;
type Collection<T> = Vec<T>;

include!("parts_a/core.rs");
include!("parts_a/options.rs");
include!("parts_a/outcome.rs");
include!("parts_a/resume.rs");
include!("parts_b/receipt.rs");
include!("parts_b/error.rs");
include!("parts_b/validate.rs");
include!("parts_b/hash.rs");

#[cfg(test)]
mod tests;
