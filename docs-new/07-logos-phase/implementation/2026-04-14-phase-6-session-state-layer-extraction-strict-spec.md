---
id: logos-phase-6-session-state-layer-extraction-strict-spec
title: LOGOS Phase 6 Session-State Layer Extraction Strict Spec
type: spec
status: active
tags: [logos, phase-6, strict-spec, extraction, session-state]
related:
  [
    [README.md],
    [../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md],
  ]
depends_on:
  [
    [../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 6 Session-State Layer Extraction Strict Spec

## Objective

Extract the clear LOGOS-owned non-canonical thread lifecycle and host-operation mechanics from `openrouter_beta_threads.rs` into a shared additive LOGOS layer without widening the change into provider-policy-sensitive controls.

## Locked Scope

Included:

1. introduce a new LOGOS-owned session-state crate under `codex-rs/`
2. move the clear non-canonical thread lifecycle ownership into that crate
3. move the clear non-canonical host-operation ownership into that crate
4. rewire the OpenRouter proving lane onto that shared layer

Excluded:

1. OpenAI canonical-path migration
2. durable resume beyond already-loaded live-thread reconnect
3. fork and review extraction
4. rollback extraction
5. compaction extraction
6. interrupt extraction
7. steering extraction
8. realtime extraction

## Required Reading

1. `AGENTS.md`
2. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
3. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md`
4. `docs-new/07-logos-phase/architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md`
5. `docs-new/07-logos-phase/implementation/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md`
6. `codex-rs/app-server/src/openrouter_beta_threads.rs`
7. `codex-rs/logos-session-engine/src/session.rs`

## Target Layer

Working target:

1. `codex-rs/logos-session-state/`

This new layer should sit:

1. above provider adapters
2. above `logos-session-engine` turn execution
3. below app-server request dispatch

## First Extraction Wave

### Slice A: Shared Non-Canonical Thread State

Move or define:

1. common non-canonical thread registry
2. common non-canonical thread-state envelope
3. shared persisted thread snapshot format
4. shared active and archived storage-path helpers

Required result:

1. OpenRouter no longer owns these as permanent private concepts

### Slice B: Shared Read And Lifecycle Surface

Move or define:

1. shared read and list helpers for non-canonical threads
2. shared archive and unarchive workflow
3. shared loaded and persisted visibility logic

Required result:

1. app-server can treat these as LOGOS-owned non-canonical thread lifecycle behavior

### Slice C: Shared Host Operations

Move or define:

1. shared shell-command ownership for non-canonical threads
2. shared background-terminal cleanup ownership for non-canonical threads
3. shared running-command bookkeeping

Required result:

1. host operations stop being permanent OpenRouter-owned logic

### Slice D: OpenRouter Rewire

Rewire the current OpenRouter proving lane so it supplies:

1. provider session bootstrap
2. provider policy validation
3. provider-specific runtime wrapper where still needed

But consumes the new shared session-state layer for:

1. thread lifecycle
2. persistence
3. archive and unarchive
4. shell-command
5. background-terminal cleanup

## Leave In OpenRouter For Now

Do not move in this package:

1. `OpenRouterBetaSession::from_config`
2. provider-specific warning and validated-route posture
3. provider-specific override validation
4. durable resume beyond loaded-thread reconnect
5. fork and review orchestration
6. rollback
7. compaction backend execution
8. interrupt
9. steering
10. realtime

## Acceptance Targets

### A1. Ownership Proof

1. The new crate owns the clear non-canonical thread lifecycle and host-operation mechanics.
2. `openrouter_beta_threads.rs` no longer owns those mechanics as permanent provider-specific implementations.

### A2. OpenRouter No-Regression Proof

1. Existing OpenRouter app-server behavior for the moved surfaces remains green after rewire.

### A3. OpenAI No-Regression Proof

1. The canonical OpenAI path remains unchanged in behavior for touched surfaces.

### A4. Boundary Honesty Proof

1. Provider-sensitive controls remain explicitly outside the extraction package.
2. No fake generic control abstraction is introduced for compaction, interrupt, steering, or realtime.

## Guardrails

1. no broad `codex-core` rewrite
2. no OpenAI degradation
3. no silent semantic flattening across providers
4. no extraction of provider-policy-sensitive controls in the first wave
5. no migration of `logos-session-engine` ownership away from turn, history, tool, and journal workflow

## Exit Verdict Shape

1. `session-state extraction wave closed`
2. or `session-state extraction blocked by X`

## Evidence

- path: `../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Current Consensus`
  lines: `22-37`
- path: `../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Move Now`
  lines: `53-87`
- path: `../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Keep Provider-Sensitive For Now`
  lines: `97-104`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThreadManager`
  lines: `73-1180`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `135-193, 290-506`

## Links

- [[README.md]]
- [[../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md]]
