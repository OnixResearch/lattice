#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "lifecycle planning uses ordered sets, maps, and the local typed error throughout"
)]
#![allow(
    assertion_density,
    reason = "untrusted retention and projection facts return typed diagnostics instead of terminating the process"
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::core::Error;
use super::model::*;

const DELETION_DOMAIN: &str = "lattice.operation-data-deletion-plan.v1";
const PROJECTION_DOMAIN: &str = "lattice.operation-data-recorded-only.v1";
const RECORDED_ONLY_ROLE: &str = "RecordedOnly";
const RETENTION_REASON: &str = "declared-retention";
const ORPHAN_REASON: &str = "orphan-preview";

pub fn retention_plan(records: &[RecordMetadata], facts: &RetentionFacts) -> Result<DeletionPlan, Error> {
    validate_candidates(records, "retention candidates")?;
    if facts.terminal_run_ids.len() > MAX_QUERY_CANDIDATES {
        return Err(Error::Limit("terminal run facts"));
    }
    let terminal: BTreeSet<u64> = facts.terminal_run_ids.iter().copied().collect();
    let mut selected = BTreeSet::new();
    for record in records {
        let is_expired = match record.lifetime {
            Lifetime::Ephemeral => true,
            Lifetime::Run { run_id } => terminal.contains(&run_id),
            Lifetime::Duration { expires_epoch_seconds } => expires_epoch_seconds <= facts.now_epoch_seconds,
            Lifetime::Retained => false,
        };
        if is_expired {
            selected.insert(record.version_identity.clone());
        }
    }
    let mut groups: BTreeMap<&str, Vec<&RecordMetadata>> = BTreeMap::new();
    for record in records {
        groups.entry(&record.record_identity).or_default().push(record);
    }
    for versions in groups.values_mut() {
        versions.sort_by_key(|record| std::cmp::Reverse(record.sequence));
        let Some(maximum) = versions.first().and_then(|record| record.retention.maximum_versions) else {
            continue;
        };
        let keep = usize::try_from(maximum).map_err(|_| Error::Limit("maximum versions"))?;
        for record in versions.iter().skip(keep) {
            selected.insert(record.version_identity.clone());
        }
    }
    deletion_plan(selected.into_iter().collect(), RETENTION_REASON)
}

pub fn orphan_prune_plan(
    records: &[RecordMetadata],
    admitted_types: &BTreeSet<String>,
    admitted_instances: &BTreeSet<String>,
) -> Result<DeletionPlan, Error> {
    validate_candidates(records, "orphan candidates")?;
    let mut selected = records
        .iter()
        .filter(|record| {
            !admitted_types.contains(&record.producer.operation_type)
                || !admitted_instances.contains(&record.producer.instance)
        })
        .map(|record| record.version_identity.clone())
        .collect::<Vec<_>>();
    selected.sort();
    selected.dedup();
    deletion_plan(selected, ORPHAN_REASON)
}

pub fn project_recorded_only(metadata: &RecordMetadata) -> Result<RecordedOnlyProjection, Error> {
    super::core::validate_metadata(metadata)?;
    if metadata.sensitivity == SensitivityClass::Sensitive {
        return Err(Error::Secret);
    }
    let mut non_claims = metadata.required_non_claims.clone();
    if !non_claims.iter().any(|value| value == PROJECTION_NON_CLAIM) {
        non_claims.push(PROJECTION_NON_CLAIM.to_string());
    }
    non_claims.sort();
    let identity = super::core::hash_value(
        PROJECTION_DOMAIN,
        &(
            &metadata.record_identity,
            &metadata.version_identity,
            &metadata.producer.invocation,
            &metadata.payload.schema_identity,
            &metadata.payload.content_identity,
            metadata.sensitivity,
            &non_claims,
        ),
    )?;
    let projection = RecordedOnlyProjection {
        schema: PROJECTION_SCHEMA.to_string(),
        identity,
        verification_role: RECORDED_ONLY_ROLE.to_string(),
        record_identity: metadata.record_identity.clone(),
        version_identity: metadata.version_identity.clone(),
        producer_invocation: metadata.producer.invocation.clone(),
        schema_identity: metadata.payload.schema_identity.clone(),
        content_identity: metadata.payload.content_identity.clone(),
        sensitivity: metadata.sensitivity,
        non_claims,
    };
    validate_recorded_only_projection(&projection)?;
    Ok(projection)
}

pub fn validate_deletion_plan(plan: &DeletionPlan) -> Result<(), Error> {
    if plan.secure_erasure_claimed || !matches!(plan.reason.as_str(), RETENTION_REASON | ORPHAN_REASON) {
        return Err(Error::Invalid("deletion plan policy"));
    }
    if plan.selected_versions.len() > MAX_QUERY_CANDIDATES {
        return Err(Error::Limit("deletion plan versions"));
    }
    if plan.selected_versions.windows(2).any(|window| window[0] >= window[1]) {
        return Err(Error::Invalid("deletion plan order"));
    }
    for version in &plan.selected_versions {
        super::core::validate_identity(version)?;
    }
    let expected = super::core::hash_value(DELETION_DOMAIN, &(&plan.reason, &plan.selected_versions))?;
    if plan.identity != expected {
        return Err(Error::Invalid("deletion plan identity"));
    }
    Ok(())
}

pub fn validate_recorded_only_projection(projection: &RecordedOnlyProjection) -> Result<(), Error> {
    if projection.schema != PROJECTION_SCHEMA
        || projection.verification_role != RECORDED_ONLY_ROLE
        || projection.sensitivity == SensitivityClass::Sensitive
    {
        return Err(Error::Invalid("recorded-only projection policy"));
    }
    for identity in [
        &projection.identity,
        &projection.record_identity,
        &projection.version_identity,
        &projection.producer_invocation,
        &projection.content_identity,
    ] {
        super::core::validate_identity(identity)?;
    }
    if let Some(schema) = &projection.schema_identity {
        super::core::validate_identity(schema)?;
    }
    if !projection.non_claims.iter().any(|value| value == PROJECTION_NON_CLAIM) {
        return Err(Error::Invalid("projection non-claim"));
    }
    for required in REQUIRED_NON_CLAIMS {
        if !projection.non_claims.iter().any(|value| value == required) {
            return Err(Error::Invalid("projection source non-claim"));
        }
    }
    if projection.non_claims.len() > MAX_NON_CLAIMS.saturating_add(1)
        || projection.non_claims.windows(2).any(|window| window[0] >= window[1])
    {
        return Err(Error::Invalid("projection non-claim order"));
    }
    for value in &projection.non_claims {
        super::core::validate_text(value)?;
    }
    let expected = super::core::hash_value(
        PROJECTION_DOMAIN,
        &(
            &projection.record_identity,
            &projection.version_identity,
            &projection.producer_invocation,
            &projection.schema_identity,
            &projection.content_identity,
            projection.sensitivity,
            &projection.non_claims,
        ),
    )?;
    if projection.identity != expected {
        return Err(Error::Invalid("projection identity"));
    }
    Ok(())
}

fn validate_candidates(records: &[RecordMetadata], label: &'static str) -> Result<(), Error> {
    if records.len() > MAX_QUERY_CANDIDATES {
        return Err(Error::Limit(label));
    }
    for record in records {
        super::core::validate_metadata(record)?;
    }
    Ok(())
}

fn deletion_plan(mut selected_versions: Vec<String>, reason: &str) -> Result<DeletionPlan, Error> {
    selected_versions.sort();
    selected_versions.dedup();
    let identity = super::core::hash_value(DELETION_DOMAIN, &(reason, &selected_versions))?;
    let plan = DeletionPlan {
        identity,
        selected_versions,
        reason: reason.to_string(),
        secure_erasure_claimed: false,
    };
    validate_deletion_plan(&plan)?;
    Ok(plan)
}
