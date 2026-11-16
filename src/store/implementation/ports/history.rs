// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl crate::store::RunHistoryStore for crate::store::Store {
    fn save_run(&self, record: &crate::store::RunRecord) -> Result<(), crate::store::StoreError> {
        crate::store::Store::save_run(self, record)
    }

    fn load_run(
        &self,
        run_id: crate::engine::RunId,
    ) -> Result<Option<crate::store::RunRecord>, crate::store::StoreError> {
        crate::store::Store::load_run(self, run_id)
    }

    fn save_run_details(&self, record: &crate::store::RunDetailRecord) -> Result<(), crate::store::StoreError> {
        crate::store::Store::save_run_details(self, record)
    }

    fn load_run_details(
        &self,
        run_id: crate::engine::RunId,
    ) -> Result<Option<crate::store::RunDetailRecord>, crate::store::StoreError> {
        crate::store::Store::load_run_details(self, run_id)
    }

    fn save_run_replay_events(
        &self,
        run_id: crate::engine::RunId,
        events: &[crate::replay::ReplayEventRecord],
    ) -> Result<(), crate::store::StoreError> {
        crate::store::Store::save_run_replay_events(self, run_id, events)
    }

    fn load_run_replay_events(
        &self,
        run_id: crate::engine::RunId,
    ) -> Result<Option<Vec<crate::replay::ReplayEventRecord>>, crate::store::StoreError> {
        crate::store::Store::load_run_replay_events(self, run_id)
    }

    fn list_runs(&self, max_record_count: usize) -> Result<Vec<crate::store::RunRecord>, crate::store::StoreError> {
        crate::store::Store::list_runs(self, max_record_count)
    }

    fn next_run_id(&self) -> Result<crate::engine::RunId, crate::store::StoreError> {
        crate::store::Store::next_run_id(self)
    }
}
