use super::*;

#[test]
fn clear_environment_excludes_ambient_bash_startup_hook() {
    let temp = tempfile::tempdir().unwrap();
    let hook = temp.path().join("startup.sh");
    let marker = temp.path().join("hook-ran");
    std::fs::write(&hook, "printf hook > \"$LATTICE_TEST_HOOK_MARKER\"\n").unwrap();
    let bash = option_env!("LATTICE_CONSTRUCTION_SHELL_DEFAULT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| which::which("bash").unwrap());
    let cat = option_env!("LATTICE_CONSTRUCTION_PATH_DEFAULT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| which::which("cat").unwrap().parent().unwrap().to_path_buf());
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "engine::effect::runtime::credential::environment_tests::environment_child",
            "--nocapture",
        ])
        .env("LATTICE_TEST_CLEAR_ENV_CHILD", "1")
        .env("LATTICE_TEST_BASH_EXE", bash)
        .env("LATTICE_TEST_CAT_PATH", cat)
        .env("LATTICE_TEST_HOOK_MARKER", &marker)
        .env("BASH_ENV", &hook)
        .status()
        .unwrap();
    assert!(status.success(), "clear-environment child must reject ambient startup hook");
}

#[test]
fn environment_child() {
    if std::env::var_os("LATTICE_TEST_CLEAR_ENV_CHILD").is_none() {
        return;
    }
    let marker = std::path::PathBuf::from(std::env::var_os("LATTICE_TEST_HOOK_MARKER").unwrap());
    let request = crate::agent_node::ProcessRequest {
        program: std::env::var("LATTICE_TEST_BASH_EXE").unwrap(),
        args: vec!["-c".to_owned(), "printf 'clean:'; cat".to_owned()],
        cwd: None,
        env: BTreeMap::from([("PATH".to_owned(), std::env::var("LATTICE_TEST_CAT_PATH").unwrap())]),
        stdin: b"payload".to_vec(),
        timeout_secs: 5,
        output_limit_bytes: 4_096,
    };
    let provider: Arc<dyn CredentialProvider> = Arc::new(crate::credentials::EmptyCredentialProvider);
    let clear = ProcessRecipe::new_clear_environment(request.clone(), None, Arc::clone(&provider)).unwrap();
    let inherited = ProcessRecipe::new(request.clone(), None, Arc::clone(&provider)).unwrap();
    assert_ne!(clear.payload(), inherited.payload(), "environment mode must be bound into payload");
    let mut changed_path = request.clone();
    changed_path.env.insert("PATH".to_owned(), "/different/explicit/bin".to_owned());
    let changed = ProcessRecipe::new_clear_environment(changed_path, None, Arc::clone(&provider)).unwrap();
    assert_ne!(clear.payload(), changed.payload(), "explicit PATH must be bound into payload");
    let mut injected = request.clone();
    injected.env.insert("BASH_ENV".to_owned(), "/unreviewed/startup".to_owned());
    assert!(ProcessRecipe::new_clear_environment(injected, None, Arc::clone(&provider)).is_err());
    let runtime = crate::process::core::RuntimeConfig::default();
    let subject = CredentialSubject::Node {
        workflow_id: "test".to_owned(),
        node_id: "shell".to_owned(),
    };
    let output = clear.consume(&runtime, None, &subject).unwrap();
    assert_eq!(output.status_code, Some(0));
    assert_eq!(output.stdout, b"clean:payload");
    assert!(!marker.exists(), "ambient BASH_ENV must not run before the authorized command");
    let output = inherited.consume(&runtime, None, &subject).unwrap();
    assert_eq!(output.stdout, b"clean:payload");
    assert_eq!(std::fs::read(&marker).unwrap(), b"hook", "legacy process inheritance remains in force");
}
