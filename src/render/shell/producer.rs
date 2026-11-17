use anyhow::Context;

impl super::Producer<'_> {
    fn call(&self, command: &str, arguments: Vec<String>) -> anyhow::Result<serde_json::Value> {
        use crate::agent_node::ProcessRunner as _;
        super::context::validate_permit(self.request, self.permit)?;
        anyhow::ensure!(
            crate::operation::identity::executor_file(std::path::Path::new(&self.request.route.executable))?
                == self.request.route.executable_identity,
            "producer executable changed"
        );
        let remaining = if matches!(command, "progress" | "get" | "cancel") {
            // Read-only recovery/cancellation does not reset computation budget.
            30_000
        } else {
            let remaining = self.request.budget.deadline_epoch_millis.saturating_sub(super::now_millis()?);
            anyhow::ensure!(remaining >= 1000, "original budget deadline expired");
            remaining
        };
        let mut args = vec![
            command.into(),
            "--root".into(),
            self.request.route.client_root.clone(),
            "--target".into(),
            self.request.route.controller.clone(),
        ];
        args.extend(arguments);
        let output = crate::agent_node::ProductionProcessRunner::new(self.process.clone()).run(
            &crate::agent_node::ProcessRequest {
                program: self.request.route.executable.clone(),
                args,
                cwd: None,
                env: std::collections::BTreeMap::new(),
                stdin: Vec::new(),
                timeout_secs: (remaining / 1000).min(30),
                output_limit_bytes: molten_batch_core::MAX_PROGRESS_REPLY_BYTES,
            },
        )?;
        // Killing/timing out this client never establishes remote stop.
        anyhow::ensure!(output.status_code == Some(0), "producer command outcome unavailable");
        let value = serde_json::from_slice(&output.stdout).context("producer observation malformed")?;
        super::check_current(self.request)?;
        Ok(value)
    }
}

impl super::Batch for super::Producer<'_> {
    fn put(
        &mut self,
        kind: molten_batch_core::ContentKind,
        file: &std::path::Path,
    ) -> anyhow::Result<molten_batch_core::Id> {
        let kind = match kind {
            molten_batch_core::ContentKind::Component => "component",
            molten_batch_core::ContentKind::Data => "data",
        };
        response_id(&self.call("put", vec!["--kind".into(), kind.into(), "--file".into(), path_text(file)?])?, "id")
    }
    fn submit(&mut self, key: &str, file: &std::path::Path) -> anyhow::Result<molten_batch_core::Id> {
        response_id(&self.call("submit", vec!["--key".into(), key.into(), "--file".into(), path_text(file)?])?, "run")
    }
    fn snapshot(&mut self, run: molten_batch_core::Id) -> anyhow::Result<molten_batch_core::ProgressSnapshot> {
        Ok(serde_json::from_value(self.call("progress", vec!["--run".into(), run.to_string()])?)?)
    }
    fn get(&mut self, id: molten_batch_core::Id, file: &std::path::Path) -> anyhow::Result<()> {
        let reply = self.call("get", vec!["--id".into(), id.to_string(), "--out".into(), path_text(file)?])?;
        anyhow::ensure!(response_id(&reply, "id")? == id, "crossed artifact response");
        Ok(())
    }
    fn cancel(&mut self, run: molten_batch_core::Id) -> anyhow::Result<()> {
        self.call("cancel", vec!["--run".into(), run.to_string()])?;
        Ok(())
    }
}

fn response_id(value: &serde_json::Value, field: &str) -> anyhow::Result<molten_batch_core::Id> {
    molten_batch_core::Id::parse(
        value.get(field).and_then(serde_json::Value::as_str).context("missing producer identity")?,
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
fn path_text(path: &std::path::Path) -> anyhow::Result<String> {
    Ok(path.to_str().context("non UTF-8 admitted path")?.into())
}
