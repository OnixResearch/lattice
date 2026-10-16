use std::collections::BTreeMap;
use std::path::PathBuf;

use lattice::engine::effect::execution;
use lattice::engine::effect::execution::RunContextProvider;
use lattice::engine::effect::standalone;
use lattice::engine::effect::standalone::core::Profile;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::Workflow;
use lattice::engine::workflow::WorkflowTimestamp;
use lattice::file_node::FileOperationKind;
use lattice::headless::HeadlessRunStatus;
use lattice::store::Store;

const FIXTURE_EXPIRY_UNIX_MS: u64 = 4_102_444_800_000;
const OBSERVED_CONTENT: &str = "operator-authorized";
const ROOT_ID: &str = "workspace";
const FILE_NAME: &str = "observed.txt";

struct Fixture {
    directory: tempfile::TempDir,
    store: Store,
    workflow: Workflow,
    profile: Profile,
    profile_path: PathBuf,
    imported_path: PathBuf,
}

impl Fixture {
    fn new(directory: tempfile::TempDir, workflow: Workflow, roots: BTreeMap<String, PathBuf>) -> Self {
        let store = Store::new(directory.path().join("state.redb")).expect("isolated store opens");
        store.save_workflow(&workflow).expect("fixture workflow saves");
        let inspection = standalone::inspect_workflow(&workflow).expect("canonical workflow requirements");
        let profile = Profile {
            schema: inspection.schema,
            workflow_revision: inspection.workflow_revision,
            machine_identity: inspection.machine_identity,
            policy_ref: inspection.policy_ref,
            expires_at_unix_ms: FIXTURE_EXPIRY_UNIX_MS,
            grants: inspection.grants,
            roots,
        };
        let profile_path = directory.path().join("authority.ncl");
        let imported_path = directory.path().join("grant.json");
        std::fs::write(&profile_path, "import \"grant.json\"\n").expect("Nickel profile imports evaluated input");
        std::fs::write(&imported_path, serde_json::to_vec(&profile).expect("profile serializes"))
            .expect("profile input writes");
        Self {
            directory,
            store,
            workflow,
            profile,
            profile_path,
            imported_path,
        }
    }

    fn source(&self) -> standalone::OperatorRunContextSource {
        standalone::OperatorRunContextSource::new(self.profile_path.clone())
    }

    async fn run(&self, provider: &dyn RunContextProvider) -> HeadlessRunStatus {
        let run = self.store.next_run_id().expect("durable run identity");
        lattice::headless::run_saved_workflow_with_authority_provider(
            &self.store,
            &self.workflow.meta.name,
            run,
            provider,
        )
        .await
        .expect("run records a terminal summary")
        .status
    }
}

fn shell_fixture() -> Fixture {
    let directory = tempfile::tempdir().expect("isolated effect directory");
    let output = directory.path().join(FILE_NAME);
    let quoted = format!("'{}'", output.to_str().expect("fixture path is UTF-8").replace('\'', "'\"'\"'"));
    let mut workflow = Workflow::new("operator-shell", WorkflowTimestamp::epoch());
    workflow.add_shell_node("write observation", format!("printf '%s' '{OBSERVED_CONTENT}' > {quoted}"));
    Fixture::new(directory, workflow, BTreeMap::new())
}

// r[verify lattice.standalone_effect_authority.provenance.scenario.local]
// r[verify lattice.standalone_effect_authority.provenance.scenario.missing]
// r[verify lattice.standalone_effect_authority.validation]
#[tokio::test]
async fn selected_operator_authority_runs_a_real_process_and_absence_has_no_effect() {
    let denied = shell_fixture();
    assert_eq!(denied.run(&execution::UnavailableRunContextSource).await, HeadlessRunStatus::Failed);
    assert!(!denied.directory.path().join(FILE_NAME).exists());

    let allowed = shell_fixture();
    assert_eq!(allowed.run(&allowed.source()).await, HeadlessRunStatus::Succeeded);
    assert_eq!(std::fs::read_to_string(allowed.directory.path().join(FILE_NAME)).unwrap(), OBSERVED_CONTENT);
}

#[derive(Clone, Copy)]
enum SourceChange {
    FormattingOnly,
    ExtendExpiry,
    Remove,
}

struct ChangeAfterAdmission {
    source: standalone::OperatorRunContextSource,
    profile_path: PathBuf,
    imported_path: PathBuf,
    profile: Profile,
    change: SourceChange,
}

impl RunContextProvider for ChangeAfterAdmission {
    fn load(
        &self,
        run_id: lattice::engine::executor::RunId,
        store: Option<Store>,
    ) -> Result<execution::RunContext, execution::Error> {
        let admitted = self.source.load(run_id, store)?;
        match self.change {
            SourceChange::FormattingOnly => {
                std::fs::write(
                    &self.profile_path,
                    "# harmless text edit\nlet current = import \"grant.json\" in current\n",
                )
                .expect("same evaluated profile");
            }
            SourceChange::ExtendExpiry => {
                let mut changed = self.profile.clone();
                changed.expires_at_unix_ms += 1;
                std::fs::write(&self.imported_path, serde_json::to_vec(&changed).unwrap())
                    .expect("changed imported meaning");
            }
            SourceChange::Remove => std::fs::remove_file(&self.profile_path).expect("selected profile removed"),
        }
        Ok(admitted)
    }
}

// r[verify lattice.standalone_effect_authority.currentness.scenario.changed]
#[tokio::test]
async fn dispatch_rechecks_imported_meaning_and_removal_but_not_source_formatting() {
    for (change, expected) in [
        (SourceChange::FormattingOnly, HeadlessRunStatus::Succeeded),
        (SourceChange::ExtendExpiry, HeadlessRunStatus::Failed),
        (SourceChange::Remove, HeadlessRunStatus::Failed),
    ] {
        let fixture = shell_fixture();
        let provider = ChangeAfterAdmission {
            source: fixture.source(),
            profile_path: fixture.profile_path.clone(),
            imported_path: fixture.imported_path.clone(),
            profile: fixture.profile.clone(),
            change,
        };
        assert_eq!(fixture.run(&provider).await, expected);
        let output = fixture.directory.path().join(FILE_NAME);
        if expected == HeadlessRunStatus::Succeeded {
            assert_eq!(std::fs::read_to_string(output).unwrap(), OBSERVED_CONTENT);
        } else {
            assert!(!output.exists(), "revoked authority must not execute the process");
        }
    }
}

// r[verify lattice.standalone_effect_authority.binding.scenario.mismatch]
#[tokio::test]
async fn a_profile_cannot_authorize_another_workflow_revision() {
    let fixture = shell_fixture();
    let mut changed = fixture.workflow.clone();
    changed.meta.name = "changed-workflow".to_owned();
    fixture.store.save_workflow(&changed).unwrap();
    let run = fixture.store.next_run_id().unwrap();
    let result = lattice::headless::run_saved_workflow_with_authority_provider(
        &fixture.store,
        &changed.meta.name,
        run,
        &fixture.source(),
    )
    .await
    .unwrap();
    assert_eq!(result.status, HeadlessRunStatus::Failed);
    assert!(!fixture.directory.path().join(FILE_NAME).exists());
}

// r[verify lattice.standalone_effect_authority.roots.scenario.unknown]
#[tokio::test]
async fn file_effects_require_an_operator_configured_root() {
    for (configure_root, relative_path, expected) in [
        (true, FILE_NAME, HeadlessRunStatus::Succeeded),
        (false, FILE_NAME, HeadlessRunStatus::Failed),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("root");
        std::fs::create_dir(&root).unwrap();
        let mut workflow = Workflow::new("operator-file", WorkflowTimestamp::epoch());
        let node = workflow.add_file_node("write observation", FileOperationKind::Write, relative_path);
        workflow.set_node_config(node, NodeConfig::File {
            operation: FileOperationKind::Write,
            path: relative_path.to_owned(),
            content: Some(OBSERVED_CONTENT.to_owned()),
            overwrite: false,
            allowed_roots: vec![ROOT_ID.to_owned()],
        });
        let roots = if configure_root {
            BTreeMap::from([(ROOT_ID.to_owned(), root.clone())])
        } else {
            BTreeMap::new()
        };
        let fixture = Fixture::new(directory, workflow, roots);
        assert_eq!(fixture.run(&fixture.source()).await, expected);
        if expected == HeadlessRunStatus::Succeeded {
            assert_eq!(std::fs::read_to_string(root.join(FILE_NAME)).unwrap(), OBSERVED_CONTENT);
        } else {
            assert!(!root.join(FILE_NAME).exists());
        }
    }
}
