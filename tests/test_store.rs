//! Tests for store — redb persistence: workflows, values, cache, run records.

mod common;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use lattice::engine::executor::RunId;
use lattice::engine::value::ContentHash;
use lattice::engine::value::Value;
use lattice::engine::workflow::Workflow;
use lattice::store::ExecutionCache;
use lattice::store::RunOwner;
use lattice::store::RunRecord;
use lattice::store::Store;
use lattice::store::WorkflowRepository;
use pretty_assertions::assert_eq;
use rat_nodegraph::model::PortId;

const INITIAL_NEXT_RUN_ID: u64 = 0;
const FIRST_SAVED_RUN_ID: u64 = 3;
const SECOND_SAVED_RUN_ID: u64 = 9;
const EXPECTED_NEXT_RUN_ID: u64 = 10;
const RUN_RECORD_NODE_COUNT: usize = 1;
const RUN_RECORD_CACHE_HITS: usize = 0;
const RUN_RECORD_FAILURES: usize = 0;
const STARTED_EPOCH_MS: i64 = 1000;
const CONCURRENT_RUN_ID_ALLOCATIONS: usize = 8;

// ---------------------------------------------------------------------------
// Store creation
// ---------------------------------------------------------------------------

#[test]
fn store_opens_successfully() {
    let (_store, _dir) = common::temp_store();
}

#[test]
fn store_open_is_idempotent() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("test.redb");
    // Open twice — should not panic or corrupt.
    Store::open(&path).unwrap();
    Store::open(&path).unwrap();
}

// ---------------------------------------------------------------------------
// Workflow CRUD
// ---------------------------------------------------------------------------

#[test]
fn save_and_load_workflow() {
    let (store, _dir) = common::temp_store();
    let (wf, _, _) = common::two_node_pipeline();

    store.save_workflow(&wf).unwrap();
    let loaded = store.load_workflow("test-pipeline").unwrap();

    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.meta.name, "test-pipeline");
    assert_eq!(loaded.graph.node_count(), 2);
    assert_eq!(loaded.graph.edge_count(), 1);
}

#[test]
fn load_nonexistent_workflow_returns_none() {
    let (store, _dir) = common::temp_store();
    let result = store.load_workflow("does-not-exist").unwrap();
    assert!(result.is_none());
}

// r[verify n8nclone.workflow_execution.trait_boundaries.storage_ports]
#[test]
fn workflow_repository_trait_saves_loads_and_reports_missing_workflows() {
    let (store, _dir) = common::temp_store();
    let repository: &dyn WorkflowRepository = &store;
    let workflow = Workflow::new("trait-workflow");

    repository.save_workflow(&workflow).expect("trait save succeeds");
    let loaded = repository.load_workflow("trait-workflow").expect("trait load succeeds");
    let missing = repository.load_workflow("missing-workflow").expect("trait missing load succeeds");

    assert!(loaded.is_some());
    assert!(missing.is_none());
}

#[test]
fn list_workflows_empty() {
    let (store, _dir) = common::temp_store();
    let names = store.list_workflows().unwrap();
    assert!(names.is_empty());
}

#[test]
fn list_workflows_returns_all() {
    let (store, _dir) = common::temp_store();

    let wf1 = Workflow::new("alpha");
    let wf2 = Workflow::new("beta");
    store.save_workflow(&wf1).unwrap();
    store.save_workflow(&wf2).unwrap();

    let mut names = store.list_workflows().unwrap();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta"]);
}

#[test]
fn save_workflow_overwrites() {
    let (store, _dir) = common::temp_store();

    let mut wf = Workflow::new("overwrite-test");
    store.save_workflow(&wf).unwrap();

    // Modify and re-save.
    wf.add_shell_node("new-node", "true");
    store.save_workflow(&wf).unwrap();

    let loaded = store.load_workflow("overwrite-test").unwrap().unwrap();
    assert_eq!(loaded.graph.node_count(), 1, "overwritten version should have 1 node");
}

#[test]
fn delete_workflow() {
    let (store, _dir) = common::temp_store();
    let wf = Workflow::new("doomed");
    store.save_workflow(&wf).unwrap();

    store.delete_workflow("doomed").unwrap();
    assert!(store.load_workflow("doomed").unwrap().is_none());
}

#[test]
fn delete_nonexistent_workflow_is_ok() {
    let (store, _dir) = common::temp_store();
    // Should not error.
    store.delete_workflow("ghost").unwrap();
}

// ---------------------------------------------------------------------------
// Value store (content-addressed)
// ---------------------------------------------------------------------------

#[test]
fn put_and_get_value() {
    let (store, _dir) = common::temp_store();
    let val = Value::from_string("hello store".into());

    store.put_value(&val).unwrap();
    let loaded = store.get_value(&val.hash).unwrap();

    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.hash, val.hash);
    assert_eq!(loaded.as_display_string(), "hello store");
}

#[test]
fn get_missing_value_returns_none() {
    let (store, _dir) = common::temp_store();
    let fake_hash = ContentHash::from_bytes(b"not stored");
    assert!(store.get_value(&fake_hash).unwrap().is_none());
}

#[test]
fn put_value_deduplicates() {
    let (store, _dir) = common::temp_store();
    let val = Value::from_string("dedup".into());

    // Store same value twice — should be a no-op on the second write.
    store.put_value(&val).unwrap();
    store.put_value(&val).unwrap();

    let loaded = store.get_value(&val.hash).unwrap().unwrap();
    assert_eq!(loaded.hash, val.hash);
}

#[test]
fn store_different_value_types() {
    let (store, _dir) = common::temp_store();

    let s = Value::from_string("text".into());
    let b = Value::from_bytes(vec![1, 2, 3]);
    let j = Value::from_json(serde_json::json!({"k": "v"}));
    let e = Value::from_exit_code(0);
    let err = Value::error("fail".into(), "trace".into());

    for val in [&s, &b, &j, &e, &err] {
        store.put_value(val).unwrap();
        let loaded = store.get_value(&val.hash).unwrap().unwrap();
        assert_eq!(loaded.hash, val.hash);
    }
}

// ---------------------------------------------------------------------------
// Execution cache
// ---------------------------------------------------------------------------

#[test]
fn put_and_get_cache() {
    let (store, _dir) = common::temp_store();

    let input_hash = ContentHash::from_bytes(b"cache key");
    let mut outputs = HashMap::new();
    outputs.insert(PortId(100), Value::from_string("cached output".into()));

    store.put_cache(&input_hash, &outputs).unwrap();

    let loaded = store.get_cache(&input_hash).unwrap();
    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert!(loaded.contains_key(&PortId(100)));
    assert_eq!(loaded.get(&PortId(100)).unwrap().as_display_string(), "cached output");
}

#[test]
fn get_missing_cache_returns_none() {
    let (store, _dir) = common::temp_store();
    let hash = ContentHash::from_bytes(b"no cache");
    assert!(store.get_cache(&hash).unwrap().is_none());
}

// r[verify n8nclone.workflow_execution.trait_boundaries.storage_ports]
#[test]
fn execution_cache_trait_reads_written_and_missing_entries() {
    let (store, _dir) = common::temp_store();
    let cache: &dyn ExecutionCache = &store;
    let input_hash = ContentHash::from_bytes(b"trait cache key");
    let missing_hash = ContentHash::from_bytes(b"trait missing cache key");
    let mut outputs = HashMap::new();
    outputs.insert(PortId(100), Value::from_string("trait cached output".into()));

    cache.put_cache(&input_hash, &outputs).expect("trait cache write succeeds");
    let loaded = cache.get_cache(&input_hash).expect("trait cache read succeeds");
    let missing = cache.get_cache(&missing_hash).expect("trait cache missing read succeeds");

    assert!(loaded.is_some());
    assert!(missing.is_none());
}

#[test]
fn cache_overwrites_on_same_key() {
    let (store, _dir) = common::temp_store();
    let hash = ContentHash::from_bytes(b"overwrite cache");

    let mut v1 = HashMap::new();
    v1.insert(PortId(1), Value::from_string("first".into()));
    store.put_cache(&hash, &v1).unwrap();

    let mut v2 = HashMap::new();
    v2.insert(PortId(1), Value::from_string("second".into()));
    store.put_cache(&hash, &v2).unwrap();

    let loaded = store.get_cache(&hash).unwrap().unwrap();
    assert_eq!(loaded.get(&PortId(1)).unwrap().as_display_string(), "second");
}

// ---------------------------------------------------------------------------
// Run records
// ---------------------------------------------------------------------------

#[test]
fn save_and_list_runs() {
    let (store, _dir) = common::temp_store();

    let record = RunRecord {
        run_id: RunId(0),
        owner: RunOwner::Unscoped,
        workflow_name: "test-wf".into(),
        started_epoch_ms: 1000,
        finished_epoch_ms: Some(2000),
        node_count: 3,
        cache_hits: 1,
        failures: 0,
    };
    store.save_run(&record).unwrap();

    let runs = store.list_runs(10).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].workflow_name, "test-wf");
    assert_eq!(runs[0].node_count, 3);
    assert_eq!(runs[0].cache_hits, 1);
}

#[test]
fn list_runs_respects_limit() {
    let (store, _dir) = common::temp_store();

    for i in 0..5 {
        store
            .save_run(&RunRecord {
                run_id: RunId(i),
                owner: RunOwner::Unscoped,
                workflow_name: format!("wf-{i}"),
                started_epoch_ms: i as i64 * 1000,
                finished_epoch_ms: None,
                node_count: 1,
                cache_hits: 0,
                failures: 0,
            })
            .unwrap();
    }

    let runs = store.list_runs(3).unwrap();
    assert_eq!(runs.len(), 3);
}

#[test]
fn list_runs_newest_first() {
    let (store, _dir) = common::temp_store();

    for i in 0..3 {
        store
            .save_run(&RunRecord {
                run_id: RunId(i),
                owner: RunOwner::Unscoped,
                workflow_name: format!("wf-{i}"),
                started_epoch_ms: i as i64 * 1000,
                finished_epoch_ms: None,
                node_count: 1,
                cache_hits: 0,
                failures: 0,
            })
            .unwrap();
    }

    let runs = store.list_runs(10).unwrap();
    // Newest (highest run_id) should come first.
    assert_eq!(runs[0].run_id.0, 2);
    assert_eq!(runs[1].run_id.0, 1);
    assert_eq!(runs[2].run_id.0, 0);
}

#[test]
fn list_runs_empty() {
    let (store, _dir) = common::temp_store();
    let runs = store.list_runs(10).unwrap();
    assert!(runs.is_empty());
}

#[test]
fn next_run_id_starts_at_initial_value() {
    let (store, _dir) = common::temp_store();

    let next_run_id = store.next_run_id().unwrap();

    assert_eq!(next_run_id, RunId(INITIAL_NEXT_RUN_ID));
}

#[test]
fn next_run_id_follows_highest_saved_run() {
    let (store, _dir) = common::temp_store();
    for run_id in [FIRST_SAVED_RUN_ID, SECOND_SAVED_RUN_ID] {
        store
            .save_run(&RunRecord {
                run_id: RunId(run_id),
                owner: RunOwner::Unscoped,
                workflow_name: format!("wf-{run_id}"),
                started_epoch_ms: STARTED_EPOCH_MS,
                finished_epoch_ms: None,
                node_count: RUN_RECORD_NODE_COUNT,
                cache_hits: RUN_RECORD_CACHE_HITS,
                failures: RUN_RECORD_FAILURES,
            })
            .unwrap();
    }

    let next_run_id = store.next_run_id().unwrap();

    assert_eq!(next_run_id, RunId(EXPECTED_NEXT_RUN_ID));
}

// r[verify lattice.agent_orchestration.run_identity]
// r[verify lattice.agent_orchestration.run_identity.scenario.concurrent]
#[test]
fn concurrent_run_id_allocations_are_unique_before_any_run_is_saved() {
    let (store, _dir) = common::temp_store();
    let shared = Arc::new(store);
    let handles = (0..CONCURRENT_RUN_ID_ALLOCATIONS)
        .map(|_| {
            let store = Arc::clone(&shared);
            std::thread::spawn(move || store.next_run_id().expect("run id allocation must succeed"))
        })
        .collect::<Vec<_>>();
    let allocated = handles
        .into_iter()
        .map(|handle| handle.join().expect("allocator thread must not panic"))
        .collect::<Vec<_>>();
    let unique = allocated.iter().map(|run_id| run_id.0).collect::<HashSet<_>>();

    assert_eq!(allocated.len(), CONCURRENT_RUN_ID_ALLOCATIONS);
    assert_eq!(unique.len(), CONCURRENT_RUN_ID_ALLOCATIONS);
    assert!(unique.contains(&INITIAL_NEXT_RUN_ID));
    assert!(unique.contains(&(CONCURRENT_RUN_ID_ALLOCATIONS as u64 - 1)));
}

// r[verify lattice.agent_orchestration.run_identity]
// r[verify lattice.agent_orchestration.run_identity.scenario.duplicate]
#[test]
fn duplicate_run_record_identity_is_rejected_without_overwrite() {
    let (store, _dir) = common::temp_store();
    let first = RunRecord {
        run_id: RunId(INITIAL_NEXT_RUN_ID),
        owner: RunOwner::Unscoped,
        workflow_name: "first".to_string(),
        started_epoch_ms: STARTED_EPOCH_MS,
        finished_epoch_ms: None,
        node_count: RUN_RECORD_NODE_COUNT,
        cache_hits: RUN_RECORD_CACHE_HITS,
        failures: RUN_RECORD_FAILURES,
    };
    let mut conflicting = first.clone();
    conflicting.workflow_name = "conflicting".to_string();
    store.save_run(&first).expect("first run must persist");

    let error = store.save_run(&conflicting).expect_err("duplicate run id must fail");
    let persisted =
        store.load_run(first.run_id).expect("run lookup must succeed").expect("first run remains persisted");

    assert!(error.to_string().contains("already persisted"));
    assert_eq!(persisted.workflow_name, first.workflow_name);
    assert_ne!(persisted.workflow_name, conflicting.workflow_name);
}

// r[verify lattice.agent_orchestration.run_identity]
#[test]
fn legacy_run_record_without_owner_decodes_as_unscoped() {
    let legacy = serde_json::json!({
        "run_id": INITIAL_NEXT_RUN_ID,
        "workflow_name": "legacy",
        "started_epoch_ms": STARTED_EPOCH_MS,
        "finished_epoch_ms": null,
        "node_count": RUN_RECORD_NODE_COUNT,
        "cache_hits": RUN_RECORD_CACHE_HITS,
        "failures": RUN_RECORD_FAILURES,
    });

    let record: RunRecord = serde_json::from_value(legacy).expect("legacy record must decode");

    assert_eq!(record.run_id, RunId(INITIAL_NEXT_RUN_ID));
    assert_eq!(record.owner, RunOwner::Unscoped);
    assert_eq!(record.workflow_name, "legacy");
}

// ---------------------------------------------------------------------------
// Cross-module: store + executor integration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn executor_output_round_trips_through_store() {
    let (store, _dir) = common::temp_store();
    let (wf, node_id) = common::standalone_node("echo", "echo stored");

    let executor = lattice::engine::executor::Executor::new();
    let result = executor.execute_node(&wf, node_id, &HashMap::new()).await;

    match &result.status {
        lattice::engine::executor::NodeStatus::Completed { outputs, .. } => {
            // Store each output value.
            for val in outputs.values() {
                store.put_value(val).unwrap();
            }

            // Store in cache.
            store.put_cache(&result.input_hash, outputs).unwrap();

            // Read back from cache.
            let cached = store.get_cache(&result.input_hash).unwrap().unwrap();
            let stdout_port = common::stdout_port_id(&wf, node_id);
            let stdout = cached.get(&stdout_port).unwrap();
            assert_eq!(stdout.as_display_string().trim(), "stored");

            // Read back individual values.
            for val in outputs.values() {
                let loaded = store.get_value(&val.hash).unwrap().unwrap();
                assert_eq!(loaded.hash, val.hash);
            }
        }
        other => panic!("expected Completed, got {:?}", other),
    }
}

#[test]
fn workflow_survives_store_round_trip_with_configs() {
    let (store, _dir) = common::temp_store();
    let (wf, _, _, _, _) = common::diamond_workflow();

    store.save_workflow(&wf).unwrap();
    let loaded = store.load_workflow("diamond").unwrap().unwrap();

    // All configs survived.
    assert_eq!(loaded.configs.len(), 4);
    for (key, cfg) in &wf.configs {
        let loaded_cfg = loaded.configs.get(key).expect("config missing after round-trip");
        assert_eq!(cfg.kind_label(), loaded_cfg.kind_label());
    }
}
