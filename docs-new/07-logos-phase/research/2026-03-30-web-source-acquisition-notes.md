---
id: logos-research-web-source-acquisition-notes
title: Web Source Acquisition Notes
type: research
status: active
tags: [research, web, sources, providers, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-research-requirements.md],
    [2026-03-30-preliminary-provider-research-scan.md]
  ]
depends_on: [[2026-03-30-logos-research-requirements.md]]
generated: false
source_of_truth: working-doc
outline: []
---

# Web Source Acquisition Notes

This document records what was successfully fetched from the web during initial Logos source gathering, what failed, and what should be acquired manually to strengthen later research.

## Confirmed Successful Fetches

### Z.AI

1. `https://docs.z.ai/llms.txt`
   - confirmed as a usable documentation index
   - useful for discovering page inventory before deeper provider research
2. `https://docs.z.ai/guides/develop/http/introduction`
   - confirmed general HTTP endpoint base:
     - `https://api.z.ai/api/paas/v4/`
   - confirmed separate coding endpoint exists:
     - `https://api.z.ai/api/coding/paas/v4`
   - confirmed Bearer auth model
   - confirmed basic chat completion and streaming examples are publicly documented
3. `https://lmstudio.ai/docs/app/api/endpoints/openai`
   - confirmed LM Studio OpenAI-compatible endpoints overview is reachable
   - confirmed it documents `/v1/responses`
   - confirmed LM Studio explicitly states Codex is supported because it implements `POST /v1/responses`

## Confirmed Partial Or Failed Fetches

### OpenAI

1. `https://platform.openai.com/docs/api-reference/responses`
   - fetch failed due to response size limit
2. `https://platform.openai.com/docs/api-reference/responses-streaming`
   - fetch failed due to response size limit

### OpenRouter

1. attempted Responses API URLs returned `404`
2. exact canonical URLs should be re-verified manually before final research

### Gemini

1. `https://ai.google.dev/gemini-api/docs/openai`
   - fetch redirected too many times in this environment
   - page likely exists, but needs manual acquisition or another fetch route

### LM Studio

1. `https://lmstudio.ai/docs/app/api/endpoints/openai-responses`
   - attempted path returned `404`
   - exact page path should be confirmed manually from the docs site navigation

## Local Source Cache Already Acquired

Stored under `../../../../input-sources-temp/`:

1. `codex-upstream/`
2. `ollama/`
3. `openai-python/`
4. `openai-node/`
5. `openai-java/`
6. `openai-dotnet/`
7. `anthropic-sdk-python/`
8. `anthropic-sdk-typescript/`
9. `google-python-genai/`
10. `mcp-typescript-sdk/`
11. `openrouter-ai-sdk-provider/`

## Highest-Value Manual Acquisitions Still Needed

1. OpenAI docs snapshots for Responses, streaming, built-in tools, and MCP/connectors.
2. Exact OpenRouter Responses docs and official SDK repos if available.
3. Gemini OpenAI-compat docs snapshot and, ideally, JS SDK source.
4. Z.AI SDK repos or a docs snapshot if public repos are not easy to locate.
5. LM Studio docs snapshots for OpenAI-compatible endpoints, Responses, and MCP/tool behavior.

## Evidence

- path: `../../../../input-sources-temp/openai-python`
  symbol: `cloned repository`
  lines: 1-1
- path: `../../../../input-sources-temp/codex-upstream`
  symbol: `cloned repository`
  lines: 1-1
- path: `../../../../input-sources-temp/ollama`
  symbol: `cloned repository`
  lines: 1-1

## Links

- [[README.md]]
- [[2026-03-30-logos-research-requirements.md]]
- [[2026-03-30-preliminary-provider-research-scan.md]]
