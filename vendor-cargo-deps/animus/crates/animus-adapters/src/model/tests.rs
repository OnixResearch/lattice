use super::*;

const VALID_PROMPT: &str = "intent: build reliable systems";
const BLANK_TEXT: &str = "   ";
const NUL_TEXT: &str = "bad\0text";
const NUL_BYTE_INDEX: usize = 3;

struct EchoAdapter;

impl crate::ComponentAdapter<Request, Response> for EchoAdapter {
    type Error = crate::AdapterContractError;

    fn call(&mut self, request: &Request) -> Result<Response, Self::Error> {
        Response::new(request.prompt())
    }
}

#[test]
fn adapter_contract_accepts_valid_request_response() {
    let request = Request::new(VALID_PROMPT).expect("valid prompt should parse");
    let mut adapter = EchoAdapter;
    let response =
        <EchoAdapter as crate::ComponentAdapter<Request, Response>>::call(&mut adapter, &request)
            .expect("valid echo response should parse");

    assert_eq!(request.prompt(), VALID_PROMPT);
    assert_eq!(response.message(), VALID_PROMPT);
    assert_eq!(
        Request::contract_identity(),
        MODEL_REQUEST_CONTRACT_IDENTITY
    );
    assert_eq!(
        Response::contract_identity(),
        MODEL_RESPONSE_CONTRACT_IDENTITY
    );
}

#[test]
fn request_from_state_uses_core_summary() {
    let intent =
        animus_core::Intent::new("build reliable systems").expect("valid intent should parse");
    let state = animus_core::AnimusState::new(intent);
    let request = Request::from_state(&state);

    assert_eq!(request.prompt(), VALID_PROMPT);
}

#[test]
fn response_accepts_multiline_text() {
    let response =
        Response::new("first line\nsecond line").expect("multiline response should parse");

    assert_eq!(response.message(), "first line\nsecond line");
}

#[test]
fn response_rejects_empty_text() {
    let error = Response::new(BLANK_TEXT).expect_err("blank response should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::EmptyField {
            field: MESSAGE_FIELD.as_str(),
        }
    );
}

#[test]
fn request_rejects_overlong_prompt() {
    let too_long_prompt_chars =
        crate::MAX_ADAPTER_FIELD_CHARS.saturating_add(crate::MIN_ADAPTER_FIELD_CHARS);
    let raw = "a".repeat(too_long_prompt_chars);
    let error = Request::new(&raw).expect_err("overlong prompt should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::FieldTooLong {
            field: PROMPT_FIELD.as_str(),
            max_chars: crate::MAX_ADAPTER_FIELD_CHARS,
            actual_chars: too_long_prompt_chars,
        }
    );
}

#[test]
fn request_rejects_nul_character() {
    let error = Request::new(NUL_TEXT).expect_err("NUL prompt should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::NulCharacter {
            field: PROMPT_FIELD.as_str(),
            byte_index: NUL_BYTE_INDEX,
        }
    );
}
