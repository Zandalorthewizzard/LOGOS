---
id: logos-host-daemon-transport-spec
title: LOGOS Host Daemon Transport Spec
type: spec
status: active
tags: [architecture, transport, daemon, ipc, host, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-approval-control-architecture-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [../research/2026-03-30-logos-provider-research-report.md]
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-approval-control-architecture-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Host Daemon Transport Spec

## Purpose

- Lock the host/daemon and IPC posture before implementation starts.
- Preserve Codex-grade session and event delivery semantics while fitting the Logos daemon-first architecture.
- Prevent transport concerns from leaking back into provider adapters or shared runtime types.

## Core Decision

- Logos uses a host-shell plus daemon-runtime split.
- The host owns UI orchestration and user interaction surfaces.
- The daemon owns session state, provider execution, tool runtime, approvals state machine, and durable journal writes.
- Transport is an outer boundary only; provider logic and session semantics must not depend on IPC shape.

## Ownership Split

### Host Owns

- user input collection
- session selection and presentation
- approval prompt presentation
- event rendering and optimistic UI state
- reconnect/retry orchestration at UI boundary

### Daemon Owns

- session lifecycle
- provider adapter invocation
- event ordering and delivery policy
- tool execution and MCP correlation
- approval state machine
- persistence and resume

## Transport Principles

### Principle 1: Event Stream Is Canonical

- Host receives an ordered daemon event stream.
- The stream is the canonical transport surface for turn progress, approvals, tool activity, and completion.

### Principle 2: Submission Is Command-Based

- Host sends explicit commands to the daemon such as start turn, resume session, answer approval, or cancel turn.
- The host does not directly manipulate provider state.

### Principle 3: Reconnect Must Be Recoverable

- If host disconnects, daemon state must remain recoverable from durable session state and active in-memory state where applicable.
- Reconnection must never require reconstructing provider state from UI memory alone.

### Principle 4: Approval Is Round-Trip Control Flow

- Approval requests originate from daemon state.
- Host presents them.
- Host returns explicit approval decisions.
- Daemon applies the decision and continues the session.

## Responses Reuse Evaluation

### What This Spec Reuses From Responses

- the value of typed event delivery instead of plain text transport
- the distinction between request submission and stream consumption
- the need to preserve ordered event semantics for tools, continuation, and completion

### Advantages Of Reusing Responses Concepts

- keeps canonical OpenAI streaming semantics translatable through the daemon boundary
- supports Codex-grade agent behavior without collapsing transport to plain text chat
- gives a strong baseline for event-oriented host rendering

### Disadvantages And Limits

- host/daemon IPC is not the same thing as provider streaming transport
- a `Responses`-shaped event vocabulary cannot be the whole transport model because approvals, tool execution state, and persistence events are Logos-owned

### Decision

- Reuse the event-oriented discipline learned from `Responses`.
- Do not make host/daemon transport a thin mirror of OpenAI provider events.
- Extend transport with Logos-owned control-flow events where needed.

## Required Transport Surfaces

### Host To Daemon Commands

- start turn
- resume session
- answer approval
- cancel turn
- subscribe/reconnect to session events

### Daemon To Host Events

- session started or resumed
- normalized model stream events
- tool call and tool result progression
- approval required
- approval resolved
- continuation updated
- completed
- failed

## Delivery Rules

- Event ordering must remain stable per session turn.
- Delivery IDs or equivalent correlation markers must support reconnect and deduplication.
- Completion and failure are terminal transport events and must not be implicit.

## Transport And Persistence Interaction

- Durable journal is the source of truth for replayable state.
- IPC transport may buffer or re-emit events, but it is not the canonical store.
- Host-side caches are convenience only.

## Non-goals

- no final protocol encoding choice yet
- no UI framework specifics
- no commitment yet to exact Tauri command/event API shape

## Acceptance Gates

1. A host can start a turn and receive ordered daemon events without provider-specific transport knowledge.
2. Approval roundtrip works across the host/daemon boundary.
3. Host reconnect can resume observing session state without relying on lost UI memory.
4. Provider adapters remain independent from IPC protocol details.

## Open Questions

- exact wire contract shape for event subscription and reconnection
- whether session snapshots should complement event replay for fast host recovery
- exact Tauri or alternative IPC binding details for the first implementation slice

## Evidence

- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Host/daemon constraint audit`
  lines: `108-126`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 2: OpenAI Canonical Turn Loop`
  lines: `106-123`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 3: Tool Loop Boundary`
  lines: `125-142`
- path: `2026-03-30-logos-crate-map.md`
  symbol: `Crate Migration Matrix`
  lines: `21-35`
- path: `2026-03-30-logos-runtime-types-v0.md`
  symbol: `Open questions`
  lines: `281-281`

## Links

- [[README.md]]
- [[2026-04-02-logos-approval-control-architecture-spec.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
