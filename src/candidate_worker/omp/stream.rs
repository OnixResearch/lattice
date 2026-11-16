pub(super) struct Progress {
    event_count: u64,
    terminal: Option<super::Terminal>,
    is_active: bool,
    last_stop: Option<String>,
}

impl Progress {
    pub(super) fn new() -> Self {
        Self {
            event_count: 0,
            terminal: None,
            is_active: false,
            last_stop: None,
        }
    }

    pub(super) fn observe(&mut self, event: &serde_json::Value) -> Result<(), String> {
        let kind = event["type"].as_str().ok_or_else(reject)?;
        self.event_count = self.event_count.checked_add(1).ok_or_else(reject)?;
        match kind {
            "session" => return Err(reject()),
            "agent_start" => {
                self.is_active = true;
                self.terminal = None;
                self.last_stop = None;
            }
            "message_end" if event["message"]["role"] == "assistant" => {
                self.last_stop = event["message"]["stopReason"].as_str().map(str::to_string);
            }
            "agent_end" => self.end(event)?,
            _ => {}
        }
        Ok(())
    }

    fn end(&mut self, event: &serde_json::Value) -> Result<(), String> {
        if !self.is_active {
            return Err(reject());
        }
        let messages = event["messages"].as_array().ok_or_else(reject)?;
        let message = messages.iter().rev().find(|message| message["role"] == "assistant").ok_or_else(reject)?;
        let reason = super::stop_reason(message)?;
        if self.last_stop.as_deref() != message["stopReason"].as_str() {
            return Err(reject());
        }
        self.terminal = Some(reason);
        self.is_active = false;
        Ok(())
    }

    pub(super) fn finish(self) -> Result<(u64, super::Terminal), String> {
        if self.is_active {
            return Err(reject());
        }
        Ok((self.event_count, self.terminal.ok_or_else(reject)?))
    }
}

pub(super) fn reject() -> String {
    "OMP JSONL stream is malformed, truncated or incomplete".to_string()
}
