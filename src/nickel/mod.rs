//! Thin Lattice adapter over the canonical Radicle Nickel evaluator.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeFailure {
    InvalidRequest,
    FileAccess,
    Evaluation,
    ContractViolation,
    TargetDecode,
    Serialization,
}

impl std::fmt::Display for SafeFailure {
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
pub const fn classify_failure(error: &nickel_eval::NickelError) -> SafeFailure {
    match error.category() {
        nickel_eval::NickelErrorCategory::InvalidRequest => SafeFailure::InvalidRequest,
        nickel_eval::NickelErrorCategory::Io => SafeFailure::FileAccess,
        nickel_eval::NickelErrorCategory::Evaluation => SafeFailure::Evaluation,
        nickel_eval::NickelErrorCategory::ContractViolation => SafeFailure::ContractViolation,
        nickel_eval::NickelErrorCategory::Deserialization => SafeFailure::TargetDecode,
        nickel_eval::NickelErrorCategory::Serialization
        | nickel_eval::NickelErrorCategory::StdlibResolution
        | nickel_eval::NickelErrorCategory::TemplateGeneration => SafeFailure::Serialization,
    }
}

pub fn evaluate_file(path: &std::path::Path) -> Result<serde_json::Value, SafeFailure> {
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

        assert_eq!(error, SafeFailure::Evaluation);
        assert!(!error.to_string().contains(path.to_string_lossy().as_ref()));
    }

    #[test]
    fn adapter_rejects_missing_files_without_path_disclosure() {
        let path = std::path::Path::new("/private/token=secret/config.ncl");

        let error = evaluate_file(path).expect_err("missing file fails");

        assert_eq!(error, SafeFailure::FileAccess);
        assert!(!error.to_string().contains("secret"));
    }
}
