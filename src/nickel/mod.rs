//! Thin Lattice adapter over the canonical Radicle Nickel evaluator.
//!
//! r[impl lattice.nickel_toolchain.boundary]

// Observed evaluator cohort metadata, not a compiled executable identity.
pub(crate) const SOURCE_REVISION: &str = nickel_eval::NICKEL_SOURCE_REVISION;
pub(crate) const LIBRARY_VERSION: &str = nickel_eval::NICKEL_LANG_VERSION;
pub(crate) const CORE_VERSION: &str = nickel_eval::NICKEL_LANG_CORE_VERSION;

pub use nickel_eval::captured::Evaluated;
pub const CAPTURED_PROFILE: &str = nickel_eval::captured::PROFILE;

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

// r[impl lattice.nickel_eval_adoption.boundary]
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

pub(crate) fn string_literal(value: &str) -> String {
    nickel_eval::serialize::render_native_string(value)
}

pub(crate) fn value_literal(value: &serde_json::Value) -> Result<String, SafeFailure> {
    nickel_eval::serialize::render_native_value(value).map_err(|error| classify_failure(&error))
}

/// The worker executable is an explicitly selected local artifact, never a PATH lookup.
/// Its bytes are bound separately from the native evaluator cohort.
pub fn captured_worker() -> Result<(std::path::PathBuf, String), SafeFailure> {
    let path = crate::config::read_path_env("LATTICE_CLOSED_EVAL_WORKER")
        .map_err(|()| SafeFailure::InvalidRequest)?
        .or_else(|| option_env!("LATTICE_CLOSED_EVAL_WORKER_DEFAULT").map(std::path::PathBuf::from))
        .ok_or(SafeFailure::InvalidRequest)?;
    if !path.is_absolute() {
        return Err(SafeFailure::InvalidRequest);
    }
    let artifact =
        crate::authoring::construction::store::immutable_executable(&path).map_err(|_| SafeFailure::FileAccess)?;
    Ok((path, artifact))
}

pub const CAPTURED_LIMITS: nickel_eval::captured::Limits = nickel_eval::captured::Limits {
    input_bytes: 5_242_880,
    module_count: 128,
    name_depth: 16,
    resolution_edges: 1024,
    output_bytes: 1_048_576,
    response_bytes: 4_194_304,
    memory_bytes: 2_147_483_648,
    execution_millis: 5_000,
};

fn captured_modules(
    files: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<Vec<nickel_eval::captured::Module<'_>>, SafeFailure> {
    files
        .iter()
        .map(|(name, bytes)| {
            std::str::from_utf8(bytes)
                .map(|source| nickel_eval::captured::Module { name, source })
                .map_err(|_| SafeFailure::InvalidRequest)
        })
        .collect()
}

/// Admits captured modules under the reviewed cohort, policy and Lattice limits.
fn admit_captured<'a>(
    entry: &'a str,
    modules: &'a [nickel_eval::captured::Module<'a>],
) -> Result<nickel_eval::captured::Admitted<'a>, SafeFailure> {
    nickel_eval::captured::admit(nickel_eval::captured::Request {
        entry,
        modules,
        cohort: nickel_eval::captured::reviewed_cohort(),
        policy: nickel_eval::captured::POLICY,
        limits: CAPTURED_LIMITS,
    })
    .map_err(|_| SafeFailure::InvalidRequest)
}

pub(crate) fn validate_captured_response(
    entry: &str,
    files: &std::collections::BTreeMap<String, Vec<u8>>,
    evaluated: &Evaluated,
) -> Result<(), SafeFailure> {
    let modules = captured_modules(files)?;
    let admitted = admit_captured(entry, &modules)?;
    nickel_eval::captured::validate_response(&admitted, evaluated.clone())
        .map(|_| ())
        .map_err(|_| SafeFailure::Evaluation)
}

/// Captured modules are admitted and evaluated only by the isolated native worker.
pub fn evaluate_captured(
    entry: &str,
    files: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<(Evaluated, String), SafeFailure> {
    let modules = captured_modules(files)?;
    let admitted = admit_captured(entry, &modules)?;
    let (worker, artifact) = captured_worker()?;
    let evaluated =
        nickel_eval::captured::evaluate_isolated(&admitted, &worker).map_err(|_| SafeFailure::Evaluation)?;
    let evaluated =
        nickel_eval::captured::validate_response(&admitted, evaluated).map_err(|_| SafeFailure::Evaluation)?;
    let (_, after) = captured_worker()?;
    if artifact != after {
        return Err(SafeFailure::Evaluation);
    }
    Ok((evaluated, artifact))
}

pub fn evaluate_file(path: &std::path::Path) -> Result<serde_json::Value, SafeFailure> {
    nickel_eval::evaluate_to_json(path).map_err(|error| classify_failure(&error))
}

/// Evaluate an already captured UTF-8 Nickel source without reopening a path.
pub fn evaluate_source(source: &str) -> Result<serde_json::Value, SafeFailure> {
    nickel_eval::evaluate_to_json_str(source).map_err(|error| classify_failure(&error))
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

    // r[verify lattice.nickel_toolchain.boundary]
    // r[verify lattice.nickel_toolchain.compatibility]
    #[test]
    fn adapter_evaluates_captured_source() {
        let value = evaluate_source("{ enabled = true }").expect("captured Nickel evaluates");

        assert_eq!(value["enabled"], true);
    }

    #[test]
    fn adapter_rejects_invalid_captured_source() {
        let error = evaluate_source("let x = in").expect_err("invalid captured Nickel fails");

        assert_eq!(error, SafeFailure::Evaluation);
    }

    #[test]
    fn adapter_returns_bounded_categories_for_invalid_source() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("config.ncl");
        std::fs::write(&path, "let x = in").expect("fixture writes");

        let error = evaluate_file(&path).expect_err("invalid Nickel fails");

        // r[verify lattice.nickel_eval_adoption.compatibility.scenario.differ]
        assert_eq!(error, SafeFailure::Evaluation);
        assert!(!error.to_string().contains(path.to_string_lossy().as_ref()));
    }

    #[test]
    fn adapter_rejects_missing_files_without_path_disclosure() {
        let path = std::path::Path::new("/private/token=secret/config.ncl");

        let error = evaluate_file(path).expect_err("missing file fails");

        // r[verify lattice.nickel_eval_adoption.compatibility.scenario.differ]
        assert_eq!(error, SafeFailure::FileAccess);
        assert!(!error.to_string().contains("secret"));
    }

    // r[verify lattice.nickel_import_merge.values]
    // r[verify lattice.nickel_import_merge.values.scenario.nested]
    #[test]
    fn imported_json_merge_preserves_nested_values_in_both_orders() {
        let directory = tempfile::tempdir().expect("temporary directory");
        std::fs::write(
            directory.path().join("identity.json"),
            r#"{"machine":{"id":"machine-1"},"grants":[{"operation":"shell","allowed":true}]}"#,
        )
        .expect("JSON fixture writes");
        let path = directory.path().join("profile.ncl");
        let expected = serde_json::json!({
            "machine": {"id": "machine-1", "reviewed": true},
            "grants": [{"operation": "shell", "allowed": true}],
            "expires": 1234
        });

        let import_path =
            serde_json::to_string(&directory.path().join("identity.json")).expect("absolute import path encodes");
        for expression in [
            "identity & { machine.reviewed = true, expires = 1234 }",
            "{ machine.reviewed = true, expires = 1234 } & identity",
        ] {
            let source = format!("let identity = import \"identity.json\" in {expression}");
            std::fs::write(&path, &source).expect("Nickel fixture writes");
            assert_eq!(evaluate_file(&path).expect("imported JSON merge evaluates"), expected);
            let captured = format!("let identity = import {import_path} in {expression}");
            assert_eq!(evaluate_source(&captured).expect("captured imported JSON merge evaluates"), expected);
        }
    }

    // r[verify lattice.nickel_import_merge.values.scenario.recursive]
    #[test]
    fn imported_json_merge_preserves_recursive_overrides_and_contracts() {
        let directory = tempfile::tempdir().expect("temporary directory");
        std::fs::write(
            directory.path().join("defaults.json"),
            r#"{"base":3,"nested":{"answer":42},"items":[{"enabled":true}]}"#,
        )
        .expect("JSON fixture writes");
        let path = directory.path().join("profile.ncl");
        std::fs::write(
            &path,
            r#"
            let defaults = import "defaults.json" in
            ({ base | Number | default = 1, derived = base + 1 } & defaults)
              & { base | force = 5, nested.extra = "retained" }
            "#,
        )
        .expect("Nickel fixture writes");

        assert_eq!(
            evaluate_file(&path).expect("recursive imported JSON merge evaluates"),
            serde_json::json!({
                "base": 5,
                "derived": 6,
                "nested": {"answer": 42, "extra": "retained"},
                "items": [{"enabled": true}]
            })
        );
    }

    // r[verify lattice.nickel_import_merge.boundary]
    // r[verify lattice.nickel_import_merge.boundary.scenario.invalid]
    #[test]
    fn imported_json_merge_rejects_invalid_imports_and_contracts_safely() {
        let directory = tempfile::tempdir().expect("temporary directory");
        std::fs::write(directory.path().join("malformed.json"), r#"{"token":"secret-sentinel","#)
            .expect("malformed JSON fixture writes");
        std::fs::write(directory.path().join("invalid.json"), r#"{"count":"secret-sentinel"}"#)
            .expect("invalid contract fixture writes");
        let path = directory.path().join("private-profile.ncl");

        for (source, expected) in [
            (r#"(import "missing-secret-sentinel.json") & { enabled = true }"#, SafeFailure::Evaluation),
            (r#"(import "malformed.json") & { enabled = true }"#, SafeFailure::Evaluation),
            (r#"(import "invalid.json") & { count | Number }"#, SafeFailure::ContractViolation),
        ] {
            std::fs::write(&path, source).expect("Nickel fixture writes");
            let error = evaluate_file(&path).expect_err("invalid imported merge fails");
            assert_eq!(error, expected);
            assert!(!error.to_string().contains("secret-sentinel"));
            assert!(!error.to_string().contains(path.to_string_lossy().as_ref()));
        }
    }
}
