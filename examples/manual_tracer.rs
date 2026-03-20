use tokvera_rust::{FinishSpanOptions, TokveraTracer, TrackOptions, Usage};

fn main() -> Result<(), String> {
    let api_base_url = std::env::var("TOKVERA_API_BASE_URL")
        .ok()
        .or_else(|| std::env::var("TOKVERA_INGEST_URL").ok().map(|value| value.trim_end_matches("/v1/events").to_string()));
    let feature = std::env::var("TOKVERA_FEATURE").unwrap_or_else(|_| "existing_app".into());
    let tenant_id = std::env::var("TOKVERA_TENANT_ID").unwrap_or_else(|_| "tenant_demo".into());
    let environment = std::env::var("TOKVERA_ENVIRONMENT").unwrap_or_else(|_| "production".into());
    let tracer = TokveraTracer::new(TrackOptions {
        api_key: Some(std::env::var("TOKVERA_API_KEY").unwrap_or_else(|_| "tok_live_replace_me".into())),
        base_url: api_base_url,
        feature: Some(feature),
        tenant_id: Some(tenant_id),
        environment: Some(environment),
        capture_content: true,
        emit_lifecycle_events: true,
        ..Default::default()
    });

    let root = tracer.start_trace(Some(TrackOptions {
        step_name: Some("request_flow".into()),
        span_kind: Some("orchestrator".into()),
        ..Default::default()
    }))?;
    let child = tracer.start_span(&root, Some(TrackOptions {
        provider: Some("openai".into()),
        event_type: Some("openai.request".into()),
        endpoint: Some("responses.create".into()),
        model: Some("gpt-4o-mini".into()),
        step_name: Some("draft_reply".into()),
        span_kind: Some("model".into()),
        ..Default::default()
    }))?;
    let child = tracer.attach_payload(&child, serde_json::json!({ "prompt": "Draft a short support reply." }), "prompt_input");
    tracer.finish_span(&child, Some(FinishSpanOptions {
        usage: Some(Usage { prompt_tokens: 24, completion_tokens: 48, total_tokens: 72 }),
        outcome: Some("success".into()),
        ..Default::default()
    }))?;
    tracer.finish_span(&root, Some(FinishSpanOptions {
        outcome: Some("success".into()),
        ..Default::default()
    }))?;

    println!("Sent lifecycle-enabled trace to Tokvera.");
    Ok(())
}
