use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

// r[impl lattice.role_protocol.persistence]
impl crate::engine::protocol::shell::SessionRepository for crate::store::Store {
    fn create(
        &self,
        session: &crate::engine::protocol::SessionRecord,
    ) -> Result<(), crate::engine::protocol::shell::StoreError> {
        let session_data = crate::store::serialize(session).map_err(map_session_error)?;
        let txn = self.db.begin_write().map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
        {
            let mut sessions = txn
                .open_table(crate::store::PROTOCOL_SESSIONS)
                .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
            if sessions
                .get(session.plan.session.as_str())
                .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?
                .is_some()
            {
                return Err(crate::engine::protocol::shell::StoreError::Conflict);
            }
            sessions
                .insert(session.plan.session.as_str(), session_data.as_slice())
                .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
        }
        txn.commit().map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)
    }

    fn load(
        &self,
        identity: &crate::engine::protocol::Identity,
    ) -> Result<Option<crate::engine::protocol::SessionRecord>, crate::engine::protocol::shell::StoreError> {
        let txn = self.db.begin_read().map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
        let sessions = txn
            .open_table(crate::store::PROTOCOL_SESSIONS)
            .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
        sessions
            .get(identity.as_str())
            .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?
            .map(|data| crate::store::deserialize(data.value()).map_err(map_session_error))
            .transpose()
    }

    fn commit(
        &self,
        expected_version: u64,
        session: &crate::engine::protocol::SessionRecord,
        event: &crate::engine::protocol::SessionEvent,
    ) -> Result<(), crate::engine::protocol::shell::StoreError> {
        let session_data = crate::store::serialize(session).map_err(map_session_error)?;
        let event_data = crate::store::serialize(event).map_err(map_session_error)?;
        let event_key = protocol_event_key(session);
        let txn = self.db.begin_write().map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
        {
            let mut sessions = txn
                .open_table(crate::store::PROTOCOL_SESSIONS)
                .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
            let persisted: crate::engine::protocol::SessionRecord = {
                let existing = sessions
                    .get(session.plan.session.as_str())
                    .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?
                    .ok_or(crate::engine::protocol::shell::StoreError::Conflict)?;
                crate::store::deserialize(existing.value()).map_err(map_session_error)?
            };
            if persisted.version != expected_version || session.version != expected_version.saturating_add(1) {
                return Err(crate::engine::protocol::shell::StoreError::Conflict);
            }
            sessions
                .insert(session.plan.session.as_str(), session_data.as_slice())
                .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
        }
        {
            let mut events = txn
                .open_table(crate::store::PROTOCOL_EVENTS)
                .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
            if events
                .get(event_key.as_str())
                .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?
                .is_some()
            {
                return Err(crate::engine::protocol::shell::StoreError::Conflict);
            }
            events
                .insert(event_key.as_str(), event_data.as_slice())
                .map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)?;
        }
        txn.commit().map_err(|_| crate::engine::protocol::shell::StoreError::Unavailable)
    }
}

fn protocol_event_key(session: &crate::engine::protocol::SessionRecord) -> String {
    format!("{}:{:020}", session.plan.session.as_str(), session.version)
}

fn map_session_error(_error: crate::store::StoreError) -> crate::engine::protocol::shell::StoreError {
    crate::engine::protocol::shell::StoreError::InvalidRecord
}
