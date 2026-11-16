use redb::ReadableTable as _;

impl crate::store::Store {
    // r[impl lattice.agent_orchestration.durability]
    pub fn commit_orchestration_transition(
        &self,
        expected_version: u64,
        instance: &crate::orchestration::Instance,
        event: &crate::orchestration::Event,
    ) -> Result<(), crate::store::StoreError> {
        let has_matching_head =
            instance.event_head.as_ref().is_some_and(|head| head.as_str() == event.event_hash.as_str());
        if event.instance_id != instance.id || !has_matching_head {
            return Err(crate::store::StoreError::Conflict(
                "orchestration transition event does not match the resulting instance head".to_string(),
            ));
        }
        let instance_data = crate::store::serialize(instance)?;
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let mut instances = txn
                .open_table(crate::store::ORCHESTRATION_INSTANCES)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let current = instances
                .get(instance.id.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
                .ok_or_else(|| {
                    crate::store::StoreError::NotFound(format!("orchestration instance '{}'", instance.id))
                })?;
            let current_instance: crate::orchestration::Instance = crate::store::deserialize(current.value())?;
            if current_instance.version != expected_version
                || instance.version != expected_version.saturating_add(crate::store::ORCHESTRATION_VERSION_INCREMENT)
            {
                return Err(crate::store::StoreError::Conflict(format!(
                    "orchestration instance '{}' changed concurrently",
                    instance.id
                )));
            }
            let mut event_table = txn
                .open_table(crate::store::ORCHESTRATION_EVENTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let mut events = match event_table
                .get(instance.id.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            {
                Some(data) => crate::store::deserialize::<Vec<crate::orchestration::Event>>(data.value())?,
                None => Vec::new(),
            };
            validate_next_event(&current_instance, &events, event)?;
            events.push(event.clone());
            let event_data = crate::store::serialize(&events)?;
            drop(current);
            instances
                .insert(instance.id.as_str(), instance_data.as_slice())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
            event_table
                .insert(instance.id.as_str(), event_data.as_slice())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(())
    }
}

fn validate_next_event(
    current: &crate::orchestration::Instance,
    events: &[crate::orchestration::Event],
    event: &crate::orchestration::Event,
) -> Result<(), crate::store::StoreError> {
    let event_prior = event.prior_hash.as_ref().map(|identity| identity.as_str());
    let current_head = current.event_head.as_ref().map(|identity| identity.as_str());
    if event.instance_id != current.id || event_prior != current_head {
        return Err(crate::store::StoreError::Conflict(
            "orchestration event does not extend the current event head".to_string(),
        ));
    }
    let expected_sequence = current
        .last_event_sequence
        .map(|sequence| sequence.saturating_add(crate::store::ORCHESTRATION_EVENT_SEQUENCE_INCREMENT))
        .unwrap_or_default();
    if event.sequence != expected_sequence || u64::try_from(events.len()).ok() != Some(expected_sequence) {
        return Err(crate::store::StoreError::Conflict("orchestration event sequence is not contiguous".to_string()));
    }
    Ok(())
}
