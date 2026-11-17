use anyhow::Context;

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Read-only identity measurement with the existing operation framing.
    ExecutorIdentity {
        #[arg(long)]
        file: std::path::PathBuf,
    },
    /// Validate and retain an inert exact operation graph. Does not grant effects.
    Prepare {
        #[arg(long)]
        request: std::path::PathBuf,
        #[arg(long)]
        out: std::path::PathBuf,
        #[arg(long)]
        lattice: std::path::PathBuf,
        #[arg(long)]
        repository: std::path::PathBuf,
        #[arg(long, default_value = "advance")]
        method: String,
    },
    /// Exact closed runtime entrypoint. Requires an owner-current binding.
    Step {
        #[arg(long)]
        binding: std::path::PathBuf,
        #[arg(long)]
        identity: String,
        #[arg(long)]
        method: String,
        #[arg(long)]
        permit: std::path::PathBuf,
        #[arg(long)]
        permit_identity: String,
    },
}

pub fn run(config: &crate::config::AppConfig, args: &Args) -> std::process::ExitCode {
    match run_inner(config, args) {
        Ok(value) => match serde_json::to_string_pretty(&value) {
            Ok(value) => {
                println!("{value}");
                std::process::ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("render receipt encoding failed: {error}");
                std::process::ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("render command rejected: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run_inner(config: &crate::config::AppConfig, args: &Args) -> anyhow::Result<serde_json::Value> {
    match &args.command {
        Command::ExecutorIdentity { file } => Ok(serde_json::json!({
            "identity": crate::operation::identity::executor_file(file)?
        })),
        Command::Prepare {
            request,
            out,
            lattice,
            repository,
            method,
        } => prepare(request, out, lattice, repository, method),
        Command::Step {
            binding,
            identity,
            method,
            permit,
            permit_identity,
        } => {
            let expected = molten_batch_core::Id::parse(identity).map_err(|e| anyhow::anyhow!("{e}"))?;
            let request = super::shell::load_request(binding, expected)?;
            let permit_id = molten_batch_core::Id::parse(permit_identity).map_err(|e| anyhow::anyhow!("{e}"))?;
            let permit = super::shell::load_permit(&request, permit, permit_id)?;
            anyhow::ensure!(permit.method == *method, "crossed dispatch method");
            let fault = if method == "advance" {
                request.route.fixture_fault.unwrap_or(super::shell::Fault::None)
            } else {
                super::shell::Fault::None
            };
            let mut producer = super::shell::Producer {
                request: &request,
                permit: &permit,
                process: config.effective_agent_runtime().local_process,
            };
            let receipt = super::shell::observe(&request, &mut producer, &permit, fault)?;
            Ok(serde_json::to_value(receipt)?)
        }
    }
}

pub fn admitted_graph(
    catalog: &crate::operation::Catalog,
    instances: &crate::operation::InstanceFile,
    executable: &std::path::Path,
    method: &str,
) -> anyhow::Result<(crate::operation::InvocationPlan, crate::engine::workflow::Workflow)> {
    let validated = crate::operation::catalog::validate(catalog.clone())?;
    let mut admitted = crate::operation::instance::validate(instances.clone(), &validated)?;
    let instance = admitted.remove("admitted-render").context("missing exact render instance")?;
    let definition =
        &validated.types.get(super::operation::TYPE_ID).context("missing render operation type")?.definition;
    anyhow::ensure!(super::operation::type_is_supported(definition), "render operation profile changed");
    anyhow::ensure!(super::operation::METHODS.contains(&method), "unsupported render method");
    let plan = crate::operation::planning::plan(crate::operation::model::PlanRequest {
        catalog: &validated,
        instance: &instance,
        method,
        arguments: std::collections::BTreeMap::new(),
        expected_catalog_identity: &validated.identity,
        expected_instance_identity: &instance.identity,
        runtime: crate::operation::model::RuntimeFact {
            kind: super::operation::RUNTIME_KIND.into(),
            key: super::operation::RUNTIME_KEY.into(),
            executor_identity: crate::operation::identity::executor_file(executable)?,
            executable_path: executable.to_str().context("executable path")?.into(),
            available: true,
            maximum_bounds: definition.bounds,
        },
        // Planning permission admits an inert mutation plan, not an execution
        // grant. Runtime still needs the existing configured graph authority.
        policy: crate::operation::model::CapabilityPolicy {
            allowed: vec![crate::operation::model::CapabilityClass::ProcessRead],
            allow_mutation: true,
        },
        preflight: crate::operation::model::PreflightFacts {
            executable_available: true,
            repository_root_admitted: true,
            credential_references_valid: true,
        },
    })?;
    assert_eq!(plan.type_id, super::operation::TYPE_ID, "planner must retain the admitted render type");
    assert_eq!(plan.method, method, "planner must retain the exact admitted method");
    let (workflow, _) = crate::operation::promotion::workflow_for_plan(&plan, "artifact-bound-render")?;
    Ok((plan, workflow))
}

fn write_new(path: &std::path::Path, bytes: &[u8]) -> anyhow::Result<()> {
    use std::io::Write as _;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::File::open(path.parent().context("parent directory")?)?.sync_all()?;
    Ok(())
}

fn prepare(
    request: &std::path::Path,
    out: &std::path::Path,
    lattice: &std::path::Path,
    repository: &std::path::Path,
    method: &str,
) -> anyhow::Result<serde_json::Value> {
    anyhow::ensure!(
        out.is_absolute() && lattice.is_absolute() && repository.is_absolute(),
        "absolute admitted paths required"
    );
    anyhow::ensure!(super::operation::METHODS.contains(&method), "unsupported render method");
    let binding: super::Request = serde_json::from_slice(&super::shell::read_bounded(request, 256 * 1024)?)?;
    binding.prepare()?;
    let id = binding.id()?;
    // The destination is create-only: graph metadata cannot change the
    // manifest/owning attempt hidden behind an already admitted plan.
    std::fs::create_dir(out)?;
    let file = out.join("binding.json");
    write_new(&file, &serde_json::to_vec_pretty(&binding)?)?;
    let catalog = super::operation::catalog(lattice)?;
    assert_eq!(catalog.types.len(), 1, "render authoring requires one closed producer profile");
    let instances = super::operation::instance(&catalog, repository, &file, id);
    assert_eq!(instances.instances.len(), 1, "render authoring must admit exactly one bound instance");
    let (plan, workflow) = admitted_graph(&catalog, &instances, lattice, method)?;
    write_new(&out.join("catalog.json"), &serde_json::to_vec_pretty(&catalog)?)?;
    write_new(&out.join("instances.json"), &serde_json::to_vec_pretty(&instances)?)?;
    write_new(&out.join("plan.json"), &serde_json::to_vec_pretty(&plan)?)?;
    let nickel = crate::workflow_format::workflow_to_nickel(&workflow);
    write_new(&out.join("render.ncl"), nickel.as_bytes())?;
    let prepared = binding.prepare()?;
    write_new(&out.join("manifest.frame"), &binding.manifest.encode()?)?;
    write_new(&out.join("job.json"), &serde_json::to_vec_pretty(&prepared.job)?)?;
    Ok(serde_json::json!({"binding": id, "producer_revision": super::PRODUCER_REVISION,
        "run": binding.run_id()?, "graph_content_identity": crate::operation::identity::content(nickel.as_bytes()),
        "operation_plan_identity": plan.identity, "catalog_identity": plan.catalog_identity,
        "authority_granted": false, "qualification": "authored_not_executed"}))
}
