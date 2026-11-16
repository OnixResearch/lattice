pub(super) mod publication;
pub(super) mod reconciliation;

impl super::AttemptResult {
    /// Project a worker outcome without retaining raw output bytes.
    pub fn from_outcome(outcome: &super::Outcome) -> Self {
        let (stdout, stderr) = outcome.streams();
        Self {
            schema: super::CANDIDATE_ATTEMPT_RESULT_SCHEMA.to_string(),
            disposition: outcome.disposition(),
            receipt: outcome.receipt(),
            published_artifacts: outcome.published_artifacts(),
            stdout_blake3: crate::pure_inet_profile::blake3_identity(stdout),
            stderr_blake3: crate::pure_inet_profile::blake3_identity(stderr),
            blocker: outcome.blocker(),
            compatibility: None,
            process_failure: outcome.failure(),
            observed_artifacts: outcome.observed_artifacts(),
        }
    }

    /// Attach the exact worker/workspace/environment identities used for this result.
    #[must_use]
    pub fn with_compatibility(mut self, request: &super::Request) -> Self {
        self.compatibility = super::compatibility::observe(request);
        self
    }

    pub fn validate(&self) -> Result<(), super::Error> {
        self.validate_identities()?;
        self.validate_observed_artifacts()?;
        self.validate_compatibility()?;
        self.validate_published_paths()?;
        match self.disposition {
            crate::candidate::CandidateDisposition::Produced => self.validate_produced(),
            _ => self.validate_nonproduced(),
        }
    }

    pub fn to_json(&self) -> Result<String, super::Error> {
        self.validate()?;
        serde_json::to_string(self).map_err(|error| super::Error::Result(error.to_string()))
    }

    pub fn from_json(text: &str) -> Result<Self, super::Error> {
        let result: Self = serde_json::from_str(text).map_err(|error| super::Error::Result(error.to_string()))?;
        result.validate()?;
        Ok(result)
    }
}

pub(super) fn is_blake3(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn absent_compatibility() -> Option<super::Compatibility> {
    None
}
pub(super) fn absent_process_failure() -> Option<super::ProcessFailure> {
    None
}
pub(super) fn empty_observed_artifacts() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::new()
}

impl super::Outcome {
    fn streams(&self) -> (&[u8], &[u8]) {
        match self {
            Self::Produced { stdout, stderr, .. }
            | Self::Failed { stdout, stderr, .. }
            | Self::Unchanged { stdout, stderr, .. }
            | Self::Cancelled { stdout, stderr }
            | Self::Interrupted { stdout, stderr, .. }
            | Self::Unknown { stdout, stderr, .. } => (stdout, stderr),
            Self::Blocked { .. } => (&[], &[]),
        }
    }

    fn disposition(&self) -> crate::candidate::CandidateDisposition {
        match self {
            Self::Produced { .. } => crate::candidate::CandidateDisposition::Produced,
            Self::Blocked { .. } => crate::candidate::CandidateDisposition::Blocked,
            Self::Unchanged { .. } | Self::Failed { .. } => crate::candidate::CandidateDisposition::Failed,
            Self::Cancelled { .. } => crate::candidate::CandidateDisposition::Cancelled,
            Self::Interrupted { .. } | Self::Unknown { .. } => crate::candidate::CandidateDisposition::Unknown,
        }
    }

    fn blocker(&self) -> Option<String> {
        match self {
            Self::Blocked { blocker } | Self::Failed { blocker, .. } | Self::Unknown { blocker, .. } => {
                Some(blocker.clone())
            }
            Self::Unchanged { .. } => {
                Some("unchanged source is a proposal requiring explicit independent no-change acceptance".into())
            }
            Self::Interrupted { .. } => {
                Some("worker failed after start; candidate and teardown require reconciliation".into())
            }
            _ => None,
        }
    }

    fn receipt(&self) -> Option<crate::candidate::CandidateReceipt> {
        match self {
            Self::Produced { receipt, .. } => Some(receipt.as_ref().clone()),
            _ => None,
        }
    }

    fn published_artifacts(&self) -> Vec<String> {
        match self {
            Self::Produced { changed_files, .. } => changed_files.keys().cloned().collect(),
            _ => Vec::new(),
        }
    }

    fn failure(&self) -> Option<super::ProcessFailure> {
        match self {
            Self::Interrupted { failure, .. } => Some(*failure),
            _ => None,
        }
    }

    fn observed_artifacts(&self) -> std::collections::BTreeMap<String, String> {
        match self {
            Self::Interrupted {
                observed_files: Some(files),
                ..
            } => files
                .iter()
                .map(|(path, bytes)| (path.clone(), crate::pure_inet_profile::blake3_identity(bytes)))
                .collect(),
            _ => std::collections::BTreeMap::new(),
        }
    }
}

impl super::AttemptResult {
    fn validate_identities(&self) -> Result<(), super::Error> {
        if self.schema != super::CANDIDATE_ATTEMPT_RESULT_SCHEMA {
            return Err(super::Error::Result("unsupported candidate result schema".into()));
        }
        if !is_blake3(&self.stdout_blake3) || !is_blake3(&self.stderr_blake3) {
            return Err(super::Error::Result("candidate output identity is malformed".into()));
        }
        Ok(())
    }

    fn validate_observed_artifacts(&self) -> Result<(), super::Error> {
        if self.process_failure.is_some() && self.disposition != crate::candidate::CandidateDisposition::Unknown {
            return Err(super::Error::Result("post-start failure must remain unknown".into()));
        }
        for (path, identity) in &self.observed_artifacts {
            crate::candidate::validate_candidate_path(path).map_err(|error| super::Error::Result(error.to_string()))?;
            if !is_blake3(identity) || self.disposition != crate::candidate::CandidateDisposition::Unknown {
                return Err(super::Error::Result("invalid interrupted artifact observation".into()));
            }
        }
        Ok(())
    }

    fn validate_compatibility(&self) -> Result<(), super::Error> {
        if let Some(value) = &self.compatibility {
            let has_invalid_schema = value.schema != super::CANDIDATE_WORKER_COMPATIBILITY_SCHEMA
                || value.result_schema != super::CANDIDATE_ATTEMPT_RESULT_SCHEMA;
            let has_invalid_identity = [
                &value.worker_program_identity,
                &value.environment_identity,
                &value.workspace_provider_identity,
            ]
            .iter()
            .any(|identity| !is_blake3(identity));
            if has_invalid_schema || has_invalid_identity {
                return Err(super::Error::Result("candidate worker compatibility identity is malformed".into()));
            }
        }
        Ok(())
    }

    fn validate_published_paths(&self) -> Result<(), super::Error> {
        for path in &self.published_artifacts {
            crate::candidate::validate_candidate_path(path).map_err(|error| super::Error::Result(error.to_string()))?;
        }
        if self.published_artifacts.windows(2).any(|window| window[0] >= window[1]) {
            return Err(super::Error::Result("published artifact paths must be sorted and unique".into()));
        }
        Ok(())
    }

    fn validate_produced(&self) -> Result<(), super::Error> {
        let receipt = self
            .receipt
            .as_ref()
            .ok_or_else(|| super::Error::Result("produced result is missing a receipt".into()))?;
        receipt.validate()?;
        if self.published_artifacts.iter().any(|path| !receipt.changed_paths.contains(path)) {
            return Err(super::Error::Result("published artifact path is not present in the receipt".into()));
        }
        if self.blocker.is_some() {
            return Err(super::Error::Result("produced result carries a blocker".into()));
        }
        Ok(())
    }

    fn validate_nonproduced(&self) -> Result<(), super::Error> {
        let label = if self.disposition == crate::candidate::CandidateDisposition::Cancelled {
            "cancelled"
        } else {
            "non-produced"
        };
        if self.receipt.is_some() {
            return Err(super::Error::Result(format!("{label} result carries a receipt")));
        }
        if !self.published_artifacts.is_empty() {
            return Err(super::Error::Result(format!("{label} result carries published artifacts")));
        }
        if self.disposition != crate::candidate::CandidateDisposition::Cancelled && self.blocker.is_none() {
            return Err(super::Error::Result("non-produced result is missing a blocker".into()));
        }
        Ok(())
    }
}
