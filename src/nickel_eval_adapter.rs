//! Thin Lattice adapter over the canonical Radicle Nickel evaluator.

use std::path::Path;

use nickel_eval::NickelError;
use nickel_eval::NickelErrorCategory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeNickelFailure {
    InvalidRequest,
    FileAccess,
    Evaluation,
    ContractViolation,
    TargetDecode,
    Serialization,
}

impl std::fmt::Display for SafeNickelFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidRequest => "invalid Nickel evaluation request",
            Self::FileAccess => "Nickel source could not be read",
            Self::Evaluation => "Nickel evaluation failed",
            Self::ContractViolation => "Nickel contract failed",
            Self::TargetDecode => "Nickel result could not be decoded",
            Self::Serialization => "Nickel result could not be serialized",
        };
        formatter.write_str(message)
    }
}

#[must_use]
pub const fn classify_failure(error: &NickelError) -> SafeNickelFailure {
    match error.category() {
        NickelErrorCategory::InvalidRequest => SafeNickelFailure::InvalidRequest,
        NickelErrorCategory::Io => SafeNickelFailure::FileAccess,
        NickelErrorCategory::Evaluation => SafeNickelFailure::Evaluation,
        NickelErrorCategory::ContractViolation => SafeNickelFailure::ContractViolation,
        NickelErrorCategory::Deserialization => SafeNickelFailure::TargetDecode,
        NickelErrorCategory::Serialization
        | NickelErrorCategory::StdlibResolution
        | NickelErrorCategory::TemplateGeneration => SafeNickelFailure::Serialization,
    }
}

pub fn evaluate_file(path: &Path) -> Result<serde_json::Value, SafeNickelFailure> {
    nickel_eval::evaluate_to_json(path).map_err(|error| classify_failure(&error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_evaluates_valid_source() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("config.ncl");
        std::fs::write(&path, "{ enabled = true }").expect("fixture writes");

        let value = evaluate_file(&path).expect("valid Nickel evaluates");

        assert_eq!(value["enabled"], true);
    }

    #[test]
    fn adapter_returns_bounded_categories_for_invalid_source() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("config.ncl");
        std::fs::write(&path, "let x = in").expect("fixture writes");

        let error = evaluate_file(&path).expect_err("invalid Nickel fails");

        assert_eq!(error, SafeNickelFailure::Evaluation);
        assert!(!error.to_string().contains(path.to_string_lossy().as_ref()));
    }

    #[test]
    fn adapter_rejects_missing_files_without_path_disclosure() {
        let path = Path::new("/private/token=secret/config.ncl");

        let error = evaluate_file(path).expect_err("missing file fails");

        assert_eq!(error, SafeNickelFailure::FileAccess);
        assert!(!error.to_string().contains("secret"));
    }
}
