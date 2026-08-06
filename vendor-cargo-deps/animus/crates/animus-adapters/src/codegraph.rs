//! Pure transport-neutral codebase-graph contracts and deterministic reducers.

use alloc::string::ToString;

const PROFILE_DOMAIN: &str = "animus.codebase-graph.profile.v1";
const GRAPH_DOMAIN: &str = "animus.codebase-graph.index.v1";
const CACHE_DOMAIN: &str = "animus.codebase-graph.cache.v1";
const RECEIPT_DOMAIN: &str = "animus.codebase-graph.receipt.v1";
const BLAKE3_HEX_LENGTH: usize = 64;
const MAX_SAFE_LABEL_BYTES: usize = 128;

/// Supported syntactic symbol class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CodeGraphSymbolKind {
    /// A syntactic definition.
    Definition,
    /// A syntactic reference.
    Reference,
}

/// Validated finite codebase-graph profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeGraphProfile {
    /// Stable profile label.
    pub profile_id: alloc::string::String,
    /// Exact language label.
    pub language: alloc::string::String,
    /// Exact parser cohort.
    pub parser_cohort: alloc::string::String,
    /// Exact query cohort.
    pub query_cohort: alloc::string::String,
    /// Capability root label, never a host path.
    pub root_label: alloc::string::String,
    /// Maximum admitted files.
    pub max_files: usize,
    /// Maximum bytes per file.
    pub max_file_bytes: usize,
    /// Maximum returned results.
    pub max_results: usize,
    /// Derived profile identity.
    pub profile_identity: alloc::string::String,
}

/// Draft used to construct a validated profile.
pub struct CodeGraphProfileDraft<'a> {
    /// Stable profile label.
    pub profile_id: &'a str,
    /// Exact language label.
    pub language: &'a str,
    /// Exact parser cohort.
    pub parser_cohort: &'a str,
    /// Exact query cohort.
    pub query_cohort: &'a str,
    /// Capability root label.
    pub root_label: &'a str,
    /// Maximum admitted files.
    pub max_files: usize,
    /// Maximum bytes per file.
    pub max_file_bytes: usize,
    /// Maximum returned results.
    pub max_results: usize,
}

/// Stable graph validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodeGraphError {
    /// A required label was empty or unsafe.
    InvalidLabel,
    /// A finite budget was zero.
    InvalidBudget,
    /// A relative path escaped its capability root.
    UnsafePath,
    /// Input exceeded its declared budget.
    BudgetExceeded,
    /// Binary or unsupported text was supplied.
    UnsupportedText,
    /// A stale graph was queried.
    StaleIndex,
    /// A rename was ambiguous or incomplete.
    AmbiguousRename,
    /// A required identity was malformed.
    InvalidIdentity,
}

/// One already-parsed syntactic contribution.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CodeGraphSymbol {
    /// Symbol text.
    pub name: alloc::string::String,
    /// Syntactic class.
    pub kind: CodeGraphSymbolKind,
    /// Capability-root-relative source path.
    pub relative_path: alloc::string::String,
    /// Start byte in the admitted file.
    pub start_byte: usize,
    /// End byte in the admitted file.
    pub end_byte: usize,
}

/// Parsed contribution for one admitted file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeGraphContribution {
    /// Capability-root-relative source path.
    pub relative_path: alloc::string::String,
    /// BLAKE3 source identity.
    pub source_identity: alloc::string::String,
    /// Deterministically ordered symbols.
    pub symbols: alloc::vec::Vec<CodeGraphSymbol>,
}

/// Trusted freshness state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodeGraphFreshness {
    /// Answers may be served.
    Fresh,
    /// A bounded rescan is required before answers may be served.
    Stale,
}

/// In-memory derived graph state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeGraphIndex {
    /// Current freshness.
    pub freshness: CodeGraphFreshness,
    /// Sorted per-file contributions.
    pub contributions: alloc::vec::Vec<CodeGraphContribution>,
    /// Derived graph identity.
    pub graph_identity: alloc::string::String,
}

/// Explicit host event class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodeGraphEventKind {
    /// A file was created.
    Create,
    /// A file was modified.
    Modify,
    /// A file was deleted.
    Delete,
    /// A file was renamed.
    Rename,
    /// Host watcher overflowed.
    Overflow,
    /// Host reported a sequence gap.
    Gap,
    /// Host reported an unsupported event.
    Unsupported,
}

/// Explicit event plus already-loaded replacement contribution.
pub struct CodeGraphEvent<'a> {
    /// Event class.
    pub kind: CodeGraphEventKind,
    /// Prior relative path.
    pub relative_path: &'a str,
    /// New relative path for rename.
    pub new_relative_path: Option<&'a str>,
    /// Parsed replacement for create, modify, or rename.
    pub contribution: Option<CodeGraphContribution>,
}

/// Bounded syntactic query.
pub struct CodeGraphQuery<'a> {
    /// Exact symbol name.
    pub symbol: &'a str,
    /// Optional requested class.
    pub kind: Option<CodeGraphSymbolKind>,
    /// Finite result limit no greater than the profile ceiling.
    pub max_results: usize,
}

/// Stable bounded query result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeGraphQueryResult {
    /// Ordered syntactic matches.
    pub matches: alloc::vec::Vec<CodeGraphSymbol>,
    /// Whether additional matches were omitted.
    pub truncated: bool,
    /// Explicit evidence boundary.
    pub evidence_class: &'static str,
}

/// Injected read-only codebase-graph query service.
pub trait CodeGraphQueryService {
    /// Executes one admitted bounded syntactic query.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic when the service is stale or the request is invalid.
    fn query_code_graph(&self, query: &CodeGraphQuery<'_>) -> Result<CodeGraphQueryResult, CodeGraphError>;
}

/// Disposable cache descriptor.
#[allow(clippy::struct_field_names)]
pub struct CodeGraphCacheDraft<'a> {
    /// Safe workspace identity.
    pub workspace_identity: &'a str,
    /// Exact source snapshot identity.
    pub source_identity: &'a str,
    /// Profile identity.
    pub profile_identity: &'a str,
    /// Parser cohort identity.
    pub parser_identity: &'a str,
    /// Query cohort identity.
    pub query_identity: &'a str,
    /// Schema identity.
    pub schema_identity: &'a str,
    /// Graph identity.
    pub graph_identity: &'a str,
}

/// Cache admission decision for a caller-owned capability-scoped store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodeGraphCacheDecision {
    /// Profile disables cache effects.
    Disabled,
    /// Exact observed cache identity may be loaded.
    Reuse,
    /// Mismatched or malformed cache must be quarantined before rebuild.
    QuarantineAndRebuild,
    /// No cache exists and a fresh graph may be saved after rebuild.
    RebuildAndSave,
}

/// Existing admission facts required before context or read-only-tool dispatch.
///
/// The independent facts are intentionally explicit: collapsing them into one
/// state enum would hide which existing admission boundary denied dispatch.
#[allow(clippy::struct_excessive_bools)]
pub struct CodeGraphDispatchAdmission {
    /// Existing context-reference or read-only-tool policy admitted the request.
    pub caller_admitted: bool,
    /// Workspace capability admits the graph root.
    pub capability_admitted: bool,
    /// A finite retrieval result remains available.
    pub retrieval_budget_available: bool,
    /// The injected graph service is available.
    pub service_available: bool,
    /// Mutation was requested through the read-only graph boundary.
    pub mutation_requested: bool,
}

/// Metadata-only graph receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeGraphReceipt {
    /// Profile identity.
    pub profile_identity: alloc::string::String,
    /// Graph identity.
    pub graph_identity: alloc::string::String,
    /// Freshness at projection time.
    pub freshness: CodeGraphFreshness,
    /// Bounded file count.
    pub file_count: usize,
    /// Bounded symbol count.
    pub symbol_count: usize,
    /// Syntactic-only evidence marker.
    pub evidence_class: &'static str,
    /// Receipt identity.
    pub receipt_identity: alloc::string::String,
}

/// Validates and canonicalizes a profile without host effects.
///
/// # Errors
///
/// Returns a stable diagnostic for unsafe labels or non-finite budgets.
pub fn validate_code_graph_profile(draft: &CodeGraphProfileDraft<'_>) -> Result<CodeGraphProfile, CodeGraphError> {
    let labels = [
        draft.profile_id,
        draft.language,
        draft.parser_cohort,
        draft.query_cohort,
        draft.root_label,
    ];
    if labels.iter().any(|label| !safe_label(label)) {
        return Err(CodeGraphError::InvalidLabel);
    }
    if draft.max_files == 0 || draft.max_file_bytes == 0 || draft.max_results == 0 {
        return Err(CodeGraphError::InvalidBudget);
    }
    debug_assert!(labels.iter().all(|label| safe_label(label)));
    debug_assert!(draft.max_files > 0);
    debug_assert!(draft.max_file_bytes > 0);
    debug_assert!(draft.max_results > 0);

    let payload = alloc::format!(
        "{PROFILE_DOMAIN}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
        draft.profile_id,
        draft.language,
        draft.parser_cohort,
        draft.query_cohort,
        draft.root_label,
        draft.max_files,
        draft.max_file_bytes,
        draft.max_results
    );
    let profile_identity = blake3_hex(payload.as_bytes());
    debug_assert!(valid_identity(&profile_identity));
    Ok(CodeGraphProfile {
        profile_id: draft.profile_id.to_string(),
        language: draft.language.to_string(),
        parser_cohort: draft.parser_cohort.to_string(),
        query_cohort: draft.query_cohort.to_string(),
        root_label: draft.root_label.to_string(),
        max_files: draft.max_files,
        max_file_bytes: draft.max_file_bytes,
        max_results: draft.max_results,
        profile_identity,
    })
}

/// Admits already-loaded file bytes before host parsing.
///
/// # Errors
///
/// Returns a stable diagnostic for unsafe paths, excess bytes, or unsupported text.
pub fn admit_code_graph_file(
    profile: &CodeGraphProfile,
    relative_path: &str,
    bytes: &[u8],
) -> Result<alloc::string::String, CodeGraphError> {
    if !safe_relative_path(relative_path) {
        return Err(CodeGraphError::UnsafePath);
    }
    if bytes.len() > profile.max_file_bytes {
        return Err(CodeGraphError::BudgetExceeded);
    }
    if bytes.contains(&0) || core::str::from_utf8(bytes).is_err() {
        return Err(CodeGraphError::UnsupportedText);
    }
    Ok(blake3_hex(bytes))
}

/// Canonicalizes one host parser contribution.
///
/// # Errors
///
/// Returns a stable diagnostic for malformed paths, identities, spans, or names.
pub fn normalize_code_graph_contribution(
    mut contribution: CodeGraphContribution,
) -> Result<CodeGraphContribution, CodeGraphError> {
    if !safe_relative_path(&contribution.relative_path) || !valid_identity(&contribution.source_identity) {
        return Err(CodeGraphError::UnsafePath);
    }
    for symbol in &contribution.symbols {
        if symbol.name.is_empty()
            || symbol.relative_path != contribution.relative_path
            || symbol.start_byte >= symbol.end_byte
        {
            return Err(CodeGraphError::InvalidLabel);
        }
    }
    contribution.symbols.sort();
    contribution.symbols.dedup();
    Ok(contribution)
}

/// Builds a fresh derived graph from explicit contributions.
///
/// # Errors
///
/// Returns a stable diagnostic for duplicate paths, malformed contributions, or excess files.
pub fn build_code_graph_index(
    profile: &CodeGraphProfile,
    contributions: alloc::vec::Vec<CodeGraphContribution>,
) -> Result<CodeGraphIndex, CodeGraphError> {
    if contributions.len() > profile.max_files {
        return Err(CodeGraphError::BudgetExceeded);
    }
    let mut normalized = contributions
        .into_iter()
        .map(normalize_code_graph_contribution)
        .collect::<Result<alloc::vec::Vec<_>, _>>()?;
    normalized.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let mut paths = alloc::collections::BTreeSet::new();
    if normalized.iter().any(|entry| !paths.insert(entry.relative_path.clone())) {
        return Err(CodeGraphError::InvalidLabel);
    }
    debug_assert!(normalized.len() <= profile.max_files);
    debug_assert!(normalized.windows(2).all(|pair| pair[0].relative_path < pair[1].relative_path));

    let identity = graph_identity(profile, &normalized);
    debug_assert!(valid_identity(&identity));
    Ok(CodeGraphIndex {
        freshness: CodeGraphFreshness::Fresh,
        contributions: normalized,
        graph_identity: identity,
    })
}

/// Applies one explicit event without reading files or invoking a parser.
///
/// # Errors
///
/// Returns a stable diagnostic for unsafe paths, ambiguous renames, or invalid replacements.
pub fn apply_code_graph_event(
    profile: &CodeGraphProfile,
    index: &CodeGraphIndex,
    event: CodeGraphEvent<'_>,
) -> Result<CodeGraphIndex, CodeGraphError> {
    if matches!(event.kind, CodeGraphEventKind::Overflow | CodeGraphEventKind::Gap | CodeGraphEventKind::Unsupported) {
        let mut stale = index.clone();
        stale.freshness = CodeGraphFreshness::Stale;
        return Ok(stale);
    }
    if !safe_relative_path(event.relative_path) {
        return Err(CodeGraphError::UnsafePath);
    }
    debug_assert!(safe_relative_path(event.relative_path));
    let mut contributions = index.contributions.clone();
    contributions.retain(|item| item.relative_path != event.relative_path);
    debug_assert!(contributions.iter().all(|item| item.relative_path != event.relative_path));
    match event.kind {
        CodeGraphEventKind::Delete => {}
        CodeGraphEventKind::Create | CodeGraphEventKind::Modify => {
            contributions.push(event.contribution.ok_or(CodeGraphError::InvalidLabel)?);
        }
        CodeGraphEventKind::Rename => {
            let new_path = event.new_relative_path.ok_or(CodeGraphError::AmbiguousRename)?;
            let replacement = event.contribution.ok_or(CodeGraphError::AmbiguousRename)?;
            if replacement.relative_path != new_path || !safe_relative_path(new_path) {
                return Err(CodeGraphError::AmbiguousRename);
            }
            contributions.push(replacement);
        }
        CodeGraphEventKind::Overflow | CodeGraphEventKind::Gap | CodeGraphEventKind::Unsupported => unreachable!(),
    }
    build_code_graph_index(profile, contributions)
}

/// Executes a stable, bounded syntactic query against a fresh index.
///
/// # Errors
///
/// Returns a stable diagnostic for stale indexes or invalid query bounds.
pub fn query_code_graph(
    profile: &CodeGraphProfile,
    index: &CodeGraphIndex,
    query: &CodeGraphQuery<'_>,
) -> Result<CodeGraphQueryResult, CodeGraphError> {
    if index.freshness != CodeGraphFreshness::Fresh {
        return Err(CodeGraphError::StaleIndex);
    }
    if query.symbol.is_empty() || query.max_results == 0 || query.max_results > profile.max_results {
        return Err(CodeGraphError::InvalidBudget);
    }
    debug_assert_eq!(index.freshness, CodeGraphFreshness::Fresh);
    debug_assert!(!query.symbol.is_empty());
    debug_assert!(query.max_results <= profile.max_results);

    let mut matches = index
        .contributions
        .iter()
        .flat_map(|entry| entry.symbols.iter())
        .filter(|symbol| symbol.name == query.symbol && query.kind.is_none_or(|kind| kind == symbol.kind))
        .cloned()
        .collect::<alloc::vec::Vec<_>>();
    matches.sort();
    let is_truncated = matches.len() > query.max_results;
    matches.truncate(query.max_results);
    Ok(CodeGraphQueryResult {
        matches,
        truncated: is_truncated,
        evidence_class: "syntactic-only",
    })
}

/// Derives an identity for a disposable cache descriptor.
///
/// # Errors
///
/// Returns a stable diagnostic when an identity or schema label is malformed.
pub fn code_graph_cache_identity(draft: &CodeGraphCacheDraft<'_>) -> Result<alloc::string::String, CodeGraphError> {
    let identities = [
        draft.workspace_identity,
        draft.source_identity,
        draft.profile_identity,
        draft.parser_identity,
        draft.query_identity,
        draft.graph_identity,
    ];
    if identities.iter().any(|identity| !valid_identity(identity)) || !safe_label(draft.schema_identity) {
        return Err(CodeGraphError::InvalidIdentity);
    }
    debug_assert!(identities.iter().all(|identity| valid_identity(identity)));
    debug_assert!(safe_label(draft.schema_identity));

    let payload = alloc::format!(
        "{CACHE_DOMAIN}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
        draft.workspace_identity,
        draft.source_identity,
        draft.profile_identity,
        draft.parser_identity,
        draft.query_identity,
        draft.schema_identity,
        draft.graph_identity
    );
    let cache_identity = blake3_hex(payload.as_bytes());
    debug_assert!(valid_identity(&cache_identity));
    Ok(cache_identity)
}

/// Plans caller-owned cache handling without cache I/O.
#[must_use]
pub fn evaluate_code_graph_cache(
    enabled: bool,
    expected_identity: &str,
    observed_identity: Option<&str>,
) -> CodeGraphCacheDecision {
    if !enabled {
        return CodeGraphCacheDecision::Disabled;
    }
    match observed_identity {
        None => CodeGraphCacheDecision::RebuildAndSave,
        Some(observed) if valid_identity(observed) && observed == expected_identity => CodeGraphCacheDecision::Reuse,
        Some(_) => CodeGraphCacheDecision::QuarantineAndRebuild,
    }
}

/// Composes existing context/tool, capability, budget, and service admission.
///
/// # Errors
///
/// Returns a stable denial without invoking a service when any required admission is absent.
pub fn admit_code_graph_dispatch(admission: &CodeGraphDispatchAdmission) -> Result<(), CodeGraphError> {
    if admission.mutation_requested {
        return Err(CodeGraphError::InvalidLabel);
    }
    if !admission.caller_admitted {
        return Err(CodeGraphError::InvalidLabel);
    }
    if !admission.capability_admitted {
        return Err(CodeGraphError::InvalidLabel);
    }
    if !admission.retrieval_budget_available {
        return Err(CodeGraphError::InvalidLabel);
    }
    if !admission.service_available {
        return Err(CodeGraphError::InvalidLabel);
    }
    debug_assert!(!admission.mutation_requested);
    debug_assert!(admission.caller_admitted);
    debug_assert!(admission.capability_admitted);
    debug_assert!(admission.retrieval_budget_available);
    debug_assert!(admission.service_available);
    Ok(())
}

/// Projects metadata-only receipt evidence.
#[must_use]
pub fn project_code_graph_receipt(profile: &CodeGraphProfile, index: &CodeGraphIndex) -> CodeGraphReceipt {
    let symbol_count = index.contributions.iter().map(|entry| entry.symbols.len()).sum::<usize>();
    let payload = alloc::format!(
        "{RECEIPT_DOMAIN}\0{}\0{}\0{}\0{}",
        profile.profile_identity,
        index.graph_identity,
        index.contributions.len(),
        symbol_count
    );
    let receipt_identity = blake3_hex(payload.as_bytes());
    debug_assert_eq!(receipt_identity.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(receipt_identity.bytes().all(|byte| byte.is_ascii_hexdigit()));
    CodeGraphReceipt {
        profile_identity: profile.profile_identity.clone(),
        graph_identity: index.graph_identity.clone(),
        freshness: index.freshness,
        file_count: index.contributions.len(),
        symbol_count,
        evidence_class: "syntactic-only",
        receipt_identity,
    }
}

fn graph_identity(profile: &CodeGraphProfile, contributions: &[CodeGraphContribution]) -> alloc::string::String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(GRAPH_DOMAIN.as_bytes());
    hasher.update(profile.profile_identity.as_bytes());
    for contribution in contributions {
        hasher.update(contribution.relative_path.as_bytes());
        hasher.update(contribution.source_identity.as_bytes());
        for symbol in &contribution.symbols {
            hasher.update(symbol.name.as_bytes());
            hasher.update(&symbol.start_byte.to_le_bytes());
            hasher.update(&symbol.end_byte.to_le_bytes());
            hasher.update(&[symbol.kind as u8]);
        }
    }
    hasher.finalize().to_hex().to_string()
}

fn safe_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SAFE_LABEL_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/'))
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\\')
        && value.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

fn valid_identity(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn blake3_hex(bytes: &[u8]) -> alloc::string::String {
    blake3::hash(bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_BUDGET: usize = 4;
    const BYTE_BUDGET: usize = 1024;
    const RESULT_BUDGET: usize = 8;

    fn profile() -> CodeGraphProfile {
        validate_code_graph_profile(&CodeGraphProfileDraft {
            profile_id: "rust-first",
            language: "rust",
            parser_cohort: "tree-sitter-rust-0.24",
            query_cohort: "rust-query-v1",
            root_label: "worktree",
            max_files: FILE_BUDGET,
            max_file_bytes: BYTE_BUDGET,
            max_results: RESULT_BUDGET,
        })
        .expect("valid profile")
    }

    struct ContributionFixture<'a> {
        path: &'a str,
        name: &'a str,
    }

    fn contribution(fixture: &ContributionFixture<'_>) -> CodeGraphContribution {
        CodeGraphContribution {
            relative_path: fixture.path.to_string(),
            source_identity: blake3_hex(fixture.path.as_bytes()),
            symbols: alloc::vec![CodeGraphSymbol {
                name: fixture.name.to_string(),
                kind: CodeGraphSymbolKind::Definition,
                relative_path: fixture.path.to_string(),
                start_byte: 1,
                end_byte: 2,
            }],
        }
    }

    #[test]
    fn positive_build_query_event_cache_and_receipt() {
        let profile = profile();
        assert!(admit_code_graph_file(&profile, "src/lib.rs", b"fn run() {}").is_ok());
        let index = build_code_graph_index(&profile, alloc::vec![contribution(&ContributionFixture {
            path: "src/lib.rs",
            name: "run",
        })])
        .expect("index");
        let result = query_code_graph(&profile, &index, &CodeGraphQuery {
            symbol: "run",
            kind: Some(CodeGraphSymbolKind::Definition),
            max_results: 1,
        })
        .expect("query");
        assert_eq!(result.matches.len(), 1);
        let updated = apply_code_graph_event(&profile, &index, CodeGraphEvent {
            kind: CodeGraphEventKind::Modify,
            relative_path: "src/lib.rs",
            new_relative_path: None,
            contribution: Some(contribution(&ContributionFixture {
                path: "src/lib.rs",
                name: "next",
            })),
        })
        .expect("update");
        assert_ne!(updated.graph_identity, index.graph_identity);
        let receipt = project_code_graph_receipt(&profile, &updated);
        assert_eq!(receipt.evidence_class, "syntactic-only");
        let cache = code_graph_cache_identity(&CodeGraphCacheDraft {
            workspace_identity: &blake3_hex(b"workspace"),
            source_identity: &blake3_hex(b"source"),
            profile_identity: &profile.profile_identity,
            parser_identity: &blake3_hex(b"parser"),
            query_identity: &blake3_hex(b"query"),
            schema_identity: "animus.codegraph.cache.v1",
            graph_identity: &updated.graph_identity,
        });
        assert!(cache.is_ok());
        assert_eq!(evaluate_code_graph_cache(false, &blake3_hex(b"cache"), None), CodeGraphCacheDecision::Disabled);
        assert_eq!(
            evaluate_code_graph_cache(true, &blake3_hex(b"cache"), Some(&blake3_hex(b"stale"))),
            CodeGraphCacheDecision::QuarantineAndRebuild
        );
        assert!(
            admit_code_graph_dispatch(&CodeGraphDispatchAdmission {
                caller_admitted: true,
                capability_admitted: true,
                retrieval_budget_available: true,
                service_available: true,
                mutation_requested: false
            })
            .is_ok()
        );
    }

    #[test]
    fn negative_paths_budgets_stale_and_rename_fail_closed() {
        let profile = profile();
        assert_eq!(admit_code_graph_file(&profile, "../secret", b"x"), Err(CodeGraphError::UnsafePath));
        assert_eq!(admit_code_graph_file(&profile, "src/x", &[0]), Err(CodeGraphError::UnsupportedText));
        let index = build_code_graph_index(&profile, alloc::vec![contribution(&ContributionFixture {
            path: "src/lib.rs",
            name: "run",
        })])
        .expect("index");
        let stale = apply_code_graph_event(&profile, &index, CodeGraphEvent {
            kind: CodeGraphEventKind::Overflow,
            relative_path: "src/lib.rs",
            new_relative_path: None,
            contribution: None,
        })
        .expect("stale");
        assert_eq!(
            query_code_graph(&profile, &stale, &CodeGraphQuery {
                symbol: "run",
                kind: None,
                max_results: 1
            }),
            Err(CodeGraphError::StaleIndex)
        );
        assert_eq!(
            admit_code_graph_dispatch(&CodeGraphDispatchAdmission {
                caller_admitted: true,
                capability_admitted: true,
                retrieval_budget_available: true,
                service_available: true,
                mutation_requested: true
            }),
            Err(CodeGraphError::InvalidLabel)
        );
        assert_eq!(
            apply_code_graph_event(&profile, &index, CodeGraphEvent {
                kind: CodeGraphEventKind::Rename,
                relative_path: "src/lib.rs",
                new_relative_path: None,
                contribution: None
            }),
            Err(CodeGraphError::AmbiguousRename)
        );
    }
}
