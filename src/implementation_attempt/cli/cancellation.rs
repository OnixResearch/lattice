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

/// `cancel`: request cancellation only; no compute port is called and nothing
/// is released. An attempt that recovery closed before execution has nothing
/// to cancel.
// r[impl lattice.execution_binding.recovery.scenario.cancellation]
pub(super) fn request(request_path: &std::path::Path, state: &std::path::Path) -> Result<(), String> {
    let (request, bytes) = super::artifacts::load_request(request_path)?;
    request.envelope.validate().map_err(|error| error.to_string())?;
    let directory = super::artifacts::directory(state, &request);
    let identity = blake3::hash(&bytes).to_hex().to_string();
    let marker = super::recovery::closed_marker(&identity);
    debug_assert!(marker.len() > identity.len(), "the closure marker never equals a request identity");
    let intent = super::artifacts::read_optional(&directory.join("intent.json"), marker.len())?;
    if intent.as_deref() == Some(marker.as_bytes()) {
        let report =
            serde_json::json!({"cancellation_requested":false,"termination_observed":false,"not_started":true});
        println!("{report}");
        return Ok(());
    }
    if intent.as_deref() != Some(identity.as_bytes()) {
        return Err("cancel request differs from persisted execution".into());
    }
    debug_assert_eq!(identity.len(), 64, "the request identity is one BLAKE3 digest");
    super::artifacts::write_once(&directory.join("cancel.request"), identity.as_bytes())?;
    println!("{}", serde_json::json!({"cancellation_requested":true,"termination_observed":false}));
    Ok(())
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
