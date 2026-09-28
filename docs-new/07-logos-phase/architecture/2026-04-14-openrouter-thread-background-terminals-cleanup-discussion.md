---
id: logos-openrouter-thread-background-terminals-cleanup-discussion
title: LOGOS OpenRouter Thread Background Terminals Cleanup Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, background-terminals, host-ops]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-14-openrouter-thread-shell-command-support-discussion.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-14-openrouter-thread-shell-command-support-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Thread Background Terminals Cleanup Discussion

## Purpose

This document isolates `thread/backgroundTerminals/clean` for OpenRouter beta threads.

The cleanup surface is easy to over-simplify because the API only exposes cleanup, not the full background-terminal lifecycle. The real question is whether OpenRouter beta threads can own and terminate the relevant host process state honestly.

## Current State

1. `thread/backgroundTerminals/clean` is still explicitly unsupported for OpenRouter beta threads.
2. The canonical path already maps this operation to `Op::CleanBackgroundTerminals`.
3. OpenRouter beta threads do not yet expose a real cleanup path for command/background process state.

## Ownership Baseline

1. Background-terminal cleanup is a LOGOS-owned host/runtime concern, not an adapter concern.
2. OpenRouter should not become the long-term owner of background-process bookkeeping that later providers would need too.
3. Provider-specific differences should appear only as policy limits or capability truth, not as a duplicated cleanup stack.

## Why Cleanup Needs Separate Attention

1. Returning `{}` without terminating anything would be false parity.
2. Cleanup semantics overlap with shell commands, interrupts, and long-running host activity.
3. OpenRouter beta needs either a real host-state owner for these processes or an explicit limitation.

## Decision Questions

1. What exact host-state owner should cleanup target for OpenRouter beta threads?
2. Does the OpenRouter beta lane need additional bookkeeping for background process state, or can it reuse the existing machinery directly?
3. What should happen if cleanup is requested on a thread with no active background terminal state?

## Current Likely Path

1. Reuse the canonical host cleanup machinery if OpenRouter threads can own compatible process state.
2. Keep cleanup host-owned and additive rather than provider-native.
3. Treat the target owner as a shared LOGOS session-state layer, even if OpenRouter remains the first proving lane.
4. Tie proof of cleanup to a real command/background activity path, not only to an empty-thread no-op.

## Closeout Expectations

1. Cleanup should actually terminate or clear the relevant background terminal state for OpenRouter threads.
2. The proof path should include a real OpenRouter thread that previously created command/background activity.
3. Any remaining limitation should be explicit about which subset of background-terminal behavior is unsupported.

## Evidence

- path: `../../../codex-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `ThreadBackgroundTerminalsCleanParams`
  lines: `2988-2995`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::thread_background_terminals_clean`
  lines: `3476-3517`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `Op::CleanBackgroundTerminals`
  lines: `215-221`
- path: `../../../codex-rs/app-server/README.md`
  symbol: `thread/backgroundTerminals/clean`
  lines: `578-586`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-02-logos-host-daemon-transport-spec.md]]
- [[2026-04-14-openrouter-thread-shell-command-support-discussion.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
