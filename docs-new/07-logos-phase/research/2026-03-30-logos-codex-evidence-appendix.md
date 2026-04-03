---
id: logos-codex-evidence-appendix
title: LOGOS Codex Evidence Appendix
type: research
status: active
tags: [research, evidence, codex, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-provider-research-report.md],
    [2026-03-30-logos-research-requirements.md],
    [2026-03-30-source-inventory-normalization.md]
  ]
depends_on: [[2026-03-30-logos-provider-research-report.md]]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Codex Evidence Appendix

## Code-backed Evidence Blocks

### 1. Event delivery and lossless stream classification

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\app-server-client\src\lib.rs`
- symbol: `event_requires_delivery`
- lines: `87-110`
- why it matters: establishes that transcript and completion events are lossless and must not be dropped casually.

### 2. Session source is part of thread metadata

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\app-server-client\src\lib.rs`
- symbol: `InProcessClientStartArgs`
- lines: `268-276`
- why it matters: confirms session origin is tracked explicitly and feeds the metadata model.

### 3. Event streaming cannot be allowed to stall

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\app-server-client\src\lib.rs`
- symbol: `InProcessAppServerClient::start`
- lines: `400-507`
- why it matters: shows the worker must keep draining events and may disable the stream if the consumer falls behind.

### 4. Approval-flow backpressure is a correctness concern

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\app-server-client\src\lib.rs`
- symbol: `next_event`
- lines: `655-660`
- why it matters: documents that callers must drain promptly or pending approvals may hang.

### 5. Resume and approval commands are separate test surfaces

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\app-server-test-client\src\lib.rs`
- symbol: `CliCommand`
- lines: `175-225`
- why it matters: the test harness intentionally separates resume, watch, approval, and follow-up flows.

### 6. Multi-approval flow is explicitly tested

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\app-server-test-client\src\lib.rs`
- symbol: `trigger_zsh_fork_multi_cmd_approval`
- lines: `720-800`
- why it matters: approval behavior is not a single callback; multiple approvals can occur per turn.

### 7. Resume behavior is a concrete protocol path

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\app-server-test-client\src\lib.rs`
- symbol: `resume_message_v2` and `thread_resume_follow`
- lines: `802-857`
- why it matters: resume is a first-class path and not a special-case afterthought.

### 8. MCP call results carry thread continuation data

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\mcp-server\src\codex_tool_runner.rs`
- symbol: `create_call_tool_result_with_thread_id`
- lines: `32-60`
- why it matters: the tool response shape includes `threadId` in `structured_content`.

### 9. MCP tool call correlation uses the original request id

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\mcp-server\src\codex_tool_runner.rs`
- symbol: `run_codex_tool_session`
- lines: `96-125`
- why it matters: the request ID is reused for correlation so tool calls can be resumed and matched.

### 10. Tool runner handles approval events and terminal completion explicitly

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\mcp-server\src\codex_tool_runner.rs`
- symbol: `run_codex_tool_session_inner`
- lines: `192-326`
- why it matters: approval, patch handling, errors, and completion are distinct protocol branches.

### 11. Session-scoped PATH injection is runtime bootstrap logic

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\arg0\src\lib.rs`
- symbol: `Arg0PathEntryGuard`
- lines: `20-40`
- why it matters: per-session helper dispatch is retained via a guard object.

### 12. Helper binaries are injected into PATH per session

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\arg0\src\lib.rs`
- symbol: `prepend_path_entry_for_codex_aliases`
- lines: `245-343`
- why it matters: child-process dispatch, temp-root cleanup, and PATH mutation are part of the process bootstrap contract.

### 13. Config origin tracking is explicit

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\config\src\state.rs`
- symbol: `origins`
- lines: `229-243`
- why it matters: merged config keeps field-origin tracking separate from requirement sources.

### 14. MCP server bootstrap loads config eagerly

- path: `C:\Work\proj\LOGOS\input-sources-temp\codex-upstream\codex-rs\mcp-server\src\lib.rs`
- symbol: `run_main`
- lines: `56-70`
- why it matters: CLI overrides and config loading happen before runtime work proceeds.

## External Source Capture Inventory

### OpenAI Docs

- canonical URL: `https://platform.openai.com/docs/guides/responses`
- source type: official docs
- local snapshot: `input-sources-temp/openai-docs/responses-overview.html`
- why it matters: canonical Responses API surface for Logos OpenAI-first behavior

- canonical URL: `https://platform.openai.com/docs/guides/streaming-responses`
- source type: official docs
- local snapshot: `input-sources-temp/openai-docs/streaming-responses.html`
- why it matters: streaming event model and partial output behavior

- canonical URL: `https://platform.openai.com/docs/guides/tools`
- source type: official docs
- local snapshot: `input-sources-temp/openai-docs/tools.html`
- why it matters: built-in tool support and tool orchestration semantics

- canonical URL: `https://platform.openai.com/docs/guides/mcp`
- source type: official docs
- local snapshot: `input-sources-temp/openai-docs/docs-mcp.html`
- why it matters: connectors / MCP integration

- canonical URL: `https://platform.openai.com/docs/guides/structured-outputs`
- source type: official docs
- local snapshot: `input-sources-temp/openai-docs/structured-outputs.html`
- why it matters: schema-constrained outputs for safe adapters

### OpenRouter Docs

- canonical URL: `https://openrouter.ai/docs/api-reference/overview`
- source type: official docs
- local snapshot: `input-sources-temp/openrouter-docs/overview.html`
- why it matters: Responses overview and canonical API entrypoint

- canonical URL: `https://openrouter.ai/docs/api-reference/streaming`
- source type: official docs
- local snapshot: `input-sources-temp/openrouter-docs/home.html`
- why it matters: streaming reference

- canonical URL: `https://openrouter.ai/docs/guides/features/tool-calling`
- source type: official docs
- local snapshot: `input-sources-temp/openrouter-docs/tool-calling-2.html`
- why it matters: tool calling semantics

- canonical URL: `https://openrouter.ai/docs/guides/features/structured-outputs`
- source type: official docs
- local snapshot: `input-sources-temp/openrouter-docs/structured-outputs.html`
- why it matters: structured output support

- canonical URL: `https://openrouter.ai/docs/api-reference/errors-and-debugging`
- source type: official docs
- local snapshot: `input-sources-temp/openrouter-docs/errors-and-debugging.html`
- why it matters: provider error model

- canonical URL: `https://openrouter.ai/docs/api-reference/limits`
- source type: official docs
- local snapshot: `input-sources-temp/openrouter-docs/limits.html`
- why it matters: rate limits and request limits

### Google Gemini Docs and SDK

- canonical URL: `https://ai.google.dev/gemini-api/docs/openai`
- source type: official docs
- local snapshot: `input-sources-temp/google-gemini-docs/openai-compat.html`
- why it matters: OpenAI compatibility guidance

- canonical URL: `https://ai.google.dev/gemini-api/docs/function-calling`
- source type: official docs
- local snapshot: `input-sources-temp/google-gemini-docs/function-calling.html`
- why it matters: native function/tool calling

- canonical URL: `https://ai.google.dev/gemini-api/docs/structured-output`
- source type: official docs
- local snapshot: `input-sources-temp/google-gemini-docs/structured-output.html`
- why it matters: structured outputs

- canonical URL: `https://ai.google.dev/gemini-api/docs/thinking`
- source type: official docs
- local snapshot: `input-sources-temp/google-gemini-docs/thinking.html`
- why it matters: thinking / reasoning semantics

- canonical URL: `https://ai.google.dev/gemini-api/docs/caching`
- source type: official docs
- local snapshot: `input-sources-temp/google-gemini-docs/caching.html`
- why it matters: context caching / continuation semantics

- canonical URL: `https://github.com/google-gemini/generative-ai-js`
- source type: official SDK repo
- local snapshot: `input-sources-temp/google-js-genai/repo-src`
- why it matters: official JS/TS SDK

### Z.AI Docs and SDK

- canonical URL: `https://docs.z.ai/guides/develop/python/introduction.md`
- source type: official docs
- local snapshot: `input-sources-temp/zai-python-sdk/introduction.html`
- why it matters: official Python SDK introduction and usage

- canonical URL: `https://docs.z.ai/guides/develop/http/introduction.md`
- source type: official docs
- local snapshot: `input-sources-temp/zai-docs/http-introduction.html`
- why it matters: native HTTP API documentation

- canonical URL: `https://docs.z.ai/guides/capabilities/function-calling.md`
- source type: official docs
- local snapshot: `input-sources-temp/zai-docs/function-calling.html`
- why it matters: native function calling

- canonical URL: `https://docs.z.ai/guides/capabilities/stream-tool.md`
- source type: official docs
- local snapshot: `input-sources-temp/zai-docs/stream-tool.html`
- why it matters: streaming tool output

- canonical URL: `https://docs.z.ai/guides/capabilities/struct-output.md`
- source type: official docs
- local snapshot: `input-sources-temp/zai-docs/structured-output.html`
- why it matters: structured output support

- canonical URL: `https://docs.z.ai/guides/capabilities/thinking-mode.md`
- source type: official docs
- local snapshot: `input-sources-temp/zai-docs/thinking-mode.html`
- why it matters: deep thinking mode

- canonical URL: `https://docs.z.ai/guides/capabilities/cache.md`
- source type: official docs
- local snapshot: `input-sources-temp/zai-docs/cache.html`
- why it matters: context caching

- canonical URL: `https://docs.z.ai/guides/capabilities/mcp-call.md`
- source type: official docs
- local snapshot: `input-sources-temp/zai-docs/mcp-call.html`
- why it matters: MCP server calling

- canonical URL: `https://docs.z.ai/api-reference/rate-limit.md`
- source type: official docs
- local snapshot: `input-sources-temp/zai-docs/rate-limit.html`
- why it matters: rate-limit semantics

- canonical URL: `https://github.com/zai-org/z-ai-sdk-python`
- source type: official SDK repo
- local snapshot: `input-sources-temp/zai-python-sdk/repo-src`
- why it matters: official Python SDK repository

### LM Studio Docs

- canonical URL: `https://lmstudio.ai/docs/app/api/endpoints/openai/`
- source type: official docs
- local snapshot: `input-sources-temp/lm-studio-docs/openai-endpoints.html`
- why it matters: OpenAI-compatible endpoints overview

- canonical URL: `https://lmstudio.ai/docs/developer/rest`
- source type: official docs
- local snapshot: `input-sources-temp/lm-studio-docs/rest-overview.html`
- why it matters: REST surface

- canonical URL: `https://lmstudio.ai/docs/developer/rest/stateful-chats`
- source type: official docs
- local snapshot: `input-sources-temp/lm-studio-docs/stateful-chats.html`
- why it matters: stateful behavior / previous_response_id

- canonical URL: `https://lmstudio.ai/docs/developer/core/mcp`
- source type: official docs
- local snapshot: `input-sources-temp/lm-studio-docs/core-mcp.html`
- why it matters: core MCP support

- canonical URL: `https://lmstudio.ai/docs/app/mcp/`
- source type: official docs
- local snapshot: `input-sources-temp/lm-studio-docs/app-mcp.html`
- why it matters: app MCP endpoint comparison

