---
id: logos-openrouter-thread-archive-support-discussion
title: LOGOS OpenRouter Thread Archive Support Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, archive, thread-lifecycle]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Thread Archive Support Discussion

## Purpose

This document isolates `thread/archive` for OpenRouter beta threads.

Archive is not only a UI flag. It is a durable thread-lifecycle transition with storage, visibility, and teardown implications.

## Current State

1. `thread/archive` is still explicitly unsupported for OpenRouter beta threads.
2. The canonical path archives rollout-backed threads by moving a materialized thread file into the archived area.
3. OpenRouter beta threads currently live primarily as LOGOS-owned in-memory state plus replay semantics, not as normal Codex rollout threads.

## Ownership Baseline

1. Archive is a LOGOS-owned thread-lifecycle and persistence concern, not an adapter concern.
2. OpenRouter should not remain the permanent owner of archive and unarchive mechanics that later providers will also need.
3. Any provider-native fast path is secondary to LOGOS-owned lifecycle truth.

## Why Archive Needs Separate Attention

1. OpenRouter archive cannot honestly reuse rollout-file assumptions if those artifacts do not exist.
2. Archiving affects thread list visibility, stale subscriptions, and later unarchive/resume behavior.
3. A fake archive that only removes a loaded thread from memory would not be enough for durable lifecycle parity.

## Decision Questions

1. What durable artifact or persisted state should represent an archived OpenRouter beta thread?
2. Should archiving require a materialized LOGOS-owned thread snapshot or journal-backed export first?
3. How should archive interact with a currently active OpenRouter thread and its subscriptions?

## Current Likely Path

1. Keep the archived state LOGOS-owned instead of pretending the thread is a normal core rollout thread.
2. Move toward a shared LOGOS session-state layer for archive and unarchive rather than a permanent OpenRouter-only file workflow.
3. Reuse existing app-server lifecycle expectations where possible, but keep persistence semantics explicit.
4. Treat archive and unarchive as a persistence-and-lifecycle problem, not merely a `ThreadStatus` toggle.

## Closeout Expectations

1. Archive should create a durable archived representation for OpenRouter beta threads.
2. Archived filter and thread visibility behavior should remain coherent.
3. Archive should clean up loaded-thread state and stale subscriptions honestly.

## Evidence

- path: `../../../codex-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `ThreadArchiveParams`
  lines: `2802-2884`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::thread_archive`
  lines: `2525-2588`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::archive_thread_common`
  lines: `5876-5985`
- path: `../../../codex-rs/app-server/tests/suite/v2/thread_archive.rs`
  symbol: `thread_archive_requires_materialized_rollout`
  lines: `31-162`
- path: `2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md`
  symbol: `What Stateless Means For LOGOS`
  lines: `23-38`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-11-phase-4-persistence-resume-discussion.md]]
- [[2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
