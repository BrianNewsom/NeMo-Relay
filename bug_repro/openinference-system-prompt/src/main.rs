// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use nemo_relay::api::llm::{LlmCallEndParams, LlmCallParams, LlmRequest, llm_call, llm_call_end};
use nemo_relay::observability::openinference::OpenInferenceSubscriber;
use opentelemetry_sdk::trace::{InMemorySpanExporterBuilder, SdkTracerProvider};
use serde_json::json;

const SYSTEM_PROMPT: &str = "You are a concise assistant.";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let subscriber = OpenInferenceSubscriber::from_tracer_provider(provider, "repro");
    subscriber.register("openinference-system-prompt-repro")?;

    let request = LlmRequest {
        headers: serde_json::Map::new(),
        content: json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 32,
            "system": SYSTEM_PROMPT,
            "messages": [{"role": "user", "content": "Hello"}]
        }),
    };
    let handle = llm_call(
        LlmCallParams::builder()
            .name("anthropic.messages")
            .request(&request)
            .build(),
    )?;
    llm_call_end(
        LlmCallEndParams::builder()
            .handle(&handle)
            .response(json!({
                "type": "message",
                "role": "assistant",
                "model": "claude-sonnet-4-20250514",
                "content": [{"type": "text", "text": "Hi"}],
                "stop_reason": "end_turn"
            }))
            .build(),
    )?;

    subscriber.deregister("openinference-system-prompt-repro")?;
    subscriber.force_flush()?;

    let spans = exporter.get_finished_spans()?;
    let attributes = spans
        .first()
        .expect("Relay should emit one LLM span")
        .attributes
        .iter()
        .map(|attribute| {
            (
                attribute.key.as_str().to_string(),
                attribute.value.to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    subscriber.shutdown()?;

    for key in [
        "llm.provider",
        "llm.system",
        "llm.input_messages.0.message.role",
        "llm.input_messages.0.message.content",
        "llm.input_messages.1.message.role",
        "llm.input_messages.1.message.content",
    ] {
        println!(
            "{key}={}",
            attributes
                .get(key)
                .map(String::as_str)
                .unwrap_or("<missing>")
        );
    }

    assert_eq!(
        attributes.get("llm.provider").map(String::as_str),
        Some("anthropic")
    );
    assert_ne!(
        attributes.get("llm.system").map(String::as_str),
        Some(SYSTEM_PROMPT)
    );
    assert_eq!(
        attributes
            .get("llm.input_messages.0.message.role")
            .map(String::as_str),
        Some("system")
    );
    assert_eq!(
        attributes
            .get("llm.input_messages.0.message.content")
            .map(String::as_str),
        Some(SYSTEM_PROMPT)
    );
    Ok(())
}
