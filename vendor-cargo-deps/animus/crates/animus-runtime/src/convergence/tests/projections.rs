const VISIBLE_BUDGET: u32 = 5;
const OMITTED_CHARS: u64 = 5;
const TELEMETRY_BOUND_MS: u64 = 20;
const TELEMETRY_OBSERVED_MS: u64 = 10;
const OUTPUT_BOUND_BYTES: u64 = 64;
const OUTPUT_OBSERVED_BYTES: u64 = 12;
const HISTORY_EVENTS: u32 = 3;
const HISTORY_EVENTS_USIZE: usize = 3;

fn handle() -> super::super::Handle {
    super::super::Handle::new(super::super::StoreKind::Result, "res_abc123").expect("handle")
}

fn output(body: &str) -> animus_adapters::tool::BoundedOutput {
    animus_adapters::tool::BoundedOutput::new(
        &animus_adapters::tool::OutputDraft {
            body,
            full_output_reference: Some("store/result/res_abc123"),
            hash_requirement: animus_adapters::tool::HashRequirement::Required,
        },
        animus_adapters::tool::OutputPolicy {
            visible_char_budget: VISIBLE_BUDGET,
        },
    )
    .expect("bounded output")
}

fn telemetry() -> super::super::ResourceTelemetry {
    super::super::ResourceTelemetry {
        cpu_bound_ms: TELEMETRY_BOUND_MS,
        cpu_observed_ms: TELEMETRY_OBSERVED_MS,
        output_bound_bytes: OUTPUT_BOUND_BYTES,
        output_observed_bytes: OUTPUT_OBSERVED_BYTES,
    }
}

fn datum() -> super::super::Datum {
    let output = output("abcdefghij");
    super::super::admit(super::super::DatumDraft {
        call_id: "call-1",
        result_kind: "text-search",
        output: &output,
        handle: Some(&handle()),
        telemetry: Some(telemetry()),
    })
    .expect("datum")
}

#[test]
fn one_typed_datum_drives_every_projection() {
    let datum = datum();
    let model = super::super::project_model(&datum).expect("model");
    let human = super::super::project_human(&datum).expect("human");
    let record = super::super::project_record(&datum).expect("record");

    assert_eq!(model.datum_identity, datum.identity());
    assert_eq!(human.datum_identity, datum.identity());
    assert_eq!(record.datum_identity, datum.identity());
    assert!(model.text.contains("omitted 5 characters"));
    assert_eq!(
        model.omission.as_ref().map(|item| item.omitted_chars),
        Some(OMITTED_CHARS)
    );
    assert_eq!(human.telemetry, Some(telemetry()));
    assert_eq!(record.telemetry, Some(telemetry()));
}

#[test]
fn silent_truncation_and_unbounded_telemetry_fail() {
    let truncated = output("abcdefghij");
    let silent = super::super::admit(super::super::DatumDraft {
        call_id: "call-1",
        result_kind: "text-search",
        output: &truncated,
        handle: None,
        telemetry: None,
    });
    let invalid_telemetry = super::super::admit(super::super::DatumDraft {
        call_id: "call-1",
        result_kind: "text-search",
        output: &truncated,
        handle: Some(&handle()),
        telemetry: Some(super::super::ResourceTelemetry {
            cpu_observed_ms: TELEMETRY_BOUND_MS.saturating_add(1),
            ..telemetry()
        }),
    });

    assert_eq!(silent, Err(super::super::Error::SilentTruncation));
    assert_eq!(
        invalid_telemetry,
        Err(super::super::Error::TelemetryOutOfBounds)
    );
}

#[test]
fn string_only_or_drifted_record_fails_inspection() {
    let datum = datum();
    let mut record = super::super::project_record(&datum).expect("record");
    record.result_kind = alloc::string::String::from("rendered-string");

    assert_eq!(
        super::super::inspect_record(&datum, &record),
        Err(super::super::Error::InvalidIdentity)
    );
}

struct MissingStore;

impl super::super::StorePort for MissingStore {
    fn resolve(
        &self,
        _handle: &super::super::Handle,
    ) -> Result<Option<super::super::Datum>, super::super::Error> {
        Ok(None)
    }
}

#[test]
fn unknown_handle_fails_closed_with_exact_token() {
    let exact = handle();
    let error = super::super::resolve(&MissingStore, &exact).expect_err("unknown");

    assert_eq!(
        error,
        super::super::Error::UnknownHandle {
            token: alloc::string::String::from(exact.token())
        }
    );
}

#[test]
fn history_is_structured_bounded_and_renders_without_a_tool_port() {
    let datum = datum();
    let record = super::super::project_record(&datum).expect("record");
    let mut history = super::super::History::new(super::super::HistoryPolicy {
        max_events: HISTORY_EVENTS,
    })
    .expect("history");
    history
        .append(super::super::EventDraft {
            kind: super::super::EventKind::Call,
            call_id: "call-1",
            span_id: None,
            result: None,
        })
        .expect("call");
    history
        .append(super::super::EventDraft {
            kind: super::super::EventKind::Result,
            call_id: "call-1",
            span_id: None,
            result: Some(&record),
        })
        .expect("result");
    history
        .append(super::super::EventDraft {
            kind: super::super::EventKind::Span,
            call_id: "call-1",
            span_id: Some("span-1"),
            result: None,
        })
        .expect("span");
    let rendered = super::super::render_history(&history);

    assert_eq!(history.events().len(), HISTORY_EVENTS_USIZE);
    assert_eq!(rendered.len(), HISTORY_EVENTS_USIZE);
    assert!(rendered[0].contains("call"));
    assert!(rendered[1].contains(datum.identity()));
    assert!(rendered[2].contains("span-1"));
    assert!(
        history
            .append(super::super::EventDraft {
                kind: super::super::EventKind::Call,
                call_id: "call-2",
                span_id: None,
                result: None,
            })
            .is_err()
    );
}
