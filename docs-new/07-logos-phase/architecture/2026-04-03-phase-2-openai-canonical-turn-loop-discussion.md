---
id: logos-phase-2-openai-canonical-turn-loop-discussion
title: LOGOS Phase 2 OpenAI Canonical Turn Loop Discussion
type: discussion
status: active
tags: [architecture, discussion, phase-2, openai, turn-loop, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-provider-openai-adapter-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-03-logos-compaction-strategy-discussion.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-provider-openai-adapter-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 2 OpenAI Canonical Turn Loop Discussion

## Purpose

- lock the architecture shape for the next phase after boundary extraction
- preserve canonical Codex/OpenAI turn semantics without smuggling OpenAI-only assumptions into shared runtime contracts
- define the minimum phase-2 scope so later implementation does not collapse into broad persistence or tool-runtime refactors

## Starting Point

- phase-1 now proves normalized text turns, explicit continuation carryover, normalized tool-call boundaries, runtime-owned tool execution, and approval-aware reinjection through the OpenAI path
- the next risk is no longer "can the boundary exist?"
- the next risk is "can LOGOS preserve the full canonical OpenAI turn loop honestly while keeping the kernel/provider/session split stable?"

## Phase-2 Problem Statement

- Codex already has a mature OpenAI turn loop with explicit per-turn session lifetime, websocket incremental carryover, compaction triggers, and turn-state routing rules
- phase-1 intentionally extracted a narrow boundary, but did not yet lock which parts of the canonical turn loop stay inside Codex-base versus move into LOGOS session ownership
- without a phase-2 design lock, future work could cause one of two bad outcomes:
  1. over-promotion of Codex/OpenAI session semantics into shared LOGOS runtime abstractions
  2. accidental reimplementation of proven Codex turn behavior in `logos-*` crates with semantic drift

## Core Decision

- Phase 2 preserves the canonical OpenAI turn loop as the reference path.
- LOGOS does not rewrite Codex turn orchestration merely to look provider-neutral.
- LOGOS does make turn ownership boundaries explicit so later providers can diverge honestly without corrupting the canonical OpenAI path.

## Architecture Goals

1. keep per-turn OpenAI transport/session lifetime canonical
2. keep continuation explicit and capability-driven
3. preserve stream ordering and terminal semantics
4. preserve OpenAI compaction behavior as an adapter-native fast path without turning provider-name checks into shared architecture truth
5. define which turn artifacts must remain durable-journal-ready for later persistence/resume phases

## Canonical Turn Loop Split

### Must Remain Canonical Codex/OpenAI Behavior

1. turn-scoped `ModelClientSession` lifetime
2. websocket incremental request construction rules
3. `x-codex-turn-state` routing-token behavior within a turn
4. explicit `previous_response_id` continuation carryover
5. OpenAI remote compaction request/response semantics

### Must Remain LOGOS-Owned Policy Orchestration

1. normalized event/message ownership
2. capability-driven session decisions
3. runtime-owned tool execution and approval policy
4. future durable journal model and replay posture
5. provider-native fast-path gating policy

## Continuation Position

- explicit continuation handles are now the only acceptable cross-turn continuation mechanism for LOGOS OpenAI turns
- cached websocket carryover is still allowed, but only as a within-turn transport optimization subordinate to explicit request state
- phase-2 should treat continuation as a canonical adapter/session seam, not as hidden transport state

## Turn-State Position

- `x-codex-turn-state` remains an OpenAI/Codex-native sticky-routing contract, not a shared runtime abstraction
- phase-2 may use a narrow Codex-base seam to transfer turn-state between fresh request-scoped sessions inside one logical LOGOS turn
- the purpose of that seam is preservation, not re-ownership: LOGOS must not reimplement sticky-routing policy itself

## Compaction Position

- OpenAI remote compaction remains canonical and preserved
- the top-level compaction policy belongs to LOGOS session/runtime ownership, not provider-name branching baked into future shared architecture
- phase-2 should preserve current OpenAI compaction semantics and isolate them cleanly enough that phase-4 persistence/resume work can add durable checkpoint behavior without another broad turn-loop rewrite
- `2026-04-03-logos-compaction-strategy-discussion.md` should be treated as a required companion design lock for phase-2 planning
- the first safe phase-2 code move is an honest compaction-backend classification plus session capability handoff, not premature portable compaction implementation

## Recommended Ownership Boundaries

### `codex-core`

- continues to own canonical OpenAI turn transport/session behavior
- may gain only narrow additive seams when external LOGOS ownership needs an explicit legal hook
- must not become the home of LOGOS-neutral session-engine orchestration

### `logos-provider-openai`

- owns request mapping, normalized stream mapping, capability projection, and explicit provider-specific policy translation
- should remain thin over canonical Codex/OpenAI behavior

### `logos-session-engine`

- owns normalized turn orchestration, tool loop policy, approval flow, and later durable journal integration points
- must not silently recreate Codex websocket/session semantics that canonically belong below the adapter seam

## Phase-2 Non-goals

1. no non-OpenAI provider implementation work
2. no broad persistence redesign yet
3. no CLI/TUI migration
4. no fake-neutral rewrite of OpenAI compaction, continuation, or turn-state behavior
5. no large `codex-core` refactor when a narrow seam is sufficient

## Open Questions To Resolve In Strict Spec

1. what exact phase-2 deliverable proves "canonical turn loop preserved" beyond current phase-1 coverage
2. whether compaction preservation needs one dedicated seam or can stay entirely within existing Codex/OpenAI public behavior
3. what minimum durable-turn artifacts phase-2 must preserve for later phase-4 journal work
4. whether any remaining OpenAI turn semantics still leak through provider-specific hidden state instead of normalized contracts

## Recommended Output Of Phase-2 Strict Spec

1. exact ownership boundaries for continuation, turn-state, websocket incrementality, and compaction
2. explicit test list for canonical OpenAI turn behavior
3. seam criteria for any additional Codex-base intervention
4. a short list of phase-2 non-goals that prevent premature persistence or multi-provider sprawl

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 2: OpenAI Canonical Turn Loop`
  lines: `107-126`
- path: `2026-04-02-logos-adapter-kernel-architecture.md`
  symbol: `Optimization Targets Before Coding`
  lines: `169-190`
- path: `2026-04-02-logos-provider-openai-adapter-spec.md`
  symbol: `Required Adapter Behavior`
  lines: `80-147`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Durable Journal Minimum Record Model`
  lines: `45-58`
- path: `2026-04-03-logos-compaction-strategy-discussion.md`
  symbol: `Design Position`
  lines: `68-116`
- path: `../../../codex-rs/core/src/client.rs`
  symbol: `ModelClientSession`
  lines: `188-215`
- path: `../../../codex-rs/core/src/client.rs`
  symbol: `ModelClientSession::prepare_websocket_request`
  lines: `831-857`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-04-02-logos-provider-openai-adapter-spec.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[2026-04-03-logos-compaction-strategy-discussion.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
