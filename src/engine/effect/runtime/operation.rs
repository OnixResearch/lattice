const RUNTIME_LIMIT_DIAGNOSTIC: &str = "operation process request exceeds current runtime limits";

impl crate::engine::effect::dispatch::AuthorizedAdapterRequest for crate::operation::runtime::AuthorizedProcessRequest {
    type Output = Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError>;

    fn execute(
        &self,
        runtime: &super::Adapters<'_>,
        authorized: &crate::engine::effect::dispatch::AuthorizedAttempt,
    ) -> Result<Self::Output, crate::engine::effect::dispatch::Error> {
        runtime.run_process(authorized, self)
    }
}

impl super::Adapters<'_> {
    pub(super) fn run_process(
        &self,
        authorized: &crate::engine::effect::dispatch::AuthorizedAttempt,
        request: &crate::operation::runtime::AuthorizedProcessRequest,
    ) -> Result<
        Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError>,
        crate::engine::effect::dispatch::Error,
    > {
        let is_ability_match = authorized.attempt.binding.ability.as_str() == super::PROCESS_ABILITY;
        let is_payload_match =
            super::plan_identity(super::PROCESS_PLAN_DOMAIN, &super::process_plan_fields(&request.process))
                == authorized.attempt.binding.payload.as_str();
        if !is_ability_match || !is_payload_match {
            return Err(crate::engine::effect::dispatch::Error::AdapterRejected);
        }
        if process_exceeds_limits(&request.process, self.process) {
            return Ok(Err(crate::agent_node::ProcessError::InvalidRequest(RUNTIME_LIMIT_DIAGNOSTIC)));
        }
        Ok(crate::process::shell::run_bounded_clear_environment_with_cancellation(
            &request.process,
            self.process,
            self.cancellation,
        ))
    }
}

fn process_exceeds_limits(
    request: &crate::agent_node::ProcessRequest,
    runtime: &crate::process::core::RuntimeConfig,
) -> bool {
    let is_timeout_over_limit = request.timeout_secs > runtime.limits.command_agent_timeout_secs;
    let is_input_over_limit = request.stdin.len() > runtime.limits.stdin_max_bytes;
    let is_stdout_over_limit = request.output_limit_bytes > runtime.limits.stdout_max_bytes;
    let is_stderr_over_limit = request.output_limit_bytes > runtime.limits.stderr_max_bytes;
    let is_input_limit_exceeded = is_timeout_over_limit || is_input_over_limit;
    let is_output_limit_exceeded = is_stdout_over_limit || is_stderr_over_limit;
    is_input_limit_exceeded || is_output_limit_exceeded
}
