use std::collections::BTreeSet;
use std::path::Path;

mod common;

use lattice::orchestration::AuthorityPolicy;
use lattice::orchestration::BudgetCharge;
use lattice::orchestration::BudgetDimensions;
use lattice::orchestration::BudgetLimits;
use lattice::orchestration::BudgetPolicy;
use lattice::orchestration::CompletionPolicy;
use lattice::orchestration::DefinitionId;
use lattice::orchestration::OrchestrationDefinition;
use lattice::orchestration::RedactionPolicy;
use lattice::workflow_template::ActivationBlock;
use lattice::workflow_template::ActivationFacts;
use lattice::workflow_template::ActivationPlan;
use lattice::workflow_template::ApplicationPlan;
use lattice::workflow_template::AssetBinding;
use lattice::workflow_template::BundleFile;
use lattice::workflow_template::BundleManifest;
use lattice::workflow_template::BundleValidationInput;
use lattice::workflow_template::CompatibilityRequirements;
use lattice::workflow_template::ContentIdentity;
use lattice::workflow_template::DiagnosticCode;
use lattice::workflow_template::FileBinding;
use lattice::workflow_template::InitializationError;
use lattice::workflow_template::MAX_BUNDLE_BYTES;
use lattice::workflow_template::MAX_CATALOG_BUNDLES;
use lattice::workflow_template::MAX_FILE_BYTES;

const CONFIG_FILE_NAME: &str = "config.ncl";
const WORKFLOW_TEMPLATE_ROOT: &str = "workflow-templates";
const BUILTIN_BUNDLE_ID: &str = "shell-preview";
const MANIFEST_FILE_NAME: &str = "manifest.ncl";
const WORKFLOW_FILE_NAME: &str = "workflow.ncl";
const EXTRA_FILE_NAME: &str = "extra.txt";
const MALFORMED_BUNDLE_ID: &str = "malformed";
const ESCAPE_BUNDLE_ID: &str = "escape";
const OPERATOR_BYTES: &str = "operator-owned";
const ACTOR: &str = "operator:template-reviewer";
const DEFINITION_ID: &str = "template-orchestration";
const ONE_ROUND: u64 = 1;
const TOKEN_LIMIT: u64 = 1_000;
const DURATION_LIMIT_MILLIS: u64 = 60_000;
const MONEY_LIMIT_MICROUNITS: u64 = 1_000;
const PURE_WORKFLOW_NAME: &str = "pure-template";
const PURE_WORKFLOW_FILE: &str = "workflow.ncl";
const PURE_WORKFLOW_BYTES: &[u8] = b"captured workflow bytes";
const PURE_MANIFEST_BYTES: &[u8] = b"captured manifest bytes";
const OPTIONAL_ASSET_PATH: &str = "assets/note.txt";
const EXPECTED_BOUND_FILE_COUNT: usize = 1;
const HEX_CHARS_PER_BYTE: usize = 2;
const TUI_FRAME_WIDTH: u16 = 100;
const TUI_FRAME_HEIGHT: u16 = 30;
const CATALOG_FIXTURE_INDEX_WIDTH: usize = 3;
const PREVIEW_RUN_RECORD_LIMIT: usize = 32;
type ActivationFactMutation = fn(&mut ActivationFacts);
type ActivationFailureCase = (ActivationFactMutation, ActivationBlock);

fn initialized_root() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary workflow-template root");
    let config = directory.path().join(CONFIG_FILE_NAME);
    lattice::workflow_template::initialize(&config).expect("fresh workflow-template initialization succeeds");
    directory
}

fn load_builtin(directory: &tempfile::TempDir) -> lattice::workflow_template::ValidatedBundle {
    let catalog = lattice::workflow_template::load_catalog(&directory.path().join(WORKFLOW_TEMPLATE_ROOT))
        .expect("workflow-template catalog loads");
    assert!(catalog.diagnostics.is_empty(), "diagnostics: {:?}", catalog.diagnostics);
    assert_eq!(catalog.bundles.len(), ONE_ROUND as usize);
    catalog.bundles.into_iter().next().expect("built-in bundle exists")
}

fn pure_validation_input() -> BundleValidationInput {
    let mut workflow =
        lattice::engine::Workflow::new(PURE_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node("preview", "true");
    BundleValidationInput {
        manifest_bytes: PURE_MANIFEST_BYTES.to_vec(),
        manifest: BundleManifest {
            schema_version: lattice::workflow_template::CURRENT_BUNDLE_SCHEMA_VERSION,
            id: PURE_WORKFLOW_NAME.to_string(),
            display_name: "Pure template".to_string(),
            description: "Pure validation fixture".to_string(),
            category: "tests".to_string(),
            workflow: FileBinding {
                path: PURE_WORKFLOW_FILE.to_string(),
                blake3: ContentIdentity::from_bytes(PURE_WORKFLOW_BYTES),
            },
            orchestration: None,
            assets: Vec::new(),
            compatibility: CompatibilityRequirements {
                workflow_schema: lattice::workflow_template::CURRENT_WORKFLOW_SCHEMA_VERSION,
                orchestration_schema: None,
                node_kinds: vec!["shell".to_string()],
                runtime_kinds: vec!["shell".to_string()],
                feature_ids: vec!["lattice.workflow-template-bundle.v1".to_string()],
            },
            operator_notes: Vec::new(),
            non_claims: Vec::new(),
        },
        files: vec![BundleFile {
            path: PURE_WORKFLOW_FILE.to_string(),
            bytes: PURE_WORKFLOW_BYTES.to_vec(),
        }],
        workflow: Some(workflow),
        orchestration: None,
    }
}

fn orchestration_bundle(
    mut bundle: lattice::workflow_template::ValidatedBundle,
) -> lattice::workflow_template::ValidatedBundle {
    let definition = OrchestrationDefinition {
        schema_version: lattice::orchestration::current_schema_version(),
        id: DefinitionId::try_from_text(DEFINITION_ID).expect("definition identity fixture is valid"),
        objective: "Review the exact template workflow".to_string(),
        workflow: bundle.workflow_revision.clone(),
        completion: CompletionPolicy::RunSucceeded,
        retry: None,
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                agent_invocations: true,
                tokens: true,
                monetary_microunits: true,
                duration_millis: true,
            },
            limits: BudgetLimits {
                rounds: ONE_ROUND,
                agent_invocations: ONE_ROUND,
                tokens: TOKEN_LIMIT,
                monetary_microunits: MONEY_LIMIT_MICROUNITS,
                duration_millis: DURATION_LIMIT_MILLIS,
            },
            per_round_upper_bound: BudgetCharge {
                rounds: ONE_ROUND,
                agent_invocations: ONE_ROUND,
                tokens: TOKEN_LIMIT,
                monetary_microunits: MONEY_LIMIT_MICROUNITS,
                duration_millis: DURATION_LIMIT_MILLIS,
            },
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([ACTOR.to_string()]),
        },
        triggers: Vec::new(),
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    };
    let revision =
        lattice::orchestration::definition_revision(&definition).expect("orchestration fixture revision is valid");
    bundle.orchestration = Some(definition);
    bundle.orchestration_revision = Some(revision);
    bundle
}

fn admitted_activation(bundle: &lattice::workflow_template::ValidatedBundle) -> ActivationFacts {
    ActivationFacts {
        explicit_save_requested: false,
        explicit_start_requested: true,
        saved_workflow_revision: Some(bundle.workflow_revision.clone()),
        current_orchestration_revision: bundle.orchestration_revision.clone(),
        runtime_admitted: true,
        effects_admitted: true,
        credentials_admitted: true,
        triggers_admitted: true,
        positive_budgets_admitted: true,
        actor_authority_admitted: true,
    }
}

// r[verify lattice.workflow_templates.catalog.scenario.valid]
// r[verify lattice.workflow_templates.loading.scenario.local]
#[test]
fn fresh_built_in_bundle_loads_with_exact_identities() {
    let directory = initialized_root();
    let bundle = load_builtin(&directory);

    assert_eq!(bundle.manifest.id, BUILTIN_BUNDLE_ID);
    assert_eq!(bundle.file_identities.len(), EXPECTED_BOUND_FILE_COUNT);
    assert_eq!(bundle.workflow.meta.name, BUILTIN_BUNDLE_ID);
    assert!(bundle.orchestration.is_none());
    assert_eq!(bundle.identity.as_str().len(), blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE));
}

// r[verify lattice.workflow_templates.core.scenario.plan]
// r[verify lattice.workflow_templates.catalog.scenario.invalid]
#[test]
fn pure_validation_accepts_optional_absence_and_rejects_invalid_inputs() {
    let valid = pure_validation_input();
    assert!(lattice::workflow_template::validate_bundle(valid).is_ok());

    let mut optional = pure_validation_input();
    optional.manifest.assets.push(AssetBinding {
        path: OPTIONAL_ASSET_PATH.to_string(),
        media_kind: "text/plain".to_string(),
        blake3: ContentIdentity::from_bytes(OPERATOR_BYTES.as_bytes()),
        required: false,
    });
    assert!(lattice::workflow_template::validate_bundle(optional).is_ok());

    let mut missing_required = pure_validation_input();
    missing_required.manifest.assets.push(AssetBinding {
        path: OPTIONAL_ASSET_PATH.to_string(),
        media_kind: "text/plain".to_string(),
        blake3: ContentIdentity::from_bytes(OPERATOR_BYTES.as_bytes()),
        required: true,
    });
    assert_eq!(
        lattice::workflow_template::validate_bundle(missing_required)
            .expect_err("missing required asset fails")
            .code,
        DiagnosticCode::MissingFile
    );

    let mut duplicate = pure_validation_input();
    duplicate.files.push(duplicate.files[0].clone());
    assert_eq!(
        lattice::workflow_template::validate_bundle(duplicate)
            .expect_err("duplicate supplied path fails")
            .code,
        DiagnosticCode::DuplicatePath
    );

    let mut drift = pure_validation_input();
    drift.files[0].bytes = OPERATOR_BYTES.as_bytes().to_vec();
    assert_eq!(
        lattice::workflow_template::validate_bundle(drift).expect_err("identity drift fails").code,
        DiagnosticCode::IdentityMismatch
    );

    let mut invalid_workflow = pure_validation_input();
    invalid_workflow.workflow = None;
    assert_eq!(
        lattice::workflow_template::validate_bundle(invalid_workflow)
            .expect_err("invalid workflow fails")
            .code,
        DiagnosticCode::InvalidWorkflow
    );

    let mut absolute = pure_validation_input();
    absolute.manifest.workflow.path = "/outside/workflow.ncl".to_string();
    assert_eq!(
        lattice::workflow_template::validate_bundle(absolute).expect_err("absolute path fails").code,
        DiagnosticCode::UnsafePath
    );
}

// r[verify lattice.workflow_templates.catalog.scenario.invalid]
#[test]
fn malformed_bundle_is_isolated_from_valid_catalog_entries() {
    let directory = initialized_root();
    let malformed = directory.path().join(WORKFLOW_TEMPLATE_ROOT).join(MALFORMED_BUNDLE_ID);
    std::fs::create_dir(&malformed).expect("malformed fixture directory creates");
    std::fs::write(malformed.join(MANIFEST_FILE_NAME), "let broken = in").expect("malformed manifest writes");

    let catalog = lattice::workflow_template::load_catalog(&directory.path().join(WORKFLOW_TEMPLATE_ROOT))
        .expect("catalog with malformed bundle loads");

    assert_eq!(catalog.bundles.len(), ONE_ROUND as usize);
    assert_eq!(catalog.bundles[0].manifest.id, BUILTIN_BUNDLE_ID);
    assert_eq!(catalog.diagnostics.len(), ONE_ROUND as usize);
    assert_eq!(catalog.diagnostics[0].bundle, MALFORMED_BUNDLE_ID);
    assert_eq!(catalog.diagnostics[0].code, DiagnosticCode::NickelEvaluation);
}

// r[verify lattice.workflow_templates.catalog.scenario.invalid]
#[test]
fn identity_drift_and_undeclared_files_fail_closed() {
    let drift_root = initialized_root();
    let workflow_path = drift_root.path().join(WORKFLOW_TEMPLATE_ROOT).join(BUILTIN_BUNDLE_ID).join(WORKFLOW_FILE_NAME);
    let mut workflow = std::fs::read_to_string(&workflow_path).expect("built-in workflow reads");
    workflow.push_str("\n# drift\n");
    std::fs::write(&workflow_path, workflow).expect("drift fixture writes");
    let drift = lattice::workflow_template::load_catalog(&drift_root.path().join(WORKFLOW_TEMPLATE_ROOT))
        .expect("drift catalog loads");
    assert!(drift.bundles.is_empty());
    assert_eq!(drift.diagnostics[0].code, DiagnosticCode::IdentityMismatch);

    let extra_root = initialized_root();
    std::fs::write(
        extra_root.path().join(WORKFLOW_TEMPLATE_ROOT).join(BUILTIN_BUNDLE_ID).join(EXTRA_FILE_NAME),
        OPERATOR_BYTES,
    )
    .expect("undeclared fixture writes");
    let extra = lattice::workflow_template::load_catalog(&extra_root.path().join(WORKFLOW_TEMPLATE_ROOT))
        .expect("extra-file catalog loads");
    assert!(extra.bundles.is_empty());
    assert_eq!(extra.diagnostics[0].code, DiagnosticCode::UndeclaredFile);
}

// r[verify lattice.workflow_templates.loading.scenario.rejected]
#[test]
fn traversal_and_oversized_files_are_rejected_without_path_disclosure() {
    let directory = initialized_root();
    let escape = directory.path().join(WORKFLOW_TEMPLATE_ROOT).join(ESCAPE_BUNDLE_ID);
    std::fs::create_dir(&escape).expect("escape fixture directory creates");
    let digest = blake3::hash(b"outside").to_hex();
    let manifest = format!(
        "{{ schema_version = 1, id = \"{ESCAPE_BUNDLE_ID}\", display_name = \"Escape\", description = \"Escape\", category = \"tests\", workflow = {{ path = \"../outside.ncl\", blake3 = \"{digest}\" }}, assets = [], compatibility = {{ workflow_schema = 1, node_kinds = [], runtime_kinds = [], feature_ids = [\"lattice.workflow-template-bundle.v1\"] }}, operator_notes = [], non_claims = [] }}"
    );
    std::fs::write(escape.join(MANIFEST_FILE_NAME), manifest).expect("escape manifest writes");

    let catalog = lattice::workflow_template::load_catalog(&directory.path().join(WORKFLOW_TEMPLATE_ROOT))
        .expect("escape catalog loads");
    let escape_diagnostic = catalog
        .diagnostics
        .iter()
        .find(|item| item.bundle == ESCAPE_BUNDLE_ID)
        .expect("escape diagnostic exists");
    assert_eq!(escape_diagnostic.code, DiagnosticCode::UnsafePath);
    assert!(!escape_diagnostic.message.contains(directory.path().to_string_lossy().as_ref()));

    let oversized_root = initialized_root();
    let oversized_path =
        oversized_root.path().join(WORKFLOW_TEMPLATE_ROOT).join(BUILTIN_BUNDLE_ID).join(WORKFLOW_FILE_NAME);
    std::fs::write(&oversized_path, vec![b'x'; MAX_FILE_BYTES.saturating_add(ONE_ROUND as usize)])
        .expect("oversized fixture writes");
    let oversized = lattice::workflow_template::load_catalog(&oversized_root.path().join(WORKFLOW_TEMPLATE_ROOT))
        .expect("oversized catalog loads");
    assert_eq!(oversized.diagnostics[0].code, DiagnosticCode::FileTooLarge);
}

// r[verify lattice.workflow_templates.loading.scenario.rejected]
#[cfg(unix)]
#[test]
fn symlink_bundle_member_is_rejected() {
    let directory = initialized_root();
    let bundle_root = directory.path().join(WORKFLOW_TEMPLATE_ROOT).join(BUILTIN_BUNDLE_ID);
    std::os::unix::fs::symlink(bundle_root.join(WORKFLOW_FILE_NAME), bundle_root.join(EXTRA_FILE_NAME))
        .expect("symlink fixture creates");

    let catalog = lattice::workflow_template::load_catalog(&directory.path().join(WORKFLOW_TEMPLATE_ROOT))
        .expect("symlink catalog loads");

    assert!(catalog.bundles.is_empty());
    assert_eq!(catalog.diagnostics[0].code, DiagnosticCode::SymlinkRejected);
}

// r[verify lattice.workflow_templates.loading.scenario.rejected]
#[test]
fn non_regular_bundle_members_and_catalog_or_bundle_overflow_are_rejected() {
    let non_regular_root = tempfile::tempdir().expect("non-regular root creates");
    let non_regular_bundle = non_regular_root.path().join("socket-bundle");
    std::fs::create_dir_all(&non_regular_bundle).expect("non-regular bundle directory creates");
    let _listener = std::os::unix::net::UnixListener::bind(non_regular_bundle.join("member.sock"))
        .expect("Unix socket fixture binds");
    let non_regular =
        lattice::workflow_template::load_catalog(non_regular_root.path()).expect("non-regular catalog scan completes");
    assert_eq!(non_regular.bundles.len(), 0);
    assert_eq!(non_regular.diagnostics.len(), 1);
    assert_eq!(non_regular.diagnostics[0].code, DiagnosticCode::NonRegularFile);

    let bundle_limit_root = tempfile::tempdir().expect("bundle-limit root creates");
    let bundle_limit_directory = bundle_limit_root.path().join("large-bundle");
    std::fs::create_dir_all(&bundle_limit_directory).expect("bundle-limit directory creates");
    std::fs::write(bundle_limit_directory.join("manifest.ncl"), "{}").expect("bundle-limit manifest creates");
    let required_large_files = MAX_BUNDLE_BYTES / MAX_FILE_BYTES + 1;
    for index in 0..required_large_files {
        std::fs::write(bundle_limit_directory.join(format!("large-{index}.bin")), vec![b'x'; MAX_FILE_BYTES])
            .expect("bundle-limit member creates");
    }
    let bundle_limit = lattice::workflow_template::load_catalog(bundle_limit_root.path())
        .expect("bundle-limit catalog scan completes");
    assert_eq!(bundle_limit.bundles.len(), 0);
    assert_eq!(bundle_limit.diagnostics.len(), 1);
    assert_eq!(bundle_limit.diagnostics[0].code, DiagnosticCode::BundleTooLarge);

    let catalog_limit_root = tempfile::tempdir().expect("catalog-limit root creates");
    for index in 0..=MAX_CATALOG_BUNDLES {
        std::fs::create_dir(catalog_limit_root.path().join(format!("bundle-{index:0CATALOG_FIXTURE_INDEX_WIDTH$}")))
            .expect("catalog-limit bundle directory creates");
    }
    let catalog_limit =
        lattice::workflow_template::load_catalog(catalog_limit_root.path()).expect("catalog-limit scan completes");
    assert!(catalog_limit.diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::LimitExceeded));
}

// r[verify lattice.workflow_templates.init.scenario.fresh]
// r[verify lattice.workflow_templates.init.scenario.existing]
#[test]
fn initialization_is_complete_and_preserves_existing_targets() {
    let fresh = tempfile::tempdir().expect("fresh initialization root");
    let config = fresh.path().join(CONFIG_FILE_NAME);
    let report = lattice::workflow_template::initialize(&config).expect("fresh initialization succeeds");
    assert!(report.created.iter().any(|path| path.ends_with(MANIFEST_FILE_NAME)));
    assert!(report.created.iter().any(|path| path.ends_with(WORKFLOW_FILE_NAME)));
    assert!(config.exists());

    let existing = tempfile::tempdir().expect("existing initialization root");
    let existing_template =
        existing.path().join("templates").join(lattice::config::DEFAULT_NODE_TEMPLATE_FILES[0].file_name);
    std::fs::create_dir_all(existing_template.parent().expect("template parent exists"))
        .expect("existing template parent creates");
    std::fs::write(&existing_template, OPERATOR_BYTES).expect("operator bytes write");
    let error = lattice::workflow_template::initialize(&existing.path().join(CONFIG_FILE_NAME))
        .expect_err("existing target blocks initialization");
    assert!(matches!(error, InitializationError::ExistingTarget { .. }));
    assert_eq!(std::fs::read_to_string(&existing_template).expect("operator bytes read"), OPERATOR_BYTES);
    assert!(!existing.path().join(CONFIG_FILE_NAME).exists());
}

// r[verify lattice.workflow_templates.core.scenario.plan]
// r[verify lattice.workflow_templates.core.scenario.conflict]
#[test]
fn application_planning_is_deterministic_idempotent_and_conflict_safe() {
    let directory = initialized_root();
    let bundle = load_builtin(&directory);
    let database = directory.path().join("catalog.redb");
    let store = lattice::store::Store::new(&database).expect("template store creates");

    let before = lattice::workflow_template::application_facts(&store, &bundle).expect("missing facts load");
    let first = lattice::workflow_template::plan_application(&bundle, &before);
    let second = lattice::workflow_template::plan_application(&bundle, &before);
    assert_eq!(first, second);
    assert!(matches!(first, ApplicationPlan::CreateDraft { .. }));

    lattice::workflow_template::save_draft(&store, &bundle).expect("workflow draft saves");
    let exact = lattice::workflow_template::application_facts(&store, &bundle).expect("exact facts load");
    assert!(matches!(
        lattice::workflow_template::plan_application(&bundle, &exact),
        ApplicationPlan::ExactAlreadyPresent { .. }
    ));

    let mut conflicting = bundle.workflow.clone();
    conflicting.meta.description = "operator changed content".to_string();
    store.save_workflow(&conflicting).expect("conflicting workflow fixture saves");
    let conflict = lattice::workflow_template::application_facts(&store, &bundle).expect("conflict facts load");
    assert!(matches!(
        lattice::workflow_template::plan_application(&bundle, &conflict),
        ApplicationPlan::Conflict(DiagnosticCode::IncompatibleContent)
    ));
    assert!(lattice::workflow_template::save_draft(&store, &bundle).is_err());
}

// r[verify lattice.workflow_templates.activation.scenario.no-auto-start]
// r[verify lattice.workflow_templates.activation.scenario.start]
#[test]
fn activation_requires_explicit_exact_admission() {
    let directory = initialized_root();
    let bundle = orchestration_bundle(load_builtin(&directory));
    let mut facts = admitted_activation(&bundle);
    facts.explicit_start_requested = false;
    assert_eq!(lattice::workflow_template::plan_activation(&bundle, &facts), ActivationPlan::PreviewOnly);

    facts.explicit_start_requested = true;
    assert!(matches!(
        lattice::workflow_template::plan_activation(&bundle, &facts),
        ActivationPlan::StartOrchestration { .. }
    ));

    let receipt =
        lattice::workflow_template::activation_receipt(&bundle, ACTOR).expect("authorized activation receipt is valid");
    let store =
        lattice::store::Store::new(directory.path().join("activation.redb")).expect("activation receipt store creates");
    store.save_workflow_template_activation(&receipt).expect("activation receipt saves before dispatch");
    store.save_workflow_template_activation(&receipt).expect("exact activation receipt is idempotent");
    assert_eq!(
        store.load_workflow_template_activation(&receipt.identity).expect("activation receipt loads"),
        Some(receipt)
    );
    assert!(lattice::workflow_template::activation_receipt(&bundle, "operator:unauthorized").is_err());
}

// r[verify lattice.workflow_templates.activation.scenario.blocked]
#[test]
fn activation_fails_closed_for_drift_and_every_missing_authority_fact() {
    let directory = initialized_root();
    let bundle = orchestration_bundle(load_builtin(&directory));
    let mut drifted = admitted_activation(&bundle);
    drifted.saved_workflow_revision = None;
    assert_eq!(
        lattice::workflow_template::plan_activation(&bundle, &drifted),
        ActivationPlan::Blocked(ActivationBlock::WorkflowNotSaved)
    );

    let cases: &[ActivationFailureCase] = &[
        (|facts| facts.runtime_admitted = false, ActivationBlock::RuntimeNotAdmitted),
        (|facts| facts.effects_admitted = false, ActivationBlock::EffectsNotAdmitted),
        (|facts| facts.credentials_admitted = false, ActivationBlock::CredentialsNotAdmitted),
        (|facts| facts.triggers_admitted = false, ActivationBlock::TriggersNotAdmitted),
        (|facts| facts.positive_budgets_admitted = false, ActivationBlock::BudgetsNotAdmitted),
        (|facts| facts.actor_authority_admitted = false, ActivationBlock::AuthorityNotAdmitted),
    ];
    for (deny, expected) in cases {
        let mut facts = admitted_activation(&bundle);
        deny(&mut facts);
        assert_eq!(lattice::workflow_template::plan_activation(&bundle, &facts), ActivationPlan::Blocked(*expected));
    }
}

// r[verify lattice.workflow_templates.activation.scenario.start]
#[tokio::test]
async fn exact_activation_records_identities_before_one_admitted_dispatch() {
    let directory = initialized_root();
    let bundle = orchestration_bundle(load_builtin(&directory));
    let store = lattice::store::Store::new(directory.path().join("admitted-activation.redb"))
        .expect("admitted activation store creates");
    lattice::workflow_template::save_draft(&store, &bundle).expect("exact workflow draft saves");
    lattice::workflow_template::save_draft(&store, &bundle).expect("exact workflow draft save is idempotent");
    let facts = admitted_activation(&bundle);
    let receipt =
        lattice::workflow_template::activation_receipt(&bundle, ACTOR).expect("admitted activation receipt computes");
    let authority = common::authority::TestRunContextProvider::new();

    let instance = lattice::workflow_template::start_orchestration_draft_with_execution(
        &store,
        &bundle,
        &facts,
        ACTOR,
        lattice::orchestration::ProductExecution {
            agent_runtime: lattice::agent_node::AgentRuntimeConfig::default(),
            authority_provider: &authority,
        },
    )
    .await
    .expect("exact admitted activation starts");

    assert_eq!(store.list_orchestrations().expect("admitted orchestrations list").len(), 1);
    assert_eq!(store.list_runs(PREVIEW_RUN_RECORD_LIMIT).expect("admitted runs list").len(), 1);
    assert_eq!(instance.definition_revision, bundle.orchestration_revision.clone().expect("revision exists"));
    assert_eq!(
        store
            .load_workflow_template_activation(&receipt.identity)
            .expect("admitted receipt lookup succeeds"),
        Some(receipt)
    );
}

// r[verify lattice.workflow_templates.activation.scenario.blocked]
// r[verify lattice.workflow_templates.activation.scenario.no-auto-start]
#[tokio::test]
async fn blocked_activation_creates_no_instance_run_or_activation_receipt() {
    let directory = initialized_root();
    let bundle = orchestration_bundle(load_builtin(&directory));
    let store = lattice::store::Store::new(directory.path().join("blocked-activation.redb"))
        .expect("blocked activation store creates");
    let mut facts = admitted_activation(&bundle);
    facts.runtime_admitted = false;
    let receipt = lattice::workflow_template::activation_receipt(&bundle, ACTOR)
        .expect("blocked activation receipt identity computes");

    lattice::workflow_template::start_orchestration_draft(
        &store,
        &bundle,
        &facts,
        ACTOR,
        lattice::agent_node::AgentRuntimeConfig::default(),
    )
    .await
    .expect_err("missing runtime admission blocks activation");

    assert!(store.list_orchestrations().expect("blocked orchestrations list").is_empty());
    assert!(store.list_runs(PREVIEW_RUN_RECORD_LIMIT).expect("blocked runs list").is_empty());
    assert_eq!(
        store.load_workflow_template_activation(&receipt.identity).expect("blocked receipt lookup succeeds"),
        None
    );
}

// r[verify lattice.workflow_templates.compatibility]
// r[verify lattice.workflow_templates.compatibility.scenario.nodes]
// r[verify lattice.workflow_templates.compatibility.scenario.cli]
#[test]
fn existing_node_templates_and_direct_format_routes_remain_compatible() {
    let directory = initialized_root();
    let node_catalog = lattice::config::load_node_templates_with_diagnostics(&directory.path().join("templates"));
    assert!(node_catalog.diagnostics.is_empty());
    assert_eq!(node_catalog.templates.len(), lattice::config::DEFAULT_NODE_TEMPLATE_FILES.len());
    assert!(node_catalog.templates.iter().all(|template| template.kind == "shell"));

    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflow =
        lattice::workflow_format::load_workflow_nickel_file(&repository.join("examples/workflows/daily-digest.ncl"))
            .expect("existing workflow import route remains valid");
    assert_eq!(workflow.meta.name, "daily-digest");
    let definition = lattice::orchestration::load_definition_nickel_file(
        &repository.join("examples/orchestrations/llm-review-loop.ncl"),
    )
    .expect("existing orchestration format route remains valid");
    assert!(!definition.objective.is_empty());
}

// r[verify lattice.workflow_templates.preview.scenario.select]
// r[verify lattice.workflow_templates.preview.scenario.cancel]
#[test]
fn tui_preview_is_unsaved_and_cancel_restores_prior_state() {
    let directory = initialized_root();
    let bundle = orchestration_bundle(load_builtin(&directory));
    let config = lattice::config::AppConfig {
        data_dir: directory.path().join("tui-state"),
        ..lattice::config::AppConfig::default()
    };
    std::fs::create_dir_all(&config.data_dir).expect("TUI state directory creates");
    let mut app = lattice::tui::App::new(config);
    let prior_name = app.workflow.meta.name.clone();
    let store = app.store.clone().expect("preview test store opens");
    let workflow_count = store.list_workflows().expect("preview workflow count reads").len();
    let run_count = store.list_runs(PREVIEW_RUN_RECORD_LIMIT).expect("preview run count reads").len();
    let orchestration_count = store.list_orchestrations().expect("preview orchestration count reads").len();
    let activation_receipt =
        lattice::workflow_template::activation_receipt(&bundle, ACTOR).expect("preview receipt identity computes");
    app.set_workflow_templates(vec![bundle]);

    app.open_workflow_template_browser();
    app.confirm_workflow_template_browser();
    assert_eq!(app.workflow.meta.name, BUILTIN_BUNDLE_ID);
    assert!(app.has_unsaved_changes());
    assert!(app.workflow_template_draft.is_some());
    assert!(
        app.store
            .as_ref()
            .expect("TUI store exists")
            .list_workflows()
            .expect("workflow list loads")
            .is_empty()
    );

    app.close_workflow_template_browser();
    assert_eq!(app.workflow.meta.name, prior_name);
    assert!(!app.has_unsaved_changes());
    assert!(app.workflow_template_draft.is_none());
    assert_eq!(store.list_workflows().expect("preview workflows reread").len(), workflow_count);
    assert_eq!(store.list_runs(PREVIEW_RUN_RECORD_LIMIT).expect("preview runs reread").len(), run_count);
    assert_eq!(store.list_orchestrations().expect("preview orchestrations reread").len(), orchestration_count);
    assert_eq!(
        store
            .load_workflow_template_activation(&activation_receipt.identity)
            .expect("preview activation receipt lookup succeeds"),
        None
    );
}

// r[verify lattice.workflow_templates.preview.scenario.select]
#[test]
fn tui_browser_renders_safe_metadata_without_bundle_commands() {
    let directory = initialized_root();
    let bundle = load_builtin(&directory);
    let config = lattice::config::AppConfig {
        data_dir: directory.path().join("render-state"),
        ..lattice::config::AppConfig::default()
    };
    std::fs::create_dir_all(&config.data_dir).expect("render TUI state directory creates");
    let mut app = lattice::tui::App::new(config);
    app.set_workflow_templates(vec![bundle]);
    app.open_workflow_template_browser();
    let area = ratatui::layout::Rect::new(0, 0, TUI_FRAME_WIDTH, TUI_FRAME_HEIGHT);
    let mut buffer = ratatui::buffer::Buffer::empty(area);
    lattice::tui::panels::render(&mut app, area, &mut buffer);
    let rendered = buffer.content().iter().map(|cell| cell.symbol()).collect::<String>();

    assert!(rendered.contains("Shell preview"));
    assert!(rendered.contains("non-claim:"));
    assert!(!rendered.contains("printf"));
    assert!(!rendered.contains("workflow template preview"));
}

// r[verify lattice.workflow_templates.preview.scenario.select]
#[test]
fn tui_refuses_dirty_replacement_and_accepts_only_an_explicit_preview() {
    let dirty_root = initialized_root();
    let dirty_bundle = load_builtin(&dirty_root);
    let dirty_config = lattice::config::AppConfig {
        data_dir: dirty_root.path().join("dirty-state"),
        ..lattice::config::AppConfig::default()
    };
    std::fs::create_dir_all(&dirty_config.data_dir).expect("dirty TUI state directory creates");
    let mut dirty = lattice::tui::App::new(dirty_config);
    dirty.set_workflow_templates(vec![dirty_bundle]);
    dirty.mark_workflow_dirty();
    let dirty_name = dirty.workflow.meta.name.clone();
    dirty.open_workflow_template_browser();
    dirty.confirm_workflow_template_browser();
    assert_eq!(dirty.workflow.meta.name, dirty_name);
    assert!(dirty.workflow_template_draft.is_none());

    let accepted_root = initialized_root();
    let accepted_bundle = load_builtin(&accepted_root);
    let accepted_config = lattice::config::AppConfig {
        data_dir: accepted_root.path().join("accepted-state"),
        ..lattice::config::AppConfig::default()
    };
    std::fs::create_dir_all(&accepted_config.data_dir).expect("accepted TUI state directory creates");
    let mut accepted = lattice::tui::App::new(accepted_config);
    accepted.set_workflow_templates(vec![accepted_bundle]);
    accepted.open_workflow_template_browser();
    accepted.confirm_workflow_template_browser();
    accepted.confirm_workflow_template_browser();
    assert!(!accepted.is_workflow_template_browser_open());
    assert_eq!(accepted.workflow.meta.name, BUILTIN_BUNDLE_ID);
    assert!(accepted.workflow_template_draft.is_some());
    assert!(accepted.has_unsaved_changes());
    accepted.save_workflow();
    assert!(!accepted.has_unsaved_changes());
    assert!(accepted.workflow_template_draft.is_none());
    assert!(
        accepted
            .store
            .as_ref()
            .expect("accepted store exists")
            .load_workflow(BUILTIN_BUNDLE_ID)
            .expect("accepted workflow lookup succeeds")
            .is_some()
    );

    let drift_root = initialized_root();
    let drift_bundle = load_builtin(&drift_root);
    let drift_config = lattice::config::AppConfig {
        data_dir: drift_root.path().join("drift-state"),
        ..lattice::config::AppConfig::default()
    };
    std::fs::create_dir_all(&drift_config.data_dir).expect("drift TUI state directory creates");
    let mut drift = lattice::tui::App::new(drift_config);
    drift.set_workflow_templates(vec![drift_bundle]);
    drift.open_workflow_template_browser();
    drift.confirm_workflow_template_browser();
    drift.confirm_workflow_template_browser();
    drift.workflow.add_shell_node("drift", "true");
    drift.save_workflow();
    assert!(drift.has_unsaved_changes());
    assert!(
        drift
            .store
            .as_ref()
            .expect("drift store exists")
            .load_workflow(BUILTIN_BUNDLE_ID)
            .expect("drift workflow lookup succeeds")
            .is_none()
    );
}
