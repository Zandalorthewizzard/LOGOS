---
id: logos-openrouter-thread-resume-support-discussion
title: LOGOS OpenRouter Thread Resume Support Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, resume, app-server]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Thread Resume Support Discussion

## Purpose

This document isolates the remaining OpenRouter `thread/resume` decisions for phase 6.

The current OpenRouter beta path already supports reconnecting to a loaded in-memory thread by `thread_id`, but that is not the same thing as full durable resume for a replay-first provider.

## Current State

1. Loaded live OpenRouter beta threads can reconnect through `thread_id` mode.
2. `history` mode is explicitly rejected.
3. `path` mode is explicitly rejected.
4. Resume config overrides are explicitly rejected.
5. The OpenRouter beta lane still relies on LOGOS-owned replay semantics rather than provider-owned continuation state.

## Ownership Baseline

1. `thread/resume` is part of LOGOS-owned session and continuity semantics, not adapter-owned wire behavior.
2. Provider-native continuation is an optional fast path, but OpenRouter currently has no native server-managed continuation to lean on.
3. Any OpenRouter resume fallback must stay replay-first and LOGOS-owned rather than pretending to be provider-native.

## Why Resume Needs Separate Attention

1. `resume` is the user-facing continuity feature for interrupted or later-returned work.
2. For OpenRouter, `resume` is inseparable from replay semantics because the provider is not the durable source of truth.
3. The already-delivered loaded-thread reconnect path does not answer what should happen after process restart, archival, or later durable recovery.

## Decision Questions

1. Does phase 6 require only loaded-thread reconnect, or durable restart-safe resume for OpenRouter beta?
2. If durable resume is required, what is the source of truth: journal, rollout-like artifact, or another LOGOS-owned persisted thread state?
3. Should `history` mode stay rejected for `0.1 beta`, or should OpenRouter support a bounded version of it?
4. Should `path` mode remain out of scope for OpenRouter beta even if archive/unarchive becomes supported?

## Current Likely Path

1. Keep the current loaded-thread reconnect path as the minimum already-proven behavior.
2. If durable resume is required, derive it from a LOGOS-owned session-state layer rather than a permanent OpenRouter-only thread manager.
3. Treat any broader resume mode as a persistence and replay decision, not merely an app-server routing task.

## Closeout Expectations

1. Phase 6 should say explicitly whether OpenRouter beta resume means loaded-thread reconnect only or durable restart-safe continuation.
2. If durable resume is required, it needs its own honest proof path.
3. If broader resume is out of scope, the beta support statement must say so explicitly.

## Evidence

- path: `../../../codex-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `ThreadResumeParams`
  lines: `2650-2733`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::thread_resume`
  lines: `4009-4074`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `validate_openrouter_resume_params`
  lines: `2056-2104`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `OpenRouter`
  lines: `140-144`
- path: `2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md`
  symbol: `What Stateless Means For LOGOS`
  lines: `23-38`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[2026-04-11-phase-4-persistence-resume-discussion.md]]
- [[2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
