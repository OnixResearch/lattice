use std::path::Path;
use std::process::Command;

use serde_json::Value;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub struct Fixture {
    pub root: tempfile::TempDir,
}

impl Fixture {
    pub fn new() -> TestResult<Self> {
        Ok(Self {
            root: tempfile::tempdir()?,
        })
    }

    pub fn write(&self, relative: &str, contents: impl AsRef<[u8]>) -> TestResult {
        let path = self.root.path().join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn command(&self) -> Command {
        let program = std::env::var_os("LATTICE_CAIRN_BIN").unwrap_or_else(|| "cairn".into());
        let mut command = Command::new(program);
        command.current_dir(self.root.path()).env_remove("CAIRN_GLOBAL_REGISTRY");
        command
    }

    pub fn run(&self, args: &[&str]) -> TestResult<(bool, Value)> {
        let output = self
            .command()
            .args(args)
            .arg("--root")
            .arg(self.root.path())
            .arg("--policy")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("cairn-policy/generated/cairn-policy.json"))
            .output()?;
        let receipt = serde_json::from_slice(&output.stdout).map_err(|error| {
            format!(
                "Cairn {args:?}: {error}; status={:?}; stdout={}; stderr={}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
        Ok((output.status.success(), receipt))
    }
}
