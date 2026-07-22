// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use nemo_relay::api::llm::{LlmCallEndParams, LlmCallParams, LlmRequest, llm_call, llm_call_end};
use nemo_relay::observability::openinference::OpenInferenceSubscriber;
use opentelemetry_sdk::trace::{InMemorySpanExporterBuilder, SdkTracerProvider};
use serde_json::{Value, json};

const REQUESTED_MODEL: &str = "switchyard";
const RESPONSE_MODEL: &str = "claude-opus-4-7";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let subscriber = OpenInferenceSubscriber::from_tracer_provider(provider, "repro");
    subscriber.register("openinference-response-model-name-repro")?;

    let request = LlmRequest {
        headers: serde_json::Map::new(),
        content: json!({
            "model": REQUESTED_MODEL,
            "input": "Report the model that served this request.",
            "temperature": 0.2
        }),
    };
    let handle = llm_call(
        LlmCallParams::builder()
            .name("openai.responses")
            .request(&request)
            .build(),
    )?;
    llm_call_end(
        LlmCallEndParams::builder()
            .handle(&handle)
            .response(json!({
                "id": "resp_1",
                "object": "response",
                "model": RESPONSE_MODEL,
                "status": "completed",
                "output": [{
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "output_text", "text": "Done"}]
                }]
            }))
            .build(),
    )?;

    subscriber.deregister("openinference-response-model-name-repro")?;
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

    let model_name = attributes
        .get("llm.model_name")
        .map(String::as_str)
        .unwrap_or("<missing>");
    let invocation_parameters = attributes
        .get("llm.invocation_parameters")
        .map(String::as_str)
        .unwrap_or("{}");
    let invocation_parameters_json: Value = serde_json::from_str(invocation_parameters)?;
    let invocation_model = invocation_parameters_json
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");

    println!("request.model={REQUESTED_MODEL}");
    println!("response.model={RESPONSE_MODEL}");
    println!("llm.model_name={model_name}");
    println!("llm.invocation_parameters={invocation_parameters}");
    println!("llm.invocation_parameters.model={invocation_model}");

    assert_eq!(model_name, RESPONSE_MODEL);
    assert_eq!(invocation_model, REQUESTED_MODEL);
    Ok(())
}
