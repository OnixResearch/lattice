//! Thin read-only persistence shell for safe run inspection.

use redb::ReadableDatabase as _;

const RUNS: redb::TableDefinition<u64, &[u8]> = redb::TableDefinition::new("runs");
const RUN_DETAILS: redb::TableDefinition<u64, &[u8]> = redb::TableDefinition::new("run_details");
const RUN_REPLAY_EVENTS: redb::TableDefinition<u64, &[u8]> = redb::TableDefinition::new("run_replay_events");
const ORCHESTRATION_INSTANCES: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("orchestration_instances");
const ORCHESTRATION_EVENTS: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new("orchestration_events");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenFailure {
    ActiveOwner,
    RecoveryRequired,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairOutcome {
    Repaired,
    AlreadyClean,
}

#[derive(Debug)]
pub enum Error {
    NotFound(crate::engine::RunId),
    Open(OpenFailure),
    Store(crate::store::StoreError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(run_id) => write!(formatter, "run {} not found", run_id.0),
            Self::Open(OpenFailure::ActiveOwner) => {
                write!(formatter, "run evidence store is owned by another process; retry after it exits")
            }
            Self::Open(OpenFailure::RecoveryRequired) => write!(
                formatter,
                "run evidence store requires recovery; run `runs repair` with the same configuration, then retry"
            ),
            Self::Open(OpenFailure::Unavailable) => write!(formatter, "run evidence store is unavailable"),
            Self::Store(_) => write!(formatter, "run evidence store contents could not be read safely"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::store::StoreError> for Error {
    fn from(error: crate::store::StoreError) -> Self {
        Self::Store(error)
    }
}

fn classify_open_failure(error: &redb::DatabaseError) -> OpenFailure {
    match error {
        redb::DatabaseError::DatabaseAlreadyOpen => OpenFailure::ActiveOwner,
        redb::DatabaseError::RepairAborted => OpenFailure::RecoveryRequired,
        _ => OpenFailure::Unavailable,
    }
}

struct ReadOnlyStore {
    database: redb::ReadOnlyDatabase,
}

impl ReadOnlyStore {
    fn open(path: &std::path::Path) -> Result<Self, Error> {
        let database =
            redb::ReadOnlyDatabase::open(path).map_err(|error| Error::Open(classify_open_failure(&error)))?;
        Ok(Self { database })
    }

    fn load_run(
        &self,
        run_id: crate::engine::RunId,
    ) -> Result<Option<crate::store::RunRecord>, crate::store::StoreError> {
        self.load_by_u64(RUNS, run_id.0)
    }

    fn load_details(
        &self,
        run_id: crate::engine::RunId,
    ) -> Result<Option<crate::store::RunDetailRecord>, crate::store::StoreError> {
        self.load_by_u64(RUN_DETAILS, run_id.0)
    }

    fn load_replay(
        &self,
        run_id: crate::engine::RunId,
    ) -> Result<Option<Vec<crate::replay::ReplayEventRecord>>, crate::store::StoreError> {
        self.load_by_u64(RUN_REPLAY_EVENTS, run_id.0)
    }

    fn load_instance(
        &self,
        instance_id: &crate::orchestration::InstanceId,
    ) -> Result<Option<crate::orchestration::OrchestrationInstance>, crate::store::StoreError> {
        self.load_by_str(ORCHESTRATION_INSTANCES, instance_id.as_str())
    }

    fn load_events(
        &self,
        instance_id: &crate::orchestration::InstanceId,
    ) -> Result<Vec<crate::orchestration::OrchestrationEvent>, crate::store::StoreError> {
        Ok(self.load_by_str(ORCHESTRATION_EVENTS, instance_id.as_str())?.unwrap_or_default())
    }

    fn load_by_u64<T: serde::de::DeserializeOwned>(
        &self,
        definition: redb::TableDefinition<u64, &[u8]>,
        key: u64,
    ) -> Result<Option<T>, crate::store::StoreError> {
        let transaction = self.database.begin_read().map_err(store_db_error)?;
        let table = transaction.open_table(definition).map_err(store_table_error)?;
        deserialize_optional(table.get(key).map_err(store_db_error)?)
    }

    fn load_by_str<T: serde::de::DeserializeOwned>(
        &self,
        definition: redb::TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> Result<Option<T>, crate::store::StoreError> {
        let transaction = self.database.begin_read().map_err(store_db_error)?;
        let table = transaction.open_table(definition).map_err(store_table_error)?;
        deserialize_optional(table.get(key).map_err(store_db_error)?)
    }
}

fn deserialize_optional<T: serde::de::DeserializeOwned>(
    data: Option<redb::AccessGuard<'_, &[u8]>>,
) -> Result<Option<T>, crate::store::StoreError> {
    data.map(|data| {
        serde_json::from_slice(data.value()).map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))
    })
    .transpose()
}

fn store_db_error(error: impl std::fmt::Display) -> crate::store::StoreError {
    crate::store::StoreError::Db(error.to_string())
}

fn store_table_error(error: impl std::fmt::Display) -> crate::store::StoreError {
    crate::store::StoreError::Table(error.to_string())
}

// r[impl lattice.workflow_execution.run_store_recovery]
pub fn repair_path(store_path: &std::path::Path) -> Result<RepairOutcome, Error> {
    let repair_observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let callback_observed = repair_observed.clone();
    let mut builder = redb::Builder::new();
    builder.set_repair_callback(move |_| {
        callback_observed.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let database = builder.open(store_path).map_err(|error| Error::Open(classify_open_failure(&error)))?;
    crate::store::Store::fence_reopened_occurrences(&database)?;
    drop(database);
    debug_assert!(store_path.exists());
    if repair_observed.load(std::sync::atomic::Ordering::Relaxed) {
        Ok(RepairOutcome::Repaired)
    } else {
        Ok(RepairOutcome::AlreadyClean)
    }
}

// r[impl lattice.workflow_execution.safe_run_inspection]
pub fn inspect_path(
    store_path: &std::path::Path,
    run_id: crate::engine::RunId,
) -> Result<super::inspection::Report, Error> {
    let store = ReadOnlyStore::open(store_path)?;
    let record = store.load_run(run_id)?.ok_or(Error::NotFound(run_id))?;
    let details = store.load_details(run_id)?;
    let replay_events = store.load_replay(run_id)?;
    let orchestration_instance = load_owner_instance(&store, &record.owner)?;
    let orchestration_events = match &orchestration_instance {
        Some(instance) => store.load_events(&instance.id)?,
        None => Vec::new(),
    };
    let orchestration = orchestration_instance.as_ref().map(|instance| super::inspection::OrchestrationInput {
        instance,
        events: &orchestration_events,
    });
    let result = super::inspection::inspect(super::inspection::Input {
        record: &record,
        details: details.as_ref(),
        replay_events: replay_events.as_deref(),
        orchestration,
    });
    debug_assert_eq!(result.run_id, run_id.0);
    Ok(result)
}

fn load_owner_instance(
    store: &ReadOnlyStore,
    owner: &crate::store::RunOwner,
) -> Result<Option<crate::orchestration::OrchestrationInstance>, crate::store::StoreError> {
    match owner {
        crate::store::RunOwner::Unscoped
        | crate::store::RunOwner::ScheduleOccurrence { .. }
        | crate::store::RunOwner::WorkflowApproval { .. } => Ok(None),
        crate::store::RunOwner::Orchestration { instance_id, .. } => store.load_instance(instance_id),
    }
}
