//! The gateway's single upstream call: the host-only credential goes on one
//! bounded HTTP request, is zeroized afterwards, and a response that echoes
//! it is withheld.

impl<P: super::super::authority::AuthorityPort> super::Server<'_, P> {
    /// Exactly one upstream call with the host-only credential.
    pub(super) fn call(&self, body: String, timeout_secs: u64) -> (super::super::admission::Observed, String) {
        let mut request = crate::http::HttpRequestPlan {
            method: "POST".into(),
            url: self.endpoint.clone(),
            headers: [("content-type".into(), "application/json".into())].into(),
            body: Some(body),
            timeout_secs,
        };
        if let Some(value) = &self.credential {
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
        match response {
            Ok(value) => {
                let is_echoed =
                    self.credential.as_ref().is_some_and(|secret| value.body.contains(secret.expose_for_execution()));
                if is_echoed {
                    (super::super::admission::Observed::Withheld, String::new())
                } else {
                    (super::super::admission::Observed::Response { status: value.status }, value.body)
                }
            }
            Err(crate::http::HttpError::Timeout(_)) => (super::super::admission::Observed::TimedOut, String::new()),
            Err(crate::http::HttpError::Cancelled) => (super::super::admission::Observed::Cancelled, String::new()),
            Err(crate::http::HttpError::InvalidRequest(_)) => {
                (super::super::admission::Observed::NotSent, String::new())
            }
            Err(crate::http::HttpError::LimitExceeded(_)) => {
                (super::super::admission::Observed::Withheld, String::new())
            }
            Err(crate::http::HttpError::Transport(_)) => {
                (super::super::admission::Observed::TransportFailed, String::new())
            }
        }
    }
}
