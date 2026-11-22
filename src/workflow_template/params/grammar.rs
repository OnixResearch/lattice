//! The strict simple-command grammar for shell nodes that receive parameter values. Pure.
//!
//! A bound command is words separated by spaces. Each word is either a literal over a safe
//! alphabet or exactly `"${KEY}"` for a key bound to that node. The first word is a literal, so
//! a value never takes the command-name position of the bound command, and the shell never
//! parses a value as shell syntax. The deny lists below are defense in depth against the
//! shell's own argument-executing builtins and common wrappers; they are not a guarantee that
//! no program the command runs will execute an argument.

/// Shell names, matched by basename, ignoring ASCII case and a trailing version suffix
/// (`bash5`, `ksh93`, `zsh-5.9`), against every literal word, so a wrapper cannot hand a value
/// to a shell named as a separate word (`flock l bash "${KEY}"`).
const SHELLS: &[&str] = &[
    "sh", "bash", "dash", "zsh", "ksh", "fish", "ash", "mksh", "lksh", "rbash", "rksh", "oksh", "yash", "posh", "csh",
    "tcsh", "pwsh", "nu", "elvish", "xonsh", "hush", "osh", "ysh", "sash",
];

/// Shell builtins, precommand modifiers, and wrappers that run, wrap, or re-parse an argument
/// as a command. Matched by basename, ignoring ASCII case, against the first word only, so
/// ordinary arguments such as `systemctl enable` or `kubectl get jobs` stay valid.
const DENIED_PROGRAMS: &[&str] = &[
    // Shell builtins that execute or load an argument.
    "eval",
    "exec",
    "command",
    "builtin",
    "source",
    ".",
    "trap",
    "jobs",
    "coproc",
    "mapfile",
    "readarray",
    "enable",
    // Builtins that evaluate an argument arithmetically or as a declaration.
    "let",
    "declare",
    "typeset",
    "local",
    // Precautionary builtins and zsh precommand modifiers.
    "fc",
    "bind",
    "complete",
    "compgen",
    "noglob",
    "nocorrect",
    "-",
    // Wrappers that run an argument as a program.
    "env",
    "tmux",
    "screen",
    "hyperfine",
    "xargs",
    "nohup",
    "timeout",
    "nice",
    "ionice",
    "setsid",
    "stdbuf",
    "taskset",
    "chrt",
    "sudo",
    "doas",
    "su",
    "runuser",
    "busybox",
    "time",
    "watch",
    "parallel",
    "flock",
    "prlimit",
    "setpriv",
    "unshare",
    "nsenter",
    "strace",
    "ltrace",
    "script",
    "systemd-run",
    "firejail",
    "bwrap",
    "chroot",
    "nix-shell",
];

/// Literal words that make common programs execute an argument.
const DENIED_WORDS: &[&str] = &["-exec", "-execdir", "-ok", "-okdir", "-c"];

/// Whether `command` is one simple command in the strict grammar for the bound `keys`.
pub(super) fn is_simple_bound_command(command: &str, keys: &[&str]) -> bool {
    let mut words = command.split(' ').filter(|word| !word.is_empty());
    let Some(program) = words.next() else {
        return false;
    };
    let is_plain_program = is_literal(program) && !program.contains('=');
    let is_denied = names_shell(program) || names_any(program, DENIED_PROGRAMS);
    if !is_plain_program || is_denied {
        return false;
    }
    words.all(|word| {
        is_reference(word, keys) || (is_literal(word) && !DENIED_WORDS.contains(&word) && !names_shell(word))
    })
}

fn is_reference(word: &str, keys: &[&str]) -> bool {
    word.strip_prefix("\"${")
        .and_then(|rest| rest.strip_suffix("}\""))
        .is_some_and(|key| keys.contains(&key))
}

/// ASCII letters, digits, and `-`, `_`, `.`, `/`, `=`, `:`, `,`, `+`, `@`, `%` only: no expansion,
/// quoting, escape, glob, redirection, grouping, comment, or control character.
fn is_literal(word: &str) -> bool {
    !word.is_empty()
        && word.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'-' | b'_' | b'.' | b'/' | b'=' | b':' | b',' | b'+' | b'@' | b'%')
        })
}

/// Basename match ignoring ASCII case, because case-insensitive filesystems resolve `ENV` to `env`.
fn names_any(word: &str, names: &[&str]) -> bool {
    let basename = word.rsplit('/').next().unwrap_or(word);
    names.iter().any(|name| name.eq_ignore_ascii_case(basename))
}

/// Whether a word names a shell by basename, ignoring ASCII case, a trailing `-static`, and a
/// trailing version suffix of digits, `.`, `-`, and `_` (`bash5`, `ksh93`, `zsh-5.9`,
/// `bash-static`).
fn names_shell(word: &str) -> bool {
    let basename = word.rsplit('/').next().unwrap_or(word);
    let unpacked = strip_static_suffix(basename);
    let unversioned =
        unpacked.trim_end_matches(|character: char| character.is_ascii_digit() || matches!(character, '.' | '-' | '_'));
    !unversioned.is_empty() && names_any(unversioned, SHELLS)
}

/// Cut a trailing `-static`, compared ignoring ASCII case (`bash-static`, `Bash-Static`).
fn strip_static_suffix(basename: &str) -> &str {
    const SUFFIX: &str = "-static";
    match basename.len().checked_sub(SUFFIX.len()) {
        Some(at) if basename.is_char_boundary(at) && basename[at..].eq_ignore_ascii_case(SUFFIX) => &basename[..at],
        _ => basename,
    }
}

#[cfg(test)]
mod tests {
    const KEYS: &[&str] = &["LATTICE_PARAM_ROOT", "LATTICE_PARAM_TOOL"];

    fn accepts(command: &str) -> bool {
        super::is_simple_bound_command(command, KEYS)
    }

    #[test]
    fn a_simple_command_with_whole_word_references_passes() {
        for command in [
            "lattice evaluation catalog --root \"${LATTICE_PARAM_ROOT}\" --output json",
            "/usr/bin/tool --flag=on \"${LATTICE_PARAM_ROOT}\" \"${LATTICE_PARAM_TOOL}\"",
            "tar -xf - -C \"${LATTICE_PARAM_ROOT}\"",
            "tool . \"${LATTICE_PARAM_ROOT}\"",
            "git -C \"${LATTICE_PARAM_ROOT}\" checkout \"${LATTICE_PARAM_TOOL}\"",
            "find . -name \"${LATTICE_PARAM_TOOL}\"",
            "systemctl enable \"${LATTICE_PARAM_TOOL}\"",
            "kubectl get jobs \"${LATTICE_PARAM_TOOL}\"",
        ] {
            assert!(accepts(command), "{command:?} passes");
        }
    }

    #[test]
    fn a_value_never_takes_the_command_name_position() {
        assert!(!accepts("echo \"$(\"${LATTICE_PARAM_TOOL}\" output.txt)\""));
        assert!(!accepts("\"${LATTICE_PARAM_TOOL}\" output.txt"));
        assert!(!accepts("LD_PRELOAD=x \"${LATTICE_PARAM_TOOL}\""));
        assert!(!accepts("A=1 tool \"${LATTICE_PARAM_ROOT}\""));
    }

    #[test]
    fn shell_builtins_that_execute_an_argument_are_denied() {
        for command in [
            "trap \"${LATTICE_PARAM_TOOL}\" EXIT",
            "jobs -x \"${LATTICE_PARAM_TOOL}\" out",
            "coproc \"${LATTICE_PARAM_TOOL}\" out",
            "mapfile -c1 -C \"${LATTICE_PARAM_TOOL}\" lines",
            "readarray -c1 -C \"${LATTICE_PARAM_TOOL}\" lines",
            "enable -f \"${LATTICE_PARAM_TOOL}\" x",
            "noglob \"${LATTICE_PARAM_TOOL}\"",
            "- \"${LATTICE_PARAM_TOOL}\"",
        ] {
            assert!(!accepts(command), "{command:?} is denied");
        }
    }

    #[test]
    fn shells_are_matched_in_any_case_and_any_word() {
        assert!(!accepts("ENV \"${LATTICE_PARAM_TOOL}\""));
        assert!(!accepts("/usr/bin/Bash -e \"${LATTICE_PARAM_TOOL}\""));
        assert!(!accepts("tool sh \"${LATTICE_PARAM_TOOL}\""));
        assert!(!accepts("tool l /bin/BASH \"${LATTICE_PARAM_TOOL}\""));
        assert!(!accepts("flock l bash \"${LATTICE_PARAM_TOOL}\""));
        assert!(!accepts("prlimit sh -ec \"${LATTICE_PARAM_TOOL}\""));
        assert!(!accepts("find . -okdir \"${LATTICE_PARAM_TOOL}\""));
        for versioned in [
            "bash5",
            "ksh93",
            "zsh-5.9",
            "/usr/bin/BASH-5.2",
            "lksh",
            "bash-static",
            "zsh-static",
            "mksh-static",
            "bash-5.2-static",
            "Bash-Static",
            "Zsh-Static",
            "hush",
            "osh",
            "ysh",
            "rksh",
            "sash",
        ] {
            assert!(!accepts(&format!("{versioned} \"${{LATTICE_PARAM_TOOL}}\"")), "{versioned} first word");
            assert!(!accepts(&format!("tool {versioned} \"${{LATTICE_PARAM_TOOL}}\"")), "{versioned} later word");
        }
        for evaluating in ["let", "declare", "typeset", "local", "tmux", "screen", "hyperfine"] {
            assert!(!accepts(&format!("{evaluating} x \"${{LATTICE_PARAM_TOOL}}\"")), "{evaluating} is denied");
        }
        assert!(accepts("tool v2 1.0 \"${LATTICE_PARAM_TOOL}\""), "version-like arguments are not shells");
    }

    #[test]
    fn every_forbidden_construct_is_rejected() {
        for construct in [
            "$LATTICE_PARAM_ROOT",
            "${LATTICE_PARAM_ROOT}",
            "\"$LATTICE_PARAM_ROOT\"",
            "\"${LATTICE_PARAM_ROOT:-/}\"",
            "\"${LATTICE_PARAM_OTHER}\"",
            "x\"${LATTICE_PARAM_ROOT}\"",
            "{{node:A:stdout}}",
            "$(id)",
            "`id`",
            "$((1+1))",
            "'quoted'",
            "\"quoted\"",
            "a\\b",
            "a;b",
            "a|b",
            "a&b",
            "a&&b",
            "<in",
            ">out",
            "<<EOF",
            "(sub)",
            "{a,b}",
            "glob*",
            "a?",
            "[ab]",
            "~",
            "#comment",
            "a\nb",
            "a\tb",
            "a\0b",
            "-exec",
            "-execdir",
            "-ok",
            "-okdir",
            "-c",
        ] {
            assert!(!accepts(&format!("tool {construct}")), "{construct:?} is rejected");
        }
        assert!(!accepts(""));
        assert!(!accepts("   "));
    }

    #[test]
    fn argument_executing_programs_are_denied_by_basename() {
        for program in super::SHELLS.iter().chain(super::DENIED_PROGRAMS) {
            assert!(!accepts(&format!("{program} \"${{LATTICE_PARAM_ROOT}}\"")), "{program} is denied");
            assert!(!accepts(&format!("/usr/bin/{program} \"${{LATTICE_PARAM_ROOT}}\"")), "{program} path is denied");
        }
        for shell in super::SHELLS {
            assert!(!accepts(&format!("tool {shell} \"${{LATTICE_PARAM_ROOT}}\"")), "{shell} argument is denied");
        }
        for program in super::DENIED_PROGRAMS {
            assert!(accepts(&format!("tool {program} \"${{LATTICE_PARAM_ROOT}}\"")), "{program} argument is allowed");
        }
    }
}
