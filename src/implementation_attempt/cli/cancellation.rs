async fn requested(path: &std::path::Path, identity: &str) -> Result<(), String> {
    loop {
        match super::artifacts::read_bounded(path, identity.len()) {
            Ok(bytes) if bytes == identity.as_bytes() => return Ok(()),
            Ok(_) => return Err("cancellation request differs from persisted execution".into()),
            Err(error) => {
                // A missing request is the normal polling state; other failures must not be hidden.
                match std::fs::symlink_metadata(path) {
                    Err(missing) if missing.kind() == std::io::ErrorKind::NotFound => {}
                    _ => return Err(error),
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

async fn signalled() -> Result<(), String> {
    #[cfg(unix)]
    {
        let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .map_err(|error| format!("cannot observe termination signal: {error}"))?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result.map_err(|error| error.to_string()),
            signal = terminate.recv() => signal.ok_or_else(|| "termination signal stream closed".into()),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.map_err(|error| error.to_string())
    }
}

pub(super) fn watch(
    signal: std::sync::Arc<std::sync::atomic::AtomicBool>,
    path: std::path::PathBuf,
    identity: String,
) -> tokio::task::JoinHandle<Result<(), String>> {
    tokio::spawn(async move {
        let result = tokio::select! {
            result = requested(&path, &identity) => result,
            result = signalled() => result,
        };
        // Failed signal/request observation also cancels; it never invents termination evidence.
        signal.store(true, std::sync::atomic::Ordering::SeqCst);
        result
    })
}

pub(super) async fn finish(task: tokio::task::JoinHandle<Result<(), String>>) -> Result<(), String> {
    task.abort();
    match task.await {
        Ok(result) => result,
        Err(error) if error.is_cancelled() => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn requests_require_exact_retained_identity() {
        let root = tempfile::tempdir().expect("request directory");
        let path = root.path().join("cancel.request");
        std::fs::write(&path, b"exact").expect("write request");
        assert!(super::requested(&path, "exact").await.is_ok());
        assert!(super::requested(&path, "other").await.is_err());
        std::fs::write(&path, b"oversized request").expect("replace request");
        assert!(super::requested(&path, "exact").await.is_err());
    }
}
