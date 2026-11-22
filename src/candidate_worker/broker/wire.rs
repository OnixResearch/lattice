use std::io::BufRead;
use std::io::Read;
use std::io::Write;

const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(500);
const REQUEST_LINE: &str = "POST /v1/chat/completions HTTP/1.1\r\n";
const HOST: &str = "127.0.0.1:41001";

pub(super) struct Handler<'a> {
    pub profile: &'a super::Profile,
    pub credentials: &'a dyn crate::credentials::CredentialProvider,
    pub gate: &'a mut dyn super::ModelRequestGate,
    pub cancellation: &'a std::sync::atomic::AtomicBool,
    pub stopped: &'a std::sync::atomic::AtomicBool,
    pub deadline: std::time::Instant,
    pub requests: u32,
    /// Request evidence that could not be retained; it fails the broker.
    pub failure: Option<String>,
    /// The host gateway of a forwarding (VM guest) broker; `None` calls the
    /// profile's endpoint directly.
    pub forwarding: Option<super::forwarded::Target>,
    /// The forwarded index of this handler's first request.
    pub first_request_index: u32,
}

impl Handler<'_> {
    pub(super) fn exchange(&mut self, mut stream: std::os::unix::net::UnixStream) -> Result<(), String> {
        stream.set_read_timeout(Some(READ_TIMEOUT)).map_err(|_| "broker input deadline is unavailable")?;
        stream.set_write_timeout(Some(READ_TIMEOUT)).map_err(|_| "broker output deadline is unavailable")?;
        let previous_requests = self.requests;
        let response = read_request(&mut stream).and_then(|body| self.forward(body));
        if let Some(failure) = self.failure.take() {
            return Err(format!("model request evidence was not retained; upstream outcome may be unknown: {failure}"));
        }
        let (status, content_type, body) = match &response {
            Ok(value) => (
                value.status,
                value.headers.get("content-type").map_or("application/json", String::as_str),
                value.body.as_str(),
            ),
            Err(()) => (403, "application/json", "{\"error\":\"model operation unavailable or not admitted\"}"),
        };
        let content_type = if content_type.starts_with("text/event-stream") {
            "text/event-stream"
        } else {
            "application/json"
        };
        debug_assert!(body.len() <= crate::http::MAX_RESPONSE_BODY_BYTES);
        debug_assert!(!content_type.contains(['\r', '\n']));
        let header = format!(
            "HTTP/1.1 {status} Broker\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",
            body.len()
        );
        let delivered = stream.write_all(header.as_bytes()).and_then(|()| stream.write_all(body.as_bytes()));
        // Readiness probes and refused clients may disconnect without any model
        // effect. An admitted call with lost response is a broker failure, not
        // evidence that the upstream operation did not execute.
        if delivered.is_err() && self.requests != previous_requests {
            return Err("admitted model response was not delivered; upstream outcome may be unknown".into());
        }
        Ok(())
    }

    pub(super) fn is_current(&self) -> bool {
        !self.cancellation.load(std::sync::atomic::Ordering::Acquire)
            && !self.stopped.load(std::sync::atomic::Ordering::Acquire)
            && crate::operation::shell::SystemClock.observe_monotonic_instant() < self.deadline
            && super::admission::now_ms().is_ok_and(|now| now < self.profile.expires_at_ms)
    }

    /// Every request passes the broker's checks and the route's gate before
    /// upstream contact; the gate settles every admitted call exactly once.
    fn forward(&mut self, body: String) -> Result<crate::http::HttpResponse, ()> {
        if !self.is_current() {
            return Err(());
        }
        // An exhausted request budget refuses before any credential or gate
        // work; a served route records that refusal once.
        if self.requests >= self.profile.max_requests {
            if let Err(failure) = self.gate.exhausted() {
                self.failure = Some(failure);
            }
            return Err(());
        }
        let value: serde_json::Value = serde_json::from_str(&body).map_err(|_| ())?;
        if value.get("model").and_then(serde_json::Value::as_str) != Some(self.profile.model.as_str()) {
            return Err(());
        }
        // Re-encode the admitted object so duplicate keys cannot have different
        // authorization and upstream interpretations.
        let body = serde_json::to_string(&value).map_err(|_| ())?;
        let timeout_secs = self.timeout_secs()?;
        // A forwarding broker never resolves a credential.
        let forwarding = self.forwarding_route()?;
        let credential = self
            .profile
            .credential
            .as_ref()
            .map(|reference| self.credentials.resolve(reference).map_err(|_| ()))
            .transpose()?;
        let checks = crate::execution_binding::serving::gate::Checks {
            is_exact_model: true,
            is_within_budget: self.requests < self.profile.max_requests,
            is_current: self.is_current(),
        };
        // r[impl lattice.vm_profile.forwarded_upstream.scenario.request_index]
        let request_index = self.first_request_index.checked_add(self.requests).ok_or(())?;
        let ticket = self.gate.admit(checks).ok_or(())?;
        debug_assert!(checks.is_within_budget && checks.is_current, "no gate admits outside the broker checks");
        self.requests += 1;
        if let Some((target, binding)) = forwarding {
            let exchange = super::forwarded::Call {
                target: &target,
                binding: &binding,
                request_index,
                body,
                timeout: std::time::Duration::from_secs(timeout_secs),
                stopped: self.stopped,
            }
            .run();
            return self.settle_forwarded(ticket, exchange);
        }
        let mut request = crate::http::HttpRequestPlan {
            method: "POST".into(),
            url: self.profile.endpoint.clone(),
            headers: [("content-type".into(), "application/json".into())].into(),
            body: Some(body),
            timeout_secs,
        };
        if let Some(value) = &credential {
            request.headers.insert("authorization".into(), format!("Bearer {}", value.expose_for_execution()));
        }
        let response = crate::http::HttpClient::execute_bounded(
            &crate::http::NativeHttpClient,
            &request,
            &crate::http::HttpBounds::default(),
            Some(self.stopped),
        );
        for value in request.headers.values_mut() {
            zeroize::Zeroize::zeroize(value);
        }
        let is_echoed = response.as_ref().is_ok_and(|response| {
            credential.as_ref().is_some_and(|secret| response.body.contains(secret.expose_for_execution()))
        });
        self.settle(ticket, response, is_echoed)
    }

    /// Record the call; a withheld, rejected or unretained response never reaches the worker.
    fn settle(
        &mut self,
        ticket: super::GateTicket,
        response: Result<crate::http::HttpResponse, crate::http::HttpError>,
        is_echoed: bool,
    ) -> Result<crate::http::HttpResponse, ()> {
        let upstream = match &response {
            Ok(_) if is_echoed => crate::execution_binding::serving::gate::Upstream::TransportFailed,
            Ok(value) => crate::execution_binding::serving::gate::Upstream::Responded(super::serving::response_facts(
                &value.body,
            )),
            Err(crate::http::HttpError::Timeout(_)) => crate::execution_binding::serving::gate::Upstream::TimedOut,
            Err(crate::http::HttpError::Cancelled) => crate::execution_binding::serving::gate::Upstream::Cancelled,
            Err(
                crate::http::HttpError::InvalidRequest(_)
                | crate::http::HttpError::Transport(_)
                | crate::http::HttpError::LimitExceeded(_),
            ) => crate::execution_binding::serving::gate::Upstream::TransportFailed,
        };
        match self.gate.settle(ticket, upstream) {
            Ok(true) if !is_echoed => response.map_err(|_| ()),
            Ok(_) => Err(()),
            Err(failure) => {
                self.failure = Some(failure);
                Err(())
            }
        }
    }

    fn timeout_secs(&self) -> Result<u64, ()> {
        let expiry_secs =
            self.profile.expires_at_ms.checked_sub(super::admission::now_ms().map_err(|_| ())?).ok_or(())? / 1000;
        let timeout_secs = expiry_secs
            .min(
                self.deadline
                    .saturating_duration_since(crate::operation::shell::SystemClock.observe_monotonic_instant())
                    .as_secs(),
            )
            .min(crate::http::MAX_TIMEOUT_SECS);
        if timeout_secs == 0 {
            return Err(());
        }
        Ok(timeout_secs)
    }
}

fn read_request(stream: &mut std::os::unix::net::UnixStream) -> Result<String, ()> {
    let mut reader = std::io::BufReader::new(stream);
    let mut used_bytes = 0;
    if line(&mut reader, &mut used_bytes)? != REQUEST_LINE {
        return Err(());
    }
    let mut content_length_bytes = None;
    let mut has_host = false;
    let mut has_content_type = false;
    for _ in 0..crate::http::MAX_HEADER_COUNT {
        let header = line(&mut reader, &mut used_bytes)?;
        if header == "\r\n" {
            if !has_host || !has_content_type {
                return Err(());
            }
            let body_bytes = content_length_bytes.ok_or(())?;
            debug_assert!(body_bytes <= crate::http::MAX_REQUEST_BODY_BYTES);
            debug_assert!(used_bytes <= crate::http::MAX_HEADER_BYTES);
            let mut body = vec![0; body_bytes];
            reader.read_exact(&mut body).map_err(|_| ())?;
            return String::from_utf8(body).map_err(|_| ());
        }
        let (name, value) = header.strip_suffix("\r\n").ok_or(())?.split_once(':').ok_or(())?;
        let value = value.trim();
        match name {
            name if name.eq_ignore_ascii_case("host") && !has_host && value == HOST => has_host = true,
            name if name.eq_ignore_ascii_case("content-type") && !has_content_type && value == "application/json" => {
                has_content_type = true
            }
            name if name.eq_ignore_ascii_case("content-length") && content_length_bytes.is_none() => {
                if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(());
                }
                let parsed_bytes: usize = value.parse().map_err(|_| ())?;
                if parsed_bytes > crate::http::MAX_REQUEST_BODY_BYTES {
                    return Err(());
                }
                content_length_bytes = Some(parsed_bytes);
            }
            name if [
                "host",
                "content-type",
                "content-length",
                "transfer-encoding",
                "expect",
                "upgrade",
            ]
            .iter()
            .any(|forbidden| name.eq_ignore_ascii_case(forbidden)) =>
            {
                return Err(());
            }
            name if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-') => {
                return Err(());
            }
            _ => (), // No worker-supplied header is forwarded.
        }
    }
    Err(())
}

fn line(reader: &mut impl BufRead, used_bytes: &mut usize) -> Result<String, ()> {
    let remaining_bytes = crate::http::MAX_HEADER_BYTES.checked_sub(*used_bytes).ok_or(())?;
    let maximum_read_bytes = (remaining_bytes as u64).checked_add(1).ok_or(())?;
    let mut bytes = Vec::new();
    Read::by_ref(reader).take(maximum_read_bytes).read_until(b'\n', &mut bytes).map_err(|_| ())?;
    if bytes.len() > remaining_bytes || !bytes.ends_with(b"\r\n") {
        return Err(());
    }
    *used_bytes = used_bytes.checked_add(bytes.len()).ok_or(())?;
    String::from_utf8(bytes).map_err(|_| ())
}
