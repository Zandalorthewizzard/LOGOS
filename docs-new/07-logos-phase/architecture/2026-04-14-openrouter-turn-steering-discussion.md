---
id: logos-openrouter-turn-steering-discussion
title: LOGOS OpenRouter Turn Steering Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, steering, control]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Turn Steering Discussion

## Purpose

This document isolates `turn/steer` for OpenRouter beta threads.

Steering is one of the least obvious control surfaces on a replay-first provider because it implies changing the direction of an already-live turn without inventing fake continuation semantics.

## Current State

1. `turn/steer` is still explicitly unsupported for OpenRouter beta threads.
2. The OpenRouter beta lane already supports replay-first follow-up turns, but not in-flight steering.
3. No current implementation claim defines what steering should mean for an active OpenRouter turn.

## Ownership Baseline

1. Steering is a LOGOS-owned control semantic, not an adapter concern.
2. A provider-native steering backend can exist later, but only behind LOGOS-owned turn semantics and explicit capability truth.
3. OpenRouter should not become the permanent owner of steering behavior that later non-canonical providers may also need.

## Why Steering Needs Separate Attention

1. Steering is a semantic control, not only a routing operation.
2. It must define whether the system edits an active turn, interrupts and restarts it, or creates another explicit branch.
3. For OpenRouter, this decision is constrained by replay-first continuity and the lack of durable provider-side turn state.

## Decision Questions

1. What exact product behavior should `turn/steer` mean on OpenRouter beta?
2. Is honest OpenRouter steering possible without first implementing interrupt plus restart semantics?
3. Should steering remain outside `0.1 beta`, or is it required to consider the OpenRouter stage complete?

## Current Likely Path

1. Treat steering as a dedicated semantic decision instead of a small implementation follow-up.
2. Avoid claiming parity until a concrete steering model is selected and tested.
3. Keep any solution above provider wires rather than faking provider-native control.
4. Prefer a shared LOGOS control owner with provider policy hooks over an OpenRouter-only steering stack.

## Closeout Expectations

1. Steering should not be implemented until its semantics are written down explicitly.
2. The final phase-6 support statement should either prove steering or exclude it honestly.

## Evidence

- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::turn_steer`
  lines: `7041-7104`
- path: `2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md`
  symbol: `What Stateless Means For LOGOS`
  lines: `23-38`
- path: `2026-04-02-logos-runtime-contract-spec.md`
  symbol: `Provider Outcomes`
  lines: `173-204`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
- [[2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
