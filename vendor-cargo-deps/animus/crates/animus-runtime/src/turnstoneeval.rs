//! Turnstone-inspired headless eval scorer contracts.
//!
//! This module scores already-collected safe action ledgers. Host shells own
//! fixture setup, provider/tool execution, workspaces, timers, concurrency, and
//! persistence.

// Cairn adaptation compatibility module keeps DTOs, validators, and fixtures compact while the change is drained.
#![allow(missing_docs)]
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports,
    path_segment_repetition,
    underscore_in_module_filename,
    usize_in_public_api,
    dead_code
)]
#![allow(numeric_units, raw_arithmetic_overflow, unbounded_collection_growth)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

const HASH_DOMAIN: &str = "animus.turnstone-headless-eval.v1";
const HASH_SEPARATOR: &str = "|";
const RECEIPT_SCHEMA: &str = "animus.turnstone-headless-eval.receipt.v1";
const MAX_LABEL_CHARS: usize = 96;
const HASH_HEX_CHARS: usize = 64;
const EMPTY_COUNT: usize = 0;
const EMPTY_SCORE_COUNT: u64 = 0;
const MIN_COUNT: usize = 1;
const PASSING_SCORE_MICRO: u32 = 1_000_000;
const FAILING_SCORE_MICRO: u32 = 0;
const UNSAFE_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "RAW_TRANSCRIPT:",
    "RAW_TOOL_OUTPUT:",
    "PROVIDER_PAYLOAD:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "sk-",
    "/home/",
    "UNBOUNDED_LOG:",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalSuite {
    pub suite_label: String,
    pub setup_hash: String,
    pub cases: Vec<EvalCase>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalCase {
    pub case_label: String,
    pub setup_summary_hash: String,
    pub expected_actions: Vec<ExpectedAction>,
    pub holdout: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedAction {
    pub action_label: String,
    pub tool_name: String,
    pub matcher: ArgumentMatcher,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentMatcher {
    Exact(String),
    Pattern(BoundedPattern),
    MalformedRawArguments,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedPattern {
    pub prefix: String,
    pub contains: String,
    pub suffix: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedAction {
    pub tool_name: String,
    pub argument_summary: String,
    pub malformed_arguments: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchMode {
    Exact,
    OrderedSubset,
    Subset,
    ContainsAny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmptyExpectationPolicy {
    PassIfNoObserved,
    FailClosed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScorePolicy {
    pub mode: MatchMode,
    pub empty_expectation: EmptyExpectationPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseScore {
    pub case_label: String,
    pub passed: bool,
    pub score_micro: u32,
    pub matched_expected: usize,
    pub unmatched_expected: usize,
    pub extra_observed: usize,
    pub malformed_observed: usize,
    pub issue_codes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalAggregate {
    pub case_count: usize,
    pub pass_count: usize,
    pub average_score_micro: u32,
    pub flaky_case_labels: Vec<String>,
    pub unmatched_action_count: usize,
    pub extra_tool_count: usize,
    pub malformed_tool_call_count: usize,
    pub holdout_case_count: usize,
    pub feedback_case_labels: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalReceipt {
    pub schema: String,
    pub suite_label: String,
    pub suite_hash: String,
    pub case_labels: Vec<String>,
    pub run_count: usize,
    pub average_score_micro: u32,
    pub policy_hash: String,
    pub fixture_hash: String,
    pub redacted: bool,
    pub receipt_hash: String,
    pub leak_marker: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalIssue {
    EmptyField,
    InvalidHash,
    UnsafeText,
    InvalidPattern,
    DuplicateMatch,
    UnmatchedAction,
    ExtraObservedAction,
    MalformedArguments,
    EmptyExpectation,
    ImpossibleScore,
    HoldoutLeak,
    ReceiptLeak,
}

impl EvalIssue {
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyField => "empty-field",
            Self::InvalidHash => "invalid-hash",
            Self::UnsafeText => "unsafe-text",
            Self::InvalidPattern => "invalid-pattern",
            Self::DuplicateMatch => "duplicate-match",
            Self::UnmatchedAction => "unmatched-action",
            Self::ExtraObservedAction => "extra-observed-action",
            Self::MalformedArguments => "malformed-arguments",
            Self::EmptyExpectation => "empty-expectation",
            Self::ImpossibleScore => "impossible-score",
            Self::HoldoutLeak => "holdout-leak",
            Self::ReceiptLeak => "receipt-leak",
        }
    }
}

pub type EvalResult<T> = Result<T, Vec<EvalIssue>>;

pub fn validate_eval_suite(suite: &EvalSuite) -> EvalResult<()> {
    let mut issues = Vec::new();
    validate_label(suite.suite_label.as_str(), &mut issues);
    validate_hash(suite.setup_hash.as_str(), &mut issues);
    if suite.cases.is_empty() {
        issues.push(EvalIssue::EmptyField);
    }
    for case in &suite.cases {
        validate_case(case, &mut issues);
    }
    result_from_issues(issues, ())
}

pub fn score_actions(
    case_label: &str,
    expected: &[ExpectedAction],
    observed: &[ObservedAction],
    policy: ScorePolicy,
) -> CaseScore {
    let mut issues = Vec::new();
    validate_label(case_label, &mut issues);
    validate_expected_actions(expected, &mut issues);
    validate_observed_actions(observed, &mut issues);
    let matched_indices = match policy.mode {
        MatchMode::Exact => match_exact(expected, observed, &mut issues),
        MatchMode::OrderedSubset => match_ordered_subset(expected, observed, &mut issues),
        MatchMode::Subset => match_subset(expected, observed, &mut issues),
        MatchMode::ContainsAny => match_contains_any(expected, observed, &mut issues),
    };
    if expected.is_empty() {
        match policy.empty_expectation {
            EmptyExpectationPolicy::PassIfNoObserved if observed.is_empty() => {}
            EmptyExpectationPolicy::PassIfNoObserved => issues.push(EvalIssue::ExtraObservedAction),
            EmptyExpectationPolicy::FailClosed => issues.push(EvalIssue::EmptyExpectation),
        }
    }
    let matched_expected = matched_indices.len();
    let unmatched_expected =
        if policy.mode == MatchMode::ContainsAny && matched_expected > EMPTY_COUNT {
            EMPTY_COUNT
        } else {
            expected.len().saturating_sub(matched_expected)
        };
    let extra_observed = observed.len().saturating_sub(matched_expected);
    if unmatched_expected > EMPTY_COUNT {
        issues.push(EvalIssue::UnmatchedAction);
    }
    if policy.mode == MatchMode::Exact && extra_observed > EMPTY_COUNT {
        issues.push(EvalIssue::ExtraObservedAction);
    }
    let malformed_observed = observed
        .iter()
        .filter(|action| action.malformed_arguments)
        .count();
    if malformed_observed > EMPTY_COUNT {
        issues.push(EvalIssue::MalformedArguments);
    }
    let passed = issues.is_empty();
    CaseScore {
        case_label: case_label.to_owned(),
        passed,
        score_micro: if passed {
            PASSING_SCORE_MICRO
        } else {
            FAILING_SCORE_MICRO
        },
        matched_expected,
        unmatched_expected,
        extra_observed,
        malformed_observed,
        issue_codes: issue_codes(issues),
    }
}

pub fn aggregate_eval_scores(suite: &EvalSuite, scores: &[CaseScore]) -> EvalResult<EvalAggregate> {
    validate_eval_suite(suite)?;
    let mut issues = Vec::new();
    if scores.len() != suite.cases.len() {
        issues.push(EvalIssue::ImpossibleScore);
    }
    for score in scores {
        if score.score_micro > PASSING_SCORE_MICRO {
            issues.push(EvalIssue::ImpossibleScore);
        }
    }
    if !issues.is_empty() {
        return Err(sorted_issues(issues));
    }
    let pass_count = scores.iter().filter(|score| score.passed).count();
    let total_score = scores
        .iter()
        .map(|score| u64::from(score.score_micro))
        .sum::<u64>();
    let average_score_micro = average_score(total_score, scores.len());
    let feedback_case_labels = suite
        .cases
        .iter()
        .filter(|case| !case.holdout)
        .map(|case| case.case_label.clone())
        .collect::<Vec<_>>();
    Ok(EvalAggregate {
        case_count: scores.len(),
        pass_count,
        average_score_micro,
        flaky_case_labels: scores
            .iter()
            .filter(|score| !score.passed && score.matched_expected > EMPTY_COUNT)
            .map(|score| score.case_label.clone())
            .collect(),
        unmatched_action_count: scores.iter().map(|score| score.unmatched_expected).sum(),
        extra_tool_count: scores.iter().map(|score| score.extra_observed).sum(),
        malformed_tool_call_count: scores.iter().map(|score| score.malformed_observed).sum(),
        holdout_case_count: suite.cases.iter().filter(|case| case.holdout).count(),
        feedback_case_labels,
    })
}

pub fn project_eval_receipt(
    suite: &EvalSuite,
    aggregate: &EvalAggregate,
    policy_hash: String,
) -> EvalReceipt {
    let mut receipt = EvalReceipt {
        schema: RECEIPT_SCHEMA.to_owned(),
        suite_label: suite.suite_label.clone(),
        suite_hash: suite.setup_hash.clone(),
        case_labels: suite
            .cases
            .iter()
            .map(|case| case.case_label.clone())
            .collect(),
        run_count: aggregate.case_count,
        average_score_micro: aggregate.average_score_micro,
        policy_hash,
        fixture_hash: suite.setup_hash.clone(),
        redacted: true,
        receipt_hash: String::new(),
        leak_marker: None,
    };
    receipt.receipt_hash = receipt_hash(&receipt);
    receipt
}

pub fn average_score(total_score: u64, score_count: usize) -> u32 {
    let Ok(divisor) = u64::try_from(score_count) else {
        return FAILING_SCORE_MICRO;
    };
    if divisor == EMPTY_SCORE_COUNT {
        return FAILING_SCORE_MICRO;
    }
    let Some(average) = total_score.checked_div(divisor) else {
        return FAILING_SCORE_MICRO;
    };
    match u32::try_from(average) {
        Ok(score) => score,
        Err(_error) => FAILING_SCORE_MICRO,
    }
}

fn validate_eval_receipt(receipt: &EvalReceipt) -> EvalResult<()> {
    let mut issues = Vec::new();
    if receipt.schema != RECEIPT_SCHEMA {
        issues.push(EvalIssue::EmptyField);
    }
    validate_label(receipt.suite_label.as_str(), &mut issues);
    validate_hash(receipt.suite_hash.as_str(), &mut issues);
    validate_hash(receipt.policy_hash.as_str(), &mut issues);
    validate_hash(receipt.fixture_hash.as_str(), &mut issues);
    validate_hash(receipt.receipt_hash.as_str(), &mut issues);
    if receipt.average_score_micro > PASSING_SCORE_MICRO {
        issues.push(EvalIssue::ImpossibleScore);
    }
    if !receipt.redacted {
        issues.push(EvalIssue::ReceiptLeak);
    }
    if let Some(marker) = &receipt.leak_marker {
        if UNSAFE_MARKERS
            .iter()
            .any(|unsafe_marker| marker.contains(unsafe_marker))
        {
            issues.push(EvalIssue::ReceiptLeak);
        }
    }
    result_from_issues(issues, ())
}

pub fn eval_hash(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn validate_case(case: &EvalCase, issues: &mut Vec<EvalIssue>) {
    validate_label(case.case_label.as_str(), issues);
    validate_hash(case.setup_summary_hash.as_str(), issues);
    validate_expected_actions(&case.expected_actions, issues);
}

fn validate_expected_actions(expected: &[ExpectedAction], issues: &mut Vec<EvalIssue>) {
    for action in expected {
        validate_label(action.action_label.as_str(), issues);
        validate_label(action.tool_name.as_str(), issues);
        match &action.matcher {
            ArgumentMatcher::Exact(value) => validate_label(value.as_str(), issues),
            ArgumentMatcher::Pattern(pattern) => validate_pattern(pattern, issues),
            ArgumentMatcher::MalformedRawArguments => {}
        }
    }
}

fn validate_observed_actions(observed: &[ObservedAction], issues: &mut Vec<EvalIssue>) {
    for action in observed {
        validate_label(action.tool_name.as_str(), issues);
        validate_label(action.argument_summary.as_str(), issues);
    }
}

fn validate_pattern(pattern: &BoundedPattern, issues: &mut Vec<EvalIssue>) {
    validate_label(pattern.prefix.as_str(), issues);
    validate_label(pattern.contains.as_str(), issues);
    validate_label(pattern.suffix.as_str(), issues);
    if pattern.prefix.is_empty() && pattern.contains.is_empty() && pattern.suffix.is_empty() {
        issues.push(EvalIssue::InvalidPattern);
    }
}

fn match_exact(
    expected: &[ExpectedAction],
    observed: &[ObservedAction],
    issues: &mut Vec<EvalIssue>,
) -> Vec<usize> {
    if expected.len() != observed.len() {
        issues.push(EvalIssue::UnmatchedAction);
    }
    expected
        .iter()
        .zip(observed.iter())
        .enumerate()
        .filter_map(|(index, (expected_action, observed_action))| {
            action_matches(expected_action, observed_action).then_some(index)
        })
        .collect()
}

fn match_ordered_subset(
    expected: &[ExpectedAction],
    observed: &[ObservedAction],
    issues: &mut Vec<EvalIssue>,
) -> Vec<usize> {
    let mut cursor = EMPTY_COUNT;
    let mut matched = Vec::new();
    for expected_action in expected {
        let found = observed
            .iter()
            .enumerate()
            .skip(cursor)
            .find(|(_, observed_action)| action_matches(expected_action, observed_action));
        if let Some((index, _)) = found {
            cursor = index + MIN_COUNT;
            matched.push(index);
        }
    }
    detect_duplicate_matches(&matched, issues);
    matched
}

fn match_subset(
    expected: &[ExpectedAction],
    observed: &[ObservedAction],
    issues: &mut Vec<EvalIssue>,
) -> Vec<usize> {
    let mut used = Vec::new();
    for expected_action in expected {
        if let Some(index) = observed
            .iter()
            .enumerate()
            .find(|(index, observed_action)| {
                !used.contains(index) && action_matches(expected_action, observed_action)
            })
            .map(|(index, _)| index)
        {
            used.push(index);
        }
    }
    detect_duplicate_matches(&used, issues);
    used
}

fn match_contains_any(
    expected: &[ExpectedAction],
    observed: &[ObservedAction],
    issues: &mut Vec<EvalIssue>,
) -> Vec<usize> {
    let matched = expected
        .iter()
        .find_map(|expected_action| {
            observed
                .iter()
                .enumerate()
                .find(|(_, observed_action)| action_matches(expected_action, observed_action))
                .map(|(index, _)| index)
        })
        .into_iter()
        .collect::<Vec<_>>();
    detect_duplicate_matches(&matched, issues);
    matched
}

fn action_matches(expected: &ExpectedAction, observed: &ObservedAction) -> bool {
    expected.tool_name == observed.tool_name
        && match &expected.matcher {
            ArgumentMatcher::Exact(value) => observed.argument_summary == *value,
            ArgumentMatcher::Pattern(pattern) => {
                observed
                    .argument_summary
                    .starts_with(pattern.prefix.as_str())
                    && observed
                        .argument_summary
                        .contains(pattern.contains.as_str())
                    && observed.argument_summary.ends_with(pattern.suffix.as_str())
            }
            ArgumentMatcher::MalformedRawArguments => observed.malformed_arguments,
        }
}

fn detect_duplicate_matches(indices: &[usize], issues: &mut Vec<EvalIssue>) {
    let mut seen = Vec::new();
    for index in indices {
        if seen.contains(index) {
            issues.push(EvalIssue::DuplicateMatch);
        }
        seen.push(*index);
    }
}

fn validate_label(value: &str, issues: &mut Vec<EvalIssue>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_LABEL_CHARS {
        issues.push(EvalIssue::EmptyField);
    }
    if UNSAFE_MARKERS.iter().any(|marker| trimmed.contains(marker)) {
        issues.push(EvalIssue::UnsafeText);
    }
}

fn validate_hash(value: &str, issues: &mut Vec<EvalIssue>) {
    if value.len() != HASH_HEX_CHARS || !value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        issues.push(EvalIssue::InvalidHash);
    }
}

fn issue_codes(issues: Vec<EvalIssue>) -> Vec<String> {
    sorted_issues(issues)
        .into_iter()
        .map(|issue| issue.code().to_owned())
        .collect()
}

fn sorted_issues(mut issues: Vec<EvalIssue>) -> Vec<EvalIssue> {
    issues.sort_by_key(|issue| issue.code());
    issues.dedup();
    issues
}

fn result_from_issues<T>(issues: Vec<EvalIssue>, value: T) -> EvalResult<T> {
    let issues = sorted_issues(issues);
    if issues.is_empty() {
        Ok(value)
    } else {
        Err(issues)
    }
}

fn receipt_hash(receipt: &EvalReceipt) -> String {
    let canonical = format!(
        "{}{}{}{}{}{}{}{}{}",
        receipt.schema,
        HASH_SEPARATOR,
        receipt.suite_label,
        HASH_SEPARATOR,
        receipt.suite_hash,
        HASH_SEPARATOR,
        receipt.run_count,
        HASH_SEPARATOR,
        receipt.average_score_micro
    );
    eval_hash("receipt", canonical.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUITE_LABEL: &str = "headless-tool-eval";
    const SETUP_BYTES: &[u8] = b"setup";
    const POLICY_BYTES: &[u8] = b"policy";
    const CASE_A: &str = "case-a";
    const CASE_B: &str = "case-b";
    const TOOL_READ: &str = "read";
    const TOOL_WRITE: &str = "write";
    const ARG_A: &str = "file=README.md";
    const ARG_B: &str = "file=src/lib.rs";

    #[test]
    fn scoring_supports_exact_ordered_subset_subset_and_contains_any() {
        // r[verify agent_harness_runtime.turnstone_headless_eval.dto]
        // r[verify agent_harness_runtime.turnstone_headless_eval.matching]
        // r[verify agent_harness_runtime.turnstone_headless_eval.scoring]
        let expected = vec![
            expected_exact(TOOL_READ, ARG_A),
            expected_pattern(TOOL_WRITE, "file=", "src", ".rs"),
        ];
        let observed_actions = vec![observed(TOOL_READ, ARG_A), observed(TOOL_WRITE, ARG_B)];
        let exact = score_actions(
            CASE_A,
            &expected,
            &observed_actions,
            policy(MatchMode::Exact),
        );
        let ordered = score_actions(
            CASE_A,
            &expected,
            &[
                observed(TOOL_READ, ARG_A),
                observed(TOOL_READ, "extra"),
                observed(TOOL_WRITE, ARG_B),
            ],
            policy(MatchMode::OrderedSubset),
        );
        let subset = score_actions(
            CASE_A,
            &expected,
            &[observed(TOOL_WRITE, ARG_B), observed(TOOL_READ, ARG_A)],
            policy(MatchMode::Subset),
        );
        let any = score_actions(
            CASE_A,
            &expected,
            &[observed(TOOL_WRITE, ARG_B)],
            policy(MatchMode::ContainsAny),
        );

        assert!(exact.passed);
        assert!(ordered.passed);
        assert!(subset.passed);
        assert!(any.passed);
        assert_eq!(ordered.extra_observed, MIN_COUNT);
    }

    #[test]
    fn aggregates_keep_holdout_details_out_of_feedback_labels() {
        // r[verify agent_harness_runtime.turnstone_headless_eval.aggregates]
        // r[verify agent_harness_runtime.turnstone_headless_eval.host_boundary]
        let suite = suite();
        let scores = vec![
            CaseScore {
                case_label: CASE_A.to_owned(),
                passed: true,
                score_micro: PASSING_SCORE_MICRO,
                matched_expected: MIN_COUNT,
                unmatched_expected: EMPTY_COUNT,
                extra_observed: EMPTY_COUNT,
                malformed_observed: EMPTY_COUNT,
                issue_codes: Vec::new(),
            },
            CaseScore {
                case_label: CASE_B.to_owned(),
                passed: false,
                score_micro: FAILING_SCORE_MICRO,
                matched_expected: EMPTY_COUNT,
                unmatched_expected: MIN_COUNT,
                extra_observed: EMPTY_COUNT,
                malformed_observed: EMPTY_COUNT,
                issue_codes: vec![EvalIssue::UnmatchedAction.code().to_owned()],
            },
        ];

        let aggregate = aggregate_eval_scores(&suite, &scores).expect("aggregate");
        let receipt = project_eval_receipt(&suite, &aggregate, eval_hash("policy", POLICY_BYTES));

        assert_eq!(aggregate.holdout_case_count, MIN_COUNT);
        assert_eq!(aggregate.feedback_case_labels, vec![CASE_A.to_owned()]);
        assert_eq!(aggregate.pass_count, MIN_COUNT);
        assert!(validate_eval_receipt(&receipt).is_ok());
    }

    #[test]
    fn malformed_specs_duplicate_matches_receipts_and_empty_policy_fail_closed() {
        // r[verify agent_harness_runtime.turnstone_headless_eval.receipts]
        // r[verify agent_harness_runtime.turnstone_headless_eval.docs]
        // r[verify agent_harness_runtime.turnstone_headless_eval.verification]
        let invalid_pattern = ExpectedAction {
            action_label: "pattern".to_owned(),
            tool_name: TOOL_READ.to_owned(),
            matcher: ArgumentMatcher::Pattern(BoundedPattern {
                prefix: String::new(),
                contains: String::new(),
                suffix: String::new(),
            }),
        };
        let score = score_actions(
            CASE_A,
            &[invalid_pattern, expected_exact(TOOL_READ, ARG_A)],
            &[ObservedAction {
                tool_name: TOOL_READ.to_owned(),
                argument_summary: "RAW_TOOL_OUTPUT:secret".to_owned(),
                malformed_arguments: true,
            }],
            ScorePolicy {
                mode: MatchMode::Exact,
                empty_expectation: EmptyExpectationPolicy::FailClosed,
            },
        );
        assert!(
            score
                .issue_codes
                .contains(&EvalIssue::InvalidPattern.code().to_owned())
        );
        assert!(
            score
                .issue_codes
                .contains(&EvalIssue::UnsafeText.code().to_owned())
        );
        assert!(
            score
                .issue_codes
                .contains(&EvalIssue::MalformedArguments.code().to_owned())
        );

        let empty = score_actions(
            CASE_A,
            &[],
            &[],
            ScorePolicy {
                mode: MatchMode::Exact,
                empty_expectation: EmptyExpectationPolicy::FailClosed,
            },
        );
        assert!(
            empty
                .issue_codes
                .contains(&EvalIssue::EmptyExpectation.code().to_owned())
        );

        let suite = suite();
        let impossible = CaseScore {
            case_label: CASE_A.to_owned(),
            passed: true,
            score_micro: PASSING_SCORE_MICRO + u32::try_from(MIN_COUNT).expect("fits"),
            matched_expected: MIN_COUNT,
            unmatched_expected: EMPTY_COUNT,
            extra_observed: EMPTY_COUNT,
            malformed_observed: EMPTY_COUNT,
            issue_codes: Vec::new(),
        };
        assert!(
            aggregate_eval_scores(&suite, &[impossible])
                .expect_err("impossible")
                .contains(&EvalIssue::ImpossibleScore)
        );

        let mut receipt = EvalReceipt {
            schema: RECEIPT_SCHEMA.to_owned(),
            suite_label: SUITE_LABEL.to_owned(),
            suite_hash: eval_hash("suite", SETUP_BYTES),
            case_labels: vec![CASE_A.to_owned()],
            run_count: MIN_COUNT,
            average_score_micro: PASSING_SCORE_MICRO,
            policy_hash: eval_hash("policy", POLICY_BYTES),
            fixture_hash: eval_hash("fixture", SETUP_BYTES),
            redacted: false,
            receipt_hash: String::new(),
            leak_marker: Some("RAW_TRANSCRIPT:secret".to_owned()),
        };
        receipt.receipt_hash = receipt_hash(&receipt);
        assert!(
            validate_eval_receipt(&receipt)
                .expect_err("receipt leak")
                .contains(&EvalIssue::ReceiptLeak)
        );
    }

    fn suite() -> EvalSuite {
        EvalSuite {
            suite_label: SUITE_LABEL.to_owned(),
            setup_hash: eval_hash("suite", SETUP_BYTES),
            cases: vec![
                EvalCase {
                    case_label: CASE_A.to_owned(),
                    setup_summary_hash: eval_hash(CASE_A, SETUP_BYTES),
                    expected_actions: vec![expected_exact(TOOL_READ, ARG_A)],
                    holdout: false,
                },
                EvalCase {
                    case_label: CASE_B.to_owned(),
                    setup_summary_hash: eval_hash(CASE_B, SETUP_BYTES),
                    expected_actions: vec![expected_exact(TOOL_WRITE, ARG_B)],
                    holdout: true,
                },
            ],
        }
    }

    fn expected_exact(tool_name: &str, argument: &str) -> ExpectedAction {
        ExpectedAction {
            action_label: format!("{tool_name}-exact"),
            tool_name: tool_name.to_owned(),
            matcher: ArgumentMatcher::Exact(argument.to_owned()),
        }
    }

    fn expected_pattern(
        tool_name: &str,
        prefix: &str,
        contains: &str,
        suffix: &str,
    ) -> ExpectedAction {
        ExpectedAction {
            action_label: format!("{tool_name}-pattern"),
            tool_name: tool_name.to_owned(),
            matcher: ArgumentMatcher::Pattern(BoundedPattern {
                prefix: prefix.to_owned(),
                contains: contains.to_owned(),
                suffix: suffix.to_owned(),
            }),
        }
    }

    fn observed(tool_name: &str, argument: &str) -> ObservedAction {
        ObservedAction {
            tool_name: tool_name.to_owned(),
            argument_summary: argument.to_owned(),
            malformed_arguments: false,
        }
    }

    const fn policy(mode: MatchMode) -> ScorePolicy {
        ScorePolicy {
            mode,
            empty_expectation: EmptyExpectationPolicy::FailClosed,
        }
    }
}
