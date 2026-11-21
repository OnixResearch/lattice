//! Canonical encoding and strict decoding of attachment documents.

use alloc::string::String;
use alloc::vec::Vec;

use super::data::{
    Attachment, AttachmentDocument, Component, ComponentBasis, ComponentKind, Costs, CoveredTime,
    EvidenceClass, EvidenceLabel, EvidenceSource, Measures, ModelDeployment, ModelFacts, Outcomes,
    Provenance, QualificationSubject, Reference, Rejection, RejectionReason, Scope, VersionBasis,
};
use super::json::{self, Fields, Value};
use super::rules::{BLAKE3_PREFIX, check_document};
use super::{ATTACHMENT_SCHEMA, MAX_ATTACHMENT_BYTES, SUBJECT_SCHEMA};

/// Validates `document` and returns its canonical bytes and identity.
///
/// # Errors
///
/// Returns a `malformed` rejection naming the first failing field.
pub fn encode_attachment(document: &AttachmentDocument) -> Result<Attachment, Rejection> {
    check_document(document)?;
    let canonical = json::encode(&document_value(document));
    if canonical.len() > MAX_ATTACHMENT_BYTES {
        return Err(Rejection::new(
            RejectionReason::Malformed,
            "attachment.bytes",
        ));
    }
    Ok(Attachment {
        identity: attachment_identity(&canonical),
        canonical,
        document: document.clone(),
    })
}

/// Decodes canonical attachment bytes without consulting any subject.
///
/// # Errors
///
/// Returns a `malformed` rejection for oversized, non-canonical, unknown-field,
/// mis-spelled, structurally invalid, or unsafe documents.
// r[impl agent_harness_runtime.qualification_attachment.attachment]
pub fn decode_attachment(bytes: &[u8]) -> Result<Attachment, Rejection> {
    let malformed = |field| Rejection::new(RejectionReason::Malformed, field);
    if bytes.len() > MAX_ATTACHMENT_BYTES {
        return Err(malformed("attachment.bytes"));
    }
    let value = json::parse(bytes).ok_or(malformed("attachment.syntax"))?;
    let document = read_document(value).ok_or(malformed("attachment.shape"))?;
    let attachment = encode_attachment(&document)?;
    if attachment.canonical != bytes {
        return Err(malformed("attachment.canonical"));
    }
    Ok(attachment)
}

/// `blake3:` identity of canonical attachment bytes.
#[must_use]
pub fn attachment_identity(canonical: &[u8]) -> String {
    alloc::format!("{BLAKE3_PREFIX}{}", blake3::hash(canonical).to_hex())
}

fn document_value(document: &AttachmentDocument) -> Value {
    let evidence = &document.evidence;
    json::object(alloc::vec![
        ("schema", json::text(ATTACHMENT_SCHEMA)),
        ("subject", subject_value(&document.subject)),
        (
            "evidence",
            json::object(alloc::vec![
                ("class", json::text(evidence.class.as_str())),
                ("protocol_identity", json::text(&evidence.protocol_identity)),
                ("verifier_identity", json::text(&evidence.verifier_identity)),
                (
                    "corpus_identity",
                    json::optional_text(evidence.corpus_identity.as_deref()),
                ),
                ("source_identity", json::text(&evidence.source_identity)),
            ]),
        ),
        (
            "outcomes",
            document
                .outcomes
                .as_ref()
                .map_or(Value::Null, outcomes_value),
        ),
        (
            "measures",
            document
                .measures
                .as_ref()
                .map_or(Value::Null, measures_value),
        ),
        (
            "scope",
            json::object(alloc::vec![
                ("partitions", json::texts(&document.scope.partitions)),
                ("limitations", json::texts(&document.scope.limitations)),
                ("missing_modes", json::texts(&document.scope.missing_modes)),
            ]),
        ),
        ("provenance", provenance_value(&document.provenance)),
        (
            "references",
            Value::List(
                document
                    .references
                    .iter()
                    .map(|reference| {
                        json::object(alloc::vec![
                            ("kind", json::text(&reference.kind)),
                            ("identity", json::text(&reference.identity)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn subject_value(subject: &QualificationSubject) -> Value {
    let components = subject
        .components
        .iter()
        .map(|component| {
            json::object(alloc::vec![
                ("kind", json::text(component.kind.as_str())),
                ("basis", json::text(component.basis.as_str())),
                (
                    "identity",
                    json::optional_text(component.identity.as_deref())
                ),
            ])
        })
        .collect();
    let model = &subject.model;
    json::object(alloc::vec![
        ("schema", json::text(SUBJECT_SCHEMA)),
        (
            "lattice_definition_identity",
            json::text(&subject.lattice_definition_identity),
        ),
        (
            "lattice_profile_revision",
            json::text(&subject.lattice_profile_revision),
        ),
        ("task_family", json::text(&subject.task_family)),
        ("components", Value::List(components)),
        (
            "model",
            json::object(alloc::vec![
                ("deployment", json::text(deployment_str(model.deployment))),
                (
                    "version_basis",
                    json::text(version_basis_str(model.version_basis))
                ),
                ("version", json::optional_text(model.version.as_deref())),
                (
                    "weights_identity",
                    json::optional_text(model.weights_identity.as_deref()),
                ),
            ]),
        ),
    ])
}

fn outcomes_value(outcomes: &Outcomes) -> Value {
    json::object(alloc::vec![
        ("planned", Value::Number(outcomes.planned)),
        ("accepted", Value::Number(outcomes.accepted)),
        ("rejected", Value::Number(outcomes.rejected)),
        ("failed", Value::Number(outcomes.failed)),
        ("cancelled", Value::Number(outcomes.cancelled)),
        ("incomplete", Value::Number(outcomes.incomplete)),
    ])
}

fn measures_value(measures: &Measures) -> Value {
    let costs = &measures.costs;
    let time = &measures.covered_time;
    json::object(alloc::vec![
        ("repetitions", Value::Number(measures.repetitions)),
        (
            "task_coverage",
            json::object(alloc::vec![
                ("covered", Value::Number(measures.tasks_covered)),
                ("declared", Value::Number(measures.tasks_declared)),
            ]),
        ),
        (
            "time_to_acceptance_ms",
            json::optional_number(measures.time_to_acceptance_ms),
        ),
        (
            "covered_time_ms",
            json::object(alloc::vec![
                ("queue", json::optional_number(time.queue_ms)),
                ("repair", json::optional_number(time.repair_ms)),
                ("verification", json::optional_number(time.verification_ms)),
            ]),
        ),
        (
            "costs",
            json::object(alloc::vec![
                ("unit", json::text(&costs.unit)),
                ("model", json::optional_number(costs.model)),
                ("tool", json::optional_number(costs.tool)),
                ("manager", json::optional_number(costs.manager)),
                ("verifier", json::optional_number(costs.verifier)),
                ("unknown_remainder", Value::Bool(costs.unknown_remainder)),
            ]),
        ),
        (
            "review_interventions",
            json::optional_number(measures.review_interventions),
        ),
        (
            "invalid_actions",
            json::optional_number(measures.invalid_actions)
        ),
        (
            "false_completion_attempts",
            json::optional_number(measures.false_completion_attempts),
        ),
    ])
}

fn provenance_value(provenance: &Provenance) -> Value {
    let label = match provenance.label {
        EvidenceLabel::Fixture => "fixture",
        EvidenceLabel::Live => "live",
    };
    json::object(alloc::vec![
        ("label", json::text(label)),
        ("producer", json::text(&provenance.producer)),
        (
            "producer_revision",
            json::text(&provenance.producer_revision)
        ),
        ("observed_at_ms", Value::Number(provenance.observed_at_ms)),
        ("valid_until_ms", Value::Number(provenance.valid_until_ms)),
    ])
}

const fn deployment_str(deployment: ModelDeployment) -> &'static str {
    match deployment {
        ModelDeployment::Hosted => "hosted",
        ModelDeployment::Local => "local",
        ModelDeployment::Scripted => "scripted",
    }
}

const fn version_basis_str(basis: VersionBasis) -> &'static str {
    match basis {
        VersionBasis::ProviderDeclared => "provider_declared",
        VersionBasis::VerifiedArtifact => "verified_artifact",
        VersionBasis::Declared => "declared",
        VersionBasis::Unavailable => "unavailable",
    }
}

fn read_document(value: Value) -> Option<AttachmentDocument> {
    let mut fields = Fields::new(value)?;
    (take_text(&mut fields, "schema")? == ATTACHMENT_SCHEMA).then_some(())?;
    let subject = read_subject(fields.take("subject")?)?;
    let evidence = read_evidence(fields.take("evidence")?)?;
    let outcomes = optional(fields.take("outcomes")?, read_outcomes)?;
    let measures = optional(fields.take("measures")?, read_measures)?;
    let scope = read_scope(fields.take("scope")?)?;
    let provenance = read_provenance(fields.take("provenance")?)?;
    let references = list(fields.take("references")?, read_reference)?;
    fields.finish()?;
    Some(AttachmentDocument {
        subject,
        evidence,
        outcomes,
        measures,
        scope,
        provenance,
        references,
    })
}

fn read_subject(value: Value) -> Option<QualificationSubject> {
    let mut fields = Fields::new(value)?;
    (take_text(&mut fields, "schema")? == SUBJECT_SCHEMA).then_some(())?;
    let subject = QualificationSubject {
        lattice_definition_identity: take_text(&mut fields, "lattice_definition_identity")?,
        lattice_profile_revision: take_text(&mut fields, "lattice_profile_revision")?,
        task_family: take_text(&mut fields, "task_family")?,
        components: list(fields.take("components")?, read_component)?,
        model: read_model(fields.take("model")?)?,
    };
    fields.finish()?;
    Some(subject)
}

fn read_component(value: Value) -> Option<Component> {
    let mut fields = Fields::new(value)?;
    let kind = ComponentKind::parse(&take_text(&mut fields, "kind")?)?;
    let basis = match take_text(&mut fields, "basis")?.as_str() {
        "declared" => ComponentBasis::Declared,
        "observed" => ComponentBasis::Observed,
        "unavailable" => ComponentBasis::Unavailable,
        _ => return None,
    };
    let identity = optional(fields.take("identity")?, as_text)?;
    fields.finish()?;
    Some(Component {
        kind,
        basis,
        identity,
    })
}

fn read_model(value: Value) -> Option<ModelFacts> {
    let mut fields = Fields::new(value)?;
    let deployment = match take_text(&mut fields, "deployment")?.as_str() {
        "hosted" => ModelDeployment::Hosted,
        "local" => ModelDeployment::Local,
        "scripted" => ModelDeployment::Scripted,
        _ => return None,
    };
    let version_basis = match take_text(&mut fields, "version_basis")?.as_str() {
        "provider_declared" => VersionBasis::ProviderDeclared,
        "verified_artifact" => VersionBasis::VerifiedArtifact,
        "declared" => VersionBasis::Declared,
        "unavailable" => VersionBasis::Unavailable,
        _ => return None,
    };
    let model = ModelFacts {
        deployment,
        version_basis,
        version: optional(fields.take("version")?, as_text)?,
        weights_identity: optional(fields.take("weights_identity")?, as_text)?,
    };
    fields.finish()?;
    Some(model)
}

fn read_evidence(value: Value) -> Option<EvidenceSource> {
    let mut fields = Fields::new(value)?;
    let evidence = EvidenceSource {
        class: EvidenceClass::parse(&take_text(&mut fields, "class")?)?,
        protocol_identity: take_text(&mut fields, "protocol_identity")?,
        verifier_identity: take_text(&mut fields, "verifier_identity")?,
        corpus_identity: optional(fields.take("corpus_identity")?, as_text)?,
        source_identity: take_text(&mut fields, "source_identity")?,
    };
    fields.finish()?;
    Some(evidence)
}

fn read_outcomes(value: Value) -> Option<Outcomes> {
    let mut fields = Fields::new(value)?;
    let outcomes = Outcomes {
        planned: take_number(&mut fields, "planned")?,
        accepted: take_number(&mut fields, "accepted")?,
        rejected: take_number(&mut fields, "rejected")?,
        failed: take_number(&mut fields, "failed")?,
        cancelled: take_number(&mut fields, "cancelled")?,
        incomplete: take_number(&mut fields, "incomplete")?,
    };
    fields.finish()?;
    Some(outcomes)
}

fn read_measures(value: Value) -> Option<Measures> {
    let mut fields = Fields::new(value)?;
    let mut coverage = Fields::new(fields.take("task_coverage")?)?;
    let tasks_covered = take_number(&mut coverage, "covered")?;
    let tasks_declared = take_number(&mut coverage, "declared")?;
    coverage.finish()?;
    let mut time = Fields::new(fields.take("covered_time_ms")?)?;
    let covered_time = CoveredTime {
        queue_ms: take_optional_number(&mut time, "queue")?,
        repair_ms: take_optional_number(&mut time, "repair")?,
        verification_ms: take_optional_number(&mut time, "verification")?,
    };
    time.finish()?;
    let measures = Measures {
        repetitions: take_number(&mut fields, "repetitions")?,
        tasks_covered,
        tasks_declared,
        time_to_acceptance_ms: take_optional_number(&mut fields, "time_to_acceptance_ms")?,
        covered_time,
        costs: read_costs(fields.take("costs")?)?,
        review_interventions: take_optional_number(&mut fields, "review_interventions")?,
        invalid_actions: take_optional_number(&mut fields, "invalid_actions")?,
        false_completion_attempts: take_optional_number(&mut fields, "false_completion_attempts")?,
    };
    fields.finish()?;
    Some(measures)
}

fn read_costs(value: Value) -> Option<Costs> {
    let mut fields = Fields::new(value)?;
    let costs = Costs {
        unit: take_text(&mut fields, "unit")?,
        model: take_optional_number(&mut fields, "model")?,
        tool: take_optional_number(&mut fields, "tool")?,
        manager: take_optional_number(&mut fields, "manager")?,
        verifier: take_optional_number(&mut fields, "verifier")?,
        unknown_remainder: match fields.take("unknown_remainder")? {
            Value::Bool(flag) => flag,
            _ => return None,
        },
    };
    fields.finish()?;
    Some(costs)
}

fn read_scope(value: Value) -> Option<Scope> {
    let mut fields = Fields::new(value)?;
    let scope = Scope {
        partitions: list(fields.take("partitions")?, as_text)?,
        limitations: list(fields.take("limitations")?, as_text)?,
        missing_modes: list(fields.take("missing_modes")?, as_text)?,
    };
    fields.finish()?;
    Some(scope)
}

fn read_provenance(value: Value) -> Option<Provenance> {
    let mut fields = Fields::new(value)?;
    let label = match take_text(&mut fields, "label")?.as_str() {
        "fixture" => EvidenceLabel::Fixture,
        "live" => EvidenceLabel::Live,
        _ => return None,
    };
    let provenance = Provenance {
        label,
        producer: take_text(&mut fields, "producer")?,
        producer_revision: take_text(&mut fields, "producer_revision")?,
        observed_at_ms: take_number(&mut fields, "observed_at_ms")?,
        valid_until_ms: take_number(&mut fields, "valid_until_ms")?,
    };
    fields.finish()?;
    Some(provenance)
}

fn read_reference(value: Value) -> Option<Reference> {
    let mut fields = Fields::new(value)?;
    let reference = Reference {
        kind: take_text(&mut fields, "kind")?,
        identity: take_text(&mut fields, "identity")?,
    };
    fields.finish()?;
    Some(reference)
}

fn as_text(value: Value) -> Option<String> {
    match value {
        Value::Text(text) => Some(text),
        _ => None,
    }
}

fn take_text(fields: &mut Fields, key: &str) -> Option<String> {
    as_text(fields.take(key)?)
}

fn take_number(fields: &mut Fields, key: &str) -> Option<u64> {
    match fields.take(key)? {
        Value::Number(number) => Some(number),
        _ => None,
    }
}

#[expect(
    clippy::option_option,
    reason = "the outer `None` is a shape error and the inner `None` is a JSON null"
)]
fn take_optional_number(fields: &mut Fields, key: &str) -> Option<Option<u64>> {
    match fields.take(key)? {
        Value::Null => Some(None),
        Value::Number(number) => Some(Some(number)),
        _ => None,
    }
}

/// `Some(None)` for null, `Some(Some(_))` for a readable value, `None` otherwise.
#[expect(
    clippy::option_option,
    reason = "the outer `None` is a shape error and the inner `None` is a JSON null"
)]
fn optional<T>(value: Value, read: impl FnOnce(Value) -> Option<T>) -> Option<Option<T>> {
    match value {
        Value::Null => Some(None),
        other => read(other).map(Some),
    }
}

fn list<T>(value: Value, read: impl FnMut(Value) -> Option<T>) -> Option<Vec<T>> {
    match value {
        Value::List(items) => items.into_iter().map(read).collect(),
        _ => None,
    }
}
