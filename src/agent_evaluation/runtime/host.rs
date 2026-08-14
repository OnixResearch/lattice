mod protocol;
mod services;

const DRIVER_PROTOCOL_ARG_COUNT: usize = 2;
const DRIVER_PROTOCOL_VERSION_ARG: &str = "lattice-agent-evaluation-driver-v1";
const REQUIRED_CLOCK_OBSERVATIONS: usize = 2;
const MAX_DRIVER_ARGS: usize = 32;
const MAX_DRIVER_ARG_BYTES: usize = 4_096;
const MAX_DRIVER_REQUEST_BYTES: usize = 1_048_576;
const MAX_DRIVER_OUTPUT_BYTES: u64 = 1_048_576;
const MAX_DRIVER_TIMEOUT_SECS: u64 = 3_600;
const SUCCESS_EXIT_CODE: i32 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverConfig {
    pub driver_identity: String,
    pub agent_profile_identity: String,
    pub environment_identity: String,
    pub policy_identity: String,
    pub program: String,
    pub args: Vec<String>,
    pub current_directory: String,
    pub timeout_secs: u64,
    pub output_max_bytes: u64,
}

pub trait DriverProcessPort {
    fn run(
        &self,
        request: &crate::agent_node::ProcessRequest,
    ) -> Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError>;
}

pub struct ProductionDriverProcess {
    config: crate::process::core::RuntimeConfig,
}

impl DriverProcessPort for ProductionDriverProcess {
    fn run(
        &self,
        request: &crate::agent_node::ProcessRequest,
    ) -> Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError> {
        crate::process::shell::run_bounded_clear_environment(request, &self.config)
    }
}

pub struct Driver<P> {
    config: DriverConfig,
    process: P,
    clock_observations: std::cell::RefCell<std::collections::VecDeque<u64>>,
}

impl Driver<ProductionDriverProcess> {
    pub fn new_production(
        config: DriverConfig,
        clock_observations: [u64; REQUIRED_CLOCK_OBSERVATIONS],
        process_config: crate::process::core::RuntimeConfig,
    ) -> Result<Self, crate::agent_evaluation::Error> {
        Self::new_with_process(config, clock_observations, ProductionDriverProcess { config: process_config })
    }
}

impl<P: DriverProcessPort> Driver<P> {
    pub fn new_with_process(
        config: DriverConfig,
        clock_observations: [u64; REQUIRED_CLOCK_OBSERVATIONS],
        process: P,
    ) -> Result<Self, crate::agent_evaluation::Error> {
        validate_config(&config)?;
        if clock_observations[0] > clock_observations[1] {
            return Err(crate::agent_evaluation::Error::ServiceUnavailable);
        }
        let observations = std::collections::VecDeque::from(clock_observations);
        debug_assert_eq!(observations.len(), REQUIRED_CLOCK_OBSERVATIONS);
        debug_assert!(clock_observations[0] <= clock_observations[1]);
        Ok(Self {
            config,
            process,
            clock_observations: std::cell::RefCell::new(observations),
        })
    }

    fn invoke(&self, invocation: Invocation<'_>) -> Result<protocol::ResultValue, crate::agent_evaluation::Error> {
        self.validate_binding(invocation.case, invocation.trial)?;
        let request = protocol::Request {
            schema: protocol::REQUEST_SCHEMA,
            driver_identity: &self.config.driver_identity,
            operation: invocation.operation,
            case_definition: &invocation.case.definition,
            case_revision_identity: &invocation.case.revision_identity,
            trial: invocation.trial,
            execution_output_identity: invocation.execution_output_identity,
            live_admission: invocation.live_admission,
            live_observation: invocation.live_observation,
            artifact_manifest: invocation.artifact_manifest,
        };
        let input = serde_json::to_vec(&request).map_err(|_| crate::agent_evaluation::Error::Encoding)?;
        if input.is_empty() || input.len() > MAX_DRIVER_REQUEST_BYTES {
            return Err(crate::agent_evaluation::Error::ServiceUnavailable);
        }
        let maximum_args = self
            .config
            .args
            .len()
            .checked_add(DRIVER_PROTOCOL_ARG_COUNT)
            .ok_or(crate::agent_evaluation::Error::ServiceUnavailable)?;
        let mut args = Vec::with_capacity(maximum_args);
        args.extend(self.config.args.iter().cloned());
        args.push(DRIVER_PROTOCOL_VERSION_ARG.to_string());
        args.push(invocation.operation.as_str().to_string());
        let output_max_bytes = usize::try_from(self.config.output_max_bytes)
            .map_err(|_| crate::agent_evaluation::Error::ServiceUnavailable)?;
        let process_request = crate::agent_node::ProcessRequest {
            program: self.config.program.clone(),
            args,
            cwd: Some(self.config.current_directory.clone()),
            env: std::collections::BTreeMap::new(),
            stdin: input,
            timeout_secs: self.config.timeout_secs,
            output_limit_bytes: output_max_bytes,
        };
        let output = self.process.run(&process_request).map_err(|_| crate::agent_evaluation::Error::ServiceFailure)?;
        if output.status_code != Some(SUCCESS_EXIT_CODE) || output.stdout.is_empty() {
            return Err(crate::agent_evaluation::Error::ServiceFailure);
        }
        let response: protocol::Response =
            serde_json::from_slice(&output.stdout).map_err(|_| crate::agent_evaluation::Error::ServiceFailure)?;
        self.validate_response(&response, invocation.trial)?;
        debug_assert_eq!(process_request.env.len(), 0);
        debug_assert!(process_request.output_limit_bytes > 0);
        Ok(response.result)
    }

    fn validate_binding(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
    ) -> Result<(), crate::agent_evaluation::Error> {
        let has_invalid_binding = [
            trial.case_revision_identity != case.revision_identity,
            trial.agent_profile_identity != self.config.agent_profile_identity,
            trial.environment_identity != self.config.environment_identity,
            trial.policy_identity != self.config.policy_identity,
        ]
        .into_iter()
        .any(|value| value);
        if has_invalid_binding {
            return Err(crate::agent_evaluation::Error::CaseDrift);
        }
        Ok(())
    }

    fn validate_response(
        &self,
        response: &protocol::Response,
        trial: &crate::agent_evaluation::planning::TrialPlan,
    ) -> Result<(), crate::agent_evaluation::Error> {
        let has_invalid_response = [
            response.schema != protocol::RESPONSE_SCHEMA,
            response.driver_identity != self.config.driver_identity,
            response.trial_identity != trial.trial_identity,
        ]
        .into_iter()
        .any(|value| value);
        if has_invalid_response {
            return Err(crate::agent_evaluation::Error::ServiceFailure);
        }
        Ok(())
    }
}

struct Invocation<'a> {
    operation: protocol::Operation,
    case: &'a crate::agent_evaluation::model::AdmittedCase,
    trial: &'a crate::agent_evaluation::planning::TrialPlan,
    execution_output_identity: Option<&'a str>,
    live_admission: Option<&'a crate::agent_evaluation::live::Admission>,
    live_observation: Option<&'a crate::agent_evaluation::live::Observation>,
    artifact_manifest: Option<&'a crate::agent_evaluation::artifacts::ArtifactManifest>,
}

fn validate_config(config: &DriverConfig) -> Result<(), crate::agent_evaluation::Error> {
    crate::agent_evaluation::identity::validate(&config.driver_identity)?;
    crate::agent_evaluation::identity::validate(&config.agent_profile_identity)?;
    crate::agent_evaluation::identity::validate(&config.environment_identity)?;
    crate::agent_evaluation::identity::validate(&config.policy_identity)?;
    let is_invalid_program = config.program.is_empty() || !std::path::Path::new(&config.program).is_absolute();
    let is_invalid_directory =
        config.current_directory.is_empty() || !std::path::Path::new(&config.current_directory).is_absolute();
    let is_invalid_timeout = config.timeout_secs == 0 || config.timeout_secs > MAX_DRIVER_TIMEOUT_SECS;
    let is_invalid_output_bound = config.output_max_bytes == 0 || config.output_max_bytes > MAX_DRIVER_OUTPUT_BYTES;
    let is_invalid_args = config.args.len() > MAX_DRIVER_ARGS
        || config.args.iter().any(|argument| argument.len() > MAX_DRIVER_ARG_BYTES || argument.contains('\0'));
    let has_invalid_config = [
        is_invalid_program,
        is_invalid_directory,
        is_invalid_timeout,
        is_invalid_output_bound,
        is_invalid_args,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_config {
        return Err(crate::agent_evaluation::Error::ServiceUnavailable);
    }
    debug_assert!(config.timeout_secs <= MAX_DRIVER_TIMEOUT_SECS);
    debug_assert!(config.output_max_bytes <= MAX_DRIVER_OUTPUT_BYTES);
    Ok(())
}
