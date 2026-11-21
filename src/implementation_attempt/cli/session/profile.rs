//! The reviewed harness template a worker profile revision frames.

/// The reviewed harness template a profile revision frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HarnessTemplate<'a> {
    pub request_schema: &'a str,
    pub launch: &'a [&'a str],
    pub print_options: &'a [&'a str],
}

/// The effective constants: this shell's request schema and the OMP launch
/// prefix and print options it actually passes to the worker.
pub(crate) const EFFECTIVE_TEMPLATE: HarnessTemplate<'static> = HarnessTemplate {
    request_schema: super::super::REQUEST_SCHEMA,
    launch: &crate::candidate_worker::omp::profile::LAUNCH_PREFIX,
    print_options: &crate::candidate_worker::omp::profile::PRINT_OPTIONS,
};

impl super::super::Request {
    /// The revision of this profile under `template`.
    pub(crate) fn framed_revision(&self, template: HarnessTemplate<'_>) -> Result<String, String> {
        let capability = match &self.worker_capability {
            None => None,
            Some(declared) => {
                let canonical = declared
                    .canonical()
                    .map_err(|defect| format!("worker capability declaration is unsupported: {defect:?}"))?;
                Some(serde_json::to_string(&canonical).map_err(|error| error.to_string())?)
            }
        };
        let compute = serde_json::to_string(&self.worker_compute).map_err(|error| error.to_string())?;
        let broker = &self.omp.model_broker;
        let credential = broker.credential.as_ref();
        let read_paths: Vec<&str> = self.read_paths.iter().map(String::as_str).collect();
        let write_paths: Vec<&str> = self.write_paths.iter().map(String::as_str).collect();
        let non_utf8 = "worker runtime path is not UTF-8";
        let relay_path = broker.relay.path.to_str().ok_or(non_utf8)?;
        let shell_path = broker.shell.path.to_str().ok_or(non_utf8)?;
        let single_shot =
            crate::content_identity::worker_profile_revision(crate::content_identity::WorkerProfileRevisionInput {
                request_schema: template.request_schema,
                harness_launch: template.launch,
                harness_print_options: template.print_options,
                program: &self.program,
                sandbox_program: &self.sandbox_program,
                harness_revision: &self.omp.revision,
                harness_executable_blake3: &self.omp.executable_blake3,
                harness_model: &self.omp.model,
                broker_schema: &broker.schema,
                broker_endpoint: &broker.endpoint,
                broker_model: &broker.model,
                credential_provider: credential.map(|reference| reference.provider.as_str()),
                credential_key: credential.map(|reference| reference.key.as_str()),
                credential_field: credential.and_then(|reference| reference.field.as_deref()),
                relay: crate::content_identity::RuntimeArtifactInput {
                    path: relay_path,
                    blake3: &broker.relay.blake3,
                },
                shell: crate::content_identity::RuntimeArtifactInput {
                    path: shell_path,
                    blake3: &broker.shell.blake3,
                },
                sandbox_blake3: &broker.sandbox_blake3,
                max_requests: u64::from(broker.max_requests),
                read_paths: &read_paths,
                write_paths: &write_paths,
                output_limit_bytes: u64::try_from(self.output_limit_bytes).map_err(|error| error.to_string())?,
                allow_unchanged: self.allow_unchanged,
                compute: &compute,
                capability: capability.as_deref(),
            })
            .map_err(|error| error.to_string())?;
        // A repair-capable profile is its single-shot revision bound to the
        // repair policy; without a policy the revision is unchanged.
        // r[impl lattice.implementation_attempt.repair.scenario.policy]
        Ok(match &self.repair {
            None => single_shot,
            Some(policy) => crate::implementation_attempt::repair::profile_revision(&single_shot, policy),
        })
    }
}
