//! Real systemd + bubblewrap qualification tests for `SystemdLauncher` (#30).
//!
//! Ignored by default so hermetic builds without a running systemd manager or
//! user namespaces stay green. Execute on a host with `systemd-run`, `systemctl`,
//! a user systemd manager (`XDG_RUNTIME_DIR`), and `LATTICE_BWRAP`:
//!
//!   LATTICE_BWRAP=/path/to/bwrap cargo test --locked --test test_systemd_sandbox_launcher --
//! --ignored --nocapture

#[cfg(unix)]
mod unix_qualification {
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use lattice::candidate_capture::admit_workspace_boundary;
    use lattice::candidate_worker;
    use lattice::candidate_worker::AttemptResult;
    use lattice::candidate_worker::Outcome;
    use lattice::candidate_worker::Request;
    use lattice::candidate_worker::sandbox::launcher::Systemd;
    use lattice::candidate_worker::sandbox::systemd::Bus;
    use lattice::candidate_worker::sandbox::systemd::Config;
    use lattice::candidate_worker::sandbox::systemd::DEFAULT_UNIT_PREFIX;
    use lattice::process::core::RuntimeConfig;

    fn bwrap_path() -> String {
        std::env::var("LATTICE_BWRAP").expect("LATTICE_BWRAP must point to a reviewed bubblewrap executable")
    }

    fn systemd_tool(name: &str) -> String {
        which::which(name)
            .unwrap_or_else(|error| panic!("{name} must be available: {error}"))
            .to_string_lossy()
            .into_owned()
    }

    fn user_launcher() -> Systemd {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR must be set");
        let bus_address = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
        Systemd::new(Config {
            systemd_run_program: systemd_tool("systemd-run"),
            systemctl_program: systemd_tool("systemctl"),
            bus: Bus::User {
                runtime_dir: Some(runtime_dir),
                bus_address,
            },
            unit_prefix: DEFAULT_UNIT_PREFIX.into(),
            slice: None,
        })
        .expect("valid user systemd launcher")
    }

    fn isolated_request(root: &Path, script: &str, timeout_secs: u64) -> Request {
        let sh = which::which("sh").expect("sh").to_string_lossy().into_owned();
        let boundary =
            admit_workspace_boundary(root, &["src".into()], &["src".into()], false, false).expect("admitted boundary");
        Request {
            work_id: "work-systemd-qual".into(),
            attempt_id: "attempt-systemd-qual".into(),
            fence: "1".into(),
            workspace_id: "ws-systemd-qual".into(),
            program: sh,
            args: vec!["-c".into(), script.into()],
            boundary,
            require_isolation: true,
            sandbox_program: Some(bwrap_path()),
            broker: None,
            timeout_secs,
            output_limit_bytes: 65_536,
        }
    }

    fn run_in_hardened_dispatcher(mode: &str, root: &Path, host_secret: &Path) -> std::process::Output {
        let exe = std::env::current_exe().expect("test executable");
        let systemd_run = systemd_tool("systemd-run");
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR");
        let mut command = Command::new(systemd_run);
        command.args([
            "--user",
            "--wait",
            "--pipe",
            "--collect",
            "--quiet",
            "--property=ProtectKernelTunables=yes",
            "--property=RestrictAddressFamilies=AF_UNIX",
        ]);
        command.arg(format!("--setenv=LATTICE_TEST_DISPATCHER_MODE={mode}"));
        command.arg(format!("--setenv=LATTICE_TEST_WORKSPACE={}", root.display()));
        command.arg(format!("--setenv=LATTICE_TEST_HOST_SECRET={}", host_secret.display()));
        command.arg(format!("--setenv=LATTICE_BWRAP={}", bwrap_path()));
        command.arg(format!("--setenv=XDG_RUNTIME_DIR={runtime_dir}"));
        if let Ok(path) = std::env::var("PATH") {
            command.arg(format!("--setenv=PATH={path}"));
        }
        if let Ok(bus) = std::env::var("DBUS_SESSION_BUS_ADDRESS") {
            command.arg(format!("--setenv=DBUS_SESSION_BUS_ADDRESS={bus}"));
            command.env("DBUS_SESSION_BUS_ADDRESS", bus);
        }
        command
            .arg("--")
            .arg(exe)
            .args([
                "--exact",
                "--ignored",
                "unix_qualification::hardened_dispatcher_child_probe",
                "--nocapture",
            ])
            .env("XDG_RUNTIME_DIR", runtime_dir);
        command.output().expect("launch hardened dispatcher unit")
    }

    #[test]
    #[ignore = "invoked by hardened_dispatcher_launches_bwrap_via_systemd_and_refuses_direct_child"]
    fn hardened_dispatcher_child_probe() {
        let Ok(mode) = std::env::var("LATTICE_TEST_DISPATCHER_MODE") else {
            return;
        };
        let root = PathBuf::from(std::env::var("LATTICE_TEST_WORKSPACE").expect("workspace"));
        let host_secret = PathBuf::from(std::env::var("LATTICE_TEST_HOST_SECRET").expect("host secret"));
        let script = format!(
            "if [ -e /etc/passwd ] || [ -e /workspace/.git ] || [ -e '{}' ]; then exit 7; fi; printf new > src/main.rs",
            host_secret.display(),
        );
        let request = isolated_request(&root, &script, 30);
        let (direct_prog, direct_args) = candidate_worker::sandbox::command(&request).expect("direct bwrap command");

        match mode.as_str() {
            "systemd" => {
                let launcher = user_launcher();
                let plan =
                    candidate_worker::sandbox::systemd::plan_launch(candidate_worker::sandbox::systemd::PlanInput {
                        config: launcher.config(),
                        process: &lattice::agent_node::ProcessRequest {
                            program: direct_prog.clone(),
                            args: direct_args.clone(),
                            cwd: Some(root.to_string_lossy().into_owned()),
                            env: std::collections::BTreeMap::new(),
                            stdin: Vec::new(),
                            timeout_secs: request.timeout_secs,
                            output_limit_bytes: request.output_limit_bytes,
                        },
                        teardown_grace_millis: RuntimeConfig::default().limits.teardown_grace_millis,
                        unit_token: "0123456789abcdef",
                    })
                    .expect("plan");
                let sep = plan.launch_args.iter().position(|arg| arg == "--").expect("separator");
                let mut expected = vec![direct_prog];
                expected.extend(direct_args);
                assert_eq!(&plan.launch_args[sep + 1..], expected.as_slice());

                let outcome = candidate_worker::run_with_launcher(
                    &request,
                    &RuntimeConfig::default(),
                    &AtomicBool::new(false),
                    &launcher,
                )
                .expect("systemd worker run");
                let result = AttemptResult::from_outcome(&outcome).with_compatibility(&request);
                result.validate().expect("compatibility validates");
                let Outcome::Produced { receipt, .. } = outcome else {
                    panic!("expected Produced inside hardened dispatcher via SystemdLauncher, got {outcome:?}");
                };
                assert_eq!(receipt.changed_paths, vec!["src/main.rs"]);
                assert_eq!(std::fs::read(root.join("src/main.rs")).expect("updated"), b"new");
            }
            "direct" => {
                let outcome = candidate_worker::run(&request, &RuntimeConfig::default(), &AtomicBool::new(false))
                    .expect("direct worker run");
                let Outcome::Failed { stderr, .. } = outcome else {
                    panic!("hardened dispatcher MUST refuse direct-child bwrap launch, got {outcome:?}");
                };
                let stderr_text = String::from_utf8_lossy(&stderr);
                assert!(
                    stderr_text.contains("NETLINK_ROUTE")
                        || stderr_text.contains("Address family not supported")
                        || stderr_text.contains("Operation not permitted"),
                    "expected bwrap refusal diagnostic under hardened dispatcher, got: {stderr_text}"
                );
                assert_eq!(std::fs::read(root.join("src/main.rs")).expect("unchanged"), b"old");
            }
            other => panic!("unexpected mode: {other}"),
        }
    }

    #[test]
    #[ignore = "requires systemd user manager and LATTICE_BWRAP"]
    fn hardened_dispatcher_launches_bwrap_via_systemd_and_refuses_direct_child() {
        let directory = tempfile::tempdir().expect("tempdir");
        let root = directory.path().join("workspace");
        let shared = tempfile::tempdir().expect("host secret dir");
        let host_secret = shared.path().join("publisher-token");
        std::fs::write(&host_secret, b"host-secret").expect("host secret");

        // Negative test first: hardened dispatcher MUST refuse direct-child bwrap.
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        std::fs::write(root.join("src/main.rs"), b"old").expect("base");
        std::fs::write(root.join(".git"), format!("gitdir: {}", shared.path().display())).expect(".git");
        let negative = run_in_hardened_dispatcher("direct", &root, &host_secret);
        assert!(
            negative.status.success(),
            "negative hardened-dispatcher probe failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&negative.stdout),
            String::from_utf8_lossy(&negative.stderr),
        );
        assert_eq!(std::fs::read(root.join("src/main.rs")).expect("still old"), b"old");

        // Positive test: same hardened dispatcher succeeds via SystemdLauncher with identical isolation.
        let positive = run_in_hardened_dispatcher("systemd", &root, &host_secret);
        assert!(
            positive.status.success(),
            "positive hardened-dispatcher probe failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&positive.stdout),
            String::from_utf8_lossy(&positive.stderr),
        );
        assert_eq!(std::fs::read(root.join("src/main.rs")).expect("updated"), b"new");
        assert_eq!(std::fs::read(&host_secret).expect("host secret intact"), b"host-secret");
    }

    #[test]
    #[ignore = "requires systemd user manager and LATTICE_BWRAP"]
    fn real_systemd_launcher_enforces_cancellation_and_deadline_with_bwrap() {
        let launcher = user_launcher();
        let sleep = which::which("sleep").expect("sleep");

        // 1. In-flight cancellation stops the transient bwrap unit before any late write.
        let cancel_dir = tempfile::tempdir().expect("cancel dir");
        let cancel_root = cancel_dir.path();
        std::fs::create_dir_all(cancel_root.join("src")).expect("src dir");
        std::fs::write(cancel_root.join("src/main.rs"), b"old").expect("base");
        let cancel_request =
            isolated_request(cancel_root, &format!("'{}' 1; printf leaked > src/leak", sleep.display()), 10);
        let signal = Arc::new(AtomicBool::new(false));
        let canceller = {
            let signal = Arc::clone(&signal);
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(200));
                signal.store(true, Ordering::Release);
            })
        };
        let cancel_outcome =
            candidate_worker::run_with_launcher(&cancel_request, &RuntimeConfig::default(), signal.as_ref(), &launcher)
                .expect("cancel run");
        canceller.join().expect("canceller");
        assert!(matches!(cancel_outcome, Outcome::Cancelled { .. }), "{cancel_outcome:?}");
        std::thread::sleep(Duration::from_millis(1_200));
        assert!(!cancel_root.join("src/leak").exists(), "cancelled bwrap unit leaked a write");

        // 2. Deadline stops the transient bwrap unit and reports the deadline blocker.
        let deadline_dir = tempfile::tempdir().expect("deadline dir");
        let deadline_root = deadline_dir.path();
        std::fs::create_dir_all(deadline_root.join("src")).expect("src dir");
        std::fs::write(deadline_root.join("src/main.rs"), b"old").expect("base");
        let deadline_request =
            isolated_request(deadline_root, &format!("'{}' 2; printf leaked > src/leak", sleep.display()), 1);
        let deadline_outcome = candidate_worker::run_with_launcher(
            &deadline_request,
            &RuntimeConfig::default(),
            &AtomicBool::new(false),
            &launcher,
        )
        .expect("deadline run");
        let Outcome::Failed { blocker, .. } = deadline_outcome else {
            panic!("expected deadline Failed outcome, got {deadline_outcome:?}");
        };
        assert!(blocker.contains("deadline"), "unexpected blocker: {blocker}");
        std::thread::sleep(Duration::from_millis(1_500));
        assert!(!deadline_root.join("src/leak").exists(), "timed-out bwrap unit leaked a write");
    }
}
