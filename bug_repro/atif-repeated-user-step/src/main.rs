// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

use nemo_relay::api::llm::{LlmCallEndParams, LlmCallParams, LlmRequest, llm_call, llm_call_end};
use nemo_relay::api::subscriber::{deregister_subscriber, register_subscriber};
use nemo_relay::observability::atif::{AtifAgentInfo, AtifExporter};
use serde_json::{Value, json};

const USER_PROMPT: &str = "Fix pip";

fn emit_llm_call(
    request_body: Value,
    response_text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = LlmRequest {
        headers: serde_json::Map::new(),
        content: request_body,
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
                "model": "switchyard",
                "output": [{
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "output_text", "text": response_text}]
                }]
            }))
            .build(),
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = AtifExporter::new(
        "session-1".to_string(),
        AtifAgentInfo {
            name: "repro-agent".to_string(),
            version: "1.0.0".to_string(),
            model_name: Some("switchyard".to_string()),
            tool_definitions: None,
            extra: None,
        },
    );
    register_subscriber("atif-repeated-user-step-repro", exporter.subscriber())?;

    emit_llm_call(
        json!({
            "model": "switchyard",
            "input": [{
                "type": "message",
                "role": "user",
                "content": [{"type": "input_text", "text": USER_PROMPT}]
            }]
        }),
        "I will inspect the environment.",
    )?;

    emit_llm_call(
        json!({
            "model": "switchyard",
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": [{"type": "input_text", "text": USER_PROMPT}]
                },
                {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "shell",
                    "arguments": "{}"
                },
                {
                    "type": "function_call_output",
                    "call_id": "call_1",
                    "output": "pip is missing"
                }
            ]
        }),
        "I found that pip is missing.",
    )?;

    let trajectory = exporter.export()?;
    deregister_subscriber("atif-repeated-user-step-repro")?;
    let user_messages = trajectory
        .steps
        .iter()
        .filter(|step| step.source == "user")
        .map(|step| step.message.clone())
        .collect::<Vec<_>>();
    let unique_user_messages = user_messages
        .iter()
        .map(Value::to_string)
        .collect::<BTreeSet<_>>();

    println!("user_step_count={}", user_messages.len());
    println!("unique_user_message_count={}", unique_user_messages.len());
    println!("user_messages={}", serde_json::to_string(&user_messages)?);

    assert_eq!(user_messages, vec![json!(USER_PROMPT)]);
    Ok(())
}
