---
id: logos-openrouter-thread-unarchive-support-discussion
title: LOGOS OpenRouter Thread Unarchive Support Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, unarchive, thread-lifecycle]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-14-openrouter-thread-archive-support-discussion.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-14-openrouter-thread-archive-support-discussion.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Thread Unarchive Support Discussion

## Purpose

This document isolates `thread/unarchive` for OpenRouter beta threads.

Unarchive is the inverse durability operation to archive, but it adds its own product question: what exact thread state should be restored on the OpenRouter beta lane after an archived thread is returned to active visibility?

## Current State

1. `thread/unarchive` is still explicitly unsupported for OpenRouter beta threads.
2. The canonical path restores a rollout-backed thread from the archived directory into normal sessions storage.
3. OpenRouter beta threads currently do not have a corresponding archived representation or restore path.

## Ownership Baseline

1. Unarchive is a LOGOS-owned thread-lifecycle restore operation, not an adapter concern.
2. OpenRouter should not become the permanent owner of restore semantics that later providers will also need.
3. Provider-native behavior may accelerate restore later, but lifecycle truth remains LOGOS-owned.

## Why Unarchive Needs Separate Attention

1. Unarchive depends on whatever durable artifact archive writes.
2. The restored thread must remain an OpenRouter beta thread, not silently convert into a core/Codex thread.
3. Visibility, status, metadata, and later resume behavior all depend on the exact restored state.

## Decision Questions

1. After unarchive, should the restored OpenRouter thread come back as loaded, not-loaded, or another explicit beta-thread status?
2. How should the restored thread be represented in `thread/list`, `thread/read`, and later app-server operations?
3. What minimum persisted state is required so unarchive is useful rather than merely decorative?

## Current Likely Path

1. Keep the restored thread on the OpenRouter beta lane with LOGOS-owned persisted state.
2. Move toward a shared LOGOS session-state layer for archive and unarchive semantics rather than a permanent OpenRouter-only restore path.
3. Reuse canonical app-server notifications and list semantics where honest.
4. Separate durable restore semantics from later loaded-thread reconnect or resume semantics.

## Closeout Expectations

1. Unarchive should restore a coherent OpenRouter beta thread object.
2. `thread/list`, archived filters, and `thread/read` should reflect the restored thread honestly.
3. The restored thread must not silently degrade into a generic core thread.

## Evidence

- path: `../../../codex-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `ThreadUnarchiveParams`
  lines: `2885-2952`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::thread_unarchive`
  lines: `3164-3354`
- path: `../../../codex-rs/app-server/tests/suite/v2/thread_unarchive.rs`
  symbol: `thread_unarchive_moves_rollout_back_into_sessions_directory`
  lines: `33-166`
- path: `2026-04-14-openrouter-thread-archive-support-discussion.md`
  symbol: `Current Likely Path`
  lines: `31-37`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-14-openrouter-thread-archive-support-discussion.md]]
- [[2026-04-11-phase-4-persistence-resume-discussion.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
