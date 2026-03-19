use tokvera_rust::{FinishSpanOptions, TokveraTracer, TrackOptions, Usage};

fn main() -> Result<(), String> {
    let tracer = TokveraTracer::new(TrackOptions {
        api_key: Some(std::env::var("TOKVERA_API_KEY").unwrap_or_else(|_| "tok_live_replace_me".into())),
        feature: Some("existing_app".into()),
        tenant_id: Some("tenant_demo".into()),
        environment: Some("production".into()),
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
