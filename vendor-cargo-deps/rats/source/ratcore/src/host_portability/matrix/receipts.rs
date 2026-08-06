//! Bounded receipt encoding, parsing, nominal binding, and freshness validation.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write as _;

const RECEIPT_COLUMNS: usize = 19;

/// Parsed receipt collection plus deterministic malformed-input diagnostics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PortabilityReceiptParse {
    /// Successfully parsed rows.
    pub receipts: Vec<super::PortabilityReceipt>,
    /// Malformed rows.
    pub issues: Vec<super::PortabilityMatrixIssue>,
}

/// Computes the hash binding every nominal receipt field.
#[must_use]
pub fn receipt_binding_hash(receipt: &super::PortabilityReceipt) -> String {
    let input = format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        receipt.schema,
        receipt.id,
        receipt.profile,
        receipt.target,
        receipt.runner_host,
        receipt.grade.as_str(),
        method_str(receipt.method),
        state_str(receipt.state),
        receipt.source_revision,
        receipt.source_hash,
        receipt.package_set_hash,
        receipt.command,
        receipt.command_hash,
        receipt.toolchain,
        freshness_str(receipt.freshness),
        receipt.diagnostic,
        receipt.non_claims,
    );
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Renders normalized receipts as deterministic TSV.
#[must_use]
pub fn render_portability_receipts_tsv(receipts: &[super::PortabilityReceipt]) -> String {
    let mut ordered = receipts.to_vec();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    let mut output = String::from(
        "schema\tid\tprofile\ttarget\trunner_host\tgrade\tmethod\tstate\tsource_revision\tsource_hash\tpackage_set_hash\tcommand\tcommand_hash\ttoolchain\tfreshness\tdiagnostic\tnon_claims\tbinding_hash\treserved\n",
    );
    for receipt in ordered {
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t-",
            receipt.schema,
            receipt.id,
            receipt.profile,
            receipt.target,
            receipt.runner_host,
            receipt.grade.as_str(),
            method_str(receipt.method),
            state_str(receipt.state),
            receipt.source_revision,
            receipt.source_hash,
            receipt.package_set_hash,
            receipt.command,
            receipt.command_hash,
            receipt.toolchain,
            freshness_str(receipt.freshness),
            receipt.diagnostic,
            receipt.non_claims,
            receipt.binding_hash,
        )
        .expect("writing to String cannot fail");
    }
    output
}

/// Parses normalized receipt TSV without filesystem or environment access.
#[must_use]
pub fn parse_portability_receipts_tsv(input: &str) -> PortabilityReceiptParse {
    let mut parsed = PortabilityReceiptParse::default();
    for (index, line) in input.lines().enumerate() {
        if index == 0 {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        let subject = format!("receipt-line-{}", index + 1);
        if fields.len() != RECEIPT_COLUMNS || fields[18] != "-" {
            parsed.issues.push(issue(
                super::PortabilityMatrixIssueCode::MalformedReceipt,
                subject,
                "receipt row has wrong bounded shape",
            ));
            continue;
        }
        let Some(grade) = parse_grade(fields[5]) else {
            parsed.issues.push(issue(
                super::PortabilityMatrixIssueCode::MalformedReceipt,
                subject,
                "receipt has unknown grade",
            ));
            continue;
        };
        let Some(method) = parse_method(fields[6]) else {
            parsed.issues.push(issue(
                super::PortabilityMatrixIssueCode::MalformedReceipt,
                subject,
                "receipt has unknown method",
            ));
            continue;
        };
        let Some(state) = parse_state(fields[7]) else {
            parsed.issues.push(issue(
                super::PortabilityMatrixIssueCode::MalformedReceipt,
                subject,
                "receipt has unknown state",
            ));
            continue;
        };
        let Some(freshness) = parse_freshness(fields[14]) else {
            parsed.issues.push(issue(
                super::PortabilityMatrixIssueCode::MalformedReceipt,
                subject,
                "receipt has unknown freshness",
            ));
            continue;
        };
        parsed.receipts.push(super::PortabilityReceipt {
            schema: fields[0].to_string(),
            id: fields[1].to_string(),
            profile: super::PortabilityProfileId::new(fields[2]),
            target: super::super::RustTargetId::new(fields[3]),
            runner_host: super::RunnerHostScopeId::new(fields[4]),
            grade,
            method,
            state,
            source_revision: fields[8].to_string(),
            source_hash: fields[9].to_string(),
            package_set_hash: fields[10].to_string(),
            command: super::PortabilityCommandId::new(fields[11]),
            command_hash: fields[12].to_string(),
            toolchain: fields[13].to_string(),
            freshness,
            diagnostic: fields[15].to_string(),
            non_claims: fields[16].to_string(),
            binding_hash: fields[17].to_string(),
        });
    }
    parsed.receipts.sort_by(|left, right| left.id.cmp(&right.id));
    parsed.issues.sort();
    parsed
}

type ProfileIndex<'a> = BTreeMap<super::PortabilityProfileId, &'a super::PortabilityPackageProfile>;
type RequirementIndex<'a> = BTreeMap<
    (super::PortabilityProfileId, super::super::RustTargetId, super::super::HostEvidenceGrade),
    &'a super::PortabilityMatrixRequirement,
>;
type TargetIndex<'a> = BTreeMap<super::super::RustTargetId, &'a super::super::RustTargetDescriptor>;

struct ReceiptIndexes<'a> {
    profiles: ProfileIndex<'a>,
    requirements: RequirementIndex<'a>,
    targets: TargetIndex<'a>,
}

/// Validates nominal scope, package/command/source freshness, privacy, and duplicates.
#[must_use]
pub fn validate_portability_receipts(
    receipts: &[super::PortabilityReceipt],
    context: super::PortabilityReceiptContext<'_>,
) -> Vec<super::PortabilityMatrixIssue> {
    let profiles = context.profiles.iter().map(|profile| (profile.id.clone(), profile)).collect::<ProfileIndex<'_>>();
    let requirements = context
        .requirements
        .iter()
        .map(|requirement| ((requirement.profile.clone(), requirement.target.clone(), requirement.grade), requirement))
        .collect::<RequirementIndex<'_>>();
    let targets = context.targets.iter().map(|target| (target.id.clone(), target)).collect::<TargetIndex<'_>>();
    let indexes = ReceiptIndexes {
        profiles,
        requirements,
        targets,
    };
    let mut issues = Vec::new();
    if !valid_git_oid(context.source_revision) {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::MalformedReceipt,
            "receipt-context",
            "gate source revision is not a full Git object identity",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut exact_grades = BTreeSet::new();

    for receipt in receipts {
        required_fields(receipt, &mut issues);
        validate_unique_receipt(receipt, &mut ids, &mut exact_grades, &mut issues);
        validate_receipt_facts(receipt, context, &indexes, &mut issues);
    }
    issues.sort();
    issues.dedup();
    issues
}

fn validate_unique_receipt(
    receipt: &super::PortabilityReceipt,
    ids: &mut BTreeSet<String>,
    exact_grades: &mut BTreeSet<(
        super::PortabilityProfileId,
        super::super::RustTargetId,
        super::super::HostEvidenceGrade,
        super::super::HostEvidenceMethod,
    )>,
    issues: &mut Vec<super::PortabilityMatrixIssue>,
) {
    if !ids.insert(receipt.id.clone()) {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::DuplicateIdentity,
            &receipt.id,
            "duplicate receipt identity",
        ));
    }
    if !exact_grades.insert((receipt.profile.clone(), receipt.target.clone(), receipt.grade, receipt.method)) {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::DuplicateReceipt,
            &receipt.id,
            "duplicate receipt for exact profile target grade method",
        ));
    }
}

fn validate_receipt_facts(
    receipt: &super::PortabilityReceipt,
    context: super::PortabilityReceiptContext<'_>,
    indexes: &ReceiptIndexes<'_>,
    issues: &mut Vec<super::PortabilityMatrixIssue>,
) {
    let Some(profile) = indexes.profiles.get(&receipt.profile) else {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::UnknownProfile,
            &receipt.id,
            "receipt references unknown profile",
        ));
        return;
    };
    if !indexes.targets.contains_key(&receipt.target) {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::UnknownTarget,
            &receipt.id,
            "receipt references unknown target",
        ));
        return;
    }
    if receipt.package_set_hash != profile.package_set_hash {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::PackageSetMismatch,
            &receipt.id,
            "receipt package set is stale",
        ));
    }
    if receipt.source_hash != context.source_hash || receipt.freshness != super::super::EvidenceFreshness::Fresh {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::StaleSource,
            &receipt.id,
            "receipt source inputs are stale",
        ));
    }
    match indexes.requirements.get(&(receipt.profile.clone(), receipt.target.clone(), receipt.grade)) {
        Some(requirement)
            if receipt.command == requirement.command && receipt.command_hash == requirement.command_hash => {}
        _ => issues.push(issue(
            super::PortabilityMatrixIssueCode::CommandMismatch,
            &receipt.id,
            "receipt command does not match exact requirement",
        )),
    }
    validate_runner(receipt, &indexes.targets, issues);
    if receipt.binding_hash != receipt_binding_hash(receipt) {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::BindingHashMismatch,
            &receipt.id,
            "receipt nominal binding hash does not match",
        ));
    }
    if !safe_token(&receipt.diagnostic, super::MAX_MATRIX_DIAGNOSTIC_BYTES)
        || !safe_token(&receipt.toolchain, super::MAX_MATRIX_DIAGNOSTIC_BYTES)
    {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::UnsafeDiagnostic,
            &receipt.id,
            "receipt diagnostic or toolchain is unsafe",
        ));
    }
    if receipt.schema != super::MATRIX_RECEIPT_SCHEMA
        || receipt.non_claims != super::MATRIX_NON_CLAIMS
        || !valid_git_oid(&receipt.source_revision)
    {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::MalformedReceipt,
            &receipt.id,
            "receipt schema or non-claims mismatch",
        ));
    }
}

fn required_fields(receipt: &super::PortabilityReceipt, issues: &mut Vec<super::PortabilityMatrixIssue>) {
    for value in [
        receipt.id.as_str(),
        receipt.profile.as_str(),
        receipt.target.as_str(),
        receipt.runner_host.as_str(),
        receipt.source_revision.as_str(),
        receipt.source_hash.as_str(),
        receipt.package_set_hash.as_str(),
        receipt.command.as_str(),
        receipt.command_hash.as_str(),
        receipt.toolchain.as_str(),
        receipt.diagnostic.as_str(),
    ] {
        if value.trim().is_empty() {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::EmptyField,
                &receipt.id,
                "receipt has an empty required field",
            ));
            break;
        }
    }
}

fn validate_runner(
    receipt: &super::PortabilityReceipt,
    targets: &BTreeMap<super::super::RustTargetId, &super::super::RustTargetDescriptor>,
    issues: &mut Vec<super::PortabilityMatrixIssue>,
) {
    if receipt.method != super::super::HostEvidenceMethod::Direct {
        return;
    }
    if receipt.target == super::super::TARGET_WASM_BROWSER {
        if !targets
            .keys()
            .any(|target| target.as_str() == receipt.runner_host.as_str() && target != &receipt.target)
        {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::WrongRunnerHost,
                &receipt.id,
                "browser receipt runner host is not a declared native target",
            ));
        }
    } else if receipt.runner_host.as_str() != receipt.target.as_str() {
        issues.push(issue(
            super::PortabilityMatrixIssueCode::WrongRunnerHost,
            &receipt.id,
            "direct native receipt runner does not match exact target",
        ));
    }
}

fn valid_git_oid(value: &str) -> bool {
    (40..=64).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn safe_token(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_'))
        && !value.contains("..")
}

/// Returns stable evidence method spelling.
#[must_use]
pub const fn method_str(method: super::super::HostEvidenceMethod) -> &'static str {
    match method {
        super::super::HostEvidenceMethod::Direct => "direct",
        super::super::HostEvidenceMethod::CrossCompile => "cross-compile",
        super::super::HostEvidenceMethod::Simulated => "simulated",
        super::super::HostEvidenceMethod::TestBackend => "test-backend",
        super::super::HostEvidenceMethod::SemanticProjection => "semantic",
        super::super::HostEvidenceMethod::Screenshot => "screenshot",
        super::super::HostEvidenceMethod::Experimental => "experimental",
    }
}

/// Returns stable evidence state spelling.
#[must_use]
pub const fn state_str(state: super::super::HostEvidenceState) -> &'static str {
    match state {
        super::super::HostEvidenceState::Passed => "passed",
        super::super::HostEvidenceState::Failed => "failed",
        super::super::HostEvidenceState::Blocked => "blocked",
        super::super::HostEvidenceState::Skipped => "skipped",
        super::super::HostEvidenceState::Unsupported => "unsupported",
    }
}

/// Returns stable evidence freshness spelling.
#[must_use]
pub const fn freshness_str(freshness: super::super::EvidenceFreshness) -> &'static str {
    match freshness {
        super::super::EvidenceFreshness::Fresh => "fresh",
        super::super::EvidenceFreshness::Stale => "stale",
    }
}

fn parse_grade(value: &str) -> Option<super::super::HostEvidenceGrade> {
    match value {
        "compile" => Some(super::super::HostEvidenceGrade::Compile),
        "native-host-test" => Some(super::super::HostEvidenceGrade::NativeHostTest),
        "native-runtime-smoke" => Some(super::super::HostEvidenceGrade::NativeRuntimeSmoke),
        "browser-runtime" => Some(super::super::HostEvidenceGrade::BrowserRuntime),
        _ => None,
    }
}

fn parse_method(value: &str) -> Option<super::super::HostEvidenceMethod> {
    match value {
        "direct" => Some(super::super::HostEvidenceMethod::Direct),
        "cross-compile" => Some(super::super::HostEvidenceMethod::CrossCompile),
        "simulated" => Some(super::super::HostEvidenceMethod::Simulated),
        "test-backend" => Some(super::super::HostEvidenceMethod::TestBackend),
        "semantic" => Some(super::super::HostEvidenceMethod::SemanticProjection),
        "screenshot" => Some(super::super::HostEvidenceMethod::Screenshot),
        "experimental" => Some(super::super::HostEvidenceMethod::Experimental),
        _ => None,
    }
}

fn parse_state(value: &str) -> Option<super::super::HostEvidenceState> {
    match value {
        "passed" => Some(super::super::HostEvidenceState::Passed),
        "failed" => Some(super::super::HostEvidenceState::Failed),
        "blocked" => Some(super::super::HostEvidenceState::Blocked),
        "skipped" => Some(super::super::HostEvidenceState::Skipped),
        "unsupported" => Some(super::super::HostEvidenceState::Unsupported),
        _ => None,
    }
}

fn parse_freshness(value: &str) -> Option<super::super::EvidenceFreshness> {
    match value {
        "fresh" => Some(super::super::EvidenceFreshness::Fresh),
        "stale" => Some(super::super::EvidenceFreshness::Stale),
        _ => None,
    }
}

fn issue(
    code: super::PortabilityMatrixIssueCode,
    subject: impl Into<String>,
    message: &'static str,
) -> super::PortabilityMatrixIssue {
    super::PortabilityMatrixIssue {
        code,
        subject: subject.into(),
        message: message.to_string(),
    }
}
