use lattice::orchestration::DefinitionId;
use lattice::orchestration::InstanceId;
use lattice::orchestration::new_instance;
use lattice::orchestration::replay_events;
use lattice::store::Store;
use redb::Database;
use redb::TableDefinition;
use serde::Deserialize;
use tempfile::tempdir;

const FIXTURE_SOURCE_REVISION: &str = "cbb5db0e4844b466dcd1877218eb1f00a2a7caf2";
const DATABASE_FILE_NAME: &str = "compatibility.redb";
const ORCHESTRATION_DEFINITIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("orchestration_definitions");
const ORCHESTRATION_INSTANCES: TableDefinition<&str, &[u8]> = TableDefinition::new("orchestration_instances");
const ORCHESTRATION_EVENTS: TableDefinition<&str, &[u8]> = TableDefinition::new("orchestration_events");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PreMigrationFixture {
    schema_version: String,
    source_revision: String,
    definition_key: String,
    definition_json: String,
    instance_key: String,
    instance_json: String,
    events_json: String,
}

fn fixture() -> PreMigrationFixture {
    serde_json::from_str(include_str!("fixtures/orchestration-identities/pre-migration-redb-v1.json"))
        .expect("pre-migration fixture must parse")
}

fn write_pre_migration_records(path: &std::path::Path, fixture: &PreMigrationFixture) {
    let initialized = Store::new(path).expect("store schema must initialize");
    drop(initialized);
    let database = Database::create(path).expect("redb fixture database must open");
    let transaction = database.begin_write().expect("fixture transaction must start");
    {
        let mut definitions = transaction.open_table(ORCHESTRATION_DEFINITIONS).expect("definition table must open");
        definitions
            .insert(fixture.definition_key.as_str(), fixture.definition_json.as_bytes())
            .expect("definition fixture must insert");
        let mut instances = transaction.open_table(ORCHESTRATION_INSTANCES).expect("instance table must open");
        instances
            .insert(fixture.instance_key.as_str(), fixture.instance_json.as_bytes())
            .expect("instance fixture must insert");
        let mut events = transaction.open_table(ORCHESTRATION_EVENTS).expect("event table must open");
        events
            .insert(fixture.instance_key.as_str(), fixture.events_json.as_bytes())
            .expect("event fixture must insert");
    }
    transaction.commit().expect("fixture transaction must commit");
}

// r[verify lattice.agent_orchestration.nominal_identities.compatibility]
// r[verify lattice.agent_orchestration.nominal_identities.wire_boundary]
// r[verify lattice.agent_orchestration.nominal_identities.events]
// r[verify lattice.workflow_approval.compatibility]
// r[verify lattice.workflow_approval.compatibility.valid]
#[test]
fn pre_migration_redb_json_loads_with_exact_wire_projection_and_replays_without_effects() {
    let fixture = fixture();
    let directory = tempdir().expect("temporary directory must exist");
    let database_path = directory.path().join(DATABASE_FILE_NAME);
    assert_eq!(fixture.schema_version, "lattice-orchestration-redb-fixture/v1");
    assert_eq!(fixture.source_revision, FIXTURE_SOURCE_REVISION);
    write_pre_migration_records(&database_path, &fixture);

    let store = Store::new(&database_path).expect("typed store must open pre-migration records");
    let definition_id = DefinitionId::try_from_text(&fixture.definition_key).expect("definition key must be admitted");
    let instance_id = InstanceId::try_from_hex(&fixture.instance_key).expect("instance key must be admitted");
    let definition = store
        .load_orchestration_definition(&definition_id)
        .expect("definition read must succeed")
        .expect("definition must exist");
    let instance = store
        .load_orchestration(&instance_id)
        .expect("instance read must succeed")
        .expect("instance must exist");
    let events = store.load_orchestration_events(&instance_id).expect("events must load");

    assert_eq!(serde_json::to_string(&definition).expect("definition serializes"), fixture.definition_json);
    assert_eq!(serde_json::to_string(&instance).expect("instance serializes"), fixture.instance_json);
    assert_eq!(serde_json::to_string(&events).expect("events serialize"), fixture.events_json);

    let initial = new_instance(definition, instance.sequence, instance.trigger_idempotency_key.clone())
        .expect("initial state must reconstruct");
    let replay = replay_events(&initial, &events, instance.event_head.as_ref());
    assert!(replay.valid);
    assert_eq!(replay.instance, instance);
}

// r[verify lattice.agent_orchestration.nominal_identities.validation]
// r[verify lattice.agent_orchestration.nominal_identities.failure_matrix.no_effect]
// r[verify lattice.workflow_approval.compatibility.denied]
#[test]
fn malformed_pre_migration_redb_key_fails_before_core_loading() {
    let fixture = fixture();
    let malformed_key = format!("{}x", fixture.instance_key);
    let error = InstanceId::try_from_hex(&malformed_key).expect_err("malformed instance key must fail");

    assert_eq!(error.to_string(), "orchestration instance identity must be one lowercase BLAKE3 hex digest");
    assert!(!error.to_string().contains(&malformed_key));
}
