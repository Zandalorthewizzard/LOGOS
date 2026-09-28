---
id: logos-session-state-ownership-and-provider-native-backend-model-discussion
title: LOGOS Session-State Ownership And Provider-Native Backend Model Discussion
type: discussion
status: active
tags: [logos, architecture, session-state, provider-backends, phase-6]
related:
  [
    [README.md],
    [2026-04-11-logos-philosophy-and-invariants.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-03-logos-compaction-strategy-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-11-logos-philosophy-and-invariants.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-03-logos-compaction-strategy-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Session-State Ownership And Provider-Native Backend Model Discussion

## Purpose

This document records the current architecture consensus exposed by the OpenRouter phase-6 work.

The problem is no longer whether LOGOS can call another provider. The problem is who owns session, state, context, replay, and control mechanics once the provider is not giving LOGOS the same cloud-side semantics as canonical OpenAI `Responses`.

## Problem Statement

OpenRouter phase 6 proved two things at once:

1. thin adapters plus the existing shared `logos-session-engine` are enough to reach real progress
2. a second ownership layer appeared between adapter wires and app-server dispatch

Today that layer lives mostly in `openrouter_beta_threads.rs`.

That is a useful proving lane, but it is the wrong long-term owner for generic session and thread mechanics that will repeat for future non-canonical providers.

## Current Consensus

1. LOGOS owns state, context, session, and thread semantics above provider wires.
2. Provider adapters stay thin and should own transport, auth, request and response mapping, capability truth, and provider-specific policy hooks.
3. Provider-native continuation, compaction, and richer control features are provider-first backends or accelerators, not the owner of product semantics.
4. Fallback from provider-native behavior to LOGOS-managed behavior is allowed only when semantic truth remains acceptable.
5. If semantic truth would change materially, LOGOS must report `degraded` or `unsupported` instead of silently falling back.
6. OpenAI remains the canonical path and must not be forced onto a weaker least-common-denominator implementation.
7. This ownership move belongs in a LOGOS-owned additive runtime layer, not in a broad `codex-core` rewrite.

## Ownership Layers

### Layer 1: Provider Adapters

Provider adapters should own:

1. auth and base URL behavior
2. transport and streaming details
3. wire payload mapping
4. provider capability truth
5. provider-specific error normalization
6. provider policy hooks and native backend entry points

They should not own:

1. generic thread lifecycle
2. generic host operations
3. durable session-state ownership
4. shared notification or thread-list semantics

### Layer 2: LOGOS Session-State Runtime Layer

The LOGOS-owned layer above provider wires should own:

1. thread and turn lifecycle
2. session and context ownership
3. durable history and continuation ownership
4. replay and resume orchestration
5. archive and unarchive semantics
6. fork and review orchestration
7. host operations such as shell and background-terminal handling
8. thread visibility, metadata, and notifications
9. compaction workflow ownership, even when the actual backend is provider-native

This is the layer that OpenRouter phase 6 has partially exposed but not yet extracted.

### Layer 3: Provider-Native Backends

Provider-native backends remain important.

Examples include:

1. OpenAI server-managed continuation
2. OpenAI native remote compaction
3. any future provider-native interrupt, realtime, or richer control surface

These should be modeled as provider-first backends behind LOGOS-owned workflow ownership.

## Execution Policy

The execution policy should be:

1. prefer provider-native backend when it exists and preserves stronger semantics
2. otherwise use a LOGOS-managed backend when LOGOS can preserve honest semantics
3. otherwise return explicit `degraded` or `unsupported`

What must not happen:

1. silent semantic downgrade hidden behind a generic success path
2. moving generic session ownership into each provider-specific lane
3. degrading the canonical OpenAI path just to make the diagram look symmetric

## Immediate Implications For OpenRouter Phase 6

1. `openrouter_beta_threads.rs` is a useful proving lane, but it is not the correct final owner for shared session and thread mechanics.
2. `thread/shellCommand`, `thread/backgroundTerminals/clean`, `thread/archive`, and `thread/unarchive` should be treated as LOGOS-owned session-state and host-operation mechanics, not as permanent OpenRouter-owned features.
3. `thread/resume`, `thread/fork`, `review/start`, `thread/rollback`, `thread/compact/start`, `turn/interrupt`, `turn/steer`, and realtime need shared ownership with provider policy hooks rather than copy-pasted provider lanes.
4. OpenAI native continuation and compaction remain valid provider-first backends and should not be flattened away.

## Immediate Implications For Future Providers

1. A future stateless or partially stateless provider should not need its own clone of `openrouter_beta_threads.rs`.
2. The provider should plug into the same LOGOS-owned session-state runtime layer with its own capability truth and policy hooks.
3. Differences between providers should remain explicit at the backend and capability level, not by duplicating the whole thread-control stack.

## Non-goals

1. no fake universal provider sameness
2. no immediate broad extraction that destabilizes already-proven phase-6 slices
3. no forced replacement of OpenAI native semantics with weaker portable logic
4. no automatic fallback that lies about what semantics are still preserved

## Closeout Expectations

1. OpenRouter mechanism discussions should now use this ownership model as their baseline.
2. Future extraction work should move shared session-state mechanics toward a LOGOS-owned layer while keeping adapters thin.
3. Phase-6 support statements should distinguish clearly between LOGOS-managed behavior and provider-native backends.

## Evidence

- path: `2026-04-11-logos-philosophy-and-invariants.md`
  symbol: `Keep Runtime Authority Visible`
  lines: `68-82`
- path: `2026-04-02-logos-runtime-contract-spec.md`
  symbol: `Provider Outcomes`
  lines: `153-198`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `OpenRouter`
  lines: `140-163`
- path: `2026-04-03-logos-compaction-strategy-discussion.md`
  symbol: `Compaction Strategy`
  lines: `40-116`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `52-53, 135-193, 290-506`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThreadManager`
  lines: `73-1180`
- path: `../../../codex-rs/logos-provider-openrouter/src/adapter.rs`
  symbol: `ProviderAdapter for OpenRouterAdapter`
  lines: `64-126`
- path: `../../../codex-rs/logos-provider-openai/src/adapter.rs`
  symbol: `openai_provider_capabilities`
  lines: `192-203`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor`
  lines: `2526-7651`

## Links

- [[README.md]]
- [[2026-04-11-logos-philosophy-and-invariants.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[2026-04-03-logos-compaction-strategy-discussion.md]]
- [[2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
