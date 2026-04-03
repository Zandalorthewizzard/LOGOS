---
id: logos-provider-research-report
title: LOGOS Provider Research Report
type: research
status: active
tags: [research, providers, codex, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-research-requirements.md],
    [2026-03-30-preliminary-provider-research-scan.md],
    [2026-03-30-web-source-acquisition-notes.md],
    [2026-03-30-source-inventory-normalization.md],
    [2026-03-30-logos-codex-evidence-appendix.md]
  ]
depends_on: [[2026-03-30-logos-research-requirements.md]]
generated: false
source_of_truth: working-doc
outline:
  [
    "Executive summary",
    "Confirmed architecture facts",
    "Codex extraction surface audit",
    "Host/daemon constraint audit",
    "Provider semantics matrix",
    "Persistence/resume findings",
    "Tool-loop findings",
    "Recommended decisions for Logos 0.1",
    "Unknowns and prototype targets",
    "Appendix of evidence"
  ]
---

# LOGOS Provider Research Report

## Executive Summary

The source capture and code audit point to one practical conclusion: Logos 0.1 should keep the OpenAI Responses path canonical, then add one strict validation adapter rather than trying to homogenize all providers up front.

The most important asymmetries are now visible:

1. OpenAI Responses is the only source in this set that is clearly stateful-by-design and already ties together responses, streaming, built-in tools, connectors/MCP, and structured outputs.
2. OpenRouter is best treated as a stateless compatibility and validation layer. It is useful for boundary testing, but it should not be used as the semantic source of truth.
3. Gemini and Z.AI both expose OpenAI-compatible layers, but they also have native capability surfaces that do not map 1:1 to OpenAI wire semantics.
4. LM Studio is valuable as a local/dev validation target because it exposes both OpenAI-compatible endpoints and native stateful behavior.
5. Codex internals already separate event streaming, approval handling, session metadata, MCP tool calling, and process/path bootstrapping. That separation should be preserved rather than flattened.

## Confirmed Architecture Facts

### 1. Codex event streaming is not a best-effort text pipe

The in-process app-server client classifies some events as lossless and others as droppable. Transcript events and completion signals are treated as correctness-critical, while output deltas and progress events are best-effort.

- The lossless classification is explicit in [`app-server-client/src/lib.rs`](/C:/Work/proj/LOGOS/input-sources-temp/codex-upstream/codex-rs/app-server-client/src/lib.rs).
- The worker disables the stream once the consumer falls behind or the queue is saturated.

Implication: Logos cannot safely reduce this to a generic text stream without risking broken completion semantics.

### 2. Session metadata is first-class

The app-server client records `session_source` in thread metadata, and the test client verifies that caller-provided session source is applied.

- [`app-server-client/src/lib.rs`](/C:/Work/proj/LOGOS/input-sources-temp/codex-upstream/codex-rs/app-server-client/src/lib.rs)
- [`app-server-test-client/src/lib.rs`](/C:/Work/proj/LOGOS/input-sources-temp/codex-upstream/codex-rs/app-server-test-client/src/lib.rs)

Implication: session origin is not incidental; it is part of the state model.

### 3. Resume and approval are separate control surfaces

The test client has distinct commands for resume, watch, command approval, patch approval, no-approval paths, and multi-approval flows.

- [`app-server-test-client/src/lib.rs`](/C:/Work/proj/LOGOS/input-sources-temp/codex-upstream/codex-rs/app-server-test-client/src/lib.rs)

Implication: resume semantics and approval semantics should not be merged into one generic `chat turn` abstraction.

### 4. MCP tool execution preserves conversation correlation

The MCP tool runner injects the original request ID as the Codex submission sub-id and returns `threadId` in structured content so the caller can resume.

- [`mcp-server/src/codex_tool_runner.rs`](/C:/Work/proj/LOGOS/input-sources-temp/codex-upstream/codex-rs/mcp-server/src/codex_tool_runner.rs)

Implication: tool-call correlation IDs are part of the protocol, not just logging metadata.

### 5. Startup/path bootstrapping is session-scoped

The arg0 helper creates a per-session temp path entry, keeps it locked for the lifetime of the process, and injects helper binaries into `PATH`.

- [`arg0/src/lib.rs`](/C:/Work/proj/LOGOS/input-sources-temp/codex-upstream/codex-rs/arg0/src/lib.rs)

Implication: child process re-exec and tool dispatch are runtime responsibilities, not application-layer conveniences.

## Confirmed Negative Findings

### 1. Z.AI JS/TS SDK is not currently verified as an official SDK

The research pass did not find a canonical Z.AI docs page that names an official JS/TS SDK repo. The available official material points JS/TS developers toward OpenAI-compatible usage instead of a native Z.AI Node SDK.

Implication: Logos should not block on a Z.AI JS/TS adapter until a verified official repo appears.

### 2. No public canonical minimal persistence schema was found

The public docs describe API semantics and continuation primitives, but not a vendor-neutral minimal persistence schema for multi-provider adapter runtimes.

Implication: persistence schema is design work informed by API semantics, not a discovered official artifact.

## Codex Extraction Surface Audit

| Module | Why it matters | Preserve or wrap first | Evidence |
|---|---|---|---|
| `app-server-client/src/lib.rs` | Owns event delivery policy, session source wiring, request draining, and approval response flow | Preserve core behavior; wrap only at the outer transport boundary | [`event_requires_delivery`](#appendix-of-evidence), `InProcessAppServerClient` |
| `app-server-test-client/src/lib.rs` | Contains executable proofs for resume, approval, follow-up, and streaming behavior | Keep as regression harness; do not refactor away before adapter boundaries are stable | `CliCommand`, `resume_message_v2`, `thread_resume_follow`, `trigger_zsh_fork_multi_cmd_approval` |
| `mcp-server/src/codex_tool_runner.rs` | Bridges MCP `tools/call` into Codex sessions and back into structured tool responses | Preserve and adapt carefully; this is the tool-loop choke point | `create_call_tool_result_with_thread_id`, `run_codex_tool_session`, `run_codex_tool_session_inner` |
| `mcp-server/src/lib.rs` | Bootstraps the MCP server and config/runtime context | Wrap around the host boundary, not inside the tool loop | `run_main` |
| `arg0/src/lib.rs` | Owns session-scoped PATH injection and helper-dispatch bootstrapping | Preserve as-is until a replacement bootstrap is fully designed | `Arg0PathEntryGuard`, `prepend_path_entry_for_codex_aliases` |
| `config/src/state.rs` | Owns config-layer origin tracking and merge semantics | Preserve and expose via a narrow interface | `origins` |

## Host / Daemon Constraint Audit

The current code points to a host-shell plus daemon-core split.

1. `mcp-server::run_main` eagerly loads CLI overrides and config before runtime work starts. That means config resolution is an early host concern, not a late adapter concern.
2. `arg0` mutates `PATH` once per session and keeps helper binaries reachable for child re-exec. This is process bootstrap logic.
3. `app-server-client` treats the server event queue as something that must be drained promptly. If the consumer is slow, request rejection is preferred over hanging approval flows.
4. `mcp-server` routes approval requests back to the caller and ignores some MCP-native events that do not matter to the session loop.

Implication for Logos: the host shell should remain the orchestration surface, while the daemon owns session state, event delivery, and tool execution.

## Provider Semantics Matrix

| Provider / mode | State model | Streaming | Tools / MCP | Structured outputs | Reasoning / thinking | Continuation / resume | Error / limits | Logos implication |
|---|---|---|---|---|---|---|---|---|
| OpenAI Responses | Stateful-by-design | Native streaming docs captured | Built-in tools, MCP/connectors docs captured | Structured outputs docs captured | Responses docs include reasoning-oriented surfaces | `previous_response_id` and stateful continuation are first-class | Separate docs captured for reference | Canonical path for Logos 0.1 |
| OpenRouter Responses | Stateless facade | Streaming docs captured | Tool calling docs captured | Structured outputs docs captured | Reasoning docs exist but reasoning-tokens detail remains `UNKNOWN` | No server-side state; client must own journal | Errors and limits docs captured | Best validation adapter, not canonical state owner |
| Gemini native + OpenAI compat | Mixed | OpenAI compat and native streaming docs captured | Native function calling docs captured | Structured output docs captured | Thinking docs captured | Native continuation is not identical to OpenAI | Compatibility and native docs differ by mode | Good secondary provider, but keep capability flags explicit |
| Z.AI OpenAI compat + native | Mixed | Stream-tool docs captured | Function calling and MCP server calling docs captured | Structured output docs captured | Thinking mode / deep thinking docs captured | Context caching docs captured | Rate-limit docs captured | Strong semantic asymmetry; keep native features separate |
| LM Studio OpenAI compat + native | Stateful in native docs; compat endpoint also supports continuation | OpenAI endpoint docs captured | MCP docs captured | Endpoint comparison docs captured | Reasoning semantics are local-model dependent | `previous_response_id` support captured | Endpoint comparison / REST docs captured | Good dev-only validation target |

### Notes on the matrix

- `OpenRouter reasoning tokens` remains `UNKNOWN` because the current source capture did not yield a canonical doc page that clearly defined the field.
- `Z.AI JS/TS SDK` remains `UNKNOWN` and should be treated as absent until a verified official repository is found.
- For Gemini, the OpenAI-compatible surface and the native API surface should be treated as separate modes.

## Persistence / Resume Findings

To resume faithfully, Logos must persist more than `conversation text`.

Minimum durable record model:

1. Provider identity and mode, including whether the turn was stateful or stateless.
2. Model name and request configuration.
3. Session identifiers such as thread id, turn id, and any provider continuation token such as `previous_response_id`.
4. User input items, including any structured prompt parts.
5. Tool call requests, tool call IDs, and tool results.
6. Approval decisions and rejected/accepted control-flow branches.
7. Completion markers and ordering information for streamed events.

The important split is this:

- Stateful providers can keep some continuation data server-side.
- Stateless providers require Logos to keep a client-side journal that is sufficient to replay or reconstruct the next turn.

That means the canonical store should be a Logos record model, not a raw provider wire dump.

This is still a Logos design decision, not a publicly documented vendor schema. The research output is therefore a requirements set, not a claim that a universal schema already exists.

## Tool-Loop Findings

The tool loop is already a protocol boundary in Codex, not an optional feature.

Key invariants:

1. Tool call IDs must remain stable across the request, the approval flow, and the result injection.
2. Event ordering matters because some events are lossless and some are not.
3. MCP `tools/call` must return a response shape that preserves `threadId` for continuation.
4. Approval requests must stay explicit and separately handled.
5. Streamed deltas and terminal completion events must not be dropped casually.

For Logos, this means provider tool calls should be normalized only up to a capability layer. The executable runtime actions, approval handling, and resume correlation should remain explicit.

## Recommended Decisions for Logos 0.1

1. Keep OpenAI Responses canonical.
2. Use OpenRouter as the first validation adapter.
3. Keep LM Studio as a dev-only local validation target.
4. Treat Gemini and Z.AI as additional providers with explicit capability flags, not as OpenAI clones.
5. Persist a Logos-native journal for resume and tool-loop state.
6. Do not erase Codex-specific approval/session/MCP behavior to force a cleaner abstraction.

## Unknowns and Prototype Targets

### Unknowns

- OpenRouter reasoning token semantics.
- Official Z.AI JS/TS SDK repository.
- Which provider-native capabilities Logos 0.1 should expose directly versus keep behind adapter-specific flags.
- The smallest persistence schema that still resumes all required provider modes faithfully.

### Prototype targets

1. Stateful OpenAI Responses turn with resume.
2. Stateless OpenRouter turn with full client-side journal replay.
3. MCP tool-call roundtrip that preserves `threadId` and approval correlation.
4. LM Studio local validation of OpenAI-compatible continuation.

## Appendix of Evidence

See [2026-03-30-logos-codex-evidence-appendix.md](2026-03-30-logos-codex-evidence-appendix.md) for code-backed evidence blocks and snapshot inventory.
