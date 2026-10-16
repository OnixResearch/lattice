impl Plane {
    pub fn execute_deletion(&self, plan: &DeletionPlan) -> Result<DeletionReceipt, Error> {
        super::lifecycle::validate_deletion_plan(plan)?;
        let _local_guard = self.mutation.lock().map_err(|_| Error::Conflict("mutation owner poisoned"))?;
        let _file_guard = self.acquire_mutation_file()?;
        let mut deleted_versions = Vec::with_capacity(plan.selected_versions.len());
        let mut missing_versions = Vec::with_capacity(plan.selected_versions.len());
        for version in &plan.selected_versions {
            let Some(metadata) = self.store.delete_operation_data_metadata(version)? else {
                missing_versions.push(version.clone());
                continue;
            };
            deleted_versions.push(version.clone());
            let remaining = self.store.list_operation_data_metadata()?;
            let is_content_referenced =
                remaining.iter().any(|record| record.payload.content_identity == metadata.payload.content_identity);
            if !is_content_referenced {
                match self.root.remove_file(&metadata.payload.safe_locator) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(_) => return Err(Error::Io("payload delete")),
                }
            }
        }
        sync_directory(&self.root)?;
        Ok(DeletionReceipt {
            plan_identity: plan.identity.clone(),
            deleted_versions,
            missing_versions,
            payload_content_omitted: true,
            secure_erasure_claimed: false,
        })
    }

    // r[impl lattice.operation_data.versions.uncertain]
    pub fn recover_publications(&self) -> Result<Vec<(String, PublicationOutcome)>, Error> {
        let records = self.list_metadata()?;
        let referenced = records
            .iter()
            .map(|record| record.payload.safe_locator.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let mut outcomes = Vec::with_capacity(records.len());
        for metadata in records {
            let outcome = match self.read_payload(&metadata) {
                Ok(_) => PublicationOutcome::Appended,
                Err(_) => PublicationOutcome::Incomplete,
            };
            outcomes.push((metadata.version_identity, outcome));
        }
        if self.root.open(MUTATION_LOCK_FILE).is_ok() {
            if outcomes.len() == MAX_QUERY_CANDIDATES {
                return Err(Error::Io("publication recovery bound"));
            }
            let identity = crate::operation::identity::hash_bytes(
                "lattice.operation-data-uncertain-owner.v1",
                MUTATION_LOCK_FILE.as_bytes(),
            );
            outcomes.push((identity, PublicationOutcome::Unknown));
        }
        let entries = self.root.read_dir(PAYLOAD_DIRECTORY).map_err(|_| Error::Io("payload recovery scan"))?;
        for entry in entries {
            if outcomes.len() == MAX_QUERY_CANDIDATES {
                return Err(Error::Io("payload recovery bound"));
            }
            let entry = entry.map_err(|_| Error::Io("payload recovery entry"))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let locator = format!("{PAYLOAD_DIRECTORY}/{name}");
            if !referenced.contains(&locator) {
                let identity =
                    crate::operation::identity::hash_bytes("lattice.operation-data-orphan-payload.v1", name.as_bytes());
                outcomes.push((identity, PublicationOutcome::Unknown));
            }
        }
        outcomes.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(outcomes)
    }

    fn acquire_mutation_file(&self) -> Result<MutationFileGuard<'_>, Error> {
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = self.root.open_with(MUTATION_LOCK_FILE, &options).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                Error::Conflict("mutation owner is active or uncertain")
            } else {
                Error::Io("mutation owner create")
            }
        })?;
        file.write_all(MUTATION_LOCK_CONTENT).map_err(|_| Error::Io("mutation owner write"))?;
        file.sync_all().map_err(|_| Error::Io("mutation owner sync"))?;
        sync_directory(&self.root)?;
        Ok(MutationFileGuard { root: &self.root })
    }

    fn publish_admitted(&self, metadata: &RecordMetadata, bytes: &[u8]) -> Result<(), Error> {
        if u64::try_from(bytes.len()).map_err(|_| Error::Io("payload length"))? != metadata.payload.byte_length_bytes {
            return Err(Error::Conflict("admitted payload length differs"));
        }
        match self.root.open(&metadata.payload.safe_locator) {
            Ok(_) => {
                let existing = self.read_payload(metadata)?;
                if existing != bytes {
                    return Err(Error::Conflict("content-addressed payload differs"));
                }
                return Ok(());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Io("payload observation")),
        }
        let version_suffix = metadata
            .version_identity
            .strip_prefix(crate::operation::model::BLAKE3_PREFIX)
            .ok_or(Error::Conflict("version identity is invalid"))?;
        let staging_locator = format!("{PAYLOAD_DIRECTORY}/.{version_suffix}.stage");
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut is_staging_owned = false;
        let publication = (|| {
            let mut file =
                self.root.open_with(&staging_locator, &options).map_err(|_| Error::Io("payload staging create"))?;
            is_staging_owned = true;
            file.write_all(bytes).map_err(|_| Error::Io("payload write"))?;
            file.sync_all().map_err(|_| Error::Io("payload sync"))?;
            match self.root.hard_link(&staging_locator, &self.root, &metadata.payload.safe_locator) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let existing = self.read_payload(metadata)?;
                    if existing == bytes {
                        Ok(())
                    } else {
                        Err(Error::Conflict("content-addressed payload differs"))
                    }
                }
                Err(_) => Err(Error::Io("payload create-only publication")),
            }
        })();
        if is_staging_owned {
            self.root.remove_file(&staging_locator).map_err(|_| Error::Io("payload staging cleanup"))?;
        }
        publication?;
        sync_directory(&self.root)
    }
}
