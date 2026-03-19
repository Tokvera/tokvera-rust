# tokvera-rust

Preview Rust SDK for Tokvera tracing.

Current Wave 3 preview surface:
- manual tracer substrate
- lifecycle-capable root and child spans
- provider wrappers for OpenAI, Anthropic, Gemini, and Mistral
- OTel bridge
- runnable examples
- canonical contract check

This repo is not official until it clears:
- native Rust CI and local toolchain validation
- canonical contract validation
- shared smoke and soak visibility in `tokvera`
- dashboard visibility in traces, live traces, and trace detail

## Install

```bash
cargo add tokvera-rust
```

## Quickstart

```rust
use tokvera_rust::{TokveraTracer, TrackOptions};

let tracer = TokveraTracer::new(TrackOptions {
    api_key: Some("tok_live_replace_me".into()),
    feature: Some("existing_app".into()),
    capture_content: true,
    emit_lifecycle_events: true,
    ..Default::default()
});

let trace = tracer.start_trace(None).unwrap();
let span = tracer.start_span(&trace, Some(TrackOptions {
    step_name: Some("plan_response".into()),
    ..Default::default()
})).unwrap();
tracer.finish_span(&span, None).unwrap();
```

## Examples

- `examples/manual_tracer.rs`
- `examples/provider_wrappers.rs`
- `examples/otel_bridge.rs`

## Contract check

```bash
node scripts/check-canonical-contract.mjs
```
