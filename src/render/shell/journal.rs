use std::io::Read as _;
use std::io::Write as _;

use anyhow::Context;

impl super::Journal {
    pub(super) fn open(request: &crate::render::model::Request) -> anyhow::Result<Self> {
        let root = std::path::PathBuf::from(&request.journal_root).join(request.id()?.to_string());
        std::fs::create_dir_all(&root)?;
        anyhow::ensure!(!std::fs::symlink_metadata(&root)?.file_type().is_symlink(), "symlink journal rejected");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))?;
        }
        let dir = cap_std::fs::Dir::open_ambient_dir(&root, cap_std::ambient_authority())?;
        let mut options = cap_std::fs::OpenOptions::new();
        options.read(true).write(true).create(true);
        let lock = dir.open_with("writer.lock", &options)?.into_std();
        lock.try_lock().context("render journal has an active writer")?;
        let mut latest = None;
        for sequence in 0..super::MAX_RECORDS {
            let name = format!("{sequence:04}.json");
            if !dir.try_exists(&name)? {
                break;
            }
            let mut bytes = Vec::new();
            dir.open(&name)?
                .take(u64::try_from(super::MAX_RECORD_BYTES)?.checked_add(1).context("record read bound overflow")?)
                .read_to_end(&mut bytes)?;
            anyhow::ensure!(bytes.len() <= super::MAX_RECORD_BYTES, "journal record bound");
            let record: super::Record = serde_json::from_slice(&bytes)?;
            anyhow::ensure!(
                record.sequence == sequence && record.binding == request.id()? && record.request == *request,
                "crossed durable record"
            );
            latest = Some(record);
        }
        let has_existing = latest.is_some();
        let is_restored = latest.as_ref().is_some_and(|record| {
            record.dispatch_possible
                || record.snapshot.is_some()
                || record.measured_adapter_wall_millis > 0
                || record.cancellation_requested
                || record.superseded
        });
        let record = latest.unwrap_or(super::Record {
            binding: request.id()?,
            request: request.clone(),
            sequence: 0,
            dispatch_possible: false,
            acknowledged: false,
            cancellation_requested: false,
            superseded: false,
            outcome: crate::render::model::Outcome::Pending,
            snapshot: None,
            accepted: None,
            measured_adapter_wall_millis: 0,
        });
        let mut journal = Self {
            dir,
            root,
            _lock: lock,
            record,
            is_restored,
        };
        if !has_existing {
            journal.save()?;
        }
        Ok(journal)
    }

    pub(super) fn save(&mut self) -> anyhow::Result<()> {
        let next = if self.dir.try_exists(format!("{:04}.json", self.record.sequence))? {
            self.record.sequence.checked_add(1).context("journal overflow")?
        } else {
            self.record.sequence
        };
        anyhow::ensure!(next < super::MAX_RECORDS, "render journal retention exhausted");
        self.record.sequence = next;
        let bytes = serde_json::to_vec(&self.record)?;
        anyhow::ensure!(bytes.len() <= super::MAX_RECORD_BYTES, "render record byte bound exceeded");
        self.write_new(&format!("{next:04}.json"), &bytes)
    }

    fn write_new(&self, name: &str, bytes: &[u8]) -> anyhow::Result<()> {
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = self.dir.open_with(name, &options)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        self.dir.open(".")?.sync_all()?;
        Ok(())
    }

    pub(super) fn exact_file(&self, name: &str, bytes: &[u8]) -> anyhow::Result<std::path::PathBuf> {
        let path = self.root.join(name);
        if self.dir.try_exists(name)? {
            anyhow::ensure!(
                super::read_bounded(&path, u64::try_from(molten_batch_core::MAX_CONTENT_BYTES)?)? == bytes,
                "retained object changed"
            );
        } else {
            self.write_new(name, bytes)?;
        }
        Ok(path)
    }

    pub(super) fn owner(&self) -> anyhow::Result<Option<super::HostExecution>> {
        let path = self.root.join("host-execution.json");
        if !path.try_exists()? {
            return Ok(None);
        }
        Ok(Some(serde_json::from_slice(&super::read_bounded(&path, 4096)?)?))
    }

    pub(super) fn receipt(&self, reused: bool) -> anyhow::Result<super::Receipt> {
        Ok(super::Receipt {
            binding: self.record.binding,
            execution: self.record.request.execution.clone(),
            attempt: self.record.request.attempt.clone(),
            budget_identity: self.record.request.budget.identity.clone(),
            outcome: self.record.outcome,
            recovery: if reused {
                "reused_committed_artifacts"
            } else if self.is_restored {
                "restored_workflow_state"
            } else {
                "new_computation_requested"
            }
            .into(),
            accepted_manifest: self
                .record
                .accepted
                .as_ref()
                .map(|m| molten_orchestrator_codec::identity("lattice.render-accepted.v1", m))
                .transpose()?,
            host_execution: self.owner()?,
            observed_host_execution: None,
            measured_adapter_wall_millis: self.record.measured_adapter_wall_millis,
            remote_usage: None,
            remote_stop_observed: self.record.outcome == crate::render::model::Outcome::Stopped,
        })
    }
}
