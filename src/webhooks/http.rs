#[derive(Debug)]
pub enum ReadError {
    Io(std::io::Error),
    Diagnostic(super::planning::RequestDiagnostic),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestHead {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    content_length_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RequestLine<'a> {
    method: &'a str,
    target: &'a str,
}

pub async fn read_raw(
    stream: &mut tokio::net::TcpStream,
    limits: super::planning::RequestLimits,
) -> Result<super::planning::RawRequest, ReadError> {
    let mut chunk = [0_u8; super::shared::HTTP_READ_CHUNK_BYTES];
    let (buffer, header_end) =
        read_header(Vec::with_capacity(super::shared::HTTP_READ_CHUNK_BYTES), stream, &mut chunk).await?;
    assert!(header_end <= buffer.len());
    assert!(buffer.len() <= super::shared::MAX_HTTP_HEADER_BYTES.saturating_add(super::shared::HTTP_READ_CHUNK_BYTES));
    let head = parse_head(&buffer[..header_end])?;

    if head.content_length_bytes > limits.max_body_bytes {
        return Err(ReadError::Diagnostic(super::planning::diagnostic(
            super::shared::PlanningCode::BodyTooLarge,
            super::shared::SAFE_BODY_TOO_LARGE_MESSAGE,
        )));
    }
    assert!(head.content_length_bytes <= limits.max_body_bytes);

    let body = read_body(stream, &buffer[header_end..], head.content_length_bytes, &mut chunk).await?;
    Ok(super::planning::RawRequest {
        method: head.method,
        target: head.target,
        headers: head.headers,
        body,
    })
}

async fn read_header(
    mut buffer: Vec<u8>,
    stream: &mut tokio::net::TcpStream,
    chunk: &mut [u8; super::shared::HTTP_READ_CHUNK_BYTES],
) -> Result<(Vec<u8>, usize), ReadError> {
    while buffer.len() <= super::shared::MAX_HTTP_HEADER_BYTES {
        if let Some(header_end) = header_end_offset(&buffer) {
            return Ok((buffer, header_end));
        }
        let bytes_read = tokio::io::AsyncReadExt::read(stream, chunk).await.map_err(ReadError::Io)?;
        if bytes_read == 0 {
            return Err(invalid_request());
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
    }
    Err(invalid_request())
}

async fn read_body(
    stream: &mut tokio::net::TcpStream,
    existing_body: &[u8],
    content_length_bytes: usize,
    chunk: &mut [u8; super::shared::HTTP_READ_CHUNK_BYTES],
) -> Result<Vec<u8>, ReadError> {
    let mut body = Vec::with_capacity(content_length_bytes);
    let initial_len = existing_body.len().min(content_length_bytes);
    body.extend_from_slice(&existing_body[..initial_len]);
    while body.len() < content_length_bytes {
        let bytes_read = tokio::io::AsyncReadExt::read(stream, chunk).await.map_err(ReadError::Io)?;
        if bytes_read == 0 {
            return Err(invalid_request());
        }
        let remaining = content_length_bytes.saturating_sub(body.len());
        let take_len = bytes_read.min(remaining);
        body.extend_from_slice(&chunk[..take_len]);
    }
    assert_eq!(body.len(), content_length_bytes);
    Ok(body)
}

fn header_end_offset(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(super::shared::HTTP_HEADER_TERMINATOR.len())
        .position(|window| window == super::shared::HTTP_HEADER_TERMINATOR)
        .map(|index| index.saturating_add(super::shared::HTTP_HEADER_TERMINATOR.len()))
}

fn parse_head(head: &[u8]) -> Result<RequestHead, ReadError> {
    let head = utf8_head(head)?;
    let mut lines = head.split(super::shared::HTTP_LINE_TERMINATOR);
    let request_line = request_line(lines.next())?;
    assert!(request_line.method.chars().all(|character| character.is_ascii_alphabetic()));

    let (headers, content_length_bytes) = parse_header_lines(lines)?;
    Ok(RequestHead {
        method: request_line.method.to_string(),
        target: request_line.target.to_string(),
        headers,
        content_length_bytes,
    })
}

fn utf8_head(head: &[u8]) -> Result<&str, ReadError> {
    std::str::from_utf8(head)
        .map(|head| head.trim_end_matches(super::shared::HTTP_LINE_TERMINATOR))
        .map_err(|_| invalid_request())
}

fn request_line(request_line: Option<&str>) -> Result<RequestLine<'_>, ReadError> {
    let request_line = request_line.ok_or_else(invalid_request)?;
    let mut parts = request_line.split_whitespace();
    let Some(method) = parts.next() else {
        return Err(invalid_request());
    };
    let Some(target) = parts.next() else {
        return Err(invalid_request());
    };
    let Some(version) = parts.next() else {
        return Err(invalid_request());
    };
    if parts.next().is_some() || !version.starts_with(super::shared::HTTP_VERSION_PREFIX) {
        return Err(invalid_request());
    }
    Ok(RequestLine { method, target })
}

fn parse_header_lines<'a>(lines: impl Iterator<Item = &'a str>) -> Result<(Vec<(String, String)>, usize), ReadError> {
    let mut headers = Vec::with_capacity(super::shared::MAX_HTTP_HEADER_LINES);
    let mut content_length_bytes = None;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if headers.len() >= super::shared::MAX_HTTP_HEADER_LINES {
            return Err(invalid_request());
        }
        let (header_name, header_value) = parse_header_line(line)?;
        if header_name.eq_ignore_ascii_case(super::shared::CONTENT_LENGTH_HEADER) {
            content_length_bytes = Some(parse_content_length(&header_value)?);
        }
        headers.push((header_name, header_value));
    }
    Ok((headers, content_length_bytes.unwrap_or_default()))
}

fn parse_header_line(line: &str) -> Result<(String, String), ReadError> {
    let Some((name, value)) = line.split_once(super::shared::HTTP_HEADER_SEPARATOR) else {
        return Err(invalid_request());
    };
    Ok((name.trim().to_string(), value.trim().to_string()))
}

fn parse_content_length(value: &str) -> Result<usize, ReadError> {
    value.parse::<usize>().map_err(|_| invalid_request())
}

fn invalid_request() -> ReadError {
    ReadError::Diagnostic(super::planning::diagnostic(
        super::shared::PlanningCode::InvalidRequest,
        super::shared::SAFE_BAD_REQUEST_MESSAGE,
    ))
}
