use super::*;

const RUN_LABEL: &str = "local-run";
const SOURCE_LOCATION: &str = "fixtures/batch-input.jsonl";
const OUTPUT_LOCATION: &str = "target/batch-output.jsonl";
const FIRST_JOB_ID: &str = "job-a";
const SECOND_JOB_ID: &str = "job-b";
const UNKNOWN_JOB_ID: &str = "job-z";
const FIRST_PROMPT: &str = "summarize local fixture";
const SECOND_PROMPT: &str = "explain local fixture";
const OBJECTIVE_LABEL: &str = "summary";
const FIRST_OUTPUT_LABEL: &str = "row-a";
const SECOND_OUTPUT_LABEL: &str = "row-b";
const MODEL_HANDLE: &str = "model.local";
const SESSION_HANDLE: &str = "session.local";
const OBJECTIVE_SUMMARY: &str = "objective-ok";
const FIRST_COMPLETION: u64 = 1;
const SECOND_COMPLETION: u64 = 2;
const CONCURRENCY_LIMIT: u16 = 2;
const REMOTE_HTTP_LOCATION: &str = "https://example.invalid/input.jsonl";
const REMOTE_S3_LOCATION: &str = "s3://bucket/output.jsonl";
const RECEIPT_REDACTION_MARKER: &str = "metadata_hash_only";
const RAW_PROMPT_LEAK: &str = "RAW_PROMPT: summarize credentials";
const PROVIDER_PAYLOAD_LEAK: &str = "provider payload body";
const CREDENTIAL_LEAK: &str = "Authorization: bearer secret=abc";
const FULL_PATH_LEAK: &str = "/home/user/private/output";
const UNBOUNDED_OUTPUT_LEAK: &str = "unbounded_output: huge log";
const VALID_RECEIPT_JOB: BatchJobReceiptDraft<'static> = BatchJobReceiptDraft {
    job_id: FIRST_JOB_ID,
    status: STATUS_SUCCEEDED,
    prompt_bytes: FIRST_PROMPT.len(),
    model_handle: Some(MODEL_HANDLE),
    session_handle: Some(SESSION_HANDLE),
    objective_summary: OBJECTIVE_SUMMARY,
    output_label: FIRST_OUTPUT_LABEL,
    output_evidence_hash: RECEIPT_REDACTION_MARKER,
    redaction_marker: RECEIPT_REDACTION_MARKER,
};
const VALID_RECEIPT_JOBS: &[BatchJobReceiptDraft<'static>] = &[VALID_RECEIPT_JOB];

struct CountingExecutionService {
    calls: usize,
}

impl BatchExecutionService for CountingExecutionService {
    fn execute(&mut self, job: &BatchJob) -> Result<BatchJobOutcome, BatchError> {
        self.calls += 1;
        BatchJobOutcome::validate(&BatchJobOutcomeDraft {
            job_id: job.job_id(),
            status: STATUS_SUCCEEDED,
            completion_sequence: self.calls as u64,
            model_handle: Some(MODEL_HANDLE),
            session_handle: Some(SESSION_HANDLE),
            objective_summary: OBJECTIVE_SUMMARY,
            output_label: job.output_label().as_str(),
            output_evidence_hash: RECEIPT_REDACTION_MARKER,
            redaction_marker: RECEIPT_REDACTION_MARKER,
        })
    }
}

include!("positive.rs");
include!("negative.rs");
include!("helpers.rs");
