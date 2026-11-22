mod admission;
mod content;
mod publication;
mod repaired;
#[cfg(test)]
mod tests;

struct Admitted {
    request: super::Request,
    /// BLAKE3 of the exact request bytes.
    identity: String,
    workspace: std::path::PathBuf,
    output: std::path::PathBuf,
    directory: std::path::PathBuf,
}

struct Retained {
    store: crate::store::Store,
    observation: serde_json::Value,
    result: crate::candidate_worker::AttemptResult,
}

pub(super) fn run(
    request_path: &std::path::Path,
    state: &std::path::Path,
    output_parent: &std::path::Path,
    revisions: super::session::revision::Store<'_>,
) -> Result<(), String> {
    println!("{}", execute(request_path, state, output_parent, revisions)?);
    Ok(())
}

fn execute(
    request_path: &std::path::Path,
    state: &std::path::Path,
    output_parent: &std::path::Path,
    revisions: super::session::revision::Store<'_>,
) -> Result<serde_json::Value, String> {
    let inputs = admission::load(request_path, state, output_parent, revisions)?;
    let (payload, observation) = if inputs.request.repair.is_some() {
        repaired::load(&inputs)?
    } else {
        let retained = admission::retained(&inputs)?;
        (content::load(&inputs, &retained)?, retained.observation)
    };
    publication::write(&inputs.output, &payload, &observation)?;
    Ok(serde_json::json!({
        "schema": "lattice.implementation-candidate-export.v1",
        "candidate_root": inputs.output,
        "host_receipt": inputs.output.join("host-receipt.json"),
        "accepted": false,
        "integrated": false,
    }))
}
