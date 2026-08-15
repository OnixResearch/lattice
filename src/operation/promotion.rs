#![allow(
    unknown_lints,
    assertion_density,
    reason = "promotion uses explicit fallible publication steps with positive, collision, and traversal tests"
)]

use std::io::Write as _;

const PROCESS_EFFECT_POLICY: &str = "process";
const DISABLED_CACHE_POLICY: &str = "disabled";
const NO_RETRY_POLICY: &str = "none";
const METADATA_ARTIFACT_POLICY: &str = "metadata-only";
const NICKEL_EXTENSION: &str = "ncl";
const TEMPORARY_EXTENSION: &str = "tmp";

pub fn workflow_for_plan(
    plan: &super::model::InvocationPlan,
    workflow_name: &str,
) -> Result<(crate::engine::workflow::Workflow, crate::engine::workflow::NodeId), super::Error> {
    super::planning::validate_plan_identity(plan)?;
    validate_no_plaintext_secret_fields(plan)?;
    if !super::identity::valid_identifier(workflow_name) {
        return Err(super::Error::Invalid("promotion workflow name is invalid".to_string()));
    }
    let mut workflow =
        crate::engine::workflow::Workflow::new(workflow_name, crate::engine::workflow::WorkflowTimestamp::epoch());
    workflow.meta.description =
        format!("Typed operation {}:{} bound to plan {}", plan.type_id, plan.method, plan.identity);
    let node_id = workflow.add_operation_node("typed-operation", plan.clone());
    let mut runner =
        crate::engine::workflow::NodeRunnerSettings::new(plan.runtime.kind.clone(), plan.runtime.key.clone());
    runner.effect_policy = PROCESS_EFFECT_POLICY.to_string();
    runner.cache_policy = DISABLED_CACHE_POLICY.to_string();
    runner.retry_policy = NO_RETRY_POLICY.to_string();
    runner.artifact_policy = METADATA_ARTIFACT_POLICY.to_string();
    workflow.set_node_runner_settings(node_id, runner);
    Ok((workflow, node_id))
}

pub fn preview(
    plan: &super::model::InvocationPlan,
    workflow_name: &str,
    destination: &std::path::Path,
) -> Result<super::model::PromotionPlan, super::Error> {
    validate_destination(destination)?;
    let (workflow, _) = workflow_for_plan(plan, workflow_name)?;
    let nickel = crate::workflow_format::workflow_to_nickel(&workflow);
    let content_identity = super::identity::content(nickel.as_bytes());
    let destination = destination
        .to_str()
        .ok_or_else(|| super::Error::Invalid("promotion destination is not valid UTF-8".to_string()))?
        .to_string();
    let identity = super::identity::promotion(&plan.identity, &destination, &content_identity)?;
    Ok(super::model::PromotionPlan {
        schema: super::model::PROMOTION_SCHEMA.to_string(),
        identity,
        invocation_identity: plan.identity.clone(),
        destination,
        content_identity,
        nickel,
        safe_summary: format!(
            "create one inert workflow source for {}:{}; no execution authority is granted",
            plan.type_id, plan.method
        ),
    })
}

pub fn create(
    plan: &super::model::InvocationPlan,
    workflow_name: &str,
    destination: &std::path::Path,
    confirmed: bool,
) -> Result<super::model::PromotionResult, super::Error> {
    ensure_destination_absent(destination)?;
    let promotion = preview(plan, workflow_name, destination)?;
    if !confirmed {
        return Ok(super::model::PromotionResult {
            plan: promotion,
            created: false,
        });
    }
    publish_create_only(destination, &promotion.content_identity, promotion.nickel.as_bytes())?;
    Ok(super::model::PromotionResult {
        plan: promotion,
        created: true,
    })
}

fn publish_create_only(
    destination: &std::path::Path,
    content_identity: &str,
    content: &[u8],
) -> Result<(), super::Error> {
    let parent = destination
        .parent()
        .ok_or_else(|| super::Error::Invalid("promotion destination has no parent directory".to_string()))?;
    let file_name = destination
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| super::Error::Invalid("promotion destination name is not valid UTF-8".to_string()))?;
    let content_suffix = content_identity
        .strip_prefix(super::model::BLAKE3_PREFIX)
        .ok_or_else(|| super::Error::Invalid("promotion content identity is invalid".to_string()))?;
    let temporary_name = format!(".{file_name}.{content_suffix}.{TEMPORARY_EXTENSION}");
    let directory = cap_std::fs::Dir::open_ambient_dir(parent, cap_std::ambient_authority())
        .map_err(|error| super::Error::Io(format!("promotion destination authority failed: {error}")))?;
    let mut is_staging_created = false;
    let publication = (|| {
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = directory
            .open_with(&temporary_name, &options)
            .map_err(|error| super::Error::Io(format!("promotion staging create failed: {error}")))?;
        is_staging_created = true;
        file.write_all(content)
            .map_err(|error| super::Error::Io(format!("promotion staging write failed: {error}")))?;
        file.sync_all()
            .map_err(|error| super::Error::Io(format!("promotion staging sync failed: {error}")))?;
        directory
            .hard_link(&temporary_name, &directory, file_name)
            .map_err(|error| super::Error::Io(format!("promotion create-only publication failed: {error}")))?;
        sync_capability_directory(&directory)?;
        Ok(())
    })();
    let cleanup = if is_staging_created {
        directory.remove_file(&temporary_name)
    } else {
        Ok(())
    };
    match (publication, cleanup) {
        (Ok(()), Ok(())) => sync_capability_directory(&directory),
        (Ok(()), Err(error)) => Err(super::Error::Io(format!("promotion staging cleanup failed: {error}"))),
        (Err(error), _) => Err(error),
    }
}

fn sync_capability_directory(directory: &cap_std::fs::Dir) -> Result<(), super::Error> {
    directory
        .open(".")
        .and_then(|opened_directory| opened_directory.sync_all())
        .map_err(|error| super::Error::Io(format!("promotion directory sync failed: {error}")))
}

fn ensure_destination_absent(destination: &std::path::Path) -> Result<(), super::Error> {
    match std::fs::symlink_metadata(destination) {
        Ok(_) => Err(super::Error::Invalid("promotion destination already exists".to_string())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(super::Error::FileAccess),
    }
}

fn validate_no_plaintext_secret_fields(plan: &super::model::InvocationPlan) -> Result<(), super::Error> {
    if contains_secret_field(&plan.arguments) || contains_secret_field(&plan.target) {
        return Err(super::Error::Invalid(
            "promotion input contains a secret-bearing field outside credential references".to_string(),
        ));
    }
    Ok(())
}

fn contains_secret_field(values: &std::collections::BTreeMap<String, serde_json::Value>) -> bool {
    values
        .iter()
        .any(|(key, value)| super::schema::key_is_secret_bearing(key) || nested_value_contains_secret_field(value))
}

fn nested_value_contains_secret_field(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(values) => values.iter().any(nested_value_contains_secret_field),
        serde_json::Value::Object(values) => values
            .iter()
            .any(|(key, value)| super::schema::key_is_secret_bearing(key) || nested_value_contains_secret_field(value)),
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => false,
    }
}

fn validate_destination(destination: &std::path::Path) -> Result<(), super::Error> {
    if !destination.is_absolute() {
        return Err(super::Error::Invalid("promotion destination must be an absolute path".to_string()));
    }
    if destination
        .components()
        .any(|component| matches!(component, std::path::Component::CurDir | std::path::Component::ParentDir))
    {
        return Err(super::Error::Invalid("promotion destination must not contain traversal components".to_string()));
    }
    if destination.extension().and_then(std::ffi::OsStr::to_str) != Some(NICKEL_EXTENSION) {
        return Err(super::Error::Invalid("promotion destination must use the .ncl extension".to_string()));
    }
    if destination.file_name().is_none() {
        return Err(super::Error::Invalid("promotion destination must name a file".to_string()));
    }
    Ok(())
}
