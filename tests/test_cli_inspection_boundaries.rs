//! Read-only CLI inspection must not create or mutate the data directory.

use std::process::Command;

const CONFIG_FILE_NAME: &str = "config.ncl";
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const SERIAL_MAX_PARALLELISM: u32 = 1;

#[test]
fn authority_inspection_does_not_create_missing_data_directory() {
    let directory = tempfile::tempdir().expect("temporary inspection directory");
    let data_dir = directory.path().join("missing-data");
    let config_path = directory.path().join(CONFIG_FILE_NAME);
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {SERIAL_MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string()
        ),
    )
    .expect("inspection config writes");

    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .arg("authority")
        .arg("inspect")
        .arg("missing-workflow")
        .output()
        .expect("authority inspect command runs");

    assert!(!data_dir.exists(), "authority inspection created state: {output:?}");
}
