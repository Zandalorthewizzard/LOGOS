---
id: logos-session-state-layer-extraction-discussion
title: LOGOS Session-State Layer Extraction Discussion
type: discussion
status: active
tags: [logos, architecture, extraction, session-state, phase-6]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
    [../implementation/2026-04-14-phase-6-session-state-layer-extraction-strict-spec.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Session-State Layer Extraction Discussion

## Purpose

This document defines the extraction boundary implied by the OpenRouter phase-6 work.

The goal is not to widen phase 6 into a speculative rewrite. The goal is to identify which mechanics currently living in `openrouter_beta_threads.rs` are actually LOGOS-owned session-state concerns and should move into a shared layer, and which mechanics should remain provider-sensitive for now.

## Problem Statement

`openrouter_beta_threads.rs` currently mixes four different responsibilities:

1. LOGOS-owned thread lifecycle and persistence
2. LOGOS-owned host operations
3. app-server protocol and notification shaping
4. OpenRouter-specific session bootstrap and policy behavior

That mix was acceptable as a proving lane.

It is not a good long-term ownership model because the same pattern will recur for other non-canonical providers, while some control surfaces still remain provider-policy-sensitive.

## Current Extraction Diagnosis

### Already Shared Enough

These mechanics already live at the right layer:

1. provider wire transport and capability truth in `logos-provider-openrouter`
2. shared turn, history, replay, tool, approval, journal, and compaction workflow ownership in `logos-session-engine`

### Over-owned By The OpenRouter Lane Today

These mechanics are LOGOS-owned in nature but currently sit inside the OpenRouter proving lane:

1. loaded-thread registry and common thread-state envelope
2. persisted non-canonical thread snapshot format and storage paths
3. read, list, archive, and unarchive lifecycle behavior
4. host operations for shell and background-terminal cleanup
5. thread visibility and subscription-facing app-server helpers

### Still Provider-Policy-Sensitive

These mechanics are not ready for blind generic extraction yet:

1. provider session bootstrap from config
2. provider-specific tool registration and session wrapper setup
3. resume semantics beyond already-loaded live-thread reconnect
4. fork and review orchestration
5. rollback semantics
6. compaction backend policy
7. interrupt, steering, and realtime semantics

## Current Consensus

1. A new LOGOS-owned session-state layer should be introduced under `codex-rs/` as an additive crate.
2. The working target name is `codex-rs/logos-session-state/` unless implementation proves a better narrow name.
3. The first extraction wave should move only the clear shared mechanics, not the full OpenRouter runtime lane.
4. OpenAI canonical path does not need to migrate into this layer in phase 6.
5. Provider-sensitive controls should remain in provider lanes until LOGOS defines the right policy hooks.

## Move Now

The first extraction package should move these mechanics out of `openrouter_beta_threads.rs`:

1. non-canonical thread registry ownership
2. non-canonical persisted thread snapshot format and path helpers
3. read and list of loaded plus persisted non-canonical threads
4. archive and unarchive lifecycle behavior for non-canonical threads
5. shell-command and background-terminal host-operation ownership

These are the clearest cases of duplicated future session-state logic.

## Move Later

The second extraction wave should reconsider these areas only after the policy surface is explicit:

1. durable resume beyond already-loaded live threads
2. fork and review orchestration
3. rollback
4. notification shaping for all turn and item variants
5. generic thread item projection from provider-agnostic internal state

## Keep Provider-Sensitive For Now

These mechanics should stay outside the first extraction package:

1. `OpenRouterBetaSession::from_config` and adapter construction
2. provider-specific validated-model and warning posture
3. provider-specific override validation
4. provider-specific continuation behavior
5. compaction backend execution
6. interrupt, steering, and realtime behavior

## Why This Boundary Is Correct

1. It removes the clearest duplicated LOGOS-owned mechanics without pretending all provider controls are identical.
2. It keeps adapters thin while preserving provider-specific truth.
3. It avoids a broad `codex-core` rewrite.
4. It preserves the canonical OpenAI path as-is.

## Non-goals

1. no forced migration of OpenAI canonical thread mechanics into the new layer during phase 6
2. no fake generic control plane that hides provider differences
3. no extraction of compaction, rollback, interrupt, steering, or realtime before their semantics are fixed
4. no replacement of `logos-session-engine` as the owner of turn, history, tool, and journal workflow

## Closeout Expectations

1. The new shared layer should own the clear non-canonical thread lifecycle and host-op mechanics.
2. `openrouter_beta_threads.rs` should shrink toward provider-specific session bootstrap and policy behavior.
3. Future providers should reuse the same extracted layer instead of cloning the OpenRouter proving lane.

## Evidence

- path: `2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Ownership Layers`
  lines: `43-103`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThreadManager`
  lines: `73-1180`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `archive_thread`
  lines: `481-624`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `background_terminals_clean`
  lines: `626-645`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `shell_command`
  lines: `646-769`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `fork_thread`
  lines: `1075-1178`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `start_review`
  lines: `1180-1769`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `135-193, 290-506`
- path: `../../../codex-rs/logos-provider-openrouter/src/adapter.rs`
  symbol: `ProviderAdapter for OpenRouterAdapter`
  lines: `64-126`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
- [[2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]]
- [[../implementation/2026-04-14-phase-6-session-state-layer-extraction-strict-spec.md]]
