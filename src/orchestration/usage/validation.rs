// r[impl lattice.agent_orchestration.usage.scenario.uncertainty]
impl super::CommandWorkerObservation {
    pub fn validate(&self) -> Result<(), super::ObservationError> {
        if self.schema != super::USAGE_OBSERVATION_SCHEMA {
            return Err(super::ObservationError::InvalidSchema);
        }
        for identifier in [
            &self.workflow_revision,
            &self.instance_id,
            &self.dispatch_id,
            &self.invocation_id,
            // Host-run receipts explicitly have no qualified worker profile.
            &self.attempt_id,
        ] {
            validate_identifier(identifier)?;
        }
        if self.scope == super::Scope::HostRun && self.accounting.is_none() {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        if let Some(accounting) = &self.accounting {
            self.validate_host_receipt(accounting)?;
        }
        if self.scope != super::Scope::HostRun {
            validate_identifier(&self.worker_profile)?;
        } else if !self.worker_profile.is_empty() || self.node_id != 0 {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        let has_any = self.agent_invocations.is_some()
            || self.model_requests.is_some()
            || self.subagent_calls.is_some()
            || self.tokens.is_some()
            || self.cost_microunits.is_some()
            || self.duration_millis.is_some();
        if self.provenance == super::UsageProvenance::Unknown && (has_any || self.complete) {
            return Err(super::ObservationError::UnknownCarriesValues);
        }
        if self.complete && !has_any {
            return Err(super::ObservationError::EmptyCompleteObservation);
        }
        Ok(())
    }

    fn validate_host_receipt(&self, accounting: &super::RunAccounting) -> Result<(), super::ObservationError> {
        if self.scope != super::Scope::HostRun
            || self.complete
            || self.provenance != super::UsageProvenance::HostMeasured
        {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        let has_worker_counts =
            self.agent_invocations.is_some() || self.model_requests.is_some() || self.subagent_calls.is_some();
        let has_worker_usage = self.tokens.is_some() || self.cost_microunits.is_some();
        if has_worker_counts || has_worker_usage {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        accounting.validate_host_accounting()?;
        if self.duration_millis != accounting.host_interval.as_ref().and_then(super::HostInterval::duration_millis) {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        Ok(())
    }

    pub fn is_estimated_or_unknown(&self) -> bool {
        matches!(self.provenance, super::UsageProvenance::Estimated | super::UsageProvenance::Unknown)
    }

    pub fn to_json(&self) -> Result<String, super::ObservationError> {
        self.validate()?;
        let json =
            serde_json::to_string(self).map_err(|error| super::ObservationError::InvalidJson(error.to_string()))?;
        if json.len() > super::MAX_USAGE_OBSERVATION_BYTES {
            return Err(super::ObservationError::InvalidJson("usage observation exceeds byte limit".to_string()));
        }
        Ok(json)
    }

    pub fn from_json(text: &str) -> Result<Self, super::ObservationError> {
        if text.len() > super::MAX_USAGE_OBSERVATION_BYTES {
            return Err(super::ObservationError::InvalidJson("usage observation exceeds byte limit".to_string()));
        }
        let observation: Self =
            serde_json::from_str(text).map_err(|error| super::ObservationError::InvalidJson(error.to_string()))?;
        observation.validate()?;
        Ok(observation)
    }
}

impl super::RunAccounting {
    fn validate_host_accounting(&self) -> Result<(), super::ObservationError> {
        if self.schema != super::RUN_USAGE_ACCOUNTING_SCHEMA {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        let is_token_and_cost_unknown = self.unknown_tokens && self.unknown_cost;
        let is_call_count_unknown = self.unknown_model_requests && self.unknown_subagent_calls;
        if !is_token_and_cost_unknown || !is_call_count_unknown {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        self.validate_host_evidence()?;
        self.validate_conservative_charge()?;
        self.validate_provider_reports()
    }

    fn validate_host_evidence(&self) -> Result<(), super::ObservationError> {
        if self.outcome.is_some_and(|outcome| outcome.succeeded && outcome.cancelled == Some(true)) {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        if self.diagnostic.len() > super::MAX_ACCOUNTING_DIAGNOSTIC_BYTES {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        if self.host_interval.as_ref().is_some_and(|interval| interval.duration_millis().is_none()) {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        Ok(())
    }

    fn validate_conservative_charge(&self) -> Result<(), super::ObservationError> {
        if self.charged.tokens < self.reserved.tokens
            || self.charged.monetary_microunits < self.reserved.monetary_microunits
        {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        if self.charged.rounds != self.reserved.rounds
            || self.charged.agent_invocations != self.reserved.agent_invocations
        {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        Ok(())
    }

    fn validate_provider_reports(&self) -> Result<(), super::ObservationError> {
        if self.provider_reports.len() > super::MAX_PROVIDER_REPORTS {
            return Err(super::ObservationError::InvalidHostReceipt);
        }
        let mut nodes = std::collections::BTreeSet::new();
        for report in &self.provider_reports {
            validate_identifier(&report.provider)?;
            if !nodes.insert(report.node_id) {
                return Err(super::ObservationError::InvalidHostReceipt);
            }
            if let Some(model) = &report.model {
                validate_identifier(model)?;
            }
        }
        Ok(())
    }
}

fn validate_identifier(value: &str) -> Result<(), super::ObservationError> {
    if value.is_empty() || value.len() > super::MAX_IDENTIFIER_BYTES || value.chars().any(char::is_control) {
        return Err(super::ObservationError::InvalidIdentifier);
    }
    Ok(())
}
