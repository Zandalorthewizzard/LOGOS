---
id: logos-implemented-items-parity-scope-analysis
title: Implemented Items Parity And Scope Remainder Analysis
type: analysis
status: active
tags:
  [
    logos,
    analysis,
    parity,
    scope,
    codex,
    goose,
    opencode,
    implemented-items,
  ]
related:
  [
    [README.md],
    [../README.md],
    [2026-03-30-codex-model-agnostic-analysis.md],
    [../architecture/2026-04-11-logos-philosophy-and-invariants.md],
    [../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [../architecture/2026-04-16-item-11-background-terminals-draft-implementation-discussion.md],
    [../implementation/2026-04-18-item-05-shell-execution-implementation-report.md],
    [../implementation/2026-04-18-item-07-archive-metadata-migration-implementation-report.md],
    [../implementation/2026-04-18-item-11-background-terminals-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-task-03-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-task-04-implementation-report.md],
    [../../02-guides/operations/2026-04-18-slice-risk-register-and-boundary-discipline.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
depends_on: [[README.md]]
generated: false
source_of_truth: working-doc
outline: []
---

# Implemented Items Parity And Scope Remainder Analysis

## Purpose

This document compares the already-realized Item slices against the Codex baseline and the borrowed reference sources, then calls out where the closed slice is intentionally narrower than maximum realizable parity.

The goal is not to reopen closed work. The goal is to prevent future agents from misreading a bounded slice as a full surface closure.

## Method

For each implemented item, this analysis records:

1. the Codex-side baseline that the slice was compared against;
2. the LOGOS implementation that actually landed;
3. the main borrowing source, if any, from Goose or OpenCode;
4. the parity remainder or narrow-scope remainder that stays open;
5. the practical misread to avoid.

## Implemented Items Matrix

| Item | Codex baseline | LOGOS implementation | Borrowed from | Parity remainder / narrow scope | How to read it |
|---|---|---|---|---|---|
| `01` Thread manager | Durable thread identity, archive state, metadata listing, and persisted thread records. | `SqliteThreadRegistry` owns non-canonical thread identity and registry-backed list/read/archive behavior. | Goose `session_manager.rs` for SQLite import/update/list patterns; OpenCode `db.ts` and `session.sql.ts` for WAL and soft-archive shape. | Scoped to the non-canonical lane; canonical thread lineage and broader Codex thread model are still outside this slice. | Good foundation slice, not a universal thread subsystem. |
| `02` Session persistence | Durable persistence boundary between metadata/config truth and runtime payload truth. | Registry-backed metadata/config truth with transitional JSONL payloads and bounded snapshot fallback. | Goose `session_manager.rs` for import and partial-update patterns; OpenCode `db.ts` and `session.sql.ts` for durability and soft-archive posture. | JSONL, transitional snapshot residue, and later journal-backed runtime truth remain open. | Honest boundary closure, not final runtime persistence. |
| `03` Fork / resume | Whole-history fork and explicit durable-state resume. | Registry-resolved durable-state fork, replay-first resume selection, and non-canonical loading helpers. | Goose durable copy-session patterns; Codex whole-history fork/resume behavior as the semantic baseline. | `thread_edges` lineage storage and richer tool-compatibility validation remain open. | Beta fork/resume slice, not a full lineage system. |
| `04` Turn lifecycle | Explicit turn lifecycle ownership and central notification derivation. | `TurnLifecycleState`, `TurnEvent`, and helper-driven notification dispatch on the OpenRouter beta lane. | Codex lifecycle/event mapping; OpenCode busy-guard and runner-style ownership pattern. | No separate stream-processor boundary and no panic-safe reset yet; shell orchestration still has its own lane residue. | Lifecycle correctness slice, not a complete turn-stream architecture. |
| `05` Shell execution | One-shot shell-command execution with streaming and cancellation. | Shared `host_exec` seam plus `run_one_shot_shell_command()` shared by canonical and OpenRouter paths. | Local Codex exec substrate first; OpenCode and Goose only as guardrails for shell selection, subprocess hygiene, and policy vocabulary. | Full background-terminal lifecycle remains open in Item 11; OpenRouter shell posture remains intentionally weaker than canonical. | Intentionally narrow extraction slice, not terminal parity. |
| `07` Archive / metadata | Archive/unarchive plus metadata coherence for list/read behavior. | `logos-session-state` registry owns archive, unarchive, name, git, and updated-at writes for the non-canonical lane. | No external borrow in this slice; reused existing LOGOS registry machinery and `ThreadUpdateBuilder`. | Compatibility snapshot residue remains; this is the non-canonical owner model, not rollout-backed canonical metadata. | Owner-shift slice, not universal metadata unification. |
| `11` Background terminals | Interactive process ownership with cleanup, visibility, stdin, buffering, and lifecycle behavior. | Shared `HostExecRegistry` owns thread-scoped cleanup and process metadata for the touched OpenRouter surface. | Local Codex host-exec/process substrate first; OpenCode and Goose only as guardrails for cleanup and process-lifetime vocabulary. | `stdin`/`write_stdin`, TTY resize, process visibility, output buffering, attach/detach, terminal-interaction pipeline, and restart handling remain open. | First-wave cleanup-owner extraction, not full background-terminal product parity. |

## Where Scope Was Most Narrow

1. `Item 05` and `Item 11` are the clearest first-wave extraction slices. Their parity remainder is the point of the closeout, not a bug in it.
2. `Item 03` and `Item 04` are narrower than the full Codex lifecycle and resume surfaces. They are correct slices, but they should not be read as full closure of lineage or stream-processing architecture.
3. `Item 02` and `Item 07` are owner-shift slices with transitional residue. That residue is honest, but it must never be mistaken for the final owner truth.
4. `Item 01` is lane-scoped. It solved the non-canonical registry problem without claiming to absorb the full canonical thread model.

## Borrowing Pattern

1. Goose was used most heavily for SQLite session-state mechanics: legacy import, partial updates, dynamic listing, and durable metadata writes.
2. OpenCode was used most heavily for WAL-oriented SQLite posture and soft-archive modeling.
3. Local Codex source was the primary substrate for shell execution, turn lifecycle, and process-lifecycle ownership.
4. `Item 07` was the only closed slice that did not need external borrow patterns in the implementation slice itself.

## Conclusion

The main risk is not that the closed slices are wrong. The risk is that a future reader sees a closed slice and assumes the remaining parity surface is also closed.

The current rule should be:

1. if a slice has a non-empty parity remainder, the remainder must be explicit;
2. if a slice keeps transitional residue, the residue must not be mistaken for owner truth;
3. if a slice borrows from Goose or OpenCode, the borrowed shape must be named, not implied;
4. if a slice is intentionally narrow, the narrower boundary must be recorded in the report, not hidden in the summary.

## Evidence Basis

The comparison above is grounded in the implemented-item reports and the slice-risk workflow now required by the docs system:

1. Session-State Foundation Task 01, 02, 03, and 04 implementation reports.
2. Item 05 shell execution implementation report.
3. Item 07 archive/metadata migration implementation report.
4. Item 11 background terminals implementation report.
5. `Slice Risk Register And Boundary Discipline` for parity remainder and residual-risk accounting.
6. `Session layer research transfer ledger` for the item-level verdict trail.
