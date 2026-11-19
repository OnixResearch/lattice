//! Normalization of the producer's `noble-ngraph-report/v1` JSON lines.
//!
//! The report is untrusted bounded process output. A complete observation
//! requires exit zero, exactly one header and one normal row, every header
//! binding equal to the admitted invocation, the per-run census check, and one
//! canonical `i64:` output. A refusal must replace the whole report with one
//! line. Producer counters are JSON numbers; they stay `u64` here and become
//! canonical decimal strings only at the published document boundary.

/// The producer report schema.
pub const SCHEMA: &str = "noble-ngraph-report/v1";
/// The producer package that emits the report.
pub const PRODUCER_PACKAGE: &str = "noble-cli";
/// The exact producer source revision qualified for this profile.
pub const PRODUCER_SOURCE_REVISION: &str = "2677ec88ddb1600d42bbea8d14409c45e39edc95";
/// The exact `net_contract` header object of the qualified producer.
pub const NET_CONTRACT: [(&str, &str); 10] = [
    ("contract", "ngraph-net-i64/3"),
    ("lowering", "ngraph-net-i64-lowering/0"),
    ("rule_table", "noble-net-i64-rules/3"),
    ("plan", "ngraph-net-plan/1"),
    ("representation", "noble-net-arena/1"),
    ("plan_checker", "noble-net-plan-checker/1"),
    ("plan_executor", "noble-net-plan-executor/1"),
    ("schedule", "lifo"),
    ("storage_policy", "reuse"),
    ("identity_scheme", "blake3"),
];
/// Producer capacities that the CLI fixes and Lattice does not lower.
pub const FIXED_CAPACITIES: [(&str, u64); 3] = [
    ("max_agent_slots", 1_048_576),
    ("max_wires", 2_097_152),
    ("max_worklist_entries", 1_048_576),
];
const MAXIMUM_REPORT_LINES: usize = 3;
const MAXIMUM_GRAPH_IDENTITY_BYTES: usize = 128;
const EXHAUSTED_EXIT_CODE: i32 = 5;
const BUDGET_CODE_PREFIX: &str = "E-NET-BUDGET-";
const OUTPUT_PREFIX: &str = "i64:";

/// The bindings a report must echo for this invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Expectation<'a> {
    pub source_revision: &'a str,
    pub input_bytes: u64,
    pub ceilings: super::route::Ceilings,
}

/// The producer's per-row counters, with their own units and provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counters {
    pub interactions: u64,
    pub expected_interactions: u64,
    pub peak_live_agents: u64,
    pub final_wires: u64,
    pub retained_bytes: u64,
    pub retained_bytes_high_water: u64,
}

/// A complete, identity-bound producer result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub result: i64,
    pub graph_identity: String,
    pub counters: Counters,
    pub interactions: super::count::Interactions,
}

/// A producer refusal at one of its declared ceilings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exhaustion {
    pub limit: String,
    pub cap: u64,
    pub observed: u64,
}

/// One normalized evaluation observation; only `Complete` carries a result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalized {
    pub outcome: crate::pure_inet_profile::PureEvaluationOutcome,
    pub reason: &'static str,
    pub completion: Option<Completion>,
    pub exhaustion: Option<Exhaustion>,
}

impl Normalized {
    fn refused(outcome: crate::pure_inet_profile::PureEvaluationOutcome, reason: &'static str) -> Self {
        debug_assert!(!matches!(outcome, crate::pure_inet_profile::PureEvaluationOutcome::Complete { .. }));
        Self {
            outcome,
            reason,
            completion: None,
            exhaustion: None,
        }
    }
}

/// Classify one bounded producer process observation.
pub fn normalize(stdout: &[u8], exit_code: Option<i32>, expectation: &Expectation<'_>) -> Normalized {
    let Some(lines) = json_lines(stdout) else {
        return Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "malformed-report");
    };
    match (exit_code, lines.as_slice()) {
        (_, []) => Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Failed, "no-report"),
        (Some(0), [header]) if header["outcome"] == "normal" => {
            Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Partial, "missing-row")
        }
        (Some(0), [header, row]) => complete(header, row, expectation),
        (Some(0), _) => Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "row-arity"),
        (Some(code), [line]) => refusal(code, line),
        (_, _) => Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "malformed-report"),
    }
}

fn json_lines(stdout: &[u8]) -> Option<Vec<serde_json::Value>> {
    let text = std::str::from_utf8(stdout).ok()?;
    let lines = text.lines().take(MAXIMUM_REPORT_LINES.saturating_add(1)).collect::<Vec<_>>();
    if lines.len() > MAXIMUM_REPORT_LINES {
        return None;
    }
    lines
        .into_iter()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).ok().filter(|value| value["schema"] == SCHEMA))
        .collect()
}

fn complete(header: &serde_json::Value, row: &serde_json::Value, expectation: &Expectation<'_>) -> Normalized {
    let graph_identity = match validate_header(header, expectation) {
        Ok(identity) => identity,
        Err(reason) => return Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Invalid, reason),
    };
    let counters = match row_counters(row) {
        Ok(counters) => counters,
        Err(reason) => return Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Invalid, reason),
    };
    let result = match row_output(row) {
        Ok(result) => result,
        Err(reason) => return Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Invalid, reason),
    };
    let Ok(interactions) = super::count::admit(header.get("semantic_count"), &counters) else {
        return Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "count-binding");
    };
    debug_assert_eq!(counters.interactions, counters.expected_interactions);
    Normalized {
        outcome: crate::pure_inet_profile::PureEvaluationOutcome::Complete { result },
        reason: "complete",
        completion: Some(Completion {
            result,
            graph_identity,
            counters,
            interactions,
        }),
        exhaustion: None,
    }
}

fn validate_header(header: &serde_json::Value, expectation: &Expectation<'_>) -> Result<String, &'static str> {
    let is_envelope_exact = header["outcome"] == "normal"
        && header["stage"] == "evaluation"
        && header["backend"] == "net"
        && header["snapshot"].is_null();
    if !is_envelope_exact {
        return Err("malformed-header");
    }
    debug_assert!(header.is_object());
    let producer = &header["producer"];
    let is_producer_exact = producer["package"] == PRODUCER_PACKAGE
        && producer["source_revision"] == expectation.source_revision
        && producer["source_revision_status"] == "build-declared"
        && producer["executable_digest"].is_null();
    if !is_producer_exact {
        return Err("crossed-producer");
    }
    let contract = NET_CONTRACT.iter().map(|(key, value)| ((*key).to_string(), serde_json::json!(value)));
    if header["net_contract"] != serde_json::Value::Object(contract.collect()) {
        return Err("crossed-contract");
    }
    if header["limits"] != super::route::limits_object(expectation.ceilings) {
        return Err("crossed-limits");
    }
    debug_assert!(expectation.ceilings.is_within_producer_defaults());
    if header["input_bytes"].as_u64() != Some(expectation.input_bytes) {
        return Err("crossed-input-bytes");
    }
    header["graph"]
        .as_str()
        .filter(|graph| !graph.is_empty() && graph.len() <= MAXIMUM_GRAPH_IDENTITY_BYTES && graph.is_ascii())
        .map(str::to_string)
        .ok_or("malformed-header")
}

fn row_counters(row: &serde_json::Value) -> Result<Counters, &'static str> {
    let is_row_exact = row["outcome"] == "normal"
        && row["vector"].as_u64() == Some(0)
        && row["backend"] == "net"
        && row["count_basis"] == "census-checked";
    if !is_row_exact {
        return Err("malformed-row");
    }
    let number = |key: &str| row[key].as_u64().ok_or("malformed-row");
    let counters = Counters {
        interactions: number("interactions")?,
        expected_interactions: number("expected_interactions")?,
        peak_live_agents: number("peak_live_agents")?,
        final_wires: number("final_wires")?,
        retained_bytes: number("retained_bytes")?,
        retained_bytes_high_water: number("retained_bytes_high_water")?,
    };
    debug_assert!(row.is_object());
    if counters.interactions != counters.expected_interactions {
        return Err("interaction-census");
    }
    debug_assert_eq!(counters.interactions, counters.expected_interactions);
    Ok(counters)
}

fn row_output(row: &serde_json::Value) -> Result<i64, &'static str> {
    let Some([output]) = row["outputs"].as_array().map(Vec::as_slice) else {
        return Err("output-arity");
    };
    let text = output.as_str().ok_or("non-string-output")?;
    let decimal = text.strip_prefix(OUTPUT_PREFIX).ok_or("malformed-output")?;
    crate::pure_inet_profile::parse_canonical_i64(decimal).map_err(|_| "malformed-output")
}

fn refusal(exit_code: i32, line: &serde_json::Value) -> Normalized {
    debug_assert!(exit_code != 0);
    let code = line["code"].as_str().unwrap_or_default();
    if let Some(limit) = code.strip_prefix(BUDGET_CODE_PREFIX) {
        let cap = line["cap"].as_u64();
        let observed = line["observed"].as_u64();
        let is_exact = exit_code == EXHAUSTED_EXIT_CODE && line["outcome"] == "exhausted" && line["limit"] == limit;
        return match (is_exact, cap, observed) {
            (true, Some(cap), Some(observed)) => Normalized {
                outcome: crate::pure_inet_profile::PureEvaluationOutcome::Exhausted,
                reason: "exhausted",
                completion: None,
                exhaustion: Some(Exhaustion {
                    limit: limit.to_string(),
                    cap,
                    observed,
                }),
            },
            _ => Normalized::refused(crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "malformed-refusal"),
        };
    }
    let (outcome, reason) = match code {
        "E-NET-INTERNAL-INVALID" => (crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "internal-invalid"),
        "E-NET-UNSUPPORTED" => (crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "unsupported"),
        "E-NET-INTERFACE" => (crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "interface"),
        "E-NET-REFUSED" => (crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "refused"),
        "C-CANCELLED" => (crate::pure_inet_profile::PureEvaluationOutcome::Cancelled, "producer-cancelled"),
        _ if line["outcome"] == "reject" => {
            (crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "producer-reject")
        }
        _ if line["outcome"] == "unsupported" => {
            (crate::pure_inet_profile::PureEvaluationOutcome::Invalid, "unsupported")
        }
        _ => (crate::pure_inet_profile::PureEvaluationOutcome::Failed, "producer-failure"),
    };
    debug_assert!(!reason.is_empty());
    Normalized::refused(outcome, reason)
}
