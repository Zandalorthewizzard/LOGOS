---
id: logos-persistence-resume-architecture-spec
title: LOGOS Persistence And Resume Architecture Spec
type: spec
status: active
tags: [architecture, persistence, resume, journal, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [../research/2026-03-30-logos-provider-research-report.md],
    [../research/2026-03-30-provider-comparison-matrix.md]
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-03-30-logos-runtime-types-v0.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Persistence And Resume Architecture Spec

## Purpose

- Lock the durable journal and resume posture before implementation begins.
- Prevent Logos from accidentally persisting raw provider wire types as canonical truth.
- Define one persistence model that supports canonical stateful providers and stateless replay adapters without lying.

## Problem Statement

- Resume is not just conversation text recovery.
- Logos must be able to resume stateful providers faithfully and replay stateless providers honestly.
- Tool calls, approvals, ordering, and continuation handles are part of the durable model, not incidental metadata.

## Core Decision

- Logos persists a Logos-native durable journal.
- Provider wire payloads may be stored optionally for debugging or compatibility work, but never as the canonical record model.
- Resume logic is driven by provider mode and continuation policy, not by a single universal mechanism.

## Durable Journal Minimum Record Model

The durable journal must capture, at minimum:

1. provider identity and provider mode
2. model identity and request configuration
3. continuation handle and continuation mode
4. normalized user and assistant messages
5. assistant tool calls and tool results
6. approval requests and decisions
7. stream ordering and terminal markers
8. normalized error outcomes
9. session metadata required for later correlation

## Resume Modes

### Mode A: Server-Managed Resume

- Canonical example: OpenAI `previous_response_id`
- Logos stores the continuation handle plus enough journal context to recover if the provider handle becomes invalid or the policy changes.

### Mode B: Client-Managed Replay

- Canonical example: OpenRouter stateless flow
- Logos reconstructs the next turn from the durable journal.
- Replay is not fallback-only; it is the primary continuation strategy for stateless adapters.

### Mode C: Hybrid Resume

- Used when a provider offers some server-side carryover plus provider-specific client-side reconstruction or cache policy.
- Hybrid mode must never be disguised as pure OpenAI-style stateful continuation.

## Responses Reuse Evaluation

### What This Spec Reuses From Responses

- the insight that continuation is a first-class part of the turn model
- the need for explicit stable identifiers such as `previous_response_id`
- typed terminal events and usage/completion framing
- the lesson that tool and reasoning artifacts may matter for follow-up turns

### Advantages Of Reusing Responses Concepts

- preserves faithful OpenAI resume behavior
- gives Logos a strong baseline for durable event/message capture
- helps prevent under-specified resume design

### Disadvantages And Limits

- `Responses` continuation is not a universal model for all providers
- overreliance on OpenAI shapes could hide the distinct needs of stateless, cached, or native providers
- some provider carryover semantics are not continuation handles at all and must not be mislabeled as such

### Decision

- Reuse `Responses` continuation lessons as canonical reference behavior.
- Do not make the durable journal depend on raw `Responses` items or OpenAI-only resume assumptions.

## Canonical Storage Boundaries

### Canonical Data

- normalized messages
- normalized tool actions
- continuation handles
- capability/mode markers
- ordering and completion markers
- approvals and branch outcomes

### Optional Non-Canonical Data

- raw provider events for observability
- raw provider request/response snapshots for debugging
- provider-specific extension payloads when needed for future compatibility work

### Prohibited As Canonical Truth

- raw `ResponseItem`
- raw provider SSE event enums
- raw OpenAI-specific or provider-specific wire requests as the primary resume model

## Session And Branching Rules

- Every turn must end with one terminal record: completed or failed.
- Tool calls and approval branches must retain stable correlation IDs.
- Resume logic must be able to reconstruct which branch was accepted, rejected, or interrupted.
- Journal writes must preserve enough order to replay the same logical turn progression.

## Provider-Specific Resume Posture

### OpenAI

- primary strategy: server-managed continuation handle
- secondary safety: enough journal detail to recover or replay if needed

### OpenRouter

- primary strategy: client-managed replay
- no fake server-managed continuation

### Gemini, Z.AI, LM Studio

- mode-sensitive strategy
- resume posture must record compat vs native mode explicitly

### Anthropic

- native/client-managed history plus provider-specific caching semantics until stronger evidence exists

## Non-goals

- no final database schema or storage engine choice yet
- no claim that one persistence shape already has full provider parity proof
- no provider-neutral mythology that hides different continuation modes

## Acceptance Gates

1. OpenAI resume is representable without exporting raw OpenAI wire types as canonical data.
2. OpenRouter replay is representable from the same durable journal family.
3. Tool calls, approvals, and terminal ordering remain reconstructible.
4. Provider mode is always persisted explicitly when a provider has compat/native split.

## Open Questions

- smallest durable record set that still supports all required early providers
- whether provider-specific reasoning artifacts need durable storage for correct replay in some modes
- when optional raw-provider snapshots are worth storing for diagnostics

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 4: Persistence And Resume`
  lines: `144-161`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Persistence / Resume Findings`
  lines: `144-166`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Recommended Decisions for Logos 0.1`
  lines: `181-188`
- path: `../research/2026-03-30-provider-comparison-matrix.md`
  symbol: `Matrix`
  lines: `30-45`
- path: `2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Resume Invariants`
  lines: `226-233`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
