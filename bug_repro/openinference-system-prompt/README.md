<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# OpenInference `llm.system` Reproduction

This standalone crate sends an Anthropic-shaped request through Relay's public
LLM lifecycle and OpenInference subscriber. It prints the relevant exported
attributes, then asserts the intended OpenInference mapping.

Run from this directory:

```bash
cargo run
```

Current Relay behavior fails the first assertion because `llm.provider` is
missing. The preceding output also shows that the system prompt is written to
`llm.system` and omitted from `llm.input_messages`.

Observed on `main` at `cd46f56c`:

```text
llm.provider=<missing>
llm.system=You are a concise assistant.
llm.input_messages.0.message.role=user
llm.input_messages.0.message.content=Hello
llm.input_messages.1.message.role=<missing>
llm.input_messages.1.message.content=<missing>

assertion `left == right` failed
  left: None
 right: Some("anthropic")
```
