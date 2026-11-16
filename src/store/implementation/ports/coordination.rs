impl crate::store::OrchestrationStore for crate::store::Store {
    fn save_orchestration_definition(
        &self,
        definition: &crate::orchestration::Definition,
    ) -> Result<(), crate::store::StoreError> {
        crate::store::Store::save_orchestration_definition(self, definition)
    }

    fn load_orchestration_definition(
        &self,
        id: &crate::orchestration::DefinitionId,
    ) -> Result<Option<crate::orchestration::Definition>, crate::store::StoreError> {
        crate::store::Store::load_orchestration_definition(self, id)
    }

    fn next_orchestration_sequence(&self) -> Result<u64, crate::store::StoreError> {
        crate::store::Store::next_orchestration_sequence(self)
    }

    fn create_orchestration(
        &self,
        instance: &crate::orchestration::Instance,
        events: &[crate::orchestration::Event],
    ) -> Result<(), crate::store::StoreError> {
        crate::store::Store::create_orchestration(self, instance, events)
    }

    fn create_triggered_orchestration(
        &self,
        scoped_key: &crate::orchestration::TriggerScopedKey,
        instance: &crate::orchestration::Instance,
    ) -> Result<crate::store::TriggerAdmissionWrite, crate::store::StoreError> {
        crate::store::Store::create_triggered_orchestration(self, scoped_key, instance)
    }

    fn commit_orchestration_transition(
        &self,
        expected_version: u64,
        instance: &crate::orchestration::Instance,
        event: &crate::orchestration::Event,
    ) -> Result<(), crate::store::StoreError> {
        crate::store::Store::commit_orchestration_transition(self, expected_version, instance, event)
    }

    fn load_orchestration(
        &self,
        id: &crate::orchestration::InstanceId,
    ) -> Result<Option<crate::orchestration::Instance>, crate::store::StoreError> {
        crate::store::Store::load_orchestration(self, id)
    }

    fn load_orchestration_events(
        &self,
        id: &crate::orchestration::InstanceId,
    ) -> Result<Vec<crate::orchestration::Event>, crate::store::StoreError> {
        crate::store::Store::load_orchestration_events(self, id)
    }

    fn list_orchestrations(&self) -> Result<Vec<crate::orchestration::Instance>, crate::store::StoreError> {
        crate::store::Store::list_orchestrations(self)
    }

    fn save_orchestration_trigger_admission(
        &self,
        scoped_key: &crate::orchestration::TriggerScopedKey,
        instance_id: &crate::orchestration::InstanceId,
    ) -> Result<crate::store::TriggerAdmissionWrite, crate::store::StoreError> {
        crate::store::Store::save_orchestration_trigger_admission(self, scoped_key, instance_id)
    }

    fn list_orchestration_trigger_admissions(
        &self,
    ) -> Result<
        std::collections::HashMap<crate::orchestration::TriggerScopedKey, crate::orchestration::InstanceId>,
        crate::store::StoreError,
    > {
        crate::store::Store::list_orchestration_trigger_admissions(self)
    }
}
