---
id: logos-opencode-goose-architecture-reference
title: LOGOS OpenCode And Goose Architecture Reference
type: research
status: active
tags: [research, competitors, opencode, goose, architecture, parity, logos]
related:
  [
    [README.md],
    [../architecture/2026-04-02-logos-runtime-contract-spec.md],
    [../architecture/2026-04-02-logos-semantics-and-guarantees-matrix.md],
    [../architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md],
    [../architecture/2026-04-02-logos-provider-adapter-profiles.md],
    [2026-03-30-logos-provider-research-report.md],
    [2026-03-30-provider-comparison-matrix.md]
  ]
depends_on:
  [
    [../architecture/2026-04-02-logos-runtime-contract-spec.md],
    [../architecture/2026-04-02-logos-semantics-and-guarantees-matrix.md],
    [../architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md],
    [../architecture/2026-04-02-logos-provider-adapter-profiles.md],
    [2026-03-30-logos-provider-research-report.md]
  ]
generated: false
source_of_truth: working-doc
outline:
  [
    "Purpose",
    "Scope",
    "Short answer",
    "OpenCode reference",
    "Goose reference",
    "Cross-tool comparison for LOGOS",
    "Parity decision for LOGOS",
    "Follow-up gaps",
    "Evidence"
  ]
---

# LOGOS OpenCode And Goose Architecture Reference

## Purpose

- Turn competitor research into a decision-grade reference for LOGOS adapter and runtime planning.
- Capture what OpenCode and Goose actually implement at the runtime, provider, tool, permission, and session layers.
- Answer one practical question for LOGOS 0.1:
  - can non-`Responses` adapters still reach competitor-grade UX and capability coverage?

## Scope

- This document uses official docs and current upstream source snapshots captured on `2026-04-02` under `input-sources-temp/opencode-upstream/` and `input-sources-temp/goose-upstream/`.
- It focuses on runtime core placement, provider strategy, tool/MCP boundary, permissions, sessions, subagents, and multi-model policy.
- It does not score model quality, pricing, or general product polish.

## Short Answer

- If `parity` means invisible OpenAI `Responses` semantic parity, the answer is `no`.
- If `parity` means current competitor-grade user experience on non-canonical providers, the answer is `yes`, conditionally.
- OpenCode and Goose both get most of their cross-provider value from runtime-owned UX surfaces:
  - orchestration shell
  - tool governance
  - permission control
  - session continuity
  - multi-agent and multi-model routing
- Neither one reads as a universal semantic-core implementation for continuation, stream ontology, or deterministic multi-provider tool-loop parity.

## OpenCode Reference

### Runtime Shape

- OpenCode explicitly runs as `TUI + server`.
- The server is not an incidental sidecar:
  - it exposes an OpenAPI `3.1` spec
  - it generates an SDK
  - it supports multiple clients
  - it exposes an SSE event stream at `/event`
- The `/tui` endpoint is used to drive the TUI remotely, including IDE plugin scenarios.

### Provider Strategy

- OpenCode positions provider support around `AI SDK + Models.dev`, not around one Logos-like semantic kernel.
- It supports `75+` providers and local models through config and model metadata.
- Custom provider config is explicit about transport shape:
  - `@ai-sdk/openai-compatible` for `/v1/chat/completions`
  - `@ai-sdk/openai` for `/v1/responses`
- In code, the provider layer merges `Models.dev` metadata with local config, then loads bundled providers or dynamic local/npm providers.

### Agent And Permission Layer

- OpenCode separates `primary agents` and `subagents`.
- The key architectural point is not the labels themselves, but permission-envelope separation:
  - `Build` is the unrestricted baseline
  - `Plan` is intentionally restricted
  - task permissions can govern which subagents can be invoked
  - example specialized agents include `Review`, `Debug`, and `Docs`
- This is a useful reference for LOGOS because it proves that different UX modes can be implemented above one shared runtime without pretending that all agents have the same authority.

### Rules And Skills

- OpenCode uses `AGENTS.md` as the primary project instruction surface.
- It resolves rule files by precedence across project, global, and Claude-compatible fallbacks.
- It also allows additional instruction files, including remote URLs.
- `SKILL.md` discovery is on-demand via a native skill tool, with project/global lookup and pattern-based permissions.
- This is directly relevant to LOGOS because it shows a mature split between:
  - always-loaded project governance
  - optional capability injection

### MCP Boundary

- OpenCode treats MCP servers as first-class external tool sources.
- It supports:
  - local MCP servers
  - remote MCP servers
  - OAuth for remote MCP
  - global and per-agent MCP enablement
- In code, MCP tools are converted into AI SDK tools, remote transports prefer `StreamableHTTP` and can fall back to `SSE`, and tools are cached/status-tracked per server.

### Runtime Hotspots That Matter For LOGOS

- `session/processor.ts` handles structured runtime events such as:
  - `tool-input-start`
  - `tool-call`
  - `tool-result`
  - `tool-error`
  - `start-step`
  - `finish-step`
  - retry notifications
- It also asks for permission when repeated tool calls look like a doom loop.
- `session/prompt.ts` makes several runtime-authority decisions that are highly relevant to LOGOS:
  - structured output is enforced via a dedicated tool contract
  - plan mode is runtime-governed, not just prompt wording
  - the loop keeps running when providers return `stop` while tool calls still exist
- `openai-compatible-chat-language-model.ts` buffers tool-call argument deltas until the JSON becomes parsable, then emits a normalized tool call.
- `server/routes/event.ts` exposes a normalized event stream with `server.connected`, heartbeats, and bus events.

### What To Borrow From OpenCode

- runtime-as-server split
- OpenAPI and SDK surface for runtime core
- agent-specific permission envelopes
- project rules plus on-demand skills
- MCP integration as an external tool ecosystem
- event bus and SSE surface as internal-runtime reference

### What Not To Over-Apply From OpenCode

- OpenCode is strong evidence for provider connectivity and runtime UX structure.
- It is weak evidence for a universal multi-provider semantic core.
- Its provider story is deliberately AI-SDK-centric and transport-shape-aware.
- That is useful, but it is not the same as preserving continuation semantics, stream ontology, or deterministic tool-loop behavior across materially different providers.

## Goose Reference

### Runtime Shape

- Goose documents a three-part architecture:
  - interface
  - agent
  - extensions
- The agent owns the interactive loop.
- Extensions expose tools and capabilities to the agent.
- MCP is the primary interoperability boundary for those extensions.

### Extension And Tool Boundary

- Goose's `Extensions Design` docs are one of the clearest public references for a tool boundary.
- The core contract is an `Extension` trait with:
  - name
  - description
  - instructions
  - tools
  - status
  - `call_tool`
- The main architectural lesson for LOGOS is that Goose treats tool integration as a unified boundary with explicit extension-owned state, not as ad hoc provider-specific JSON glue.

### Provider And Protocol Strategy

- Goose supports many providers, but the shape of support matters.
- The built-in OpenAI provider explicitly supports OpenAI-compatible endpoints.
- ACP agents are also supported as providers, with goose extensions passed through as MCP servers.
- Deprecated CLI providers are still documented, but Goose now recommends ACP providers instead.
- The CLI provider docs are especially important because they make an architectural admission explicit:
  - pass-through provider reuse is useful
  - it adds session management and workflow integration
  - but it does not fully support Goose features
  - and it does not expose the Goose extension ecosystem
- This is a close analog to the LOGOS question about non-`Responses` adapters:
  - useful UX parity is possible without semantic parity
  - but the boundary and the downgrade must be honest

### Governance, Permissions, And Continuity

- Goose uses YAML config files for provider/model selection, extensions, tool permissions, secrets, and runtime permission decisions.
- It exposes explicit permission modes:
  - autonomous
  - manual approval
  - smart approval
  - chat only
- It also layers fine-grained tool permissions on top.
- Goose projects track working directories and associated session IDs for continuity.
- Current docs say projects store path, last instruction, and session ID in `projects.json`.
- Current code shows deeper persistence in `sessions.db`, with session rows carrying at least:
  - `working_dir`
  - `provider_name`
  - `model_config_json`
  - `goose_mode`
  - `extension_data`
- Session copy/import paths preserve provider, model config, mode, and conversation content.

### Multi-Model And Subagents

- Goose explicitly supports multi-model strategies inside one session.
- It documents lead/worker and planner model configurations.
- Subagents can be internal or external.
- Internal subagents inherit session context and extensions, but run with restricted operations.
- External subagents can be other tools or providers, including Codex via MCP.

### ACP Boundary

- Goose supports ACP in two directions:
  - goose as an ACP server
  - ACP agents as providers
- Current ACP server code exposes mode and model as session config options and translates tool calls, tool results, and permission prompts into ACP session updates.
- The current code also rejects MCP `SSE` transport in favor of `streamable_http`, which is a strong present-day signal that Goose is standardizing the transport boundary rather than keeping older custom SSE assumptions alive.

### Provider Hotspots That Matter For LOGOS

- `provider_registry.rs` shows a registry model with builtin, preferred, and declarative providers.
- `openai_compatible.rs` still targets `chat/completions`, not a universal continuation or `Responses` contract.
- `formats/openai.rs` is especially relevant:
  - it normalizes `reasoning_content`
  - it parses tool calls into internal message content
  - it validates and repairs tool schemas
  - it safely repairs some broken JSON argument payloads
  - it parses streaming SSE chunks into internal messages
- `session_manager.rs` shows that Goose continuity is increasingly client/runtime-owned through storage rather than provider-native continuation.

### What To Borrow From Goose

- extension and tool abstraction
- permission modes plus per-tool permission policy
- project and session continuity
- multi-model planning and lead/worker policy
- ACP boundary treatment as a first-class integration seam

### What Not To Over-Apply From Goose

- Goose is not a proof that one runtime can erase provider differences safely.
- Its strongest patterns are around:
  - extensions
  - permissions
  - continuity
  - protocol boundaries
- That is different from a canonical semantic core for continuation, event ontology, and tool-loop determinism.

## Cross-Tool Comparison For LOGOS

| Axis | OpenCode | Goose | LOGOS implication |
| --- | --- | --- | --- |
| Runtime nucleus | TUI plus server with OpenAPI and SSE event stream | Interface plus agent plus extensions, plus ACP surfaces | LOGOS host/daemon runtime-as-service direction is already competitive |
| Provider philosophy | AI SDK plus Models.dev plus config-driven endpoint selection | Registry plus declarative providers plus ACP providers | LOGOS still needs an adapter kernel because connectivity is not semantic parity |
| Non-canonical provider path | Mostly transport and config normalization | Compatibility providers, ACP delegation, and explicit degraded pass-through | LOGOS can support non-`Responses` providers honestly without pretending they are OpenAI |
| Tool ecosystem | Built-in tools plus MCP servers as tools | Extensions over MCP, ACP providers pass MCP through | Runtime-owned tool loop remains the safest common denominator |
| Approval governance | Per-agent permissions and task permissions | Global modes plus tool permissions plus provider permission bridges | Approval UX can stay runtime-owned even when provider semantics differ |
| Session continuity | Server/session/event model with multi-client support | Projects plus SQLite session persistence plus copy/import/export | LOGOS journal and replay can be competitive without provider-native state |
| Multi-agent strategy | Primary agents and subagents with permission envelopes | Internal and external subagents, including external tool ecosystems | Adapter class and authority boundaries should stay explicit in UX |
| Semantic parity stance | Useful multi-provider runtime, but not a universal semantic-core claim | Useful protocol and extension boundary, but not universal provider sameness | LOGOS should continue separating `canonical`, `translated`, `emulated`, and `unknown` semantics |

## Parity Decision For LOGOS

### 1. Define The Right Parity Target

- `Canonical parity`:
  - OpenAI `Responses` semantic parity
  - continuation handle parity
  - strongest tool-loop and hosted-tool parity
- `Competitive parity`:
  - user-visible runtime UX comparable to OpenCode and Goose on non-canonical providers
- `Feature-exposure parity`:
  - planning
  - approvals
  - session continuity
  - MCP and tool access
  - subagents
  - multi-model routing

Only the OpenAI canonical adapter should claim `canonical parity`.

### 2. Direct Answer

- OpenRouter-like stateless adapters can still reach `competitive parity`.
- Gemini-, Anthropic-, and Z.AI-like native adapters can also reach `competitive parity`.
- They cannot honestly claim invisible `Responses` parity unless the provider really exposes those semantics.

### 3. Why The Answer Is Yes At The Competitor Level

- OpenCode already demonstrates that a strong runtime UX can sit above mixed provider surfaces and mixed endpoint shapes.
- Goose already demonstrates that session management, permissions, extension control, and protocol delegation can be runtime-owned even when provider backends are heterogeneous.
- In both systems, the durable value is mostly above the provider wire format.

### 4. Conditions For A Real Yes

- LOGOS runtime must own:
  - normalized event stream
  - approval state
  - portable tool policy
  - durable journal and replay
  - mode and model control
- Adapter class and runtime tier must stay visible:
  - canonical
  - compat
  - native
  - mixed-mode
  - translated
  - emulated
- Provider-native features must be capability-gated, not silently treated as portable defaults.
- Contract tests must cover:
  - stream parsing
  - tool-call argument buffering and repair
  - restart resume
  - approval bridging
  - downgrade behavior

### 5. What Would Make The Answer No

- claiming `previous_response_id`-like semantics where the provider does not have them
- hiding replay-based continuation behind fake server-state language
- silently bypassing runtime approvals for provider-native tools
- flattening native provider semantics into lowest-common-denominator wire compatibility

## Follow-Up Gaps

### OpenCode

- Current source snapshot is strong on runtime and MCP surfaces.
- Remaining deeper work:
  - tighter persistence and resume audit
  - exact provider abstraction and drift-handling contract
  - explicit tests for streaming and broken tool arguments beyond current code hotspots

### Goose

- Current source snapshot is strong on extensions, ACP, permissions, and session continuity.
- Remaining deeper work:
  - current `Responses` and OpenAI-specific semantics coverage across providers
  - exact ACP-over-HTTP product direction beyond the current code and docs surface
  - tighter mapping of provider-native formats outside the OpenAI-compatible path

## Evidence

- path: `../../input-sources-temp/opencode-upstream/packages/web/src/content/docs/server.mdx`
  symbol: `How it works`, `Spec`
  lines: `47-80`
- path: `../../input-sources-temp/opencode-upstream/packages/web/src/content/docs/providers.mdx`
  symbol: `Providers`, `OpenAI-compatible providers`
  lines: `9-34`, `1890-2030`
- path: `../../input-sources-temp/opencode-upstream/packages/web/src/content/docs/agents.mdx`
  symbol: `Primary agents`, `Permissions`, `Task permissions`, `Examples`
  lines: `22-67`, `420-445`, `571-600`, `692-696`
- path: `../../input-sources-temp/opencode-upstream/packages/web/src/content/docs/rules.mdx`
  symbol: `Rules`, `Precedence`, `Custom Instructions`
  lines: `6-28`, `65-131`
- path: `../../input-sources-temp/opencode-upstream/packages/web/src/content/docs/skills.mdx`
  symbol: `Agent Skills`, `Understand discovery`, `Configure permissions`
  lines: `6-30`, `105-132`
- path: `../../input-sources-temp/opencode-upstream/packages/web/src/content/docs/mcp-servers.mdx`
  symbol: `Enable`, `Local`, `Remote`, `OAuth`, `Global`, `Per agent`
  lines: `6-40`, `72-92`, `131-168`, `294-390`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/provider/provider.ts`
  symbol: `fromModelsDevModel`, `provider initialization`
  lines: `26-35`, `894-906`, `983-1058`, `1388-1421`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/session/processor.ts`
  symbol: `SessionProcessor.create`
  lines: `82-110`, `153-230`, `457-474`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/session/prompt.ts`
  symbol: `structured output`, `plan mode`, `tool loop`
  lines: `57-65`, `313-316`, `1369-1375`, `1460-1519`, `1529-1536`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/server/routes/event.ts`
  symbol: `EventRoutes`
  lines: `11-80`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/mcp/index.ts`
  symbol: `convertMcpTool`, `connectRemote`, `connectLocal`, `tools`, `authenticate`
  lines: `132-170`, `273-327`, `380-435`, `611-632`, `713-767`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/provider/sdk/copilot/chat/openai-compatible-chat-language-model.ts`
  symbol: `tool call streaming`
  lines: `537-672`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/goose-architecture/goose-architecture.md`
  symbol: `goose Components`, `Interoperability with Extensions`, `Agent Client Protocol`, `Interactive Loop`
  lines: `10-59`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/goose-architecture/extensions-design.md`
  symbol: `Extension`, `Tools`, `Architecture`
  lines: `7-70`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/getting-started/providers.md`
  symbol: `Available Providers`, `ACP Providers`, `Using Custom OpenAI Endpoints`
  lines: `14-22`, `41-53`, `64-73`, `262-271`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/guides/cli-providers.md`
  symbol: `CLI provider deprecation and limitations`, `Message Translation`, `Response Processing`
  lines: `11-17`, `43-47`, `370-398`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/guides/acp-providers.md`
  symbol: `ACP providers`, `Codex ACP`, `Gemini ACP`
  lines: `10-15`, `36-45`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/guides/config-files.md`
  symbol: `Configuration Files`, `Environment Variables`, `Extensions Configuration`
  lines: `9-21`, `34-54`, `112-150`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/guides/goose-permissions.md`
  symbol: `Permission Modes`, `CLI Provider Permission Integration`
  lines: `11-32`, `143-155`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/guides/managing-projects.md`
  symbol: `Managing Projects`
  lines: `7-19`, `36-40`, `77-80`, `108-113`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/guides/multi-model/index.mdx`
  symbol: `Multi-Model Configuration`
  lines: `13-29`
- path: `../../input-sources-temp/goose-upstream/documentation/docs/guides/subagents.mdx`
  symbol: `subagents`, `internal subagents`, `external subagents`, `restricted operations`
  lines: `27`, `76-89`, `184-229`, `326-345`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/providers/provider_registry.rs`
  symbol: `ProviderRegistry`
  lines: `1-40`, `79-115`, `187-203`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/providers/openai_compatible.rs`
  symbol: `OpenAiCompatibleProvider`, `stream`, `stream_openai_compat`
  lines: `21-27`, `102-127`, `236-249`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/providers/formats/openai.rs`
  symbol: `response_to_message`, `validate_tool_schemas`, `response_to_streaming_message`
  lines: `320-323`, `368-473`, `544-623`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/providers/utils.rs`
  symbol: `safely_parse_json`
  lines: `465-520`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `Session`, `SessionManager`, `SessionStorage`
  lines: `23-24`, `52-65`, `262-307`, `510-550`, `713-729`, `1439-1457`
- path: `../../input-sources-temp/goose-upstream/crates/goose-acp/src/server.rs`
  symbol: `MCP server conversion`, `mode and model state`, `tool and permission updates`
  lines: `87-109`, `316-377`, `600-720`, `889-899`, `1215-1240`, `1732-1739`
- path: `../architecture/2026-04-02-logos-runtime-contract-spec.md`
  symbol: `Core Position`, `Logos Runtime Owns`, `Logos Runtime Guarantees`
  lines: `32-57`, `87-126`
- path: `../architecture/2026-04-02-logos-semantics-and-guarantees-matrix.md`
  symbol: `Why This Exists`, `Decision`, `Matrix`
  lines: `27-38`, `53-59`, `74-80`
- path: `../architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md`
  symbol: `Purpose`, `Responses Reuse Evaluation`, `Provider-Specific Likely Gaps`
  lines: `31-41`, `51-65`, `95-101`
- path: `../architecture/2026-04-02-logos-provider-adapter-profiles.md`
  symbol: `OpenAI Canonical Adapter`, `OpenRouter Validation Adapter`, `Gemini Adapter`, `Anthropic Adapter`, `Z.AI Adapter`, `LM Studio Adapter`
  lines: `34-45`, `54-73`, `75-94`, `96-114`, `116-135`, `137-154`

## Links

- [[README.md]]
- [[../architecture/2026-04-02-logos-runtime-contract-spec.md]]
- [[../architecture/2026-04-02-logos-semantics-and-guarantees-matrix.md]]
- [[../architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md]]
- [[../architecture/2026-04-02-logos-provider-adapter-profiles.md]]
- [[2026-03-30-logos-provider-research-report.md]]
- [[2026-03-30-provider-comparison-matrix.md]]
