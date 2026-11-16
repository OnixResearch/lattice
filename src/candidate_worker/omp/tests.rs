use super::*;

fn events(reason: &str) -> Vec<u8> {
    format!("{{\"type\":\"session\",\"version\":3,\"id\":\"session-1\",\"cwd\":\"/workspace\"}}\n{{\"type\":\"agent_start\"}}\n{{\"type\":\"message_update\",\"assistantMessageEvent\":{{\"type\":\"text_delta\",\"delta\":\"done\"}}}}\n{{\"type\":\"message_end\",\"message\":{{\"role\":\"assistant\",\"stopReason\":\"{reason}\"}}}}\n{{\"type\":\"agent_end\",\"messages\":[{{\"role\":\"assistant\",\"stopReason\":\"{reason}\"}}]}}\n").into_bytes()
}

#[test]
fn normalizes_pinned_terminal_outcomes_without_inventing_usage() {
    for (reason, terminal) in [
        ("stop", Terminal::Completed),
        ("error", Terminal::Failed),
        ("aborted", Terminal::Aborted),
    ] {
        let raw = events(reason);
        let observation = normalize_jsonl(&raw, 4096, "/workspace").expect("bounded stream");
        assert_eq!(observation.terminal, terminal);
        assert_eq!(observation.raw_events_blake3, crate::pure_inet_profile::blake3_identity(&raw));
        assert!(!observation.usage_complete);
    }
}

// r[verify lattice.candidate_worker.omp_jsonl.scenario.invalid]
#[test]
fn rejects_malformed_truncated_crossed_and_excessive_events() {
    let raw = events("stop");
    assert!(normalize_jsonl(&raw[..raw.len() - 1], 4096, "/workspace").is_err());
    assert!(normalize_jsonl(&raw, raw.len() - 1, "/workspace").is_err());
    assert!(normalize_jsonl(&raw, 4096, "/other").is_err());
    assert!(normalize_jsonl(b"{}\n", 4096, "/workspace").is_err());
    assert!(normalize_jsonl(&events("toolUse"), 4096, "/workspace").is_err());
    let text = String::from_utf8(raw).expect("text");
    assert!(normalize_jsonl(text.replace("agent_end", "not_terminal").as_bytes(), 4096, "/workspace").is_err());
    assert!(normalize_jsonl(text.replace("message_update", "session").as_bytes(), 4096, "/workspace").is_err());
}

#[test]
fn intermediate_tool_use_is_not_mistaken_for_terminal_failure() {
    let raw = concat!(
        "{\"type\":\"session\",\"version\":3,\"id\":\"s\",\"cwd\":\"/workspace\"}\n",
        "{\"type\":\"agent_start\"}\n",
        "{\"type\":\"message_end\",\"message\":{\"role\":\"assistant\",\"stopReason\":\"toolUse\"}}\n",
        "{\"type\":\"message_end\",\"message\":{\"role\":\"toolResult\"}}\n",
        "{\"type\":\"message_end\",\"message\":{\"role\":\"assistant\",\"stopReason\":\"stop\"}}\n",
        "{\"type\":\"agent_end\",\"messages\":[{\"role\":\"assistant\",\"stopReason\":\"toolUse\"},{\"role\":\"toolResult\"},{\"role\":\"assistant\",\"stopReason\":\"stop\"}]}\n",
    );
    assert_eq!(
        normalize_jsonl(raw.as_bytes(), 4096, "/workspace").expect("complete tool turn").terminal,
        Terminal::Completed
    );
}

#[test]
fn header_directory_admission_and_zero_byte_bound_remain_fail_closed() {
    let raw = events("stop");
    assert!(normalize_jsonl(&raw, 0, "/workspace").is_err());
    let text = String::from_utf8(raw).expect("JSONL text");
    for directories in ["[\"/outside\"]", "{}", "null"] {
        let crossed = text.replacen(
            "\"cwd\":\"/workspace\"",
            &format!("\"cwd\":\"/workspace\",\"additionalDirectories\":{directories}"),
            1,
        );
        assert!(normalize_jsonl(crossed.as_bytes(), 4096, "/workspace").is_err());
    }
    let admitted = text.replacen("\"cwd\":\"/workspace\"", "\"cwd\":\"/workspace\",\"additionalDirectories\":[]", 1);
    assert!(normalize_jsonl(admitted.as_bytes(), 4096, "/workspace").is_ok());
}
