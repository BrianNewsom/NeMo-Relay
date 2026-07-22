<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# OpenInference Response Model Name Reproduction

This standalone crate sends an OpenAI Responses request through Relay's public
LLM lifecycle and OpenInference subscriber. The request uses the routing alias
`switchyard`, while the response identifies the actual serving model as
`claude-opus-4-7`.

Run from this directory:

```bash
cargo run
```

Current Relay behavior retains the request alias as `llm.model_name` and omits
the requested model from `llm.invocation_parameters`:

```text
request.model=switchyard
response.model=claude-opus-4-7
llm.model_name=switchyard
llm.invocation_parameters={"temperature":0.2}
llm.invocation_parameters.model=<missing>
```

The correct span should use the response-reported model for `llm.model_name`
and retain the request alias as the invocation parameter `model`. This matches
the [OpenInference semantic conventions](https://arize-ai.github.io/openinference/spec/semantic_conventions.html#system-and-model-identification)
and the response-first fallback order in the [OpenInference LangChain tracer](https://github.com/Arize-ai/openinference/blob/main/python/instrumentation/openinference-instrumentation-langchain/src/openinference/instrumentation/langchain/_tracer.py).
