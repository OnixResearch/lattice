fn write_executable(path: &std::path::Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, body).expect("write script");
    let mut permissions = std::fs::metadata(path).expect("script metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("script permissions");
}

fn isolated_request(root: &std::path::Path, sandbox: &std::path::Path, script: &str) -> super::super::Request {
    let boundary =
        crate::candidate_capture::admit_workspace_boundary(root, &["src".into()], &["src".into()], false, false)
            .expect("admitted workspace boundary");
    super::super::Request {
        work_id: "work-systemd".into(),
        attempt_id: "attempt-systemd".into(),
        fence: "7".into(),
        workspace_id: "workspace-systemd".into(),
        program: "sh".into(),
        args: vec!["-c".into(), script.into()],
        boundary,
        require_isolation: true,
        sandbox_program: Some(sandbox.to_string_lossy().into_owned()),
        broker: None,
        timeout_secs: 30,
        output_limit_bytes: 65_536,
    }
}

#[test]
fn systemd_plan_wraps_exact_bwrap_argv_with_minimal_properties() {
    let directory = tempfile::tempdir().expect("tempdir");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    let fake_bwrap = root.join("fake-bwrap");
    write_executable(&fake_bwrap, "#!/bin/sh\nexit 0\n");

    let request = isolated_request(root, &fake_bwrap, "printf ok > src/out");
    let (bwrap_program, bwrap_args) = super::command(&request).expect("bwrap command");
    let process = crate::agent_node::ProcessRequest {
        program: bwrap_program.clone(),
        args: bwrap_args.clone(),
        cwd: Some(root.to_string_lossy().into_owned()),
        env: std::collections::BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: request.timeout_secs,
        output_limit_bytes: request.output_limit_bytes,
    };

    let config = super::systemd::Config {
        systemd_run_program: "/run/current-system/sw/bin/systemd-run".into(),
        systemctl_program: "/run/current-system/sw/bin/systemctl".into(),
        bus: super::systemd::Bus::System,
        unit_prefix: super::systemd::DEFAULT_UNIT_PREFIX.into(),
        slice: Some("lattice-workers.slice".into()),
    };
    let plan = super::systemd::plan_launch(super::systemd::PlanInput {
        config: &config,
        process: &process,
        teardown_grace_millis: 500,
        unit_token: "0123456789abcdef",
    })
    .expect("valid systemd plan");

    assert_eq!(plan.unit_name, "lattice-worker-0123456789abcdef.service");
    assert!(plan.client_env.is_empty());
    assert_eq!(plan.stop_args, vec!["--system", "stop", "lattice-worker-0123456789abcdef.service",]);
    assert!(plan.launch_args.contains(&"--system".to_string()));
    assert!(plan.launch_args.contains(&"--wait".to_string()));
    assert!(plan.launch_args.contains(&"--pipe".to_string()));
    assert!(plan.launch_args.contains(&"--collect".to_string()));
    assert!(plan.launch_args.contains(&"--service-type=exec".to_string()));
    assert!(plan.launch_args.contains(&"--slice=lattice-workers.slice".to_string()));
    assert!(plan.launch_args.contains(&"--property=ProtectKernelTunables=no".to_string()));
    assert!(plan.launch_args.contains(&"--property=RestrictAddressFamilies=AF_UNIX AF_NETLINK".to_string()));
    assert!(plan.launch_args.contains(&"--property=KillMode=control-group".to_string()));
    assert!(plan.launch_args.contains(&"--property=KillSignal=SIGKILL".to_string()));
    assert!(plan.launch_args.contains(&"--property=TimeoutStopSec=500ms".to_string()));
    assert!(plan.launch_args.contains(&"--property=RuntimeMaxSec=36s".to_string()));

    let separator = plan.launch_args.iter().position(|arg| arg == "--").expect("argument separator");
    let wrapped_tail = &plan.launch_args[separator + 1..];
    let mut expected_tail = vec![bwrap_program];
    expected_tail.extend(bwrap_args);
    assert_eq!(wrapped_tail, expected_tail.as_slice());
}

#[test]
fn systemd_plan_user_bus_sets_only_minimal_client_env_and_rejects_invalid_configs() {
    let base_process = crate::agent_node::ProcessRequest {
        program: "/usr/bin/bwrap".into(),
        args: vec!["--version".into()],
        cwd: None,
        env: std::collections::BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: 10,
        output_limit_bytes: 1024,
    };
    let valid_user = super::systemd::Config {
        systemd_run_program: "/run/current-system/sw/bin/systemd-run".into(),
        systemctl_program: "/run/current-system/sw/bin/systemctl".into(),
        bus: super::systemd::Bus::User {
            runtime_dir: Some("/run/user/1000".into()),
            bus_address: None,
        },
        unit_prefix: "lattice-worker".into(),
        slice: None,
    };
    let plan = super::systemd::plan_launch(super::systemd::PlanInput {
        config: &valid_user,
        process: &base_process,
        teardown_grace_millis: 200,
        unit_token: "fedcba9876543210",
    })
    .expect("valid user bus plan");
    assert_eq!(plan.launch_args.first().map(String::as_str), Some("--user"));
    assert_eq!(plan.stop_args.first().map(String::as_str), Some("--user"));
    assert_eq!(plan.client_env.len(), 1);
    assert_eq!(plan.client_env.get("XDG_RUNTIME_DIR").map(String::as_str), Some("/run/user/1000"));

    let mut bad_relative = valid_user.clone();
    bad_relative.systemd_run_program = "systemd-run".into();
    assert!(super::systemd::validate_config(&bad_relative).is_err());

    let mut bad_prefix = valid_user.clone();
    bad_prefix.unit_prefix = "../escape".into();
    assert!(super::systemd::validate_config(&bad_prefix).is_err());

    let mut bad_slice = valid_user.clone();
    bad_slice.slice = Some("not-a-slice".into());
    assert!(super::systemd::validate_config(&bad_slice).is_err());

    let mut empty_user = valid_user;
    empty_user.bus = super::systemd::Bus::User {
        runtime_dir: None,
        bus_address: None,
    };
    assert!(super::systemd::validate_config(&empty_user).is_err());
}

#[cfg(unix)]
#[test]
fn systemd_launcher_stops_detached_unit_on_cancellation_and_prevents_late_write() {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    let directory = tempfile::tempdir().expect("tempdir");
    let root = directory.path().join("workspace");
    let state = directory.path().join("systemd-state");
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    std::fs::create_dir_all(&state).expect("state dir");
    std::fs::write(root.join("src/main.rs"), b"initial").expect("base file");

    let sh_path = which::which("sh").expect("sh");
    let sleep_path = which::which("sleep").expect("sleep");
    let kill_path = which::which("kill").expect("kill");
    let cat_path = which::which("cat").expect("cat");
    let fake_bwrap = directory.path().join("fake-bwrap");
    write_executable(&fake_bwrap, "#!/bin/sh\nexit 0\n");

    // Simulate systemd-run --wait where the service process runs in its own session
    // and would survive SIGTERM to systemd-run unless systemctl stop kills it.
    let fake_systemd_run = directory.path().join("fake-systemd-run");
    write_executable(
        &fake_systemd_run,
        &format!(
            "#!{sh}\n\
             unit=\"\"\n\
             for arg in \"$@\"; do\n\
               case \"$arg\" in\n\
                 --unit=*) unit=\"${{arg#--unit=}}\" ;;\n\
               esac\n\
             done\n\
             (\n\
               trap '' TERM\n\
               {sleep} 0.8\n\
               printf leaked > '{leak}'\n\
             ) </dev/null >/dev/null 2>&1 &\n\
             child=$!\n\
             printf '%s' \"$child\" > '{state}/'\"$unit\".pid\n\
             printf partial-out\n\
             {sleep} 5\n",
            sh = sh_path.display(),
            sleep = sleep_path.display(),
            leak = root.join("src/leak").display(),
            state = state.display(),
        ),
    );
    let fake_systemctl = directory.path().join("fake-systemctl");
    write_executable(
        &fake_systemctl,
        &format!(
            "#!{sh}\n\
             unit=\"$3\"\n\
             printf '%s' \"$unit\" > '{state}/stopped-unit'\n\
             pidfile='{state}/'\"$unit\".pid\n\
             if [ -f \"$pidfile\" ]; then\n\
               pid=$({cat} \"$pidfile\")\n\
               {kill} -KILL \"$pid\" 2>/dev/null || true\n\
             fi\n\
             exit 0\n",
            sh = sh_path.display(),
            cat = cat_path.display(),
            kill = kill_path.display(),
            state = state.display(),
        ),
    );

    let launcher = super::launcher::Systemd::new(super::systemd::Config {
        systemd_run_program: fake_systemd_run.to_string_lossy().into_owned(),
        systemctl_program: fake_systemctl.to_string_lossy().into_owned(),
        bus: super::systemd::Bus::System,
        unit_prefix: "lattice-test".into(),
        slice: None,
    })
    .expect("launcher");

    let request = isolated_request(&root, &fake_bwrap, "sleep 5");
    let cancellation = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let signal = Arc::clone(&cancellation);
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(150));
        signal.store(true, Ordering::Release);
    });

    let outcome = super::super::run_with_launcher(
        &request,
        &crate::process::core::RuntimeConfig::default(),
        cancellation.as_ref(),
        &launcher,
    )
    .expect("run");
    canceller.join().expect("canceller");

    assert!(matches!(outcome, super::super::Outcome::Cancelled { .. }), "{outcome:?}");
    let stopped = std::fs::read_to_string(state.join("stopped-unit")).expect("systemctl stop called");
    assert!(stopped.starts_with("lattice-test-") && stopped.ends_with(".service"), "stopped={stopped}");
    std::thread::sleep(Duration::from_millis(900));
    assert!(!root.join("src/leak").exists(), "detached worker leaked after cancellation");
}

#[cfg(unix)]
#[test]
fn systemd_launcher_stops_detached_unit_on_deadline() {
    let directory = tempfile::tempdir().expect("tempdir");
    let root = directory.path().join("workspace");
    let state = directory.path().join("systemd-state");
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    std::fs::create_dir_all(&state).expect("state dir");
    std::fs::write(root.join("src/main.rs"), b"initial").expect("base file");

    let sh_path = which::which("sh").expect("sh");
    let sleep_path = which::which("sleep").expect("sleep");
    let kill_path = which::which("kill").expect("kill");
    let cat_path = which::which("cat").expect("cat");
    let fake_bwrap = directory.path().join("fake-bwrap");
    write_executable(&fake_bwrap, "#!/bin/sh\nexit 0\n");

    let fake_systemd_run = directory.path().join("fake-systemd-run");
    write_executable(
        &fake_systemd_run,
        &format!(
            "#!{sh}\n\
             unit=\"\"\n\
             for arg in \"$@\"; do\n\
               case \"$arg\" in\n\
                 --unit=*) unit=\"${{arg#--unit=}}\" ;;\n\
               esac\n\
             done\n\
             (\n\
               trap '' TERM\n\
               {sleep} 1.6\n\
               printf leaked > '{leak}'\n\
             ) </dev/null >/dev/null 2>&1 &\n\
             printf '%s' \"$!\" > '{state}/'\"$unit\".pid\n\
             {sleep} 5\n",
            sh = sh_path.display(),
            sleep = sleep_path.display(),
            leak = root.join("src/leak").display(),
            state = state.display(),
        ),
    );
    let fake_systemctl = directory.path().join("fake-systemctl");
    write_executable(
        &fake_systemctl,
        &format!(
            "#!{sh}\n\
             unit=\"$3\"\n\
             pidfile='{state}/'\"$unit\".pid\n\
             if [ -f \"$pidfile\" ]; then\n\
               pid=$({cat} \"$pidfile\")\n\
               {kill} -KILL \"$pid\" 2>/dev/null || true\n\
             fi\n\
             exit 0\n",
            sh = sh_path.display(),
            cat = cat_path.display(),
            kill = kill_path.display(),
            state = state.display(),
        ),
    );

    let launcher = super::launcher::Systemd::new(super::systemd::Config {
        systemd_run_program: fake_systemd_run.to_string_lossy().into_owned(),
        systemctl_program: fake_systemctl.to_string_lossy().into_owned(),
        bus: super::systemd::Bus::System,
        unit_prefix: "lattice-deadline".into(),
        slice: None,
    })
    .expect("launcher");

    let mut request = isolated_request(&root, &fake_bwrap, "sleep 5");
    request.timeout_secs = 1;
    let outcome = super::super::run_with_launcher(
        &request,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
        &launcher,
    )
    .expect("run");

    let super::super::Outcome::Failed { blocker, .. } = outcome else {
        panic!("expected deadline failure, got {outcome:?}");
    };
    assert_eq!(blocker, super::super::execution::DEADLINE_BLOCKER);
    std::thread::sleep(std::time::Duration::from_millis(800));
    assert!(!root.join("src/leak").exists(), "detached worker leaked after deadline");
}

#[cfg(unix)]
#[test]
fn systemd_launcher_degrades_to_interrupted_when_teardown_fails_and_accepts_exit_5() {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    let directory = tempfile::tempdir().expect("tempdir");
    let root = directory.path().join("workspace");
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    std::fs::write(root.join("src/main.rs"), b"initial").expect("base file");

    let sh_path = which::which("sh").expect("sh");
    let sleep_path = which::which("sleep").expect("sleep");
    let fake_bwrap = directory.path().join("fake-bwrap");
    write_executable(&fake_bwrap, "#!/bin/sh\nexit 0\n");

    let fake_systemd_run = directory.path().join("fake-systemd-run");
    write_executable(&fake_systemd_run, &format!("#!{}\n{} 5\n", sh_path.display(), sleep_path.display()));
    let failing_systemctl = directory.path().join("failing-systemctl");
    write_executable(&failing_systemctl, &format!("#!{}\nexit 1\n", sh_path.display()));
    let not_loaded_systemctl = directory.path().join("not-loaded-systemctl");
    write_executable(&not_loaded_systemctl, &format!("#!{}\nexit 5\n", sh_path.display()));

    let request = isolated_request(&root, &fake_bwrap, "sleep 5");

    // Case 1: systemctl stop fails with exit 1 -> fail-closed Interrupted(Teardown, Unknown).
    let failing_launcher = super::launcher::Systemd::new(super::systemd::Config {
        systemd_run_program: fake_systemd_run.to_string_lossy().into_owned(),
        systemctl_program: failing_systemctl.to_string_lossy().into_owned(),
        bus: super::systemd::Bus::System,
        unit_prefix: "lattice-fail".into(),
        slice: None,
    })
    .expect("failing launcher");
    let cancellation = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let signal = Arc::clone(&cancellation);
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(120));
        signal.store(true, Ordering::Release);
    });
    let outcome = super::super::run_with_launcher(
        &request,
        &crate::process::core::RuntimeConfig::default(),
        cancellation.as_ref(),
        &failing_launcher,
    )
    .expect("run");
    canceller.join().expect("canceller");
    let super::super::Outcome::Interrupted { failure, .. } = outcome else {
        panic!("expected fail-closed Interrupted on teardown failure, got {outcome:?}");
    };
    assert_eq!(failure.phase, crate::agent_node::ProcessFailurePhase::Teardown);
    assert_eq!(failure.termination, crate::agent_node::ProcessTermination::Unknown);

    // Case 2: systemctl stop returns 5 (unit not loaded / already collected) -> Cancelled preserved.
    let not_loaded_launcher = super::launcher::Systemd::new(super::systemd::Config {
        systemd_run_program: fake_systemd_run.to_string_lossy().into_owned(),
        systemctl_program: not_loaded_systemctl.to_string_lossy().into_owned(),
        bus: super::systemd::Bus::System,
        unit_prefix: "lattice-gone".into(),
        slice: None,
    })
    .expect("not-loaded launcher");
    let cancellation = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let signal = Arc::clone(&cancellation);
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(120));
        signal.store(true, Ordering::Release);
    });
    let outcome = super::super::run_with_launcher(
        &request,
        &crate::process::core::RuntimeConfig::default(),
        cancellation.as_ref(),
        &not_loaded_launcher,
    )
    .expect("run");
    canceller.join().expect("canceller");
    assert!(matches!(outcome, super::super::Outcome::Cancelled { .. }), "{outcome:?}");
}

#[cfg(unix)]
#[test]
fn systemd_launcher_produces_candidate_with_identical_compatibility_identity() {
    let directory = tempfile::tempdir().expect("tempdir");
    let root = directory.path().join("workspace");
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    std::fs::write(root.join("src/main.rs"), b"old").expect("base file");

    let sh_path = which::which("sh").expect("sh");
    let fake_bwrap = directory.path().join("fake-bwrap");
    write_executable(&fake_bwrap, &format!("#!{}\nprintf new > '{}/src/main.rs'\n", sh_path.display(), root.display()));
    let fake_systemd_run = directory.path().join("fake-systemd-run");
    write_executable(
        &fake_systemd_run,
        &format!(
            "#!{sh}\n\
             while [ $# -gt 0 ]; do\n\
               if [ \"$1\" = \"--\" ]; then\n\
                 shift\n\
                 exec \"$@\"\n\
               fi\n\
               shift\n\
             done\n\
             exit 2\n",
            sh = sh_path.display()
        ),
    );
    let fake_systemctl = directory.path().join("fake-systemctl");
    write_executable(&fake_systemctl, &format!("#!{}\nexit 0\n", sh_path.display()));

    let request = isolated_request(&root, &fake_bwrap, "ignored");
    let direct_compat = super::super::compatibility::observe(&request).expect("direct compat");

    let launcher = super::launcher::Systemd::new(super::systemd::Config {
        systemd_run_program: fake_systemd_run.to_string_lossy().into_owned(),
        systemctl_program: fake_systemctl.to_string_lossy().into_owned(),
        bus: super::systemd::Bus::System,
        unit_prefix: "lattice-prod".into(),
        slice: None,
    })
    .expect("launcher");

    let outcome = super::super::run_with_launcher(
        &request,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
        &launcher,
    )
    .expect("run");

    let result = super::super::AttemptResult::from_outcome(&outcome).with_compatibility(&request);
    assert_eq!(result.compatibility, Some(direct_compat));
    let super::super::Outcome::Produced { receipt, .. } = outcome else {
        panic!("expected Produced, got {outcome:?}");
    };
    assert_eq!(receipt.changed_paths, vec!["src/main.rs"]);
    assert_eq!(std::fs::read(root.join("src/main.rs")).expect("updated"), b"new");
}
