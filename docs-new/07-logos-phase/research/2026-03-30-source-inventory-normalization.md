---
id: logos-source-inventory-normalization
title: Source Inventory Normalization
type: research
status: active
tags: [research, sources, inventory, providers, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-research-requirements.md],
    [2026-03-30-preliminary-provider-research-scan.md],
    [2026-03-30-web-source-acquisition-notes.md],
    [../../../input-sources-temp/provider-source-inventory.md]
  ]
depends_on: [[2026-03-30-logos-research-requirements.md]]
generated: false
source_of_truth: working-doc
outline: []
---

# Source Inventory Normalization

This document defines the local on-disk shape for provider source capture in
`input-sources-temp/`.

## Goal

Make the next discovery pass cheap:

1. every provider folder has a stable `index.md`;
2. each captured source has a canonical URL recorded close to the file;
3. source status is visible without reopening the web;
4. missing official repos are marked `UNKNOWN`, not guessed.

## Normalized fields

Use these fields for each source entry:

- `provider`
- `source kind`
- `canonical URL`
- `local path`
- `status`
- `why it matters for Logos`

## Status values

- `confirmed`
- `partial`
- `unknown`
- `needs-refresh`

## Folder contract

Each folder under `input-sources-temp/` should contain:

- `index.md`
- optional `*.source.json`
- one or more snapshots or repo checkouts

## Recommended snapshot formats

- `*.html` for canonical page capture
- `*.md` when a page can be cleanly reduced to markdown
- `*.pdf` when visual fidelity matters
- `README.md` and selected source files for official repos

## Normalized inventory

| Provider | Source kind | Canonical URL | Local path | Status | Why it matters for Logos |
| --- | --- | --- | --- | --- | --- |
| OpenAI | official docs | https://platform.openai.com/docs/api-reference/responses/tutorials-and-guides | `input-sources-temp/openai-docs` | partial | Primary Responses contract for adapter-first semantics |
| OpenAI | official docs | https://platform.openai.com/docs/guides/streaming-responses | `input-sources-temp/openai-docs` | partial | Streaming event parsing and tool-call deltas |
| OpenAI | official docs | https://platform.openai.com/docs/guides/tools?api-mode=responses | `input-sources-temp/openai-docs` | partial | Built-in tools and MCP entry point |
| OpenAI | official docs | https://platform.openai.com/docs/guides/structured-outputs?api-mode=chat | `input-sources-temp/openai-docs` | partial | Structured outputs and refusal behavior |
| OpenRouter | official docs | https://openrouter.ai/docs/api/reference/overview/ | `input-sources-temp/openrouter-docs` | partial | OpenAI-like facade semantics |
| OpenRouter | official docs | https://openrouter.ai/docs/features/tool-calling | `input-sources-temp/openrouter-docs` | partial | Tool calling model and model support |
| OpenRouter | official docs | https://openrouter.ai/docs/features/structured-outputs | `input-sources-temp/openrouter-docs` | partial | Schema enforcement and output shape |
| OpenRouter | sdk repo | https://github.com/OpenRouterTeam/typescript-sdk | `input-sources-temp/openrouter-ts-sdk` | partial | Official TS SDK surface |
| OpenRouter | sdk repo | https://github.com/OpenRouterTeam/python-sdk | `input-sources-temp/openrouter-python-sdk` | partial | Official Python SDK surface |
| Gemini | official docs | https://ai.google.dev/gemini-api/docs/openai | `input-sources-temp/google-gemini-docs` | partial | OpenAI compatibility boundary |
| Gemini | official docs | https://ai.google.dev/gemini-api/docs/function-calling | `input-sources-temp/google-gemini-docs` | partial | Native tool calling |
| Gemini | official docs | https://ai.google.dev/gemini-api/docs/structured-output | `input-sources-temp/google-gemini-docs` | partial | Native schema outputs |
| Gemini | sdk repo | https://github.com/googleapis/js-genai | `input-sources-temp/google-js-genai` | partial | Official JS/TS SDK |
| Z.AI | official docs | https://docs.z.ai/ | `input-sources-temp/zai-docs` | partial | Platform entry point |
| Z.AI | official docs | https://docs.z.ai/guides/capabilities/function-calling | `input-sources-temp/zai-docs` | partial | Tool calling semantics |
| Z.AI | official docs | https://docs.z.ai/guides/capabilities/stream-tool | `input-sources-temp/zai-docs` | partial | Streamed tool output |
| Z.AI | official docs | https://docs.z.ai/guides/capabilities/thinking-mode | `input-sources-temp/zai-docs` | partial | Thinking / deep thinking semantics |
| Z.AI | sdk repo | UNKNOWN | `input-sources-temp/zai-js-sdk` | unknown | No verified official JS/TS repo found yet |
| Z.AI | sdk repo | https://github.com/zai-org/z-ai-sdk-python | `input-sources-temp/zai-python-sdk` | partial | Official Python SDK surface |
| LM Studio | official docs | https://lmstudio.ai/docs/app/api/endpoints/openai/ | `input-sources-temp/lm-studio-docs` | partial | OpenAI-compatible endpoint matrix |
| LM Studio | official docs | https://lmstudio.ai/docs/developer/rest/stateful-chats | `input-sources-temp/lm-studio-docs` | partial | Stateful continuation semantics |
| LM Studio | official docs | https://lmstudio.ai/docs/app/mcp/ | `input-sources-temp/lm-studio-docs` | partial | MCP support in the app |

## Acquisition order

1. OpenAI docs.
2. OpenRouter docs.
3. Gemini docs and JS SDK.
4. Z.AI docs and SDKs.
5. LM Studio docs snapshots.

## Resume point

Next agent should read:

1. this document;
2. `input-sources-temp/provider-source-inventory.md`;
3. `docs-new/07-logos-phase/research/2026-03-30-web-source-acquisition-notes.md`.

