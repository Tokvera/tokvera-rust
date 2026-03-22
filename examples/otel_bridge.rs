use chrono::{Duration, Utc};
use std::collections::HashMap;
use tokvera_rust::{OtelReadableSpan, TokveraOtelBridge, TrackOptions};

fn main() -> Result<(), String> {
    let api_base_url = std::env::var("TOKVERA_API_BASE_URL").ok().or_else(|| {
        std::env::var("TOKVERA_INGEST_URL")
            .ok()
            .map(|value| value.trim_end_matches("/v1/events").to_string())
    });
    let feature = std::env::var("TOKVERA_FEATURE").unwrap_or_else(|_| "otel_bridge".into());
    let tenant_id = std::env::var("TOKVERA_TENANT_ID").unwrap_or_else(|_| "tenant_demo".into());
    let bridge = TokveraOtelBridge::new(TrackOptions {
        api_key: Some(
            std::env::var("TOKVERA_API_KEY").unwrap_or_else(|_| "tok_live_replace_me".into()),
        ),
        base_url: api_base_url,
        feature: Some(feature),
        tenant_id: Some(tenant_id),
        ..Default::default()
    });

    bridge.export(&[OtelReadableSpan {
        name: "llm_call".into(),
        trace_id: "trc_rust_otel".into(),
        span_id: "spn_rust_otel".into(),
        parent_span_id: None,
        start_time: Utc::now() - Duration::milliseconds(300),
        end_time: Utc::now(),
        status_code: "ok".into(),
        attributes: HashMap::from([
            ("llm.provider".into(), serde_json::json!("openai")),
            (
                "gen_ai.request.model".into(),
                serde_json::json!("gpt-4o-mini"),
            ),
            (
                "tokvera.event_type".into(),
                serde_json::json!("openai.request"),
            ),
            (
                "tokvera.endpoint".into(),
                serde_json::json!("responses.create"),
            ),
            ("gen_ai.usage.total_tokens".into(), serde_json::json!(19)),
        ]),
    }])?;

    println!("Forwarded OTel spans to Tokvera.");
    Ok(())
}
