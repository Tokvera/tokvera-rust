use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Default, Debug)]
pub struct TrackOptions {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub feature: Option<String>,
    pub tenant_id: Option<String>,
    pub customer_id: Option<String>,
    pub environment: Option<String>,
    pub plan: Option<String>,
    pub trace_id: Option<String>,
    pub run_id: Option<String>,
    pub conversation_id: Option<String>,
    pub span_id: Option<String>,
    pub parent_span_id: Option<String>,
    pub provider: Option<String>,
    pub event_type: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub step_name: Option<String>,
    pub span_kind: Option<String>,
    pub tool_name: Option<String>,
    pub attempt_type: Option<String>,
    pub outcome: Option<String>,
    pub retry_reason: Option<String>,
    pub fallback_reason: Option<String>,
    pub quality_label: Option<String>,
    pub feedback_score: Option<f64>,
    pub capture_content: bool,
    pub emit_lifecycle_events: bool,
    pub payload_refs: Vec<String>,
    pub payload_blocks: Vec<Value>,
    pub metrics: HashMap<String, Value>,
    pub decision: HashMap<String, Value>,
    pub schema_version: Option<String>,
}

impl TrackOptions {
    pub fn merge(base: &TrackOptions, override_opts: Option<&TrackOptions>) -> TrackOptions {
        let mut merged = base.clone();
        if let Some(override_opts) = override_opts {
            macro_rules! apply_opt {
                ($field:ident) => {
                    if override_opts.$field.is_some() {
                        merged.$field = override_opts.$field.clone();
                    }
                };
            }
            apply_opt!(api_key);
            apply_opt!(base_url);
            apply_opt!(feature);
            apply_opt!(tenant_id);
            apply_opt!(customer_id);
            apply_opt!(environment);
            apply_opt!(plan);
            apply_opt!(trace_id);
            apply_opt!(run_id);
            apply_opt!(conversation_id);
            apply_opt!(span_id);
            apply_opt!(parent_span_id);
            apply_opt!(provider);
            apply_opt!(event_type);
            apply_opt!(endpoint);
            apply_opt!(model);
            apply_opt!(step_name);
            apply_opt!(span_kind);
            apply_opt!(tool_name);
            apply_opt!(attempt_type);
            apply_opt!(outcome);
            apply_opt!(retry_reason);
            apply_opt!(fallback_reason);
            apply_opt!(quality_label);
            if override_opts.feedback_score.is_some() {
                merged.feedback_score = override_opts.feedback_score;
            }
            if !override_opts.payload_refs.is_empty() {
                merged.payload_refs = override_opts.payload_refs.clone();
            }
            if !override_opts.payload_blocks.is_empty() {
                merged.payload_blocks = override_opts.payload_blocks.clone();
            }
            if !override_opts.metrics.is_empty() {
                merged.metrics = override_opts.metrics.clone();
            }
            if !override_opts.decision.is_empty() {
                merged.decision = override_opts.decision.clone();
            }
            if override_opts.capture_content {
                merged.capture_content = true;
            }
            if override_opts.emit_lifecycle_events {
                merged.emit_lifecycle_events = true;
            }
            if override_opts.schema_version.is_some() {
                merged.schema_version = override_opts.schema_version.clone();
            }
        }
        merged
    }
}

#[derive(Clone, Default, Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Clone, Default, Debug)]
pub struct FinishSpanOptions {
    pub usage: Option<Usage>,
    pub outcome: Option<String>,
    pub quality_label: Option<String>,
    pub feedback_score: Option<f64>,
    pub metrics: HashMap<String, Value>,
    pub decision: HashMap<String, Value>,
    pub payload_blocks: Vec<Value>,
    pub error: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct TraceHandle {
    pub trace_id: String,
    pub run_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub provider: String,
    pub event_type: String,
    pub endpoint: String,
    pub model: String,
    pub options: TrackOptions,
}

#[derive(Clone, Default, Debug)]
pub struct ProviderRequest {
    pub model: Option<String>,
    pub input: Option<Value>,
    pub event_type: Option<String>,
    pub endpoint: Option<String>,
    pub step_name: Option<String>,
    pub span_kind: Option<String>,
    pub tool_name: Option<String>,
}

#[derive(Clone, Default, Debug)]
pub struct ProviderResult {
    pub output: Option<Value>,
    pub usage: Option<Usage>,
    pub outcome: Option<String>,
    pub quality_label: Option<String>,
    pub feedback_score: Option<f64>,
    pub metrics: HashMap<String, Value>,
    pub decision: HashMap<String, Value>,
}

#[derive(Clone, Default, Debug)]
pub struct OtelReadableSpan {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status_code: String,
    pub attributes: HashMap<String, Value>,
}

pub trait IngestClient: Send + Sync {
    fn ingest_event(&self, event: Value) -> Result<(), String>;
}

pub struct HttpClient {
    api_key: String,
    base_url: String,
}

impl HttpClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.tokvera.org".to_string()),
        }
    }
}

impl IngestClient for HttpClient {
    fn ingest_event(&self, event: Value) -> Result<(), String> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/v1/events", self.base_url.trim_end_matches('/')))
            .bearer_auth(&self.api_key)
            .json(&event)
            .send()
            .map_err(|error| error.to_string())?;
        if !response.status().is_success() {
            return Err(format!("tokvera ingest failed: {}", response.status()));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct TokveraTracer {
    base: TrackOptions,
    client: Arc<dyn IngestClient>,
}

impl TokveraTracer {
    pub fn new(base: TrackOptions) -> Self {
        let api_key = base.api_key.clone().unwrap_or_default();
        Self {
            base: base.clone(),
            client: Arc::new(HttpClient::new(api_key, base.base_url.clone())),
        }
    }

    pub fn with_client(base: TrackOptions, client: Arc<dyn IngestClient>) -> Self {
        Self { base, client }
    }

    pub fn start_trace(&self, options: Option<TrackOptions>) -> Result<TraceHandle, String> {
        let merged = TrackOptions::merge(&self.base, options.as_ref());
        let handle = TraceHandle {
            trace_id: choose_owned(vec![options.as_ref().and_then(|o| o.trace_id.clone()), merged.trace_id.clone()], "trc"),
            run_id: choose_owned(vec![options.as_ref().and_then(|o| o.run_id.clone()), merged.run_id.clone()], "run"),
            span_id: choose_owned(vec![options.as_ref().and_then(|o| o.span_id.clone()), merged.span_id.clone()], "spn"),
            parent_span_id: None,
            started_at: Utc::now(),
            provider: choose_owned(vec![options.as_ref().and_then(|o| o.provider.clone()), merged.provider.clone()], "tokvera"),
            event_type: choose_owned(vec![options.as_ref().and_then(|o| o.event_type.clone()), merged.event_type.clone()], "tokvera.trace"),
            endpoint: choose_owned(vec![options.as_ref().and_then(|o| o.endpoint.clone()), merged.endpoint.clone()], "manual.trace"),
            model: choose_owned(vec![options.as_ref().and_then(|o| o.model.clone()), merged.model.clone()], "manual"),
            options: merged,
        };
        let hydrated = self.hydrate(handle);
        if hydrated.options.emit_lifecycle_events {
            self.client.ingest_event(self.build_event(&hydrated, "in_progress", FinishSpanOptions::default()))?;
        }
        Ok(hydrated)
    }

    pub fn start_span(&self, parent: &TraceHandle, options: Option<TrackOptions>) -> Result<TraceHandle, String> {
        let merged = TrackOptions::merge(&TrackOptions::merge(&self.base, Some(&parent.options)), options.as_ref());
        let handle = TraceHandle {
            trace_id: options.as_ref().and_then(|o| o.trace_id.clone()).or_else(|| merged.trace_id.clone()).unwrap_or_else(|| parent.trace_id.clone()),
            run_id: options.as_ref().and_then(|o| o.run_id.clone()).or_else(|| merged.run_id.clone()).unwrap_or_else(|| parent.run_id.clone()),
            span_id: options.as_ref().and_then(|o| o.span_id.clone()).or_else(|| merged.span_id.clone()).unwrap_or_else(|| format!("spn_{}", Uuid::new_v4().simple())),
            parent_span_id: options.as_ref().and_then(|o| o.parent_span_id.clone()).or_else(|| merged.parent_span_id.clone()).or_else(|| Some(parent.span_id.clone())),
            started_at: Utc::now(),
            provider: options.as_ref().and_then(|o| o.provider.clone()).or_else(|| merged.provider.clone()).unwrap_or_else(|| parent.provider.clone()),
            event_type: options.as_ref().and_then(|o| o.event_type.clone()).or_else(|| merged.event_type.clone()).unwrap_or_else(|| "tokvera.trace".to_string()),
            endpoint: options.as_ref().and_then(|o| o.endpoint.clone()).or_else(|| merged.endpoint.clone()).unwrap_or_else(|| "manual.span".to_string()),
            model: options.as_ref().and_then(|o| o.model.clone()).or_else(|| merged.model.clone()).unwrap_or_else(|| parent.model.clone()),
            options: merged,
        };
        let hydrated = self.hydrate(handle);
        if hydrated.options.emit_lifecycle_events {
            self.client.ingest_event(self.build_event(&hydrated, "in_progress", FinishSpanOptions::default()))?;
        }
        Ok(hydrated)
    }

    pub fn attach_payload(&self, handle: &TraceHandle, payload: Value, payload_type: &str) -> TraceHandle {
        let mut updated = handle.clone();
        updated.options.payload_blocks.push(json!({
            "payload_type": payload_type,
            "content": if payload.is_string() { payload } else { Value::String(payload.to_string()) },
        }));
        updated
    }

    pub fn finish_span(&self, handle: &TraceHandle, options: Option<FinishSpanOptions>) -> Result<(), String> {
        self.client.ingest_event(self.build_event(handle, "success", options.unwrap_or_default()))
    }

    pub fn fail_span(&self, handle: &TraceHandle, message: &str, options: Option<FinishSpanOptions>) -> Result<(), String> {
        let mut effective = options.unwrap_or_default();
        if effective.error.is_none() {
            effective.error = Some(json!({ "type": "runtime_error", "message": message }));
        }
        self.client.ingest_event(self.build_event(handle, "failure", effective))
    }

    pub fn get_track_options_from_trace_context(&self, handle: &TraceHandle, overrides: Option<TrackOptions>) -> TrackOptions {
        let mut merged = TrackOptions::merge(&TrackOptions::merge(&self.base, Some(&handle.options)), overrides.as_ref());
        if overrides.as_ref().and_then(|o| o.trace_id.clone()).is_none() {
            merged.trace_id = Some(handle.trace_id.clone());
        }
        if overrides.as_ref().and_then(|o| o.run_id.clone()).is_none() {
            merged.run_id = Some(handle.run_id.clone());
        }
        if overrides.as_ref().and_then(|o| o.parent_span_id.clone()).is_none() {
            merged.parent_span_id = Some(handle.span_id.clone());
        }
        if overrides.as_ref().and_then(|o| o.span_id.clone()).is_none() {
            merged.span_id = None;
        }
        if overrides.as_ref().and_then(|o| o.provider.clone()).is_none() {
            merged.provider = Some(handle.provider.clone());
        }
        if overrides.as_ref().and_then(|o| o.event_type.clone()).is_none() {
            merged.event_type = Some(handle.event_type.clone());
        }
        if overrides.as_ref().and_then(|o| o.endpoint.clone()).is_none() {
            merged.endpoint = Some(handle.endpoint.clone());
        }
        if overrides.as_ref().and_then(|o| o.model.clone()).is_none() {
            merged.model = Some(handle.model.clone());
        }
        merged
    }

    pub fn track_openai<F>(&self, parent: &TraceHandle, request: ProviderRequest, operation: F) -> Result<ProviderResult, String>
    where
        F: FnOnce() -> Result<ProviderResult, String>,
    {
        self.track_provider(parent, "openai", request, operation)
    }

    pub fn track_anthropic<F>(&self, parent: &TraceHandle, request: ProviderRequest, operation: F) -> Result<ProviderResult, String>
    where
        F: FnOnce() -> Result<ProviderResult, String>,
    {
        self.track_provider(parent, "anthropic", request, operation)
    }

    pub fn track_gemini<F>(&self, parent: &TraceHandle, request: ProviderRequest, operation: F) -> Result<ProviderResult, String>
    where
        F: FnOnce() -> Result<ProviderResult, String>,
    {
        self.track_provider(parent, "gemini", request, operation)
    }

    pub fn track_mistral<F>(&self, parent: &TraceHandle, request: ProviderRequest, operation: F) -> Result<ProviderResult, String>
    where
        F: FnOnce() -> Result<ProviderResult, String>,
    {
        self.track_provider(parent, "mistral", request, operation)
    }

    fn track_provider<F>(&self, parent: &TraceHandle, provider: &str, request: ProviderRequest, operation: F) -> Result<ProviderResult, String>
    where
        F: FnOnce() -> Result<ProviderResult, String>,
    {
        let mut child = self.start_span(parent, Some(TrackOptions {
            provider: Some(provider.to_string()),
            event_type: Some(request.event_type.unwrap_or_else(|| format!("{}.request", provider))),
            endpoint: Some(request.endpoint.unwrap_or_else(|| default_provider_endpoint(provider))),
            model: request.model.clone(),
            step_name: Some(request.step_name.unwrap_or_else(|| format!("{}_call", provider))),
            span_kind: Some(request.span_kind.unwrap_or_else(|| "model".to_string())),
            tool_name: request.tool_name.clone(),
            ..Default::default()
        }))?;
        if let Some(input) = request.input.clone() {
            if child.options.capture_content {
                child = self.attach_payload(&child, input, "prompt_input");
            }
        }
        let result = operation()?;
        if let Some(output) = result.output.clone() {
            if child.options.capture_content {
                child = self.attach_payload(&child, output, "model_output");
            }
        }
        self.finish_span(&child, Some(FinishSpanOptions {
            usage: result.usage.clone(),
            outcome: result.outcome.clone(),
            quality_label: result.quality_label.clone(),
            feedback_score: result.feedback_score,
            metrics: result.metrics.clone(),
            decision: result.decision.clone(),
            ..Default::default()
        }))?;
        Ok(result)
    }

    fn hydrate(&self, mut handle: TraceHandle) -> TraceHandle {
        handle.options.trace_id = Some(handle.trace_id.clone());
        handle.options.run_id = Some(handle.run_id.clone());
        handle.options.span_id = Some(handle.span_id.clone());
        handle.options.parent_span_id = handle.parent_span_id.clone();
        handle.options.provider = Some(handle.provider.clone());
        handle.options.event_type = Some(handle.event_type.clone());
        handle.options.endpoint = Some(handle.endpoint.clone());
        handle.options.model = Some(handle.model.clone());
        if handle.options.step_name.is_none() {
            handle.options.step_name = Some(if handle.parent_span_id.is_none() { "trace_root" } else { "span_step" }.to_string());
        }
        if handle.options.span_kind.is_none() {
            handle.options.span_kind = Some("orchestrator".to_string());
        }
        if handle.options.schema_version.is_none() {
            handle.options.schema_version = Some("2026-04-01".to_string());
        }
        handle
    }

    fn build_event(&self, handle: &TraceHandle, status: &str, options: FinishSpanOptions) -> Value {
        let usage = options.usage.clone().unwrap_or_default();
        let latency_ms = options
            .metrics
            .get("latency_ms")
            .and_then(|value| value.as_i64())
            .unwrap_or_else(|| (Utc::now() - handle.started_at).num_milliseconds().max(1));
        let outcome = options
            .outcome
            .clone()
            .or_else(|| handle.options.outcome.clone())
            .unwrap_or_else(|| if status == "failure" { "failure".to_string() } else { "success".to_string() });
        let retry_reason = handle.options.retry_reason.clone().or_else(|| options.decision.get("retry_reason").and_then(|value| value.as_str().map(str::to_string)));
        let fallback_reason = handle.options.fallback_reason.clone().or_else(|| options.decision.get("fallback_reason").and_then(|value| value.as_str().map(str::to_string)));
        let quality_label = options.quality_label.clone().or_else(|| handle.options.quality_label.clone());
        let feedback_score = options.feedback_score.or(handle.options.feedback_score);

        let mut metrics = handle.options.metrics.clone();
        metrics.extend(options.metrics.clone());
        metrics.insert("latency_ms".to_string(), json!(latency_ms));
        metrics.insert("prompt_tokens".to_string(), json!(usage.prompt_tokens));
        metrics.insert("completion_tokens".to_string(), json!(usage.completion_tokens));
        metrics.insert("total_tokens".to_string(), json!(usage.total_tokens));

        let mut decision = handle.options.decision.clone();
        decision.extend(options.decision.clone());

        let mut payload_blocks = handle.options.payload_blocks.clone();
        payload_blocks.extend(options.payload_blocks.clone());

        let evaluation = if outcome.is_empty() && retry_reason.is_none() && fallback_reason.is_none() && quality_label.is_none() && feedback_score.is_none() {
            Value::Null
        } else {
            json!({
                "outcome": outcome,
                "retry_reason": retry_reason,
                "fallback_reason": fallback_reason,
                "quality_label": quality_label,
                "feedback_score": feedback_score,
            })
        };

        json!({
            "schema_version": handle.options.schema_version.clone().unwrap_or_else(|| "2026-04-01".to_string()),
            "event_type": handle.event_type,
            "provider": handle.provider,
            "endpoint": handle.endpoint,
            "status": status,
            "timestamp": Utc::now().to_rfc3339(),
            "latency_ms": latency_ms,
            "model": handle.model,
            "usage": usage,
            "tags": {
                "feature": handle.options.feature,
                "tenant_id": handle.options.tenant_id,
                "customer_id": handle.options.customer_id,
                "environment": handle.options.environment,
                "plan": handle.options.plan,
                "attempt_type": handle.options.attempt_type,
                "trace_id": handle.trace_id,
                "run_id": handle.run_id,
                "conversation_id": handle.options.conversation_id,
                "span_id": handle.span_id,
                "parent_span_id": handle.parent_span_id,
                "step_name": handle.options.step_name,
                "outcome": outcome,
                "retry_reason": retry_reason,
                "fallback_reason": fallback_reason,
                "quality_label": quality_label,
                "feedback_score": feedback_score,
            },
            "evaluation": evaluation,
            "span_kind": handle.options.span_kind,
            "tool_name": handle.options.tool_name,
            "payload_refs": handle.options.payload_refs,
            "payload_blocks": payload_blocks,
            "metrics": metrics,
            "decision": decision,
            "error": options.error,
        })
    }
}

#[derive(Clone)]
pub struct TokveraOtelBridge {
    client: Arc<dyn IngestClient>,
}

impl TokveraOtelBridge {
    pub fn new(base: TrackOptions) -> Self {
        let api_key = base.api_key.clone().unwrap_or_default();
        Self {
            client: Arc::new(HttpClient::new(api_key, base.base_url.clone())),
        }
    }

    pub fn with_client(client: Arc<dyn IngestClient>) -> Self {
        Self { client }
    }

    pub fn export(&self, spans: &[OtelReadableSpan]) -> Result<(), String> {
        for span in spans {
            let latency_ms = (span.end_time - span.start_time).num_milliseconds().max(1);
            let status = if span.status_code == "error" { "failure" } else { "success" };
            let payload = json!({
                "schema_version": "2026-04-01",
                "event_type": span.attributes.get("tokvera.event_type").cloned().unwrap_or_else(|| json!("tokvera.trace")),
                "provider": span.attributes.get("llm.provider").cloned().unwrap_or_else(|| json!("tokvera")),
                "endpoint": span.attributes.get("tokvera.endpoint").cloned().unwrap_or_else(|| json!("otel.span")),
                "status": status,
                "timestamp": span.end_time.to_rfc3339(),
                "latency_ms": latency_ms,
                "model": span.attributes.get("gen_ai.request.model").cloned().unwrap_or_else(|| json!("otel")),
                "usage": {
                    "prompt_tokens": span.attributes.get("gen_ai.usage.input_tokens").cloned().unwrap_or_else(|| json!(0)),
                    "completion_tokens": span.attributes.get("gen_ai.usage.output_tokens").cloned().unwrap_or_else(|| json!(0)),
                    "total_tokens": span.attributes.get("gen_ai.usage.total_tokens").cloned().unwrap_or_else(|| json!(0)),
                },
                "tags": {
                    "feature": span.attributes.get("tokvera.feature").cloned().unwrap_or_else(|| json!("otel_bridge")),
                    "tenant_id": span.attributes.get("tokvera.tenant_id").cloned().unwrap_or_else(|| json!("otel")),
                    "trace_id": span.trace_id,
                    "run_id": span.attributes.get("tokvera.run_id").cloned().unwrap_or_else(|| json!(span.trace_id)),
                    "span_id": span.span_id,
                    "parent_span_id": span.parent_span_id,
                    "step_name": span.name,
                    "outcome": if status == "failure" { "failure" } else { "success" },
                },
                "span_kind": span.attributes.get("tokvera.span_kind").cloned().unwrap_or_else(|| json!("orchestrator")),
                "metrics": {
                    "latency_ms": latency_ms,
                    "prompt_tokens": span.attributes.get("gen_ai.usage.input_tokens").cloned().unwrap_or_else(|| json!(0)),
                    "completion_tokens": span.attributes.get("gen_ai.usage.output_tokens").cloned().unwrap_or_else(|| json!(0)),
                    "total_tokens": span.attributes.get("gen_ai.usage.total_tokens").cloned().unwrap_or_else(|| json!(0)),
                }
            });
            self.client.ingest_event(payload)?;
        }
        Ok(())
    }
}

fn choose_owned(values: Vec<Option<String>>, prefix: &str) -> String {
    values.into_iter().flatten().find(|value| !value.is_empty()).unwrap_or_else(|| format!("{}_{}", prefix, Uuid::new_v4().simple()))
}

fn default_provider_endpoint(provider: &str) -> String {
    match provider {
        "openai" => "responses.create".to_string(),
        "anthropic" => "messages.create".to_string(),
        "gemini" => "models.generate_content".to_string(),
        "mistral" => "chat.complete".to_string(),
        _ => "manual.span".to_string(),
    }
}

#[derive(Default)]
struct RecordingClient {
    events: Mutex<Vec<Value>>,
}

impl IngestClient for RecordingClient {
    fn ingest_event(&self, event: Value) -> Result<(), String> {
        self.events.lock().map_err(|error| error.to_string())?.push(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_tracing_keeps_lifecycle_on_one_trace() {
        let recorder = Arc::new(RecordingClient::default());
        let client: Arc<dyn IngestClient> = recorder.clone();
        let tracer = TokveraTracer::with_client(
            TrackOptions {
                api_key: Some("tok_test".into()),
                feature: Some("existing_app".into()),
                tenant_id: Some("tenant_test".into()),
                capture_content: true,
                emit_lifecycle_events: true,
                ..Default::default()
            },
            client,
        );

        let root = tracer.start_trace(Some(TrackOptions {
            step_name: Some("request_flow".into()),
            ..Default::default()
        })).unwrap();
        let child = tracer.start_span(&root, Some(TrackOptions {
            provider: Some("openai".into()),
            event_type: Some("openai.request".into()),
            endpoint: Some("responses.create".into()),
            model: Some("gpt-4o-mini".into()),
            ..Default::default()
        })).unwrap();
        let child = tracer.attach_payload(&child, json!({"prompt": "Hello"}), "prompt_input");
        tracer.finish_span(&child, Some(FinishSpanOptions {
            usage: Some(Usage { prompt_tokens: 12, completion_tokens: 8, total_tokens: 20 }),
            ..Default::default()
        })).unwrap();
        tracer.finish_span(&root, Some(FinishSpanOptions {
            outcome: Some("success".into()),
            ..Default::default()
        })).unwrap();

        let events = recorder.events.lock().unwrap().clone();
        assert_eq!(events.len(), 4);
        assert_eq!(events[0]["status"], "in_progress");
        assert_eq!(events[2]["status"], "success");
        assert_eq!(events[2]["tags"]["trace_id"], root.trace_id);
        assert_eq!(events[2]["tags"]["span_id"], child.span_id);
    }

    #[test]
    fn provider_wrapper_emits_mistral_child_span() {
        let recorder = Arc::new(RecordingClient::default());
        let client: Arc<dyn IngestClient> = recorder.clone();
        let tracer = TokveraTracer::with_client(
            TrackOptions {
                api_key: Some("tok_test".into()),
                feature: Some("router".into()),
                tenant_id: Some("tenant_test".into()),
                capture_content: true,
                ..Default::default()
            },
            client,
        );
        let root = tracer.start_trace(Some(TrackOptions {
            step_name: Some("router_root".into()),
            ..Default::default()
        })).unwrap();
        let result = tracer.track_mistral(&root, ProviderRequest {
            model: Some("mistral-small".into()),
            input: Some(json!({"prompt": "Classify"})),
            ..Default::default()
        }, || {
            Ok(ProviderResult {
                output: Some(json!({"label": "billing"})),
                usage: Some(Usage { prompt_tokens: 10, completion_tokens: 2, total_tokens: 12 }),
                ..Default::default()
            })
        }).unwrap();
        assert_eq!(result.output.unwrap()["label"], "billing");
        let events = recorder.events.lock().unwrap().clone();
        assert_eq!(events[1]["provider"], "mistral");
        assert_eq!(events[1]["event_type"], "mistral.request");
    }

    #[test]
    fn otel_bridge_exports_canonical_span() {
        let recorder = Arc::new(RecordingClient::default());
        let client: Arc<dyn IngestClient> = recorder.clone();
        let bridge = TokveraOtelBridge::with_client(client);
        bridge.export(&[OtelReadableSpan {
            name: "llm_call".into(),
            trace_id: "trc_otel".into(),
            span_id: "spn_otel".into(),
            parent_span_id: None,
            start_time: Utc::now() - chrono::Duration::seconds(1),
            end_time: Utc::now(),
            status_code: "ok".into(),
            attributes: HashMap::from([
                ("llm.provider".into(), json!("openai")),
                ("gen_ai.request.model".into(), json!("gpt-4o-mini")),
                ("tokvera.event_type".into(), json!("openai.request")),
                ("tokvera.endpoint".into(), json!("responses.create")),
                ("gen_ai.usage.total_tokens".into(), json!(17)),
            ]),
        }]).unwrap();
        let events = recorder.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event_type"], "openai.request");
        assert_eq!(events[0]["tags"]["trace_id"], "trc_otel");
    }
}
