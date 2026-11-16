mod admission;
mod content;
mod publication;
#[cfg(test)]
mod tests;

struct Admitted {
    request: super::Request,
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
) -> Result<(), String> {
    println!("{}", execute(request_path, state, output_parent)?);
    Ok(())
}

fn execute(
    request_path: &std::path::Path,
    state: &std::path::Path,
    output_parent: &std::path::Path,
) -> Result<serde_json::Value, String> {
    let inputs = admission::load(request_path, state, output_parent)?;
    let retained = admission::retained(&inputs)?;
    let payload = content::load(&inputs, &retained)?;
    publication::write(&inputs.output, &payload, &retained.observation)?;
    Ok(serde_json::json!({
        "schema": "lattice.implementation-candidate-export.v1",
        "candidate_root": inputs.output,
        "host_receipt": inputs.output.join("host-receipt.json"),
        "accepted": false,
        "integrated": false,
    }))
}
