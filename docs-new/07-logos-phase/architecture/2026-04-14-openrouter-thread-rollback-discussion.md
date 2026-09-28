---
id: logos-openrouter-thread-rollback-discussion
title: LOGOS OpenRouter Thread Rollback Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, rollback, thread-lifecycle]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Thread Rollback Discussion

## Purpose

This document isolates `thread/rollback` for OpenRouter beta threads.

Rollback looks like a thread-management convenience, but for OpenRouter it is really a question about authoritative turn boundaries, persisted history ownership, and what can be safely removed or rewound on a replay-first provider path.

## Current State

1. `thread/rollback` is still explicitly unsupported for OpenRouter beta threads.
2. OpenRouter beta threads now have fork and review state plus replay-first history, but no rollback contract.
3. Turn and item state are already visible in the app-server thread model, but rollback semantics are not yet defined for the OpenRouter lane.

## Ownership Baseline

1. Rollback is a LOGOS-owned session and history semantics problem, not an adapter concern.
2. Provider-native rollback backends may exist later, but they should plug into LOGOS-owned lifecycle truth rather than replace it.
3. OpenRouter should not become the permanent owner of rollback semantics that later non-canonical providers will also need.

## Why Rollback Needs Separate Attention

1. Rollback must define what gets removed, what stays durable, and what later resume or fork behavior should see.
2. For OpenRouter, rollback may affect replayable history, tool records, and detached review or fork lineage.
3. A naive rollback could make the visible thread history diverge from the actual replay source of truth.

## Decision Questions

1. What is the rollback unit for OpenRouter beta: last turn, last user turn, or another boundary?
2. How should rollback interact with tool calls, tool results, and review or forked threads?
3. Does rollback require journal-backed rewriting, in-memory masking, or another LOGOS-owned mechanism?

## Current Likely Path

1. Treat rollback as a journal and history semantics problem, not only as app-server routing.
2. Define the rollback unit explicitly before implementation.
3. Keep any eventual OpenRouter rollback path additive and LOGOS-owned.
4. Prefer a shared LOGOS session-state owner with provider policy hooks over an OpenRouter-only rollback stack.

## Closeout Expectations

1. Rollback must be defined against explicit turn and history boundaries.
2. The visible thread state after rollback must match the future replay source of truth.
3. If phase 6 cannot support rollback honestly, the beta support statement must call that out explicitly.

## Evidence

- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::thread_rollback`
  lines: `3362-3434`
- path: `2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
  symbol: `Compaction Interaction Rules`
  lines: `129-134`
- path: `2026-04-11-phase-4-persistence-resume-discussion.md`
  symbol: `Durable Resume`
  lines: `1-260`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-11-phase-4-journal-record-taxonomy-discussion.md]]
- [[2026-04-11-phase-4-persistence-resume-discussion.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
