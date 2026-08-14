fn parse_output_format(value: &str) -> Result<OutputFormat, &'static str> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        _ => Err("approval output format is unsupported"),
    }
}

fn render_list(store: &crate::store::Store, count: u32, output: OutputFormat) -> Result<String, &'static str> {
    let views = super::shell::list_safe(store, count).map_err(|_| "approval list failed")?;
    match output {
        OutputFormat::Text => Ok(views.iter().map(super::shell::render_safe_text).collect()),
        OutputFormat::Json => serde_json::to_string(&views).map_err(|_| "approval output failed"),
    }
}

fn render_show(store: &crate::store::Store, request: &str, output: OutputFormat) -> Result<String, &'static str> {
    let request_id =
        super::ApprovalRequestId::try_from_hex(request).map_err(|_| "approval request identity is invalid")?;
    let view = super::shell::inspect_safe(store, &request_id)
        .map_err(|_| "approval inspection failed")?
        .ok_or("approval request was not found")?;
    match output {
        OutputFormat::Text => Ok(super::shell::render_safe_text(&view)),
        OutputFormat::Json => serde_json::to_string(&view).map_err(|_| "approval output failed"),
    }
}

fn render_receipt(store: &crate::store::Store, request: &str, output: OutputFormat) -> Result<String, &'static str> {
    let request_id =
        super::ApprovalRequestId::try_from_hex(request).map_err(|_| "approval request identity is invalid")?;
    let receipt = super::shell::receipt(store, &request_id)
        .map_err(|_| "approval receipt failed")?
        .ok_or("approval request was not found")?;
    debug_assert_eq!(receipt.request_id, request_id);
    debug_assert!(!receipt.required_non_claims.is_empty());
    match output {
        OutputFormat::Text => Ok(format!(
            concat!(
                "receipt={} request={} run={} status={:?} gate={} node={}\n",
                "revision={} authority_profile={:?} authority_policy={} expiry_policy={}\n",
                "delegated_policy={} verification_receipts={} clock={:?} events={} event_head={}\n",
                "choregraph_profile={} suspension_class={} non_claims={}\n"
            ),
            receipt.id,
            receipt.request_id,
            receipt.run_id,
            receipt.status,
            receipt.gate_id,
            receipt.node_id.0,
            receipt.workflow_revision,
            receipt.authority_profile,
            receipt.authority_policy_id,
            receipt.expiry_policy_id,
            receipt.delegated_policy_reference.as_deref().unwrap_or("none"),
            receipt.verification_receipt_references.len(),
            receipt.clock_source,
            receipt.event_count,
            receipt.event_head,
            receipt.choregraph_profile,
            receipt.suspension_class,
            receipt.required_non_claims.len(),
        )),
        OutputFormat::Json => serde_json::to_string(&receipt).map_err(|_| "approval output failed"),
    }
}

struct DecisionArgs<'a> {
    request: &'a str,
    workflow_revision: &'a str,
    actor: &'a str,
    decision: &'a str,
    reason: Option<String>,
    observed_epoch_ms: i64,
    output: OutputFormat,
}

fn render_decision(store: &crate::store::Store, args: DecisionArgs<'_>) -> Result<String, &'static str> {
    let request_id =
        super::ApprovalRequestId::try_from_hex(args.request).map_err(|_| "approval request identity is invalid")?;
    let revision = crate::orchestration::WorkflowRevision::try_from_hex(args.workflow_revision)
        .map_err(|_| "approval workflow revision is invalid")?;
    let actor = super::ActorId::try_from_text(args.actor).map_err(|_| "approval actor identity is invalid")?;
    let command = super::DecisionCommand {
        request_id,
        expected_workflow_revision: revision,
        actor,
        decision: match args.decision {
            "approve" => super::DecisionClass::Approve,
            "reject" => super::DecisionClass::Reject,
            _ => return Err("approval decision is unsupported"),
        },
        reason: args.reason,
        observed_epoch_millis: args.observed_epoch_ms,
        evidence: super::DecisionAuthorityEvidence::LocalOperator,
    };
    debug_assert_eq!(command.request_id.as_str(), args.request);
    debug_assert_eq!(command.expected_workflow_revision.as_str(), args.workflow_revision);
    let outcome = super::shell::decide(store, &command).map_err(|_| "approval decision failed")?;
    match args.output {
        OutputFormat::Text => Ok(format!("approval decision {outcome:?}\n")),
        OutputFormat::Json => serde_json::to_string(&outcome).map_err(|_| "approval output failed"),
    }
}

struct ResumeArgs<'a> {
    run_id: u64,
    request: &'a str,
    workflow_revision: &'a str,
    event_head: &'a str,
    owner: &'a str,
    values: &'a [String],
    credentials: &'a [String],
    observed_epoch_ms: i64,
}

fn resume_command(store: &crate::store::Store, args: ResumeArgs<'_>) -> Result<super::ResumeCommand, &'static str> {
    let request_id =
        super::ApprovalRequestId::try_from_hex(args.request).map_err(|_| "approval request identity is invalid")?;
    let revision = crate::orchestration::WorkflowRevision::try_from_hex(args.workflow_revision)
        .map_err(|_| "approval workflow revision is invalid")?;
    let event_head =
        super::ApprovalEventId::try_from_hex(args.event_head).map_err(|_| "approval event identity is invalid")?;
    let owner =
        super::ResumeOwnerId::try_from_text(args.owner).map_err(|_| "approval resume owner identity is invalid")?;
    let active = store
        .load_workflow_approval_for_run(crate::engine::executor::RunId(args.run_id))
        .map_err(|_| "active approval lookup failed")?
        .ok_or("active approval was not found")?;
    if active.id != request_id || active.workflow_revision != revision {
        return Err("approval request binding does not match the active run");
    }
    debug_assert_eq!(active.id, request_id);
    debug_assert_eq!(active.workflow_revision, revision);
    let values = args.values.iter().map(|value| parse_resume_value(value)).collect::<Result<_, _>>()?;
    let credentials = args.credentials.iter().map(|value| parse_resume_credential(value)).collect::<Result<_, _>>()?;
    Ok(super::ResumeCommand {
        request_id,
        expected_workflow_revision: revision,
        expected_event_head: event_head,
        owner,
        observed_epoch_millis: args.observed_epoch_ms,
        values,
        credentials,
    })
}

fn parse_resume_value(value: &str) -> Result<super::ResumeValue, &'static str> {
    let (name, typed_value) = value.split_once('=').ok_or("resume value must use name=type:value")?;
    let (type_name, raw_value) = typed_value.split_once(':').ok_or("resume value must use name=type:value")?;
    let name = super::ResumeBindingName::try_from_text(name).map_err(|_| "resume value name is invalid")?;
    let value = match type_name {
        "string" => crate::engine::value::Value::from_string(raw_value.to_string()),
        "json" => crate::engine::value::Value::from_json(
            serde_json::from_str(raw_value).map_err(|_| "resume JSON value is invalid")?,
        ),
        "bytes" => crate::engine::value::Value::from_bytes(parse_hex_bytes(raw_value)?),
        "exitcode" => {
            crate::engine::value::Value::from_exit_code(raw_value.parse().map_err(|_| "resume exit code is invalid")?)
        }
        _ => return Err("resume value type is unsupported"),
    };
    Ok(super::ResumeValue { name, value })
}

fn parse_hex_bytes(value: &str) -> Result<Vec<u8>, &'static str> {
    let (pairs, remainder) = value.as_bytes().as_chunks::<HEX_CHARS_PER_BYTE>();
    if !remainder.is_empty() {
        return Err("resume byte value must be even-length hexadecimal");
    }
    pairs
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(|_| "resume byte value is invalid")?;
            u8::from_str_radix(text, HEX_RADIX).map_err(|_| "resume byte value is invalid")
        })
        .collect()
}

fn parse_resume_credential(value: &str) -> Result<super::ResumeCredentialBinding, &'static str> {
    let (name, reference) = value.split_once('=').ok_or("resume credential must use name=provider:key")?;
    let (provider, key) = reference.split_once(':').ok_or("resume credential must use name=provider:key")?;
    let name = super::ResumeBindingName::try_from_text(name).map_err(|_| "resume credential name is invalid")?;
    let reference = crate::credentials::SecretRef::new(provider, key, None);
    crate::credentials::validate_secret_ref(&reference).map_err(|_| "resume credential reference is invalid")?;
    Ok(super::ResumeCredentialBinding { name, reference })
}

fn open_store(config: &crate::config::AppConfig) -> Result<crate::store::Store, String> {
    crate::store::Store::new(crate::config::store_path(&config.data_dir)).map_err(|error| error.to_string())
}

fn write_result(prefix: &str, result: Result<String, &'static str>) -> std::process::ExitCode {
    match result {
        Ok(rendered) => {
            let mut stdout = std::io::stdout().lock();
            if stdout.write_all(rendered.as_bytes()).and_then(|()| stdout.flush()).is_ok() {
                std::process::ExitCode::SUCCESS
            } else {
                fail(FailureArgs {
                    prefix,
                    error: "output failed",
                })
            }
        }
        Err(error) => fail(FailureArgs { prefix, error }),
    }
}

struct FailureArgs<'a> {
    prefix: &'a str,
    error: &'a str,
}

fn fail(args: FailureArgs<'_>) -> std::process::ExitCode {
    eprintln!("{}: {}", args.prefix, args.error);
    std::process::ExitCode::FAILURE
}
