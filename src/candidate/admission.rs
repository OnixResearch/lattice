pub(super) fn identifier(value: &str) -> Result<(), super::CandidateReceiptError> {
    if value.is_empty() || value.len() > super::MAX_IDENTIFIER_BYTES || value.chars().any(char::is_control) {
        return Err(super::CandidateReceiptError::InvalidIdentifier);
    }
    Ok(())
}

pub(super) fn changed_paths(paths: &[String]) -> Result<(), super::CandidateReceiptError> {
    if paths.len() > super::MAX_CHANGED_PATHS {
        return Err(super::CandidateReceiptError::TooManyChangedPaths);
    }
    let mut previous: Option<&str> = None;
    for path in paths {
        self::path(path)?;
        if previous.is_some_and(|previous| previous >= path.as_str()) {
            return Err(super::CandidateReceiptError::ChangedPathsNotCanonical);
        }
        previous = Some(path);
    }
    Ok(())
}

pub(super) fn path(path: &str) -> Result<(), super::CandidateReceiptError> {
    let has_invalid_shape =
        path.starts_with(['/', '\\']) || path.split('/').any(|component| component == ".." || component.is_empty());
    if path.is_empty() || path.len() > super::MAX_IDENTIFIER_BYTES || has_invalid_shape {
        return Err(super::CandidateReceiptError::InvalidChangedPath);
    }
    Ok(())
}
