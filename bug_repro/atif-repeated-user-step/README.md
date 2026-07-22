<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# ATIF Repeated User Step Reproduction

This standalone crate registers Relay's public `AtifExporter`, emits two
complete OpenAI Responses LLM lifecycles, and exports the in-memory trajectory.
The second request carries the original user message plus a function call and
function-call output, but no new user message.

Run from this directory:

```bash
cargo run
```

Current Relay behavior successfully exports the trajectory, then fails the
one-user-step assertion:

```text
user_step_count=2
unique_user_message_count=1
user_messages=["Fix pip","Fix pip"]

assertion `left == right` failed
  left: [String("Fix pip"), String("Fix pip")]
 right: [String("Fix pip")]
```

The same `handle_llm_start` reconstruction logic is present in `0.6.0-rc.3`,
`0.7.0-alpha.20260721`, and `main` at `cd46f56c`. Every LLM start creates a
`source="user"` step and extracts the latest user message from the request's
complete history, causing historical prompts to be emitted again.
