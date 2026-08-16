#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the operation CLI uses its local clap, serialization, and contract vocabulary"
)]
#![allow(
    assertion_density,
    reason = "fallible CLI orchestration delegates invariants to typed admission functions and direct negative tests"
)]
use serde::Serialize;

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// List admitted operation types and data-only instances.
    Catalog(CatalogArgs),
    /// Show one operation type or instance without loading credentials or executors.
    Show(ShowArgs),
    /// List typed methods for one operation type.
    Methods(MethodsArgs),
    /// Build one pure identity-bound invocation plan.
    Plan(Box<PlanArgs>),
    /// Preview or explicitly run one read-only invocation.
    Run(Box<RunArgs>),
    /// Preview or create one inert Nickel workflow source.
    Promote(Box<PromoteArgs>),
}

#[derive(clap::Args, Debug)]
pub struct CatalogArgs {
    #[arg(long)]
    pub catalog: std::path::PathBuf,
    #[arg(long)]
    pub instances: std::path::PathBuf,
}

#[derive(clap::Args, Debug)]
pub struct ShowArgs {
    #[command(flatten)]
    pub source: CatalogArgs,
    #[arg(long)]
    pub id: String,
}

#[derive(clap::Args, Debug)]
pub struct MethodsArgs {
    #[command(flatten)]
    pub source: CatalogArgs,
    #[arg(long)]
    pub type_id: String,
}

#[derive(clap::Args, Debug)]
pub struct PlanArgs {
    #[command(flatten)]
    pub source: CatalogArgs,
    #[arg(long)]
    pub instance: String,
    #[arg(long)]
    pub method: String,
    #[arg(long, default_value = "{}")]
    pub arguments_json: String,
    #[arg(long)]
    pub executable: std::path::PathBuf,
}

#[derive(clap::Args, Debug)]
pub struct RunArgs {
    #[command(flatten)]
    pub plan: PlanArgs,
    /// Grant one exact local read-only process request after preview.
    #[arg(long)]
    pub confirm_read_only: bool,
}

#[derive(clap::Args, Debug)]
pub struct PromoteArgs {
    #[command(flatten)]
    pub plan: PlanArgs,
    #[arg(long)]
    pub workflow_name: String,
    #[arg(long)]
    pub destination: std::path::PathBuf,
    #[arg(long)]
    pub confirm_create: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CatalogReport {
    pub catalog_identity: String,
    pub types: Vec<super::format::TypeMetadata>,
    pub instances: Vec<super::format::InstanceMetadata>,
}

pub struct PlanInput<'a> {
    pub catalog_path: &'a std::path::Path,
    pub instances_path: &'a std::path::Path,
    pub instance_id: &'a str,
    pub method: &'a str,
    pub arguments_json: &'a str,
    pub executable_path: &'a std::path::Path,
}

pub(crate) struct AdmittedPlan {
    pub(crate) catalog: super::model::ValidatedCatalog,
    pub(crate) instance: super::model::ValidatedInstance,
    pub(crate) plan: super::model::InvocationPlan,
}

pub fn catalog_report(
    catalog_path: &std::path::Path,
    instances_path: &std::path::Path,
) -> Result<CatalogReport, super::Error> {
    let catalog = super::format::load_catalog(catalog_path)?;
    let instances = super::format::load_instances(instances_path, &catalog)?;
    Ok(CatalogReport {
        catalog_identity: catalog.identity.clone(),
        types: super::format::type_metadata(&catalog),
        instances: super::format::instance_metadata(&instances),
    })
}

pub fn admit_plan(input: PlanInput<'_>) -> Result<super::model::InvocationPlan, super::Error> {
    Ok(admit_for_pack(input)?.plan)
}

pub(crate) fn admit_for_pack(input: PlanInput<'_>) -> Result<AdmittedPlan, super::Error> {
    admit_for_pack_with_runtime(input, &crate::process::core::RuntimeConfig::default())
}

#[allow(
    unknown_lints,
    function_length,
    reason = "one admission function keeps the exact catalog, instance, executable, policy, and preflight sequence visible"
)]
pub(crate) fn admit_for_pack_with_runtime(
    input: PlanInput<'_>,
    process_runtime: &crate::process::core::RuntimeConfig,
) -> Result<AdmittedPlan, super::Error> {
    let catalog = super::format::load_catalog(input.catalog_path)?;
    let mut instances = super::format::load_instances(input.instances_path, &catalog)?;
    let instance = instances.remove(input.instance_id).ok_or_else(|| {
        super::Error::Diagnostics(vec![super::diagnostic(
            super::model::DiagnosticCode::InvalidValue,
            "instance_id",
            "operation instance is not present",
        )])
    })?;
    let operation_type = catalog.types.get(&instance.definition.type_id).ok_or(super::Error::Encoding)?;
    if !operation_type.definition.methods.iter().any(|method| method.name == input.method) {
        return Err(super::Error::Diagnostics(vec![super::diagnostic(
            super::model::DiagnosticCode::MissingMethod,
            "method",
            "operation method is not present in the admitted type",
        )]));
    }
    if input.arguments_json.len() > operation_type.definition.bounds.input_bytes {
        return Err(super::Error::Diagnostics(vec![super::diagnostic(
            super::model::DiagnosticCode::Limit,
            "arguments",
            "operation argument source exceeds the admitted input bound",
        )]));
    }
    let arguments = super::format::parse_arguments(input.arguments_json)?;
    let executor_identity = super::identity::executor_file(input.executable_path)?;
    let executable_path = input.executable_path.to_str().ok_or(super::Error::FileAccess)?.to_string();
    let is_repository_root_admitted = repository_root_admitted(&instance);
    let is_credential_reference_set_valid = instance
        .definition
        .credentials
        .values()
        .all(|reference| crate::credentials::validate_secret_ref(reference).is_ok());
    let runtime = super::model::RuntimeFact {
        kind: operation_type.definition.runtime.kind.clone(),
        key: operation_type.definition.runtime.key.clone(),
        executor_identity,
        executable_path,
        available: true,
        maximum_bounds: super::model::Bounds {
            timeout_seconds: process_runtime.limits.command_agent_timeout_secs,
            input_bytes: process_runtime.limits.stdin_max_bytes.min(super::model::MAXIMUM_OPERATION_BYTES),
            output_bytes: process_runtime
                .limits
                .stdout_max_bytes
                .min(process_runtime.limits.stderr_max_bytes)
                .min(super::model::MAXIMUM_OPERATION_BYTES),
            artifact_bytes: super::model::MAXIMUM_OPERATION_BYTES,
        },
    };
    let policy = super::model::CapabilityPolicy {
        allowed: operation_type.definition.capabilities.clone(),
        allow_mutation: false,
    };
    let expected_catalog_identity = catalog.identity.clone();
    let expected_instance_identity = instance.identity.clone();
    let plan = super::planning::plan(super::model::PlanRequest {
        catalog: &catalog,
        instance: &instance,
        method: input.method,
        arguments,
        expected_catalog_identity: &expected_catalog_identity,
        expected_instance_identity: &expected_instance_identity,
        runtime,
        policy,
        preflight: super::model::PreflightFacts {
            executable_available: true,
            repository_root_admitted: is_repository_root_admitted,
            credential_references_valid: is_credential_reference_set_valid,
        },
    })?;
    super::planning::validate_plan_admission(&plan, &catalog, &instance)?;
    Ok(AdmittedPlan {
        catalog,
        instance,
        plan,
    })
}

include!("command.rs");
