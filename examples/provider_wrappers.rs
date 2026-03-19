use tokvera_rust::{FinishSpanOptions, ProviderRequest, ProviderResult, TokveraTracer, TrackOptions, Usage};

fn main() -> Result<(), String> {
    let tracer = TokveraTracer::new(TrackOptions {
        api_key: Some(std::env::var("TOKVERA_API_KEY").unwrap_or_else(|_| "tok_live_replace_me".into())),
        feature: Some("router".into()),
        tenant_id: Some("tenant_demo".into()),
        capture_content: true,
        emit_lifecycle_events: true,
        ..Default::default()
    });

    let root = tracer.start_trace(Some(TrackOptions {
        step_name: Some("router_root".into()),
        span_kind: Some("orchestrator".into()),
        ..Default::default()
    }))?;

    tracer.track_mistral(&root, ProviderRequest {
        model: Some("mistral-small".into()),
        input: Some(serde_json::json!({ "prompt": "Classify this ticket." })),
        ..Default::default()
    }, || {
        Ok(ProviderResult {
            output: Some(serde_json::json!({ "route": "billing" })),
            usage: Some(Usage { prompt_tokens: 11, completion_tokens: 3, total_tokens: 14 }),
            ..Default::default()
        })
    })?;

    tracer.finish_span(&root, Some(FinishSpanOptions {
        outcome: Some("success".into()),
        ..Default::default()
    }))?;

    println!("Tracked provider child span.");
    Ok(())
}
