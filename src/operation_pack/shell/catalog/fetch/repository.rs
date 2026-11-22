//! Bounded `git` invocations inside one private fetch staging directory.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "repository observations return typed errors instead of terminating the process"
)]
#![allow(
    ambient_env,
    reason = "the fetch shell forwards only an allowlisted transport environment to the git process"
)]

const GIT_PROGRAM: &str = "git";
const SHELL_PROGRAM: &str = "sh";
/// Cap every file the child writes, then replace the shell with `git` and its exact arguments.
const FILE_SIZE_WRAPPER: &str = "ulimit -f \"$1\" && shift && exec \"$@\"";
const WRAPPER_NAME: &str = "lattice-catalog-fetch";
const GIT_DIRECTORY: &str = "repository.git";
pub(super) const GIT_DIRECTORY_ARGUMENT: &str = "--git-dir=repository.git";
const REMOTE: &str = "origin";
const FETCH_TIMEOUT_SECONDS: u64 = 600;
const LOCAL_TIMEOUT_SECONDS: u64 = 60;
const MESSAGE_OUTPUT_BYTES: usize = 65_536;
const LISTING_OUTPUT_BYTES: usize = 1_048_576;
const MAXIMUM_REPOSITORY_ENTRIES: usize = 65_536;
const FORWARDED_ENVIRONMENT: [&str; 4] = ["PATH", "SSL_CERT_FILE", "SSL_CERT_DIR", "SSH_AUTH_SOCK"];
const BOUNDED_FETCH: [&str; 8] = [
    "-c",
    "fetch.unpackLimit=1",
    "-c",
    "transfer.unpackLimit=1",
    "-c",
    "transfer.fsckObjects=true",
    "-c",
    "fetch.negotiationAlgorithm=noop",
];
const FETCH_OPTIONS: [&str; 6] = [
    "fetch",
    "--quiet",
    "--no-tags",
    "--no-write-fetch-head",
    "--recurse-submodules=no",
    "--filter=blob:none",
];

/// One staging repository and its total transfer budget.
pub(super) struct Git<'git> {
    pub(super) staging: &'git std::path::Path,
    pub(super) process: &'git crate::process::core::RuntimeConfig,
    pub(super) limit_bytes: u64,
}

impl Git<'_> {
    /// Create a repository of the commit's object format whose only remote is a promisor for
    /// blob-less fetches. Lazy fetching stays disabled for every later command.
    pub(super) fn prepare(&self, request: &super::Request) -> Result<(), super::super::super::super::Error> {
        let format = format!(
            "--object-format={}",
            super::super::super::super::catalog::transfer::object_format(&request.commit)
        );
        self.run_local(
            &["init", "--bare", "--quiet", "--template=", &format, GIT_DIRECTORY],
            &[],
            MESSAGE_OUTPUT_BYTES,
        )?;
        for (key, value) in [
            ("remote.origin.url", request.repository.as_str()),
            ("remote.origin.promisor", "true"),
            ("remote.origin.partialclonefilter", "blob:none"),
        ] {
            self.run_local(&[GIT_DIRECTORY_ARGUMENT, "config", key, value], &[], MESSAGE_OUTPUT_BYTES)?;
        }
        Ok(())
    }

    /// Fetch the commit and its trees, without blob content, then confirm the exact commit.
    pub(super) fn fetch_commit(&self, commit: &str) -> Result<(), super::super::super::super::Error> {
        let mut arguments = BOUNDED_FETCH.to_vec();
        arguments.push(GIT_DIRECTORY_ARGUMENT);
        arguments.extend(FETCH_OPTIONS);
        arguments.extend(["--depth=1", REMOTE, commit]);
        self.bounded_fetch(&arguments, &[])?;
        let peeled = format!("{commit}^{{commit}}");
        let revision = self.run_local(
            &[
                GIT_DIRECTORY_ARGUMENT,
                "rev-parse",
                "--verify",
                "--end-of-options",
                &peeled,
            ],
            &[],
            MESSAGE_OUTPUT_BYTES,
        )?;
        if revision.trim_ascii() != commit.as_bytes() {
            return Err(super::super::super::super::Error::Invalid("fetched commit differs"));
        }
        Ok(())
    }

    pub(super) fn list(
        &self,
        commit: &str,
        subdirectory: Option<&str>,
    ) -> Result<Vec<u8>, super::super::super::super::Error> {
        let mut arguments = vec![GIT_DIRECTORY_ARGUMENT, "ls-tree", "-r", "-z", "--full-tree", commit];
        if let Some(subdirectory) = subdirectory {
            arguments.extend(["--", subdirectory]);
        }
        self.run_local(&arguments, &[], LISTING_OUTPUT_BYTES)
    }

    /// Fetch exactly the selected blobs by object name within the remaining budget.
    pub(super) fn fetch_blobs(
        &self,
        entries: &[super::super::super::super::catalog::source::TreeEntry],
    ) -> Result<(), super::super::super::super::Error> {
        let mut arguments = BOUNDED_FETCH.to_vec();
        arguments.push(GIT_DIRECTORY_ARGUMENT);
        arguments.extend(FETCH_OPTIONS);
        arguments.extend(["--stdin", REMOTE]);
        self.bounded_fetch(&arguments, &super::object_lines(entries))
    }

    /// Bytes the staging repository holds on disk.
    pub(super) fn used_bytes(&self) -> Result<u64, super::super::super::super::Error> {
        Ok(self.observe()?.0)
    }

    /// Run one fetch with every written file capped at the remaining budget, then check the total.
    fn bounded_fetch(&self, arguments: &[&str], stdin: &[u8]) -> Result<(), super::super::super::super::Error> {
        let remaining_bytes = self.limit_bytes.saturating_sub(self.used_bytes()?);
        if remaining_bytes < super::super::super::super::catalog::transfer::FILE_SIZE_BLOCK_BYTES {
            return Err(super::super::super::super::Error::Invalid("catalog fetch transfer bound"));
        }
        let blocks = super::super::super::super::catalog::transfer::file_size_blocks(remaining_bytes);
        let cap_bytes = blocks.saturating_mul(super::super::super::super::catalog::transfer::FILE_SIZE_BLOCK_BYTES);
        let block_text = blocks.to_string();
        let mut wrapped = vec!["-c", FILE_SIZE_WRAPPER, WRAPPER_NAME, block_text.as_str(), GIT_PROGRAM];
        wrapped.extend_from_slice(arguments);
        let result = self.execute(SHELL_PROGRAM, &wrapped, stdin, MESSAGE_OUTPUT_BYTES, FETCH_TIMEOUT_SECONDS);
        let (used_bytes, largest_bytes) = self.observe()?;
        if largest_bytes >= cap_bytes || used_bytes > self.limit_bytes {
            return Err(super::super::super::super::Error::Invalid("catalog fetch transfer bound"));
        }
        result.map(|_| ())
    }

    /// Run one local `git` command that never contacts the remote.
    pub(super) fn run_local(
        &self,
        arguments: &[&str],
        stdin: &[u8],
        output_limit_bytes: usize,
    ) -> Result<Vec<u8>, super::super::super::super::Error> {
        self.execute(GIT_PROGRAM, arguments, stdin, output_limit_bytes, LOCAL_TIMEOUT_SECONDS)
    }

    fn execute(
        &self,
        program: &str,
        arguments: &[&str],
        stdin: &[u8],
        output_limit_bytes: usize,
        timeout_secs: u64,
    ) -> Result<Vec<u8>, super::super::super::super::Error> {
        let staging = self
            .staging
            .to_str()
            .ok_or(super::super::super::super::Error::Invalid("fetch staging path encoding"))?;
        let mut environment = FORWARDED_ENVIRONMENT
            .iter()
            .filter_map(|key| {
                let value = std::env::var(key).ok().filter(|value| !value.is_empty())?;
                Some(((*key).to_string(), value))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        for (key, value) in [
            ("HOME", staging),
            ("GIT_CONFIG_NOSYSTEM", "1"),
            ("GIT_CONFIG_GLOBAL", "/dev/null"),
            ("GIT_TERMINAL_PROMPT", "0"),
            ("GIT_ALLOW_PROTOCOL", "file:https:ssh"),
            ("GIT_NO_LAZY_FETCH", "1"),
        ] {
            environment.insert(key.to_string(), value.to_string());
        }
        let request = crate::agent_node::ProcessRequest {
            program: program.to_string(),
            args: arguments.iter().map(|argument| (*argument).to_string()).collect(),
            cwd: Some(staging.to_string()),
            env: environment,
            stdin: stdin.to_vec(),
            timeout_secs,
            output_limit_bytes,
        };
        let output = crate::process::shell::run_bounded_clear_environment(&request, self.process)?;
        Ok(output.stdout)
    }

    /// Total and largest regular-file sizes under the staging repository, without following links.
    fn observe(&self) -> Result<(u64, u64), super::super::super::super::Error> {
        let mut pending = vec![self.staging.join(GIT_DIRECTORY)];
        let (mut total_bytes, mut largest_bytes, mut entries) = (0u64, 0u64, 0usize);
        while let Some(directory) = pending.pop() {
            let listing = match std::fs::read_dir(&directory) {
                Ok(listing) => listing,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return Err(super::super::super::super::Error::Io("fetch repository observation")),
            };
            for entry in listing {
                entries = entries.saturating_add(1);
                if entries > MAXIMUM_REPOSITORY_ENTRIES {
                    return Err(super::super::super::super::Error::Invalid("catalog fetch transfer bound"));
                }
                let entry = entry.map_err(|_| super::super::super::super::Error::Io("fetch repository observation"))?;
                let metadata = std::fs::symlink_metadata(entry.path())
                    .map_err(|_| super::super::super::super::Error::Io("fetch repository observation"))?;
                if metadata.is_dir() {
                    pending.push(entry.path());
                } else {
                    total_bytes = total_bytes.saturating_add(metadata.len());
                    largest_bytes = largest_bytes.max(metadata.len());
                }
            }
        }
        Ok((total_bytes, largest_bytes))
    }
}
