//! Pure release scratch layout; filesystem admission belongs to the runner shell.
use super::model::{
    CLANKERS_TARGET_DIR, DEFAULT_TARGET_DIR, PACKAGE_TARGET_DIR, VENDORED_UCAN_CHECK_BINARY,
    VENDORED_UCAN_TEST_BINARY,
};

pub(super) const ENV: &str = "RELEASE_VERIFY_SCRATCH_DIR";

pub(super) fn validate(root: &str) -> Result<(), &'static str> {
    if !root.starts_with('/')
        || root == "/"
        || root.chars().any(char::is_control)
        || root[1..]
            .split('/')
            .any(|part| matches!(part, "" | "." | ".."))
    {
        return Err("release scratch directory must be an absolute normalized non-root path");
    }
    Ok(())
}

pub(super) fn relocate(root: Option<&str>, value: &str) -> String {
    match root {
        Some(root)
            if matches!(
                value,
                DEFAULT_TARGET_DIR
                    | PACKAGE_TARGET_DIR
                    | CLANKERS_TARGET_DIR
                    | VENDORED_UCAN_TEST_BINARY
                    | VENDORED_UCAN_CHECK_BINARY
            ) =>
        {
            format!("{root}/{}", &value[5..])
        }
        _ => value.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_ambiguous_roots() {
        for root in [
            "", "/", "relative", "/a/../b", "/a/./b", "/a//b", "/a/", "/a\n", "/a\0",
        ] {
            assert!(validate(root).is_err(), "{root:?}");
        }
        assert!(validate("/home/with spaces/scratch").is_ok());
    }

    #[test]
    fn relocates_only_exact_owned_paths_and_preserves_default() {
        for value in [
            DEFAULT_TARGET_DIR,
            PACKAGE_TARGET_DIR,
            CLANKERS_TARGET_DIR,
            VENDORED_UCAN_TEST_BINARY,
            VENDORED_UCAN_CHECK_BINARY,
        ] {
            assert_eq!(relocate(None, value), value);
            assert_eq!(
                relocate(Some("/scratch"), value),
                format!("/scratch/{}", &value[5..])
            );
        }
        for value in [
            "rustc",
            "--test",
            "/tmp/foreign",
            "/tmp/basalt-release-target/child",
        ] {
            assert_eq!(relocate(Some("/scratch"), value), value);
        }
    }

    #[test]
    fn compiler_outputs_and_programs_resolve_to_the_same_paths() {
        let steps = crate::plan::steps();
        for binary in [VENDORED_UCAN_TEST_BINARY, VENDORED_UCAN_CHECK_BINARY] {
            assert!(
                steps
                    .iter()
                    .any(|step| step.args.windows(2).any(|args| args == ["-o", binary]))
            );
            assert!(steps.iter().any(|step| step.program == binary));
        }
        let paths = [DEFAULT_TARGET_DIR, PACKAGE_TARGET_DIR, CLANKERS_TARGET_DIR]
            .map(|value| relocate(Some("/scratch"), value));
        assert!(paths[0] != paths[1] && paths[1] != paths[2] && paths[0] != paths[2]);
    }
}
