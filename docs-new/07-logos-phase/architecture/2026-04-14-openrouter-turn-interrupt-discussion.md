---
id: logos-openrouter-turn-interrupt-discussion
title: LOGOS OpenRouter Turn Interrupt Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, interrupt, control]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Turn Interrupt Discussion

## Purpose

This document isolates `turn/interrupt` for OpenRouter beta threads.

Interrupt is a live-control surface over an active turn. It is not just another unsupported RPC flag.

## Current State

1. `turn/interrupt` is still explicitly unsupported for OpenRouter beta threads.
2. OpenRouter beta threads already support active turns, tool calls, and detached review activity.
3. There is no OpenRouter-specific cancellation path yet for an active app-server turn.

## Ownership Baseline

1. Interrupt is a LOGOS-owned runtime control surface, not an adapter concern.
2. A provider-native interrupt backend can exist later, but only as a provider-first backend under LOGOS-owned turn-control semantics.
3. OpenRouter should not become the permanent owner of interrupt semantics that later non-canonical providers will also need.

## Why Interrupt Needs Separate Attention

1. Interrupt requires real cancellation semantics, not only a client-facing error code.
2. It interacts with host operations, background terminals, and final turn notifications.
3. A fake interrupt path could leave the runtime active while the UI believes the turn stopped.

## Decision Questions

1. What exact runtime handle owns cancellation for an active OpenRouter beta turn?
2. What notification sequence should the app-server emit after interrupt?
3. How should interrupt interact with any still-running background terminal state?

## Current Likely Path

1. Reuse the existing app-server turn lifecycle expectations as far as the OpenRouter beta lane can support them honestly.
2. Keep cancellation host and runtime-owned rather than provider-native.
3. Prefer a shared LOGOS control owner with provider policy hooks over an OpenRouter-only interrupt stack.
4. Separate interrupt semantics from background-terminal cleanup semantics.

## Closeout Expectations

1. Interrupt should stop the active OpenRouter beta turn honestly.
2. The app-server should emit a coherent final state after interruption.
3. Remaining coupling with background terminals should be explicit.

## Evidence

- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::turn_interrupt`
  lines: `7621-7689`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `Op`
  lines: `215-221`
- path: `../../../codex-rs/app-server/README.md`
  symbol: `turn interrupt`
  lines: `560-586`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-02-logos-host-daemon-transport-spec.md]]
- [[2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
