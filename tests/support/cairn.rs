use std::path::Path;
use std::process::Command;

use serde_json::Value;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Fixture options one Cairn subcommand accepts.
#[derive(Clone, Copy)]
enum Options {
    /// No `--root` or `--policy`; the command reads paths from the fixture directory.
    None,
    /// `--root` only.
    Root,
    /// `--root` and Lattice's generated `--policy`.
    RootAndPolicy,
}

/// Options per subcommand, from the pinned Cairn's `cairn --help` usage lines.
/// Cairn refuses an option a subcommand does not take, so an undeclared
/// subcommand is an error here rather than a guess.
const SUBCOMMAND_OPTIONS: &[(&[&str], Options)] = &[
    (&["init"], Options::Root),
    (&["status"], Options::Root),
    (&["change", "create"], Options::RootAndPolicy),
    (&["gate"], Options::RootAndPolicy),
    (&["sync"], Options::RootAndPolicy),
    (&["workflow", "status"], Options::RootAndPolicy),
    (&["traceability", "coverage"], Options::RootAndPolicy),
    (&["migrate"], Options::RootAndPolicy),
    (&["release-readiness"], Options::RootAndPolicy),
    (&["preserves-corpus", "update"], Options::None),
];

fn subcommand_options(args: &[&str]) -> Result<Options, String> {
    SUBCOMMAND_OPTIONS
        .iter()
        .find(|(subcommand, _)| args.starts_with(subcommand))
        .map(|(_, options)| *options)
        .ok_or_else(|| format!("Cairn {args:?}: declare its options in SUBCOMMAND_OPTIONS"))
}

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
        let program = std::env::var_os("LATTICE_CAIRN_BIN")
            .expect("LATTICE_CAIRN_BIN must point at the pinned native Cairn binary; run inside nix develop");
        let mut command = Command::new(program);
        command.current_dir(self.root.path()).env_remove("CAIRN_GLOBAL_REGISTRY");
        command
    }

    pub fn run(&self, args: &[&str]) -> TestResult<(bool, Value)> {
        let mut command = self.command();
        command.args(args);
        match subcommand_options(args)? {
            Options::None => {}
            Options::Root => {
                command.arg("--root").arg(self.root.path());
            }
            Options::RootAndPolicy => {
                command
                    .arg("--root")
                    .arg(self.root.path())
                    .arg("--policy")
                    .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("cairn-policy/generated/cairn-policy.json"));
            }
        }
        let output = command.output()?;
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
