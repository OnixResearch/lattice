use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::os::fd::AsRawFd;

pub(super) const ERROR_MARKER: &str = "lattice-injected-http-400";
const ACCEPT_TIMEOUT_MS: libc::c_int = 60_000;
const SOCKET_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
const HOLD_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(90);

pub(super) struct Endpoint {
    address: std::net::SocketAddr,
    release: Option<std::sync::mpsc::SyncSender<()>>,
    worker: Option<std::thread::JoinHandle<Result<serde_json::Value, String>>>,
}

impl Endpoint {
    pub(super) fn start(cancellation: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("local fault endpoint");
        listener.set_nonblocking(true).expect("bounded listener");
        let address = listener.local_addr().expect("local endpoint address");
        let (release, held) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || serve(listener, cancellation, held));
        Self {
            address,
            release: Some(release),
            worker: Some(worker),
        }
    }

    pub(super) fn url(&self) -> String {
        format!("http://{}/v1/chat/completions", self.address)
    }

    pub(super) fn finish(mut self) -> serde_json::Value {
        drop(self.release.take());
        self.worker
            .take()
            .expect("server thread")
            .join()
            .expect("server joins")
            .expect("real admitted request")
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        drop(self.release.take());
        if let Some(worker) = self.worker.take() {
            // Wake an accept on panic/early worker failure; all socket reads and
            // the held-response receiver also have finite deadlines.
            drop(std::net::TcpStream::connect_timeout(&self.address, SOCKET_TIMEOUT));
            assert!(worker.join().is_ok(), "fault server thread panicked during cleanup");
        }
    }
}

fn serve(
    listener: std::net::TcpListener,
    cancellation: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    held: std::sync::mpsc::Receiver<()>,
) -> Result<serde_json::Value, String> {
    let mut stream = accept(&listener).map_err(|error| error.to_string())?;
    stream.set_read_timeout(Some(SOCKET_TIMEOUT)).map_err(|error| error.to_string())?;
    stream.set_write_timeout(Some(SOCKET_TIMEOUT)).map_err(|error| error.to_string())?;
    let request = read_request(&mut stream).map_err(|error| error.to_string())?;
    if let Some(signal) = cancellation {
        // This is the only writer of the cancellation flag: a complete brokered
        // request, not a timer or OMP-looking stdout, establishes that OMP ran.
        signal.store(true, std::sync::atomic::Ordering::Release);
        match held.recv_timeout(HOLD_TIMEOUT) {
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
            other => return Err(format!("held endpoint was not stopped after worker return: {other:?}")),
        }
    } else {
        respond_bad_request(&mut stream).map_err(|error| error.to_string())?;
    }
    drop(stream);
    Ok(request)
}

fn accept(listener: &std::net::TcpListener) -> std::io::Result<std::net::TcpStream> {
    let mut descriptor = libc::pollfd {
        fd: listener.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: the single pollfd is live and writable for the duration of poll;
    // its listener descriptor remains owned by this thread until accept returns.
    let ready = unsafe { libc::poll(&mut descriptor, 1, ACCEPT_TIMEOUT_MS) };
    if ready < 0 {
        return Err(std::io::Error::last_os_error());
    }
    if ready == 0 {
        return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "OMP never reached local endpoint"));
    }
    listener.accept().map(|(stream, _)| stream)
}

fn read_request(stream: &mut std::net::TcpStream) -> std::io::Result<serde_json::Value> {
    let mut reader = std::io::BufReader::new(stream);
    let mut remaining_bytes = crate::http::MAX_HEADER_BYTES;
    let first = header_line(&mut reader, &mut remaining_bytes)?;
    if first != "POST /v1/chat/completions HTTP/1.1\r\n" {
        return Err(invalid("unexpected upstream request target"));
    }
    let mut length_bytes = None;
    for _ in 0..crate::http::MAX_HEADER_COUNT {
        let line = header_line(&mut reader, &mut remaining_bytes)?;
        if line == "\r\n" {
            return read_body(&mut reader, length_bytes);
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            if length_bytes.is_some() {
                return Err(invalid("duplicate upstream content length"));
            }
            length_bytes = Some(value.trim().parse::<usize>().map_err(|_| invalid("invalid upstream body length"))?);
        }
    }
    Err(invalid("too many upstream headers"))
}

fn header_line(reader: &mut impl BufRead, remaining_bytes: &mut usize) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    let limit_bytes = u64::try_from(*remaining_bytes).map_err(|_| invalid("header bound"))?;
    Read::by_ref(reader).take(limit_bytes).read_until(b'\n', &mut bytes)?;
    *remaining_bytes = remaining_bytes.checked_sub(bytes.len()).ok_or_else(|| invalid("header bound"))?;
    if !bytes.ends_with(b"\r\n") {
        return Err(invalid("incomplete upstream header"));
    }
    String::from_utf8(bytes).map_err(|_| invalid("header UTF-8"))
}

fn read_body(reader: &mut impl Read, length_bytes: Option<usize>) -> std::io::Result<serde_json::Value> {
    let length_bytes = length_bytes.ok_or_else(|| invalid("missing upstream body length"))?;
    if length_bytes > crate::http::MAX_REQUEST_BODY_BYTES {
        return Err(invalid("upstream body exceeds broker bound"));
    }
    let mut body = vec![0; length_bytes];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body).map_err(|_| invalid("upstream body JSON"))
}

fn respond_bad_request(stream: &mut std::net::TcpStream) -> std::io::Result<()> {
    let body = serde_json::json!({
        "error": { "message": ERROR_MARKER, "type": "invalid_request_error", "code": "invalid_request" }
    })
    .to_string();
    let header = format!(
        "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body.as_bytes())
}

fn invalid(reason: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, reason)
}
