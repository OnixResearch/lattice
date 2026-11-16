use redb::ReadableTable as _;

impl crate::store::Store {
    // r[impl lattice.agent_orchestration.durability]
    pub fn create_orchestration(
        &self,
        instance: &crate::orchestration::Instance,
        events: &[crate::orchestration::Event],
    ) -> Result<(), crate::store::StoreError> {
        validate_event_batch(instance, events)?;
        let instance_data = crate::store::serialize(instance)?;
        let event_data = crate::store::serialize(events)?;
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let mut instances = txn
                .open_table(crate::store::ORCHESTRATION_INSTANCES)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if instances
                .get(instance.id.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
                .is_some()
            {
                return Err(crate::store::StoreError::Conflict(format!(
                    "orchestration instance '{}' already exists",
                    instance.id
                )));
            }
            instances
                .insert(instance.id.as_str(), instance_data.as_slice())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
            let mut event_table = txn
                .open_table(crate::store::ORCHESTRATION_EVENTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            event_table
                .insert(instance.id.as_str(), event_data.as_slice())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(())
    }
    // r[impl lattice.agent_orchestration.triggers]
    pub fn create_triggered_orchestration(
        &self,
        scoped_key: &crate::orchestration::TriggerScopedKey,
        instance: &crate::orchestration::Instance,
    ) -> Result<crate::store::TriggerAdmissionWrite, crate::store::StoreError> {
        validate_event_batch(instance, &[])?;
        let instance_data = crate::store::serialize(instance)?;
        let empty_events = crate::store::serialize(&Vec::<crate::orchestration::Event>::new())?;
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let outcome = {
            let mut admissions = txn
                .open_table(crate::store::ORCHESTRATION_TRIGGER_ADMISSIONS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let existing_instance_id = admissions
                .get(scoped_key.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
                .map(|existing| {
                    crate::orchestration::InstanceId::try_from_hex(existing.value())
                        .map_err(super::identity_admission_error)
                })
                .transpose()?;
            match existing_instance_id {
                Some(existing_instance_id) => crate::store::TriggerAdmissionWrite::Duplicate {
                    instance_id: existing_instance_id,
                },
                None => {
                    let mut instances = txn
                        .open_table(crate::store::ORCHESTRATION_INSTANCES)
                        .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
                    if instances
                        .get(instance.id.as_str())
                        .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
                        .is_some()
                    {
                        return Err(crate::store::StoreError::Conflict(format!(
                            "orchestration instance '{}' already exists",
                            instance.id
                        )));
                    }
                    instances
                        .insert(instance.id.as_str(), instance_data.as_slice())
                        .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
                    let mut events = txn
                        .open_table(crate::store::ORCHESTRATION_EVENTS)
                        .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
                    events
                        .insert(instance.id.as_str(), empty_events.as_slice())
                        .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
                    admissions
                        .insert(scoped_key.as_str(), instance.id.as_str())
                        .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
                    crate::store::TriggerAdmissionWrite::Inserted
                }
            }
        };
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(outcome)
    }
}

fn validate_event_batch(
    instance: &crate::orchestration::Instance,
    events: &[crate::orchestration::Event],
) -> Result<(), crate::store::StoreError> {
    let mut prior_hash = None;
    let mut prior_sequence = None;
    for event in events {
        validate_event_link(instance, event, prior_hash, prior_sequence)?;
        prior_hash = Some(event.event_hash.as_str());
        prior_sequence = Some(event.sequence);
    }
    if instance.event_head.as_ref().map(|identity| identity.as_str()) != prior_hash
        || instance.last_event_sequence != prior_sequence
    {
        return Err(crate::store::StoreError::Conflict(
            "orchestration creation events do not match the persisted instance head".to_string(),
        ));
    }
    Ok(())
}

fn validate_event_link(
    instance: &crate::orchestration::Instance,
    event: &crate::orchestration::Event,
    prior_hash: Option<&str>,
    prior_sequence: Option<u64>,
) -> Result<(), crate::store::StoreError> {
    if event.instance_id != instance.id || event.prior_hash.as_ref().map(|identity| identity.as_str()) != prior_hash {
        return Err(crate::store::StoreError::Conflict(
            "orchestration creation event chain does not match the instance".to_string(),
        ));
    }
    if let Some(sequence) = prior_sequence
        && event.sequence != sequence.saturating_add(crate::store::ORCHESTRATION_EVENT_SEQUENCE_INCREMENT)
    {
        return Err(crate::store::StoreError::Conflict("orchestration creation events are not contiguous".to_string()));
    }
    Ok(())
}
