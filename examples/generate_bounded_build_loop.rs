//! Freeze inert Nickel fixtures for the bounded runtime-iteration example.
#[path = "bounded-build-loop/graph.rs"]
mod bounded_build_loop;

fn main() -> bounded_build_loop::Result<()> {
    let workflow = bounded_build_loop::workflow(std::path::Path::new("examples/bounded-build-loop"))?;
    let definition = bounded_build_loop::definition(&workflow)?;
    std::fs::write(
        "examples/workflows/bounded-build-loop.ncl",
        lattice::workflow_format::workflow_to_nickel(&workflow),
    )?;
    std::fs::write(
        "examples/orchestrations/bounded-build-loop.ncl",
        lattice::orchestration::definition_to_nickel(&definition)?,
    )?;
    Ok(())
}
