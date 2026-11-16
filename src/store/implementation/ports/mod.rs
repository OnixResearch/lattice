mod coordination;
mod history;

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait WorkflowRepository {
    fn save_workflow(&self, workflow: &crate::engine::Workflow) -> Result<(), crate::store::StoreError>;
    fn load_workflow(&self, name: &str) -> Result<Option<crate::engine::Workflow>, crate::store::StoreError>;
    fn list_workflows(&self) -> Result<Vec<String>, crate::store::StoreError>;
    fn delete_workflow(&self, name: &str) -> Result<(), crate::store::StoreError>;
}
// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait ExecutionCache {
    fn put_cache(
        &self,
        input_hash: &crate::engine::ContentHash,
        outputs: &std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::Value>,
    ) -> Result<(), crate::store::StoreError>;
    fn get_cache(
        &self,
        input_hash: &crate::engine::ContentHash,
    ) -> Result<
        Option<std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::Value>>,
        crate::store::StoreError,
    >;
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait ValueStore {
    fn put_value(&self, value: &crate::engine::Value) -> Result<(), crate::store::StoreError>;
    fn get_value(
        &self,
        hash: &crate::engine::ContentHash,
    ) -> Result<Option<crate::engine::Value>, crate::store::StoreError>;
}

pub trait EffectAttemptRepository {
    fn reserve_effect_attempt(
        &self,
        binding: crate::engine::effect::authority::IntentBinding,
    ) -> Result<crate::engine::effect::authority::Attempt, crate::store::StoreError>;
    fn save_effect_outcome(
        &self,
        event: &crate::engine::effect::authority::AuthorizationOutcomeEvent,
    ) -> Result<(), crate::store::StoreError>;
    fn load_effect_attempt(
        &self,
        identity: &crate::engine::effect::authority::AttemptIdentity,
    ) -> Result<Option<crate::engine::effect::authority::Attempt>, crate::store::StoreError>;
    fn load_effect_outcome(
        &self,
        identity: &str,
    ) -> Result<Option<crate::engine::effect::authority::AuthorizationOutcomeEvent>, crate::store::StoreError>;
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait ScheduleStore {
    fn save_schedule(&self, schedule: &crate::scheduler::ScheduleDefinition) -> Result<(), crate::store::StoreError>;
    fn load_schedule(
        &self,
        schedule_id: &str,
    ) -> Result<Option<crate::scheduler::ScheduleDefinition>, crate::store::StoreError>;
    fn list_schedules(&self) -> Result<Vec<crate::scheduler::ScheduleDefinition>, crate::store::StoreError>;
    fn delete_schedule(&self, schedule_id: &str) -> Result<(), crate::store::StoreError>;
}

pub trait TriggerStore {
    fn save_trigger(&self, trigger: &crate::triggers::TriggerDefinition) -> Result<(), crate::store::StoreError>;
    fn load_trigger(
        &self,
        trigger_id: &str,
    ) -> Result<Option<crate::triggers::TriggerDefinition>, crate::store::StoreError>;
    fn list_triggers(&self) -> Result<Vec<crate::triggers::TriggerDefinition>, crate::store::StoreError>;
    fn delete_trigger(&self, trigger_id: &str) -> Result<(), crate::store::StoreError>;
    fn save_trigger_run(&self, record: &crate::triggers::TriggerRunRecord) -> Result<(), crate::store::StoreError>;
    fn list_trigger_runs(
        &self,
        max_record_count: usize,
    ) -> Result<Vec<crate::triggers::TriggerRunRecord>, crate::store::StoreError>;
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl crate::store::WorkflowRepository for crate::store::Store {
    fn save_workflow(&self, workflow: &crate::engine::Workflow) -> Result<(), crate::store::StoreError> {
        crate::store::Store::save_workflow(self, workflow)
    }

    fn load_workflow(&self, name: &str) -> Result<Option<crate::engine::Workflow>, crate::store::StoreError> {
        crate::store::Store::load_workflow(self, name)
    }

    fn list_workflows(&self) -> Result<Vec<String>, crate::store::StoreError> {
        crate::store::Store::list_workflows(self)
    }

    fn delete_workflow(&self, name: &str) -> Result<(), crate::store::StoreError> {
        crate::store::Store::delete_workflow(self, name)
    }
}
// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl crate::store::ExecutionCache for crate::store::Store {
    fn put_cache(
        &self,
        input_hash: &crate::engine::ContentHash,
        outputs: &std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::Value>,
    ) -> Result<(), crate::store::StoreError> {
        crate::store::Store::put_cache(self, input_hash, outputs)
    }

    fn get_cache(
        &self,
        input_hash: &crate::engine::ContentHash,
    ) -> Result<
        Option<std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::Value>>,
        crate::store::StoreError,
    > {
        crate::store::Store::get_cache(self, input_hash)
    }
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl crate::store::ValueStore for crate::store::Store {
    fn put_value(&self, value: &crate::engine::Value) -> Result<(), crate::store::StoreError> {
        crate::store::Store::put_value(self, value)
    }

    fn get_value(
        &self,
        hash: &crate::engine::ContentHash,
    ) -> Result<Option<crate::engine::Value>, crate::store::StoreError> {
        crate::store::Store::get_value(self, hash)
    }
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl crate::store::ScheduleStore for crate::store::Store {
    fn save_schedule(&self, schedule: &crate::scheduler::ScheduleDefinition) -> Result<(), crate::store::StoreError> {
        crate::store::Store::save_schedule(self, schedule)
    }

    fn load_schedule(
        &self,
        schedule_id: &str,
    ) -> Result<Option<crate::scheduler::ScheduleDefinition>, crate::store::StoreError> {
        crate::store::Store::load_schedule(self, schedule_id)
    }

    fn list_schedules(&self) -> Result<Vec<crate::scheduler::ScheduleDefinition>, crate::store::StoreError> {
        crate::store::Store::list_schedules(self)
    }

    fn delete_schedule(&self, schedule_id: &str) -> Result<(), crate::store::StoreError> {
        crate::store::Store::delete_schedule(self, schedule_id)
    }
}

impl crate::store::TriggerStore for crate::store::Store {
    fn save_trigger(&self, trigger: &crate::triggers::TriggerDefinition) -> Result<(), crate::store::StoreError> {
        crate::store::Store::save_trigger(self, trigger)
    }

    fn load_trigger(
        &self,
        trigger_id: &str,
    ) -> Result<Option<crate::triggers::TriggerDefinition>, crate::store::StoreError> {
        crate::store::Store::load_trigger(self, trigger_id)
    }

    fn list_triggers(&self) -> Result<Vec<crate::triggers::TriggerDefinition>, crate::store::StoreError> {
        crate::store::Store::list_triggers(self)
    }

    fn delete_trigger(&self, trigger_id: &str) -> Result<(), crate::store::StoreError> {
        crate::store::Store::delete_trigger(self, trigger_id)
    }

    fn save_trigger_run(&self, record: &crate::triggers::TriggerRunRecord) -> Result<(), crate::store::StoreError> {
        crate::store::Store::save_trigger_run(self, record)
    }

    fn list_trigger_runs(
        &self,
        max_record_count: usize,
    ) -> Result<Vec<crate::triggers::TriggerRunRecord>, crate::store::StoreError> {
        crate::store::Store::list_trigger_runs(self, max_record_count)
    }
}
