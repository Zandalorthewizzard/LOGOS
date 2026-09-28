---
id: logos-session-state-foundation-task-03-turn-lifecycle-strict-spec
title: Session-State Foundation Task 03 Turn Lifecycle Strict Spec
type: spec
status: active
tags:
  [logos, phase-6, strict-spec, sprint, session-state, turn-lifecycle]
related:
  [
    [README.md],
    [2026-04-17-session-state-foundation-sprint-strict-spec.md],
    [../architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md],
    [2026-04-17-item-04-implementation-reference.md],
    [2026-04-17-session-state-foundation-slice-implementation-report-template.md],
  ]
depends_on:
  [
    [2026-04-17-session-state-foundation-sprint-strict-spec.md],
    [2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md],
    [2026-04-17-session-state-foundation-task-02-session-persistence-strict-spec.md],
    [../architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md],
    [2026-04-17-item-04-implementation-reference.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Session-State Foundation Task 03 Turn Lifecycle Strict Spec

## Start Here

1. `AGENTS.md`
2. `docs-new/07-logos-phase/architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md`
3. `docs-new/07-logos-phase/implementation/2026-04-17-item-04-implementation-reference.md`
4. the exact Codex, OpenCode, and Goose snapshot files cited in that implementation reference evidence section
5. `codex-rs/app-server/src/openrouter_beta_threads.rs`
6. `codex-rs/app-server/src/thread_state.rs`
7. `codex-rs/protocol/src/protocol.rs`

## Intent

Replace manual notification assembly and fragile active-turn bookkeeping on the targeted non-canonical path with authoritative lifecycle state and derived notifications.

Non-goals:

1. no interrupt implementation
2. no steering implementation
3. no review orchestration implementation
4. no shell execution extraction

## Locked Decisions

1. Lifecycle state transitions are authoritative.
2. Notifications must be derived from those transitions, not assembled ad hoc at call sites.
3. The task uses the Item 04 implementation reference and cited source snapshots as required inputs.
4. Production code naming must be LOGOS-native.

## Required External References

1. Codex event and status model patterns cited in the Item 04 implementation reference
2. OpenCode runner pattern and doom-loop detection cited in the Item 04 implementation reference
3. Goose inspection patterns cited in the Item 04 implementation reference where relevant

## Current Invalid Or Stale Code

1. `active_turn_id` as the main concurrency guard in `openrouter_beta_threads.rs`
2. repeated `send_notification(...)` call chains in `start_turn`
3. duplicated manual notification sequences in review-related paths
4. stream processing, persistence, lifecycle, and notification dispatch mixed into one spawned task closure

## Transitional Code Allowed During This Task

1. delivery helper functions may remain while lifecycle ownership moves
2. provider-specific streaming internals may remain if lifecycle and notification ownership moves first

## Code That Must Be Removed Or Downgraded Before Task Close

1. manual notification ownership in the targeted non-canonical turn path must be removed or reduced to thin delivery-only glue
2. `active_turn_id` must stop being the final concurrency model
3. monolithic lifecycle closures must stop being the final owner of turn state transitions

## Implementation Tasks

### T1. Lifecycle State Machine

1. introduce explicit lifecycle state for targeted non-canonical turns
2. introduce a concurrency guard or runner model

### T2. Derived Notifications

1. centralize mapping from lifecycle state changes to app-server notifications
2. remove manual per-path notification sequencing as final ownership

### T3. Stream Processor Boundary

1. separate stream handling from lifecycle state transitions
2. keep notification delivery thin and downstream of lifecycle truth

### T4. Reporting

1. trace all imported runner or event patterns to source file and line range
2. list every manual notification site removed
3. list any remaining manual notification sites that stay transitional

## Acceptance And Tests

1. targeted tests for lifecycle state transitions and derived notifications
2. focused `cargo test -p codex-app-server` coverage for the touched turn path
3. no claim of closure if `start_turn` still owns the final notification choreography directly

## Evidence

- path: `../architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md`
  symbol: `Manual Notification Construction Is The Core Problem`
  lines: `56-83`
- path: `../architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md`
  symbol: `Turn Lifecycle State Machine`
  lines: `135-153`
- path: `2026-04-17-item-04-implementation-reference.md`
  symbol: `Stage A: Turn Lifecycle State Machine`
  lines: `28-93`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `start_turn`
  lines: `771-1044`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `send_notification`
  lines: `1973-1988`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `EventMsg`
  lines: `1223-1302`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `AgentStatus`
  lines: `1606-1622`

## Links

- [[README.md]]
- [[2026-04-17-session-state-foundation-sprint-strict-spec.md]]
- [[../architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md]]
- [[2026-04-17-item-04-implementation-reference.md]]
- [[2026-04-17-session-state-foundation-slice-implementation-report-template.md]]
