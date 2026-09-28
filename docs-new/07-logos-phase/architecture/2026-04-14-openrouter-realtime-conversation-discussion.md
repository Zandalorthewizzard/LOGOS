---
id: logos-openrouter-realtime-conversation-discussion
title: LOGOS OpenRouter Realtime Conversation Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, realtime, transport]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-02-logos-runtime-tier-contract-spec.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-02-logos-runtime-tier-contract-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Realtime Conversation Discussion

## Purpose

This document isolates realtime conversation operations for the OpenRouter beta lane.

Realtime is not just another control flag. It is a separate transport/runtime class with its own latency, streaming, interruption, and session-management implications.

## Current State

1. Realtime conversation operations are still explicitly unsupported for OpenRouter beta threads.
2. The current OpenRouter beta lane is built around replay-first request/response turns and tool loops.
3. No current implementation claim provides an OpenRouter realtime transport or session model.

## Ownership Baseline

1. Realtime is a LOGOS-owned transport and session-control surface above provider wires, not an adapter-only concern.
2. Any provider-native realtime backend should remain a provider-first transport backend under LOGOS-owned session and control semantics.
3. OpenRouter should not become the permanent owner of a separate realtime session stack that later providers would also need.

## Why Realtime Needs Separate Attention

1. Realtime work would introduce a different transport class than the current OpenRouter beta lane.
2. It overlaps with turn interrupt, steering, and audio or live-stream semantics.
3. Treating it as a simple follow-on RPC would understate the implementation and support burden.

## Decision Questions

1. Is realtime part of the required OpenRouter phase-6 parity target, or a later extension?
2. If realtime is required, what transport and session model should LOGOS own for OpenRouter?
3. How would realtime coexist with the current replay-first turn model?

## Current Likely Path

1. Keep realtime outside the clear host-ops remainder.
2. Treat it as a separate transport and runtime decision with its own support boundary.
3. Model any future solution as LOGOS-owned session and control orchestration with provider-first transport backends.
4. Do not imply realtime readiness from the current turn-based OpenRouter beta lane.

## Closeout Expectations

1. Realtime should be explicitly classified as proven, bounded limitation, or blocker before phase-6 closeout.
2. The OpenRouter beta support statement should not imply realtime parity unless a real transport path exists.

## Evidence

- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `openrouter_thread_operation_unsupported`
  lines: `7160-7230`
- path: `2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Host Daemon Transport`
  lines: `1-240`
- path: `2026-04-02-logos-runtime-tier-contract-spec.md`
  symbol: `Runtime Tiers`
  lines: `1-220`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-02-logos-host-daemon-transport-spec.md]]
- [[2026-04-02-logos-runtime-tier-contract-spec.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
