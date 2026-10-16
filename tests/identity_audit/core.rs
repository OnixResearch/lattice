use std::collections::BTreeSet;

const LEGACY_BRAND_LOWER: &str = concat!("n8n", "clone");
const LEGACY_REQUIREMENT_PREFIX: &str = concat!("n8n", "clone.");
const MARKER_PREFIX: &str = "r[";
const MARKER_SUFFIX: char = ']';
const MARKER_VERBS: &[&str] = &["impl", "verify", "depends", "related"];
const ACCEPTED_ID_MANIFEST_DOMAIN: &str = "lattice-accepted-legacy-requirement-ids-v1";
const ARCHIVE_MANIFEST_DOMAIN: &str = "lattice-rename-archive-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSnapshot {
    pub path: String,
    pub text: String,
}

impl TextSnapshot {
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinarySnapshot {
    pub path: String,
    pub bytes: Vec<u8>,
}

impl BinarySnapshot {
    pub fn new(path: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            bytes: bytes.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandingFinding {
    pub path: String,
    pub line_number: usize,
    pub line: String,
}

pub fn stale_branding_findings(files: &[TextSnapshot]) -> Vec<BrandingFinding> {
    let mut findings = Vec::new();
    for file in files {
        for (line_index, line) in file.text.lines().enumerate() {
            if line.to_ascii_lowercase().contains(LEGACY_BRAND_LOWER)
                && !legacy_occurrences_are_allowed(&file.path, line)
            {
                findings.push(BrandingFinding {
                    path: file.path.clone(),
                    line_number: line_index.saturating_add(1),
                    line: line.to_string(),
                });
            }
        }
    }
    findings
}

pub fn accepted_legacy_id_manifest_hash(files: &[TextSnapshot]) -> String {
    let mut identities = BTreeSet::new();
    for file in files {
        for line in file.text.lines() {
            if let Some(identity) = accepted_requirement_id(line)
                && identity.starts_with(LEGACY_REQUIREMENT_PREFIX)
            {
                identities.insert(format!("{}#{identity}", file.path));
            }
        }
    }
    let snapshots =
        identities.into_iter().map(|identity| BinarySnapshot::new(identity, Vec::new())).collect::<Vec<_>>();
    snapshot_manifest_hash(ACCEPTED_ID_MANIFEST_DOMAIN, &snapshots)
}

pub fn archive_manifest_hash(files: &[BinarySnapshot]) -> String {
    snapshot_manifest_hash(ARCHIVE_MANIFEST_DOMAIN, files)
}

fn legacy_occurrences_are_allowed(path: &str, line: &str) -> bool {
    if marker_only_line(line) {
        return true;
    }

    let allowed_tokens = allowed_tokens_for_path(path);
    if allowed_tokens.is_empty() {
        return false;
    }

    let mut remainder = line.to_ascii_lowercase();
    for token in allowed_tokens {
        remainder = remainder.replace(&token.to_ascii_lowercase(), "");
    }
    !remainder.contains(LEGACY_BRAND_LOWER)
}

fn marker_only_line(line: &str) -> bool {
    let Some(mut remaining) = line.trim().strip_prefix("//").map(str::trim) else {
        return false;
    };
    if remaining.is_empty() {
        return false;
    }

    loop {
        let Some(after_prefix) = remaining.strip_prefix(MARKER_PREFIX) else {
            return false;
        };
        let Some(close_index) = after_prefix.find(MARKER_SUFFIX) else {
            return false;
        };
        let content = &after_prefix[..close_index];
        if !valid_marker_content(content) {
            return false;
        }
        remaining = after_prefix[close_index.saturating_add(1)..].trim();
        if remaining.is_empty() {
            return true;
        }
    }
}

fn valid_marker_content(content: &str) -> bool {
    let Some((verb, identity)) = content.split_once(' ') else {
        return false;
    };
    MARKER_VERBS.contains(&verb)
        && identity.starts_with(LEGACY_REQUIREMENT_PREFIX)
        && identity.chars().all(valid_requirement_id_character)
}

fn valid_requirement_id_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
}

fn allowed_tokens_for_path(path: &str) -> &'static [&'static str] {
    const CONFIG_TOKENS: &[&str] = &[
        concat!("~/.", "n8n", "clone/config.ncl"),
        concat!(".", "n8n", "clone"),
        concat!("N8N", "CLONE_DATA_DIR"),
        concat!("n8n", "clone.redb"),
    ];
    const WEBHOOK_TOKENS: &[&str] = &[concat!("n8n", "clone-webhook-signature:v1")];
    const VAULT_TOKENS: &[&str] = &[
        concat!("n8n", "clone-local-vault-stream-v1"),
        concat!("n8n", "clone-local-vault-integrity-v1"),
    ];
    const FLAKE_TOKENS: &[&str] = &[concat!(".cairn/archive/2026-07-20-rename-n8n", "clone-to-lattice/")];

    match path {
        "src/config/identity.rs" => CONFIG_TOKENS,
        "src/triggers.rs" => WEBHOOK_TOKENS,
        "src/credentials.rs" => VAULT_TOKENS,
        "flake.nix" => FLAKE_TOKENS,
        _ => &[],
    }
}

fn accepted_requirement_id(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let after_prefix = trimmed.strip_prefix(MARKER_PREFIX)?;
    let close_index = after_prefix.find(MARKER_SUFFIX)?;
    let identity = &after_prefix[..close_index];
    if identity.contains(' ') || !identity.chars().all(valid_requirement_id_character) {
        return None;
    }
    Some(identity)
}

fn snapshot_manifest_hash(domain: &str, files: &[BinarySnapshot]) -> String {
    assert!(!domain.is_empty());
    let mut ordered = files.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.path.cmp(&right.path).then(left.bytes.cmp(&right.bytes)));

    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, domain.as_bytes());
    for file in ordered {
        hash_field(&mut hasher, file.path.as_bytes());
        hash_field(&mut hasher, &file.bytes);
    }
    hasher.finalize().to_hex().to_string()
}

fn hash_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).expect("manifest field length must fit in u64");
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
}

#[cfg(test)]
mod tests {
    use super::BinarySnapshot;
    use super::TextSnapshot;
    use super::archive_manifest_hash;
    use super::stale_branding_findings;

    fn legacy_name() -> String {
        ["n8n", "clone"].concat()
    }

    #[test]
    fn classifier_accepts_only_exact_stable_identities() {
        let legacy = legacy_name();
        let files = vec![
            TextSnapshot::new("src/config/identity.rs", format!("const LEGACY: &str = \".{legacy}\";")),
            TextSnapshot::new(
                "src/triggers.rs",
                format!("// r[impl {legacy}.workflow]\nconst TAG: &str = \"{legacy}-webhook-signature:v1\";"),
            ),
        ];

        assert!(stale_branding_findings(&files).is_empty());
    }

    #[test]
    fn classifier_rejects_public_text_and_marker_bypasses() {
        let legacy = legacy_name();
        let files = vec![
            TextSnapshot::new("README.md", format!("run {legacy} --config config.ncl")),
            TextSnapshot::new("tests/stale.rs", format!("// {legacy} remains the product")),
            TextSnapshot::new("src/runtime.rs", format!("// r[impl {legacy}.workflow] stale {legacy} branding")),
            TextSnapshot::new("tests/incomplete_marker.rs", format!("// r[verify {legacy}.workflow")),
        ];

        let findings = stale_branding_findings(&files);
        assert_eq!(findings.len(), files.len());
    }

    #[test]
    fn manifest_hash_changes_when_path_or_bytes_change() {
        let original = vec![BinarySnapshot::new("archive/file.md", b"stable".to_vec())];
        let changed_path = vec![BinarySnapshot::new("archive/other.md", b"stable".to_vec())];
        let changed_bytes = vec![BinarySnapshot::new("archive/file.md", b"changed".to_vec())];
        let original_hash = archive_manifest_hash(&original);

        assert_ne!(archive_manifest_hash(&changed_path), original_hash);
        assert_ne!(archive_manifest_hash(&changed_bytes), original_hash);
    }
}
