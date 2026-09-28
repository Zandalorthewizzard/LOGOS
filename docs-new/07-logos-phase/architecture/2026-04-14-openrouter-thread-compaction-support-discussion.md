---
id: logos-openrouter-thread-compaction-support-discussion
title: LOGOS OpenRouter Thread Compaction Support Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, compaction, replay]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-03-logos-compaction-strategy-discussion.md],
    [2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-03-logos-compaction-strategy-discussion.md],
    [2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Thread Compaction Support Discussion

## Purpose

This document isolates `thread/compact/start` for OpenRouter beta threads.

Compaction is the highest-risk remaining OpenRouter control because it changes the replay boundary for a provider that already depends on LOGOS-owned replay semantics.

## Current State

1. `thread/compact/start` is still explicitly unsupported for OpenRouter beta threads.
2. The OpenRouter observation doc already marks compaction semantics as incomplete.
3. The current OpenRouter beta lane remains replay-first and does not yet prove compaction parity.

## Ownership Baseline

1. Compaction workflow ownership belongs to LOGOS even when a provider-native compaction backend exists.
2. OpenAI native remote compaction remains a valid provider-first backend, but not the owner of product compaction semantics.
3. Any portable or replay-first compaction fallback must remain honest about checkpoint semantics and follow-up replay behavior.

## Why Compaction Needs Separate Attention

1. Compaction changes the future replay input, not only the current visible thread transcript.
2. A bad compaction implementation could corrupt follow-up turns, tool-result replay, or resume semantics.
3. Phase 6 explicitly cannot close by silently implying OpenRouter compaction parity.

## Decision Questions

1. Can OpenRouter beta support a compaction checkpoint that still preserves honest replay semantics?
2. What exact durable record should represent an OpenRouter compaction boundary?
3. Is compaction a beta limitation that must remain excluded from `LOGOS 0.1 beta`, or is it a required parity surface for the OpenRouter stage?

## Current Likely Path

1. Keep compaction out of the easy host-ops remainder and treat it as a dedicated replay and persistence decision.
2. Base any eventual implementation on LOGOS-owned checkpoint semantics with provider-first backend selection where available.
3. Do not claim fallback parity unless the LOGOS-managed backend preserves honest replay and resume semantics.
4. If not implemented in phase 6, record it as a bounded limitation rather than a silent gap.

## Closeout Expectations

1. The final phase-6 acceptance report must classify OpenRouter compaction as proven, bounded limitation, or remaining blocker.
2. No OpenRouter beta support statement should imply compaction parity unless replay after compaction is actually proven.

## Evidence

- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::thread_compact_start`
  lines: `3436-3474`
- path: `2026-04-03-logos-compaction-strategy-discussion.md`
  symbol: `Compaction Strategy`
  lines: `1-260`
- path: `2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md`
  symbol: `Risk Register`
  lines: `82-151`
- path: `2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
  symbol: `Compaction Interaction Rules`
  lines: `129-134`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-03-logos-compaction-strategy-discussion.md]]
- [[2026-04-11-phase-4-journal-record-taxonomy-discussion.md]]
- [[2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
