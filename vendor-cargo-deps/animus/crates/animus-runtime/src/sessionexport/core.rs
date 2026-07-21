//! Pure policy validation and redacted tree projection.

type OrderedMap<K, V> = std::collections::BTreeMap<K, V>;
type OrderedSet<T> = std::collections::BTreeSet<T>;
type SessionTreeExportEntry = super::data::SessionTreeExportEntry;
type SessionTreeExportEntryInput = super::data::SessionTreeExportEntryInput;
type SessionTreeExportEntryKind = super::data::SessionTreeExportEntryKind;
type SessionTreeExportError = super::data::SessionTreeExportError;
type SessionTreeExportFormat = super::data::SessionTreeExportFormat;
type SessionTreeExportIssue = super::data::SessionTreeExportIssue;
type SessionTreeExportPolicy = super::data::SessionTreeExportPolicy;
type SessionTreeExportPolicyInput = super::data::SessionTreeExportPolicyInput;
type SessionTreeExportProjection = super::data::SessionTreeExportProjection;
type SessionTreeExportSourceInput = super::data::SessionTreeExportSourceInput;

const BLAKE3_HEX_CHARS: usize = super::data::BLAKE3_HEX_CHARS;
const HASH_SEPARATOR: &str = super::data::HASH_SEPARATOR;
const MAX_ARTIFACT_BYTES: u32 = super::data::MAX_ARTIFACT_BYTES;
const MAX_DEPTH: u32 = super::data::MAX_DEPTH;
const MAX_ENTRY_COUNT: u32 = super::data::MAX_ENTRY_COUNT;
const MAX_RECEIPT_REFERENCE_COUNT: u32 = super::data::MAX_RECEIPT_REFERENCE_COUNT;
const MAX_SUMMARY_CHARS: u32 = super::data::MAX_SUMMARY_CHARS;
const MIN_COUNT: u32 = super::data::MIN_COUNT;
const SESSION_TREE_EXPORT_POLICY_HASH_DOMAIN: &str =
    super::data::SESSION_TREE_EXPORT_POLICY_HASH_DOMAIN;
const SESSION_TREE_EXPORT_POLICY_SCHEMA: &str = super::data::SESSION_TREE_EXPORT_POLICY_SCHEMA;
const SESSION_TREE_EXPORT_SOURCE_HASH_DOMAIN: &str =
    super::data::SESSION_TREE_EXPORT_SOURCE_HASH_DOMAIN;

#[derive(Clone, Copy)]
pub(super) struct ExportField(pub(super) &'static str);

const FIELD_ACTIVE_LEAF: ExportField = ExportField("session_tree_export_active_leaf");
const FIELD_ENTRY: ExportField = ExportField("session_tree_export_entry");
const FIELD_POLICY: ExportField = ExportField("session_tree_export_policy");
const FIELD_SOURCE: ExportField = ExportField("session_tree_export_source");
const FIELD_TOPOLOGY: ExportField = ExportField("session_tree_export_topology");
const REQUIRED_BRANCH_VISIBILITY: &str = "all_redacted";
const REQUIRED_TOOL_METADATA_POSTURE: &str = "status_only";
const REQUIRED_CUSTOM_ENTRY_POSTURE: &str = "reject";
const REQUIRED_CONTENT_POSTURE: &str = "metadata_only";
const REQUIRED_PATH_POSTURE: &str = "omit";
const REQUIRED_MALFORMED_POSTURE: &str = "reject";
const REQUIRED_SCRIPT_POSTURE: &str = "forbidden";
const REQUIRED_REMOTE_POSTURE: &str = "forbidden";
const REQUIRED_RECEIPT_FIELDS: &[&str] = &[
    "source_identity",
    "projection_identity",
    "artifact_identity",
    "counts",
    "format",
    "byte_count",
    "redaction",
    "write_status",
    "non_claim",
];
const PRIVATE_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "RAW_MODEL_OUTPUT:",
    "RAW_TOOL_BODY:",
    "PROVIDER_PAYLOAD:",
    "HIDDEN_REASONING:",
    "ENVIRONMENT:",
    "RAW_ERROR:",
    "PRIVATE_CUSTOM_ENTRY:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "authorization:",
    "/home/",
    "UNBOUNDED_LOG:",
];

/// Revalidates a generated redacted export policy and its BLAKE3 identity.
///
/// # Errors
///
/// Returns [`SessionTreeExportError`] for stale identity, unsupported posture,
/// duplicates, unknown classes, or invalid bounds.
// r[impl agent_harness_runtime.session_tree_export.policy]
pub fn validate_session_tree_export_policy(
    input: &SessionTreeExportPolicyInput,
) -> Result<SessionTreeExportPolicy, SessionTreeExportError> {
    validate_policy_shape(input)?;
    let allowed_entry_kinds = parse_entry_kinds(input.allowed_entry_kinds.as_slice())?;
    let formats = parse_formats(input.formats.as_slice())?;
    let expected_hash = session_tree_export_policy_content_hash(input);
    if expected_hash != input.policy_hash {
        return Err(error(SessionTreeExportIssue::StalePolicy, FIELD_POLICY));
    }
    let policy = SessionTreeExportPolicy {
        policy_id: input.policy_id.clone(),
        max_entries: input.max_entries,
        max_depth: input.max_depth,
        max_summary_chars: input.max_summary_chars,
        max_html_bytes: input.max_html_bytes,
        max_safe_lines_bytes: input.max_safe_lines_bytes,
        allowed_entry_kinds,
        formats,
        branch_visibility: input.branch_visibility.clone(),
        tool_metadata_posture: input.tool_metadata_posture.clone(),
        custom_entry_posture: input.custom_entry_posture.clone(),
        receipt_fields: input.receipt_fields.clone(),
        policy_hash: input.policy_hash.clone(),
    };
    debug_assert!(!policy.allowed_entry_kinds.is_empty());
    debug_assert!(!policy.formats.is_empty());
    Ok(policy)
}

/// Computes canonical generated-policy content identity.
#[must_use]
pub fn session_tree_export_policy_content_hash(input: &SessionTreeExportPolicyInput) -> String {
    debug_assert!(!input.schema.is_empty());
    debug_assert!(!input.policy_id.is_empty());
    let mut parts = vec![
        input.schema.as_str(),
        input.policy_id.as_str(),
        input.content_posture.as_str(),
        input.path_posture.as_str(),
        input.malformed_input.as_str(),
        input.scripts.as_str(),
        input.remote_resources.as_str(),
        input.branch_visibility.as_str(),
        input.tool_metadata_posture.as_str(),
        input.custom_entry_posture.as_str(),
    ];
    let max_entries = input.max_entries.to_string();
    let max_depth = input.max_depth.to_string();
    let max_summary_chars = input.max_summary_chars.to_string();
    let max_html_bytes = input.max_html_bytes.to_string();
    let max_safe_lines_bytes = input.max_safe_lines_bytes.to_string();
    parts.extend([
        max_entries.as_str(),
        max_depth.as_str(),
        max_summary_chars.as_str(),
        max_html_bytes.as_str(),
        max_safe_lines_bytes.as_str(),
    ]);
    parts.extend(input.allowed_entry_kinds.iter().map(String::as_str));
    parts.extend(input.formats.iter().map(String::as_str));
    parts.extend(input.receipt_fields.iter().map(String::as_str));
    hash_parts(SESSION_TREE_EXPORT_POLICY_HASH_DOMAIN, parts.as_slice())
}

/// Validates and projects already-loaded session metadata into a redacted tree.
///
/// # Errors
///
/// Returns [`SessionTreeExportError`] for unsafe metadata, malformed topology,
/// stale active leaf, cycles, duplicates, unsupported kinds, or exceeded bounds.
// r[impl agent_harness_runtime.session_tree_export.source]
// r[impl agent_harness_runtime.session_tree_export.projection]
pub fn project_session_tree_export(
    policy: &SessionTreeExportPolicy,
    source: &SessionTreeExportSourceInput,
) -> Result<SessionTreeExportProjection, SessionTreeExportError> {
    validate_source_header(policy, source)?;
    let entries_by_id = validate_entries(policy, source.entries.as_slice())?;
    validate_parent_references(&entries_by_id)?;
    if !entries_by_id.contains_key(source.active_leaf_id.as_str()) {
        return Err(error(
            SessionTreeExportIssue::StaleActiveLeaf,
            FIELD_ACTIVE_LEAF,
        ));
    }
    let active_path = parent_chain(
        source.active_leaf_id.as_str(),
        &entries_by_id,
        policy.max_depth,
    )?;
    let child_counts = child_counts(&entries_by_id)?;
    let projected_entries = project_entries(&EntryProjectionInput {
        policy,
        source,
        entries_by_id: &entries_by_id,
        child_counts: &child_counts,
        active_path: &active_path,
    })?;
    debug_assert_eq!(entries_by_id.len(), source.entries.len());
    debug_assert!(!active_path.is_empty());
    let source_hash = source_hash(source);
    let active_path_count = u32::try_from(active_path.len())
        .map_err(|_error| error(SessionTreeExportIssue::BoundExceeded, FIELD_TOPOLOGY))?;
    let projection_hash = projection_hash(&ProjectionHashInput {
        session_id: source.session_id.as_str(),
        active_leaf_id: source.active_leaf_id.as_str(),
        source_hash: source_hash.as_str(),
        policy_hash: policy.policy_hash.as_str(),
        entries: projected_entries.entries.as_slice(),
    });
    Ok(SessionTreeExportProjection {
        session_id: source.session_id.clone(),
        active_leaf_id: source.active_leaf_id.clone(),
        source_hash,
        policy_hash: policy.policy_hash.clone(),
        entries: projected_entries.entries,
        active_path_count,
        branch_point_count: projected_entries.branch_point_count,
        omission_count: projected_entries.omission_count,
        projection_hash,
    })
}

struct EntryProjectionInput<'a> {
    policy: &'a SessionTreeExportPolicy,
    source: &'a SessionTreeExportSourceInput,
    entries_by_id: &'a OrderedMap<&'a str, &'a SessionTreeExportEntryInput>,
    child_counts: &'a OrderedMap<&'a str, u32>,
    active_path: &'a OrderedSet<String>,
}

struct ProjectedEntries {
    entries: Vec<SessionTreeExportEntry>,
    branch_point_count: u32,
    omission_count: u32,
}

fn project_entries(
    input: &EntryProjectionInput<'_>,
) -> Result<ProjectedEntries, SessionTreeExportError> {
    let mut entries = Vec::with_capacity(input.source.entries.len());
    let mut branch_point_count = 0_u32;
    let mut omission_count = 0_u32;
    for entry in &input.source.entries {
        let depth = entry_depth(
            entry.entry_id.as_str(),
            input.entries_by_id,
            input.policy.max_depth,
        )?;
        let child_count = match input.child_counts.get(entry.entry_id.as_str()) {
            Some(count) => *count,
            None => 0_u32,
        };
        if child_count > MIN_COUNT {
            branch_point_count = checked_add(CheckedAddInput {
                value: branch_point_count,
                increment: MIN_COUNT,
            })?;
        }
        omission_count = checked_add(CheckedAddInput {
            value: omission_count,
            increment: entry.omitted_field_count,
        })?;
        entries.push(SessionTreeExportEntry {
            entry_id: entry.entry_id.clone(),
            parent_id: entry.parent_id.clone(),
            storage_sequence: entry.storage_sequence,
            depth,
            child_count,
            kind: entry.kind,
            repair_target_id: entry.repair_target_id.clone(),
            safe_summary: entry.safe_summary.clone(),
            entry_hash: entry.entry_hash.clone(),
            status: entry.status.clone(),
            omitted_field_count: entry.omitted_field_count,
            is_active_path: input.active_path.contains(entry.entry_id.as_str()),
        });
    }
    debug_assert_eq!(entries.len(), input.source.entries.len());
    debug_assert!(!input.active_path.is_empty());
    Ok(ProjectedEntries {
        entries,
        branch_point_count,
        omission_count,
    })
}

fn validate_policy_shape(
    input: &SessionTreeExportPolicyInput,
) -> Result<(), SessionTreeExportError> {
    let has_required_postures = input.schema == SESSION_TREE_EXPORT_POLICY_SCHEMA
        && input.content_posture == REQUIRED_CONTENT_POSTURE
        && input.path_posture == REQUIRED_PATH_POSTURE;
    let has_safe_artifact_posture = input.malformed_input == REQUIRED_MALFORMED_POSTURE
        && input.scripts == REQUIRED_SCRIPT_POSTURE
        && input.remote_resources == REQUIRED_REMOTE_POSTURE;
    let has_safe_projection_posture = input.branch_visibility == REQUIRED_BRANCH_VISIBILITY
        && input.tool_metadata_posture == REQUIRED_TOOL_METADATA_POSTURE
        && input.custom_entry_posture == REQUIRED_CUSTOM_ENTRY_POSTURE;
    if !has_required_postures || !has_safe_artifact_posture || !has_safe_projection_posture {
        return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
    }
    safe_label(input.policy_id.as_str(), FIELD_POLICY)?;
    lowercase_blake3(input.policy_hash.as_str(), FIELD_POLICY)?;
    validate_policy_bounds(input)?;
    validate_receipt_fields(input.receipt_fields.as_slice())?;
    Ok(())
}

fn validate_receipt_fields(fields: &[String]) -> Result<(), SessionTreeExportError> {
    let has_exact_fields = fields.len() == REQUIRED_RECEIPT_FIELDS.len()
        && fields
            .iter()
            .map(String::as_str)
            .eq(REQUIRED_RECEIPT_FIELDS.iter().copied());
    if !has_exact_fields {
        return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
    }
    debug_assert!(!fields.is_empty());
    debug_assert_eq!(fields.len(), REQUIRED_RECEIPT_FIELDS.len());
    Ok(())
}

fn validate_policy_bounds(
    input: &SessionTreeExportPolicyInput,
) -> Result<(), SessionTreeExportError> {
    let is_entry_bound_invalid =
        input.max_entries < MIN_COUNT || input.max_entries > MAX_ENTRY_COUNT;
    let is_depth_bound_invalid = input.max_depth < MIN_COUNT || input.max_depth > MAX_DEPTH;
    let is_summary_bound_invalid =
        input.max_summary_chars < MIN_COUNT || input.max_summary_chars > MAX_SUMMARY_CHARS;
    if is_entry_bound_invalid || is_depth_bound_invalid {
        return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
    }
    if is_summary_bound_invalid {
        return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
    }
    validate_artifact_bound(input.max_html_bytes)?;
    validate_artifact_bound(input.max_safe_lines_bytes)
}

fn validate_artifact_bound(max_bytes: u32) -> Result<(), SessionTreeExportError> {
    if (MIN_COUNT..=MAX_ARTIFACT_BYTES).contains(&max_bytes) {
        Ok(())
    } else {
        Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY))
    }
}

fn parse_entry_kinds(
    labels: &[String],
) -> Result<Vec<SessionTreeExportEntryKind>, SessionTreeExportError> {
    if labels.is_empty() {
        return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
    }
    let mut seen = OrderedSet::new();
    let mut kinds = Vec::with_capacity(labels.len());
    for label in labels {
        let Some(kind) = SessionTreeExportEntryKind::parse(label.as_str()) else {
            return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
        };
        if !seen.insert(kind) {
            return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
        }
        kinds.push(kind);
    }
    Ok(kinds)
}

fn parse_formats(
    labels: &[String],
) -> Result<Vec<SessionTreeExportFormat>, SessionTreeExportError> {
    if labels.is_empty() {
        return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
    }
    let mut seen = OrderedSet::new();
    let mut formats = Vec::with_capacity(labels.len());
    for label in labels {
        let Some(format) = SessionTreeExportFormat::parse(label.as_str()) else {
            return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
        };
        if !seen.insert(format) {
            return Err(error(SessionTreeExportIssue::InvalidPolicy, FIELD_POLICY));
        }
        formats.push(format);
    }
    Ok(formats)
}

fn validate_source_header(
    policy: &SessionTreeExportPolicy,
    source: &SessionTreeExportSourceInput,
) -> Result<(), SessionTreeExportError> {
    safe_label(source.session_id.as_str(), FIELD_SOURCE)?;
    safe_label(source.active_leaf_id.as_str(), FIELD_ACTIVE_LEAF)?;
    lowercase_blake3(source.snapshot_hash.as_str(), FIELD_SOURCE)?;
    lowercase_blake3(source.replay_hash.as_str(), FIELD_SOURCE)?;
    let entry_count = u32::try_from(source.entries.len())
        .map_err(|_error| error(SessionTreeExportIssue::BoundExceeded, FIELD_SOURCE))?;
    let receipt_count = u32::try_from(source.safe_receipt_hashes.len())
        .map_err(|_error| error(SessionTreeExportIssue::BoundExceeded, FIELD_SOURCE))?;
    if entry_count < MIN_COUNT || entry_count > policy.max_entries {
        return Err(error(SessionTreeExportIssue::BoundExceeded, FIELD_SOURCE));
    }
    if receipt_count > MAX_RECEIPT_REFERENCE_COUNT {
        return Err(error(SessionTreeExportIssue::BoundExceeded, FIELD_SOURCE));
    }
    for receipt_hash in &source.safe_receipt_hashes {
        lowercase_blake3(receipt_hash.as_str(), FIELD_SOURCE)?;
    }
    debug_assert!(entry_count >= MIN_COUNT);
    debug_assert!(entry_count <= policy.max_entries);
    Ok(())
}

fn validate_entries<'a>(
    policy: &SessionTreeExportPolicy,
    entries: &'a [SessionTreeExportEntryInput],
) -> Result<OrderedMap<&'a str, &'a SessionTreeExportEntryInput>, SessionTreeExportError> {
    let mut by_id = OrderedMap::new();
    let mut sequences = OrderedSet::new();
    for entry in entries {
        validate_entry(policy, entry)?;
        if by_id.insert(entry.entry_id.as_str(), entry).is_some() {
            return Err(error(SessionTreeExportIssue::DuplicateEntry, FIELD_ENTRY));
        }
        if !sequences.insert(entry.storage_sequence) {
            return Err(error(SessionTreeExportIssue::DuplicateEntry, FIELD_ENTRY));
        }
    }
    let mut expected_sequence = MIN_COUNT;
    for entry in entries {
        if entry.storage_sequence != expected_sequence {
            return Err(error(SessionTreeExportIssue::InvalidSource, FIELD_ENTRY));
        }
        expected_sequence = checked_add(CheckedAddInput {
            value: expected_sequence,
            increment: MIN_COUNT,
        })?;
    }
    debug_assert_eq!(by_id.len(), entries.len());
    debug_assert_eq!(sequences.len(), entries.len());
    Ok(by_id)
}

fn validate_entry(
    policy: &SessionTreeExportPolicy,
    entry: &SessionTreeExportEntryInput,
) -> Result<(), SessionTreeExportError> {
    safe_label(entry.entry_id.as_str(), FIELD_ENTRY)?;
    if let Some(parent_id) = &entry.parent_id {
        safe_label(parent_id.as_str(), FIELD_ENTRY)?;
    }
    safe_label(entry.status.as_str(), FIELD_ENTRY)?;
    if let Some(repair_target_id) = &entry.repair_target_id {
        safe_label(repair_target_id.as_str(), FIELD_ENTRY)?;
    }
    lowercase_blake3(entry.entry_hash.as_str(), FIELD_ENTRY)?;
    if !policy.allowed_entry_kinds.contains(&entry.kind) {
        return Err(error(SessionTreeExportIssue::InvalidSource, FIELD_ENTRY));
    }
    let summary_chars = u32::try_from(entry.safe_summary.chars().count())
        .map_err(|_error| error(SessionTreeExportIssue::BoundExceeded, FIELD_ENTRY))?;
    if summary_chars > policy.max_summary_chars || contains_private_marker(&entry.safe_summary) {
        return Err(error(SessionTreeExportIssue::InvalidSource, FIELD_ENTRY));
    }
    debug_assert!(policy.allowed_entry_kinds.contains(&entry.kind));
    debug_assert!(summary_chars <= policy.max_summary_chars);
    Ok(())
}

fn validate_parent_references(
    entries: &OrderedMap<&str, &SessionTreeExportEntryInput>,
) -> Result<(), SessionTreeExportError> {
    let root_count = entries
        .values()
        .filter(|entry| entry.parent_id.is_none())
        .count();
    if root_count != 1 {
        return Err(error(SessionTreeExportIssue::InvalidSource, FIELD_TOPOLOGY));
    }
    for entry in entries.values() {
        if let Some(parent_id) = &entry.parent_id {
            if !entries.contains_key(parent_id.as_str()) {
                return Err(error(SessionTreeExportIssue::MissingParent, FIELD_TOPOLOGY));
            }
            if parent_id == &entry.entry_id {
                return Err(error(SessionTreeExportIssue::Cycle, FIELD_TOPOLOGY));
            }
        }
        validate_repair_target(entry, entries)?;
    }
    debug_assert_eq!(root_count, 1);
    debug_assert!(!entries.is_empty());
    Ok(())
}

fn validate_repair_target(
    entry: &SessionTreeExportEntryInput,
    entries: &OrderedMap<&str, &SessionTreeExportEntryInput>,
) -> Result<(), SessionTreeExportError> {
    if entry.kind != SessionTreeExportEntryKind::Repair {
        if entry.repair_target_id.is_some() {
            return Err(error(SessionTreeExportIssue::InvalidSource, FIELD_ENTRY));
        }
        return Ok(());
    }
    let target = entry
        .repair_target_id
        .as_deref()
        .and_then(|target_id| entries.get(target_id).copied())
        .ok_or_else(|| error(SessionTreeExportIssue::StaleRepairTarget, FIELD_ENTRY))?;
    if target.entry_id == entry.entry_id || target.storage_sequence >= entry.storage_sequence {
        return Err(error(
            SessionTreeExportIssue::StaleRepairTarget,
            FIELD_ENTRY,
        ));
    }
    debug_assert!(target.storage_sequence < entry.storage_sequence);
    Ok(())
}

fn parent_chain(
    start_id: &str,
    entries: &OrderedMap<&str, &SessionTreeExportEntryInput>,
    max_depth: u32,
) -> Result<OrderedSet<String>, SessionTreeExportError> {
    let mut path = OrderedSet::new();
    let mut current = Some(start_id);
    let mut depth = 0_u32;
    while let Some(entry_id) = current {
        if !path.insert(String::from(entry_id)) {
            return Err(error(SessionTreeExportIssue::Cycle, FIELD_TOPOLOGY));
        }
        depth = checked_add(CheckedAddInput {
            value: depth,
            increment: MIN_COUNT,
        })?;
        if depth > max_depth {
            return Err(error(SessionTreeExportIssue::BoundExceeded, FIELD_TOPOLOGY));
        }
        let entry = entries
            .get(entry_id)
            .ok_or_else(|| error(SessionTreeExportIssue::MissingParent, FIELD_TOPOLOGY))?;
        current = entry.parent_id.as_deref();
    }
    debug_assert!(!path.is_empty());
    debug_assert!(depth <= max_depth);
    Ok(path)
}

fn entry_depth(
    entry_id: &str,
    entries: &OrderedMap<&str, &SessionTreeExportEntryInput>,
    max_depth: u32,
) -> Result<u32, SessionTreeExportError> {
    let path = parent_chain(entry_id, entries, max_depth)?;
    let count = u32::try_from(path.len())
        .map_err(|_error| error(SessionTreeExportIssue::BoundExceeded, FIELD_TOPOLOGY))?;
    count
        .checked_sub(MIN_COUNT)
        .ok_or_else(|| error(SessionTreeExportIssue::InvalidSource, FIELD_TOPOLOGY))
}

fn child_counts<'a>(
    entries: &OrderedMap<&'a str, &'a SessionTreeExportEntryInput>,
) -> Result<OrderedMap<&'a str, u32>, SessionTreeExportError> {
    let mut counts: OrderedMap<&str, u32> =
        entries.keys().map(|entry_id| (*entry_id, 0_u32)).collect();
    for entry in entries.values() {
        if let Some(parent_id) = &entry.parent_id {
            let count = counts
                .get_mut(parent_id.as_str())
                .ok_or_else(|| error(SessionTreeExportIssue::MissingParent, FIELD_TOPOLOGY))?;
            *count = checked_add(CheckedAddInput {
                value: *count,
                increment: MIN_COUNT,
            })?;
        }
    }
    Ok(counts)
}

fn source_hash(source: &SessionTreeExportSourceInput) -> String {
    debug_assert!(!source.entries.is_empty());
    debug_assert!(!source.session_id.is_empty());
    let mut owned = vec![
        source.session_id.clone(),
        source.snapshot_hash.clone(),
        source.replay_hash.clone(),
        source.active_leaf_id.clone(),
    ];
    for entry in &source.entries {
        owned.push(entry.entry_id.clone());
        owned.push(optional_text(entry.parent_id.as_deref()));
        owned.push(entry.storage_sequence.to_string());
        owned.push(String::from(entry.kind.as_str()));
        owned.push(optional_text(entry.repair_target_id.as_deref()));
        owned.push(entry.safe_summary.clone());
        owned.push(entry.entry_hash.clone());
        owned.push(entry.status.clone());
        owned.push(entry.omitted_field_count.to_string());
    }
    owned.extend(source.safe_receipt_hashes.iter().cloned());
    let parts: Vec<&str> = owned.iter().map(String::as_str).collect();
    hash_parts(SESSION_TREE_EXPORT_SOURCE_HASH_DOMAIN, parts.as_slice())
}

#[derive(Clone, Copy)]
struct CheckedAddInput {
    value: u32,
    increment: u32,
}

struct ProjectionHashInput<'a> {
    session_id: &'a str,
    active_leaf_id: &'a str,
    source_hash: &'a str,
    policy_hash: &'a str,
    entries: &'a [SessionTreeExportEntry],
}

fn projection_hash(input: &ProjectionHashInput<'_>) -> String {
    debug_assert!(!input.entries.is_empty());
    debug_assert!(!input.active_leaf_id.is_empty());
    let mut owned = vec![
        String::from(input.session_id),
        String::from(input.active_leaf_id),
        String::from(input.source_hash),
        String::from(input.policy_hash),
    ];
    for entry in input.entries {
        owned.push(entry.entry_id.clone());
        owned.push(optional_text(entry.parent_id.as_deref()));
        owned.push(entry.storage_sequence.to_string());
        owned.push(entry.depth.to_string());
        owned.push(entry.child_count.to_string());
        owned.push(String::from(entry.kind.as_str()));
        owned.push(optional_text(entry.repair_target_id.as_deref()));
        owned.push(entry.safe_summary.clone());
        owned.push(entry.entry_hash.clone());
        owned.push(entry.status.clone());
        owned.push(entry.omitted_field_count.to_string());
        owned.push(entry.is_active_path.to_string());
    }
    let parts: Vec<&str> = owned.iter().map(String::as_str).collect();
    hash_parts(
        super::data::SESSION_TREE_EXPORT_PROJECTION_HASH_DOMAIN,
        parts.as_slice(),
    )
}

pub(super) fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        let part_length_bytes = part.len().to_string();
        hasher.update(HASH_SEPARATOR.as_bytes());
        hasher.update(part_length_bytes.as_bytes());
        hasher.update(b":");
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

pub(super) fn safe_label(value: &str, field: ExportField) -> Result<(), SessionTreeExportError> {
    let character_count = u32::try_from(value.chars().count())
        .map_err(|_error| error(SessionTreeExportIssue::BoundExceeded, field))?;
    let is_label_bounded = !value.is_empty() && character_count <= MAX_SUMMARY_CHARS;
    let has_safe_chars = value.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | ':' | '-')
    });
    if !is_label_bounded || !has_safe_chars {
        Err(error(SessionTreeExportIssue::InvalidSource, field))
    } else {
        Ok(())
    }
}

pub(super) fn lowercase_blake3(
    value: &str,
    field: ExportField,
) -> Result<(), SessionTreeExportError> {
    let is_hash_width_valid = value.len() == BLAKE3_HEX_CHARS;
    let is_lowercase_hex = value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_hash_width_valid || !is_lowercase_hex {
        Err(error(SessionTreeExportIssue::InvalidSource, field))
    } else {
        Ok(())
    }
}

fn optional_text(value: Option<&str>) -> String {
    match value {
        Some(text) => String::from(text),
        None => String::new(),
    }
}

fn contains_private_marker(value: &str) -> bool {
    PRIVATE_MARKERS.iter().any(|marker| {
        value
            .to_ascii_lowercase()
            .contains(&marker.to_ascii_lowercase())
    })
}

fn checked_add(input: CheckedAddInput) -> Result<u32, SessionTreeExportError> {
    input
        .value
        .checked_add(input.increment)
        .ok_or_else(|| error(SessionTreeExportIssue::BoundExceeded, FIELD_TOPOLOGY))
}

fn error(issue: SessionTreeExportIssue, field: ExportField) -> SessionTreeExportError {
    SessionTreeExportError::new(issue, field.0)
}
