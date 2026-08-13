use crate::model::{BLAKE3_HEX_LENGTH, GIT_COMMIT_SUFFIX, SOURCE_SELECTION};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) fn extract_source_revision(
    source: &Path,
    staging: &Path,
    revision: &str,
) -> Result<(), String> {
    verify_source_revision(source, revision)?;
    crate::filesystem::remove_directory_if_present(staging)?;
    std::fs::create_dir_all(staging)
        .map_err(|error| format!("cannot create {}: {error}", staging.display()))?;

    let mut archive_command = Command::new("git");
    archive_command
        .arg("-C")
        .arg(source)
        .arg("archive")
        .arg("--format=tar")
        .arg(revision)
        .args(SOURCE_SELECTION)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut archive = archive_command
        .spawn()
        .map_err(|error| format!("cannot start git archive: {error}"))?;
    let archive_stdout = archive
        .stdout
        .take()
        .ok_or_else(|| String::from("git archive stdout was not piped"))?;
    let extraction = Command::new("tar")
        .arg("--extract")
        .arg("--file=-")
        .arg("--directory")
        .arg(staging)
        .arg("--no-same-owner")
        .stdin(Stdio::from(archive_stdout))
        .output()
        .map_err(|error| format!("cannot start tar extraction: {error}"))?;
    let archive_output = archive
        .wait_with_output()
        .map_err(|error| format!("cannot wait for git archive: {error}"))?;
    if !archive_output.status.success() {
        return Err(format!(
            "git archive failed: {}",
            String::from_utf8_lossy(&archive_output.stderr)
        ));
    }
    if !extraction.status.success() {
        return Err(format!(
            "tar extraction failed: {}",
            String::from_utf8_lossy(&extraction.stderr)
        ));
    }
    Ok(())
}

fn verify_source_revision(source: &Path, revision: &str) -> Result<(), String> {
    let commit = format!("{revision}{GIT_COMMIT_SUFFIX}");
    let output = Command::new("git")
        .arg("-C")
        .arg(source)
        .arg("cat-file")
        .arg("-e")
        .arg(&commit)
        .output()
        .map_err(|error| format!("cannot inspect source revision: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "source repository does not contain commit {revision}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub(crate) fn blake3_file(path: &Path) -> Result<String, String> {
    let output = Command::new("b3sum")
        .arg("--no-names")
        .arg(path)
        .output()
        .map_err(|error| format!("cannot start b3sum for {}: {error}", path.display()))?;
    parse_blake3_output(&output, path.to_string_lossy().as_ref())
}

pub(crate) fn blake3_bytes(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("b3sum")
        .arg("--no-names")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start b3sum for manifest material: {error}"))?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| String::from("b3sum stdin was not piped"))?;
    input
        .write_all(bytes)
        .map_err(|error| format!("cannot write manifest material to b3sum: {error}"))?;
    drop(input);
    let output = child
        .wait_with_output()
        .map_err(|error| format!("cannot wait for manifest b3sum: {error}"))?;
    parse_blake3_output(&output, "manifest material")
}

fn parse_blake3_output(output: &std::process::Output, source: &str) -> Result<String, String> {
    if !output.status.success() {
        return Err(format!(
            "b3sum failed for {source}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let digest = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("b3sum output was not UTF-8 for {source}: {error}"))?
        .trim()
        .to_owned();
    if digest.len() != BLAKE3_HEX_LENGTH
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "b3sum returned a non-canonical digest for {source}"
        ));
    }
    Ok(digest)
}
