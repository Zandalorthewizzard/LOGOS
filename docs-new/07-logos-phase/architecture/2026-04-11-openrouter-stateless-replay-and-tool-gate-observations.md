---
id: logos-openrouter-stateless-replay-and-tool-gate-observations
title: LOGOS OpenRouter Stateless Replay And Tool-Gate Observations
type: discussion
status: active
tags: [logos, openrouter, phase-6, replay, tools, risks]
related:
  [
    [README.md],
    [2026-04-02-logos-provider-openrouter-adapter-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
    [../implementation/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md],
  ]
depends_on:
  [
    [2026-04-02-logos-provider-openrouter-adapter-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [../implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Stateless Replay And Tool-Gate Observations

## Purpose

This document records the practical implications of the OpenRouter stateless model for the current phase-6 implementation lane.

It exists to prevent one specific failure mode:

1. closing the OpenRouter stage on a happy-path tool demo while leaving the actual replay and runtime risks unexamined

Phase-6 closeout must explicitly walk this observation set before the OpenRouter stage can be considered complete.

## What Stateless Means For LOGOS

For the current LOGOS architecture, OpenRouter is not treated as a stateful semantic source of truth.

That means:

1. `ContinuationHandle::ClientManagedHistory` is a mode marker, not a durable provider-side handle
2. continuity lives in LOGOS-owned normalized history and durable journal state
3. follow-up turns are reconstructed from replayable LOGOS records, not from provider-owned conversation state
4. tool-loop continuation for OpenRouter must replay `AssistantToolCall` and `ToolResult`, not only user and assistant text

This does not mean LOGOS must always send the full raw transcript forever. It means the current OpenRouter path is replay-first, and correctness is owned by the LOGOS journal and replay policy rather than by provider-side state.

## Immediate Phase-6 Implications

The narrow phase-6 tool-gate proof is the correct starting point because the main unknown is not the shared LOGOS runtime anymore. The main unknown is whether a live OpenRouter model or route can sustain the existing LOGOS tool loop through adapter-local request, event, and replay mapping.

Practical consequence:

1. phase 6 should first prove one adapter-local end-to-end tool roundtrip on OpenRouter
2. only after that proof passes should broader beta hardening widen

## Current Enabling State

The current codebase already has the minimum building blocks for the proof slice:

1. OpenRouter request mapping already serializes runtime-owned tool declarations
2. OpenRouter replay mapping already serializes `AssistantToolCall` and `ToolResult`
3. OpenRouter response mapping already recognizes `function_call` output items
4. `logos-session-engine` already builds turns from normalized accumulated history plus pending messages
5. `logos-tool-runtime` already owns tool execution and reinjection

This means the phase-6 gate is not blocked by missing shared-runtime seams by default. It is blocked only if live OpenRouter behavior cannot sustain those seams honestly.

## Risk Register

### 1. Live tool-call emission may be inconsistent

Risk:

1. a validated OpenRouter model or route may not return `function_call` items consistently enough to sustain agent-grade behavior

Why it matters:

1. the whole OpenRouter agent-beta claim depends on one honest runtime-owned tool loop, not only on text turns

Closeout expectation:

1. prove at least one stable validated model or route
2. document the validated set explicitly in the beta support statement

### 2. Tool-result replay may not be accepted reliably

Risk:

1. OpenRouter may emit a tool call but fail to continue reliably after replayed `function_call_output`

Why it matters:

1. the real tool gate is not call emission alone
2. the gate is the full loop: declare tool, receive call, execute tool, replay result, receive final continuation

Closeout expectation:

1. prove a full roundtrip through the existing LOGOS tool-runtime seam
2. prove at least one follow-up continuation after tool-result replay

### 3. Support is likely model- or route-dependent

Risk:

1. OpenRouter tool behavior is unlikely to be uniformly reliable across the whole router surface

Why it matters:

1. a provider-wide `OpenRouter supports tools` claim would be dishonest if only a subset of models or routes sustain the tool path

Closeout expectation:

1. treat OpenRouter agent support as a validated model or route set
2. keep capability truth honest at the beta support layer

### 4. Replay-first operation grows payload size and context pressure

Risk:

1. because OpenRouter is replay-first, request payload size will grow with normalized history until a replay boundary reduces it

Why it matters:

1. long sessions may become operationally fragile before the beta if replay pressure is ignored

Closeout expectation:

1. measure or at least characterize replay growth in the validated OpenRouter path
2. document practical beta limitations if replay growth is not yet mitigated

### 5. OpenRouter compaction semantics are still incomplete

Risk:

1. the current OpenRouter adapter still rejects replay of `CompactionCheckpoint`

Why it matters:

1. phase-6 must not silently claim OpenRouter compaction parity if replay after compaction is still unsupported or only partially defined

Closeout expectation:

1. either implement the required OpenRouter compaction/replay behavior
2. or explicitly scope OpenRouter beta support so compaction is excluded or clearly degraded

## Closeout Rule

The OpenRouter stage is not closable just because one happy-path tool roundtrip exists.

Before phase 6 is marked closed, the acceptance report must explicitly walk all five risks above and record for each one:

1. `proven`
2. `bounded limitation`
3. `remaining blocker`

If a risk is only a bounded limitation, the beta support statement must name it explicitly.

## Evidence

- path: `2026-04-02-logos-provider-openrouter-adapter-spec.md`
  symbol: `Responses Reuse Evaluation`
  lines: `45-91`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `OpenRouter`
  lines: `140-144`
- path: `2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md`
  symbol: `OpenRouter Tool Gate Proof Plan`
  lines: `223-249`
- path: `../../../codex-rs/logos-provider-openrouter/src/map_request.rs`
  symbol: `map_turn_request`
  lines: `72-101`
- path: `../../../codex-rs/logos-provider-openrouter/src/map_request.rs`
  symbol: `map_message`
  lines: `158-200`
- path: `../../../codex-rs/logos-provider-openrouter/src/map_request.rs`
  symbol: `map_tool_def`
  lines: `204-221`
- path: `../../../codex-rs/logos-provider-openrouter/src/responses.rs`
  symbol: `map_response`
  lines: `76-169`
- path: `../../../codex-rs/logos-provider-openrouter/src/adapter.rs`
  symbol: `ProviderAdapter for OpenRouterAdapter`
  lines: `62-87`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine::run_turn_with_tools_inner`
  lines: `300-385`
- path: `2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
  symbol: `Compaction Interaction Rules`
  lines: `129-134`

## Links

- [[README.md]]
- [[2026-04-02-logos-provider-openrouter-adapter-spec.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
- [[../implementation/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md]]
