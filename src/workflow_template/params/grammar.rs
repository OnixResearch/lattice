//! The strict simple-command grammar for shell nodes that receive parameter values. Pure.
//!
//! A bound command is words separated by spaces. Each word is either a literal over a safe
//! alphabet or exactly `"${KEY}"` for a key bound to that node. The first word is a literal
//! program name that is not an argument-executing program, so a value can never choose the
//! program or reach shell syntax.

/// Programs that run, wrap, or re-parse their arguments as commands, matched by basename.
const DENIED_PROGRAMS: &[&str] = &[
    "sh", "bash", "dash", "zsh", "ksh", "fish", "eval", "exec", "env", "xargs", "command", "builtin", "source", ".",
    "nohup", "timeout", "nice", "ionice", "setsid", "stdbuf", "taskset", "chrt", "sudo", "doas", "su", "runuser",
    "busybox", "time", "watch", "parallel", "find", "git",
];

/// Literal words that make common programs execute an argument.
const DENIED_WORDS: &[&str] = &["-exec", "-execdir", "-ok", "-c"];

/// Whether `command` is one simple command in the strict grammar for the bound `keys`.
pub(super) fn is_simple_bound_command(command: &str, keys: &[&str]) -> bool {
    let mut words = command.split(' ').filter(|word| !word.is_empty());
    let Some(program) = words.next() else {
        return false;
    };
    if !is_literal(program) || program.contains('=') || is_denied_program(program) {
        return false;
    }
    words.all(|word| is_reference(word, keys) || (is_literal(word) && !DENIED_WORDS.contains(&word)))
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

fn is_denied_program(program: &str) -> bool {
    let basename = program.rsplit('/').next().unwrap_or(program);
    DENIED_PROGRAMS.contains(&basename)
}

#[cfg(test)]
mod tests {
    const KEYS: &[&str] = &["LATTICE_PARAM_ROOT", "LATTICE_PARAM_TOOL"];

    fn accepts(command: &str) -> bool {
        super::is_simple_bound_command(command, KEYS)
    }

    #[test]
    fn a_simple_command_with_whole_word_references_passes() {
        assert!(accepts("lattice evaluation catalog --root \"${LATTICE_PARAM_ROOT}\" --output json"));
        assert!(accepts("/usr/bin/tool --flag=on \"${LATTICE_PARAM_ROOT}\" \"${LATTICE_PARAM_TOOL}\""));
    }

    #[test]
    fn a_value_can_never_choose_the_program() {
        assert!(!accepts("echo \"$(\"${LATTICE_PARAM_TOOL}\" output.txt)\""));
        assert!(!accepts("\"${LATTICE_PARAM_TOOL}\" output.txt"));
        assert!(!accepts("LD_PRELOAD=x \"${LATTICE_PARAM_TOOL}\""));
        assert!(!accepts("A=1 tool \"${LATTICE_PARAM_ROOT}\""));
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
            "-c",
        ] {
            assert!(!accepts(&format!("tool {construct}")), "{construct:?} is rejected");
        }
        assert!(!accepts(""));
        assert!(!accepts("   "));
    }

    #[test]
    fn argument_executing_programs_are_denied_by_basename() {
        for program in super::DENIED_PROGRAMS {
            assert!(!accepts(&format!("{program} \"${{LATTICE_PARAM_ROOT}}\"")), "{program} is denied");
            assert!(!accepts(&format!("/usr/bin/{program} \"${{LATTICE_PARAM_ROOT}}\"")), "{program} path is denied");
        }
    }
}
