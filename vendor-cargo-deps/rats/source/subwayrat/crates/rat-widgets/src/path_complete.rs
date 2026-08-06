//! Pure completion projection over caller-supplied resource facts.

/// Maximum entries accepted in one immutable snapshot.
pub const MAX_COMPLETION_ENTRIES: usize = 4_096;
/// Maximum candidates returned by one projection.
pub const MAX_COMPLETION_RESULTS: usize = 256;
/// Maximum UTF-8 bytes accepted in query and entry presentation fields.
pub const MAX_COMPLETION_TEXT_BYTES: usize = 4_096;

/// Opaque application lookup token. Display equality does not merge identities.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompletionEntryId(String);

impl CompletionEntryId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, CompletionInputError> {
        let value = value.into();
        if value.is_empty() || value.len() > MAX_COMPLETION_TEXT_BYTES {
            return Err(CompletionInputError::InvalidIdentity);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Explicit matching policy chosen by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionCasePolicy {
    Sensitive,
    Insensitive,
}

/// Entry eligibility supplied by the resource-owning adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionEntryEligibility {
    Eligible,
    Ineligible,
    Unrepresentable,
}

/// One caller-parsed entry. Insertion and display text are never fabricated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionEntrySnapshot {
    pub id: CompletionEntryId,
    pub match_text: String,
    pub display_text: String,
    pub insertion_text: String,
    pub is_container: bool,
    pub eligibility: CompletionEntryEligibility,
}

impl CompletionEntrySnapshot {
    pub fn try_new(
        id: CompletionEntryId,
        match_text: impl Into<String>,
        display_text: impl Into<String>,
        insertion_text: impl Into<String>,
        is_container: bool,
        eligibility: CompletionEntryEligibility,
    ) -> Result<Self, CompletionInputError> {
        let match_text = match_text.into();
        let display_text = display_text.into();
        let insertion_text = insertion_text.into();
        if [&match_text, &display_text, &insertion_text]
            .iter()
            .any(|value| value.len() > MAX_COMPLETION_TEXT_BYTES)
        {
            return Err(CompletionInputError::TextCapacityExceeded);
        }
        Ok(Self {
            id,
            match_text,
            display_text,
            insertion_text,
            is_container,
            eligibility,
        })
    }
}

/// Caller-parsed query for one immutable resource generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionQuery {
    pub generation: u64,
    pub prefix: String,
    pub case_policy: CompletionCasePolicy,
    pub result_limit: usize,
}

impl CompletionQuery {
    pub fn try_new(
        generation: u64,
        prefix: impl Into<String>,
        case_policy: CompletionCasePolicy,
        result_limit: usize,
    ) -> Result<Self, CompletionInputError> {
        let prefix = prefix.into();
        if prefix.len() > MAX_COMPLETION_TEXT_BYTES {
            return Err(CompletionInputError::TextCapacityExceeded);
        }
        if result_limit == 0 || result_limit > MAX_COMPLETION_RESULTS {
            return Err(CompletionInputError::InvalidResultLimit);
        }
        Ok(Self {
            generation,
            prefix,
            case_policy,
            result_limit,
        })
    }
}

/// Availability of a caller-owned resource snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionSnapshotState {
    Ready(Vec<CompletionEntrySnapshot>),
    Missing,
    Denied,
    Failed,
}

/// Immutable directory/virtual-resource facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionSnapshot {
    pub generation: u64,
    pub state: CompletionSnapshotState,
}

impl CompletionSnapshot {
    pub fn try_ready(generation: u64, entries: Vec<CompletionEntrySnapshot>) -> Result<Self, CompletionInputError> {
        if entries.len() > MAX_COMPLETION_ENTRIES {
            return Err(CompletionInputError::EntryCapacityExceeded);
        }
        Ok(Self {
            generation,
            state: CompletionSnapshotState::Ready(entries),
        })
    }
}

/// Content-free projection diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionDiagnostic {
    StaleSnapshot,
    ResourceMissing,
    PolicyDenied,
    SnapshotFailed,
}

/// Validated input failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionInputError {
    InvalidIdentity,
    TextCapacityExceeded,
    InvalidResultLimit,
    EntryCapacityExceeded,
}

/// Candidate retaining application identity and exact supplied presentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    pub id: CompletionEntryId,
    pub display_text: String,
    pub insertion_text: String,
    pub is_container: bool,
}

/// Deterministic completion result. Rejections contain no partial candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionProjection {
    Candidates(Vec<CompletionCandidate>),
    Rejected(CompletionDiagnostic),
}

/// Filters an immutable snapshot without parsing paths or acquiring resources.
pub fn project_path_completions(query: &CompletionQuery, snapshot: &CompletionSnapshot) -> CompletionProjection {
    if query.generation != snapshot.generation {
        return CompletionProjection::Rejected(CompletionDiagnostic::StaleSnapshot);
    }
    let entries = match &snapshot.state {
        CompletionSnapshotState::Ready(entries) => entries,
        CompletionSnapshotState::Missing => {
            return CompletionProjection::Rejected(CompletionDiagnostic::ResourceMissing);
        }
        CompletionSnapshotState::Denied => {
            return CompletionProjection::Rejected(CompletionDiagnostic::PolicyDenied);
        }
        CompletionSnapshotState::Failed => {
            return CompletionProjection::Rejected(CompletionDiagnostic::SnapshotFailed);
        }
    };

    let normalized_prefix = normalize_for_case(&query.prefix, query.case_policy);
    let mut matches: Vec<_> = entries
        .iter()
        .filter(|entry| entry.eligibility == CompletionEntryEligibility::Eligible)
        .filter(|entry| normalize_for_case(&entry.match_text, query.case_policy).starts_with(&normalized_prefix))
        .map(|entry| CompletionCandidate {
            id: entry.id.clone(),
            display_text: entry.display_text.clone(),
            insertion_text: entry.insertion_text.clone(),
            is_container: entry.is_container,
        })
        .collect();
    matches.sort_by(|left, right| {
        left.display_text
            .cmp(&right.display_text)
            .then(left.insertion_text.cmp(&right.insertion_text))
            .then(left.id.cmp(&right.id))
    });
    matches.truncate(query.result_limit);
    CompletionProjection::Candidates(matches)
}

fn normalize_for_case(value: &str, policy: CompletionCasePolicy) -> String {
    match policy {
        CompletionCasePolicy::Sensitive => value.to_string(),
        CompletionCasePolicy::Insensitive => value.to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        id: &str,
        match_text: &str,
        display: &str,
        insertion: &str,
        is_container: bool,
    ) -> CompletionEntrySnapshot {
        CompletionEntrySnapshot::try_new(
            CompletionEntryId::try_new(id).unwrap(),
            match_text,
            display,
            insertion,
            is_container,
            CompletionEntryEligibility::Eligible,
        )
        .unwrap()
    }

    #[test]
    fn supplied_unix_windows_virtual_and_opaque_insertions_are_preserved() {
        let entries = vec![
            entry("unix", "app", "Unix app", "/srv/app/", true),
            entry("windows", "app", "Windows app", r"C:\srv\app\", true),
            entry("browser", "app", "Browser app", "vfs:app", true),
            entry("remote", "app", "Remote app", "resource://node/7", false),
        ];
        let snapshot = CompletionSnapshot::try_ready(3, entries).unwrap();
        let query = CompletionQuery::try_new(3, "app", CompletionCasePolicy::Sensitive, 10).unwrap();
        let CompletionProjection::Candidates(candidates) = project_path_completions(&query, &snapshot) else {
            panic!("supplied entries must project");
        };

        assert_eq!(candidates.len(), 4);
        assert!(candidates.iter().any(|item| item.insertion_text == "/srv/app/"));
        assert!(candidates.iter().any(|item| item.insertion_text == r"C:\srv\app\"));
        assert!(candidates.iter().any(|item| item.insertion_text == "vfs:app"));
        assert!(candidates.iter().any(|item| item.insertion_text == "resource://node/7"));
    }

    #[test]
    fn case_policy_order_limit_and_opaque_identity_are_deterministic() {
        let snapshot = CompletionSnapshot::try_ready(8, vec![
            entry("b", "Apple", "same", "second", false),
            entry("a", "apple", "same", "first", false),
            entry("c", "banana", "banana", "banana", false),
        ])
        .unwrap();
        let insensitive = CompletionQuery::try_new(8, "APP", CompletionCasePolicy::Insensitive, 2).unwrap();
        let sensitive = CompletionQuery::try_new(8, "APP", CompletionCasePolicy::Sensitive, 2).unwrap();

        let CompletionProjection::Candidates(first) = project_path_completions(&insensitive, &snapshot) else {
            panic!("ready snapshot must project");
        };
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].id.as_str(), "a");
        assert_eq!(first[1].id.as_str(), "b");
        assert_eq!(project_path_completions(&insensitive, &snapshot), CompletionProjection::Candidates(first));
        assert_eq!(project_path_completions(&sensitive, &snapshot), CompletionProjection::Candidates(Vec::new()));
    }

    #[test]
    fn denied_stale_failed_and_unrepresentable_facts_fail_closed() {
        let query = CompletionQuery::try_new(5, "", CompletionCasePolicy::Sensitive, 4).unwrap();
        let denied = CompletionSnapshot {
            generation: 5,
            state: CompletionSnapshotState::Denied,
        };
        let missing = CompletionSnapshot {
            generation: 5,
            state: CompletionSnapshotState::Missing,
        };
        let failed = CompletionSnapshot {
            generation: 5,
            state: CompletionSnapshotState::Failed,
        };
        let mut hidden = entry("bytes", "", "", "", false);
        hidden.eligibility = CompletionEntryEligibility::Unrepresentable;
        let ready = CompletionSnapshot::try_ready(4, vec![hidden.clone()]).unwrap();

        assert_eq!(
            project_path_completions(&query, &denied),
            CompletionProjection::Rejected(CompletionDiagnostic::PolicyDenied)
        );
        assert_eq!(
            project_path_completions(&query, &missing),
            CompletionProjection::Rejected(CompletionDiagnostic::ResourceMissing)
        );
        assert_eq!(
            project_path_completions(&query, &failed),
            CompletionProjection::Rejected(CompletionDiagnostic::SnapshotFailed)
        );
        assert_eq!(
            project_path_completions(&query, &ready),
            CompletionProjection::Rejected(CompletionDiagnostic::StaleSnapshot)
        );
        let current = CompletionSnapshot::try_ready(5, vec![hidden]).unwrap();
        assert_eq!(project_path_completions(&query, &current), CompletionProjection::Candidates(Vec::new()));
    }
}
