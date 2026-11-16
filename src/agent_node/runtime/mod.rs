pub(crate) mod host;
mod invocation;
mod profile;

pub use host::ExecutionContext;
pub use host::FakeProcessRunner;
pub use host::ProcessError;
pub use host::ProcessFailurePhase;
pub use host::ProcessOutput;
pub use host::ProcessRequest;
pub use host::ProcessRunner;
pub use host::ProcessTermination;
pub use host::ProductionProcessRunner;
pub use invocation::CommandOutput;
pub use invocation::CommandPlan;
pub use invocation::plan_command;
pub use invocation::run_command_with_runner;
pub use profile::CONFIGURED_RUNTIME_SCHEMA;
pub use profile::CompatibilitySupport;
pub use profile::ConfiguredBinding;
pub use profile::ConfiguredProfile;

pub trait Adapter: Send + Sync {
    /// Material-free identity of the selected backend and its execution configuration.
    /// Planning must not resolve credentials, contact a backend, or perform an effect.
    fn execution_identity(&self, request: &super::Request) -> Result<String, super::Error>;
    fn execute(
        &self,
        request: &super::Request,
        context: &super::ExecutionContext,
    ) -> Result<super::Response, super::Error>;

    /// Execute with an optional run cancellation signal.
    ///
    /// Adapters that can propagate cancellation into their effects should
    /// override this method. The default preserves the existing behavior.
    fn execute_with_cancellation(
        &self,
        request: &super::Request,
        context: &super::ExecutionContext,
        cancellation: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<super::Response, super::Error> {
        let _ = cancellation;
        self.execute(request, context)
    }
}

#[derive(Debug, Default)]
pub struct NoopAdapter;

impl Adapter for NoopAdapter {
    fn execution_identity(&self, request: &super::Request) -> Result<String, super::Error> {
        Err(super::Error::UnsupportedTarget(super::target_label(request.target).to_owned()))
    }
    fn execute(
        &self,
        request: &super::Request,
        _context: &super::ExecutionContext,
    ) -> Result<super::Response, super::Error> {
        Err(super::Error::UnsupportedTarget(super::target_label(request.target).to_string()))
    }
}

enum Route<'a> {
    Command,
    OpenAi(&'a crate::openai::RuntimeConfig),
    Animus(&'a crate::openai::RuntimeConfig),
}

// r[impl n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.route]
// r[impl lattice.agent_runtime.animus_leaf]
// r[impl lattice.agent_runtime.animus_compatibility]
fn route_for_target(config: &super::RuntimeOptions, target: super::Target) -> Result<Route<'_>, super::Error> {
    let target_label = super::target_label(target);
    let has_command_runtime = config.runtimes.contains_key(target_label);
    let openai_runtime = config.openai_compatible.get(target_label);
    match (has_command_runtime, openai_runtime, target) {
        (true, None, super::Target::Pi | super::Target::Clankers) => Ok(Route::Command),
        (false, Some(runtime), super::Target::Pi | super::Target::Clankers) => Ok(Route::OpenAi(runtime)),
        (false, Some(runtime), super::Target::Animus) => Ok(Route::Animus(runtime)),
        (false, None, _) => {
            Err(super::Error::PreflightRejected(format!("missing {target_label} agent runtime configuration")))
        }
        (true, Some(_), _) => {
            Err(super::Error::PreflightRejected(format!("ambiguous {target_label} agent runtime configuration")))
        }
        (true, None, super::Target::Animus) => Err(super::Error::PreflightRejected(
            "Animus target requires an OpenAI-compatible model runtime and cannot use a command runtime".to_string(),
        )),
    }
}

#[derive(Clone)]
pub struct ProductionAdapter {
    config: super::RuntimeOptions,
    openai_transport: std::sync::Arc<dyn crate::openai::Transport>,
}

impl ProductionAdapter {
    pub fn new(config: super::RuntimeOptions) -> Self {
        Self {
            config,
            openai_transport: crate::openai::default_transport(),
        }
    }

    pub fn new_with_openai_transport(
        config: super::RuntimeOptions,
        openai_transport: std::sync::Arc<dyn crate::openai::Transport>,
    ) -> Self {
        Self {
            config,
            openai_transport,
        }
    }
}

impl Default for ProductionAdapter {
    fn default() -> Self {
        Self::new(super::RuntimeOptions::default())
    }
}

// r[impl lattice.agent_runtime.structured_input_projection.scenario.selected]
// r[impl lattice.agent_runtime.projection_boundaries.scenario.evidence]
impl Adapter for ProductionAdapter {
    fn execution_identity(&self, request: &super::Request) -> Result<String, super::Error> {
        Ok(configured_binding(&self.config, request.target)?.revision)
    }
    fn execute(
        &self,
        request: &super::Request,
        context: &super::ExecutionContext,
    ) -> Result<super::Response, super::Error> {
        self.execute_with_cancellation(request, context, None)
    }
    fn execute_with_cancellation(
        &self,
        request: &super::Request,
        context: &super::ExecutionContext,
        cancellation: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<super::Response, super::Error> {
        let projection = self.config.structured_input_projection.for_target(request.target);
        let binding = configured_binding(&self.config, request.target)?;
        let response = match route_for_target(&self.config, request.target)? {
            Route::Command => {
                let plan = super::plan_command(&self.config, request)?;
                let output = invocation::run_command(&plan, &self.config.local_process, context, cancellation)?;
                super::decode_command(&plan, &output)
            }
            Route::OpenAi(runtime) => {
                let global_output_max_bytes = global_output_max_bytes(&self.config)?;
                crate::openai::execute_with_projection(
                    crate::openai::ProjectedChat {
                        runtime,
                        request,
                        projection,
                        global_output_max_bytes,
                    },
                    context,
                    self.openai_transport.as_ref(),
                )
            }
            Route::Animus(runtime) => crate::animus_agent::execute_with_projection(
                crate::openai::ProjectedChat {
                    runtime,
                    request,
                    projection,
                    global_output_max_bytes: global_output_max_bytes(&self.config)?,
                },
                context,
                self.openai_transport.as_ref(),
            ),
        }?;
        let mut response = attach_projection_metadata(response, projection, request.json_inputs.len());
        if let Some(metadata) = response.metadata.as_object_mut() {
            metadata.insert("configured_runtime".to_string(), serde_json::json!(binding));
        }
        Ok(response)
    }
}

fn attach_projection_metadata(
    mut response: super::Response,
    projection: crate::agent_prompt::StructuredInputProjection,
    input_count: usize,
) -> super::Response {
    let facts = serde_json::json!({
        "profile": projection.profile_id(),
        "implementation": projection.implementation(),
        "structured_input_count": input_count,
    });
    if let Some(metadata) = response.metadata.as_object_mut() {
        metadata.insert("structured_input_projection".to_string(), facts);
    } else {
        response.metadata = serde_json::json!({
            "provider_metadata": response.metadata,
            "structured_input_projection": facts,
        });
    }
    response
}

fn global_output_max_bytes(config: &super::RuntimeOptions) -> Result<u64, super::Error> {
    u64::try_from(config.max_output_bytes)
        .map_err(|_| super::Error::PreflightRejected("global agent output bound is unsupported".to_string()))
}

fn configured_binding(
    config: &super::RuntimeOptions,
    target: super::Target,
) -> Result<ConfiguredBinding, super::Error> {
    let route = match route_for_target(config, target)? {
        Route::Command => "command",
        Route::OpenAi(_) => "openai-compatible",
        Route::Animus(_) => "animus",
    };
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"lattice.agent.selected-runtime.v1");
    serde_json::to_writer(
        &mut hasher,
        &(route, config, &config.local_process.shell_program, &config.local_process.limits),
    )
    .map_err(|_| super::Error::PreflightRejected("agent runtime identity unavailable".to_owned()))?;
    Ok(ConfiguredBinding {
        schema: CONFIGURED_RUNTIME_SCHEMA.to_string(),
        target,
        revision: format!("blake3:{}", hasher.finalize().to_hex()),
    })
}
