---
id: logos-guide-op-slice-risk-register-and-boundary-discipline
title: Slice Risk Register And Boundary Discipline
type: guide
status: active
tags: [operations, risk, boundaries, specs, implementation]
related:
  [
    [2026-04-14-orchestrated-agent-loop.md],
    [2026-04-14-glm-5.1-agent-profile.md],
    [../agent-navigation.md],
    [../../../AGENTS.md],
    [../../07-logos-phase/implementation/2026-04-03-codex-core-seam-index.md],
    [../../07-logos-phase/implementation/2026-04-18-item-05-shell-execution-strict-spec.md],
    [../../07-logos-phase/implementation/2026-04-18-item-05-shell-execution-implementation-report.md],
    [../../07-logos-phase/implementation/2026-04-17-item-11-background-terminals-strict-spec.md]
  ]
depends_on: []
generated: false
source_of_truth: process
outline: []
---

# Slice Risk Register And Boundary Discipline

## Purpose

Make residual-boundary discipline mandatory for bounded LOGOS slices so specs, tasks, sprints, implementation reports, and review passes do not overclaim closure.

## Why This Exists

1. LOGOS work often closes one narrow runtime surface while an adjacent surface remains explicitly open.
2. Extraction work can honestly move ownership without closing the neighboring product area.
3. Transitional residue can remain correct for compatibility while still being dangerous if later agents mistake it for final owner truth.
4. A narrow Codex-base seam can be justified for one slice and still become a future governance risk if later work treats it as a generic expansion point.
5. Slice closure is only useful if future agents can tell exactly what was closed, what was not, and what would become false discovery if skimmed carelessly.

## Mandatory Reading Triggers

Read this guide before:

1. writing or revising a strict spec;
2. writing an external-agent task;
3. writing a sprint package or sprint checkpoint;
4. starting implementation on a bounded slice;
5. writing an implementation report;
6. accepting or rejecting a closure claim.

This guide is mandatory even when the slice feels obvious. The point is to prevent obvious-looking bounded work from being misread as broader closure.

## Required Risk Register Categories

Every spec, task, sprint, implementation report, and review pass must explicitly account for the categories below when they are relevant.

### 1. Adjacent-Open-Surface Risk

Identify the neighboring surface that remains open even if the touched slice lands cleanly.

Examples:

1. `Item 05` can close one-shot `thread/shellCommand` parity while `Item 11` background-terminal lifecycle remains open.
2. Archive owner migration can close while rollback semantics remain open.

### 2. Transitional Residue Risk

State what compatibility or migration residue remains and why it is not the final owner.

Examples:

1. legacy snapshot fallback;
2. compatibility archive directories;
3. duplicate config JSON kept only for restart or import safety.

### 3. Stale-Owner Risk

Name the old lane-local or provider-owned logic that is invalid as final architecture.

This must distinguish:

1. code already removed;
2. code still present but downgraded to compatibility-only residue;
3. code still incorrectly acting as owner and therefore blocking honest closure.

### 4. Seam Expansion Risk

If a slice introduces or extends a Codex-base seam, state:

1. why a wrapper or outer additive seam was insufficient;
2. what exact new public or cross-crate surface was introduced;
3. what future misuse would turn that seam into architecture drift.

### 5. Security And Semantics Honesty Risk

Do not let shared mechanics imply shared safety or shared semantics when they are not actually implemented.

Examples:

1. a shared host-exec runner does not automatically imply shared sandbox posture;
2. provider parity on one command surface does not imply parity on approvals, PTY lifecycle, or provider-native controls.

### 6. Process-State Risk

Prevent docs from drifting ahead of code or from preserving two competing truths.

Check explicitly:

1. whether ledger, spec, discussion, and implementation report agree;
2. whether a report name or sprint task label overstates what was actually implemented;
3. whether a later summary accidentally turns a bounded slice into a broader closure claim.

### 7. Migration / Double-Move Risk

When an owner shift is still in flight, state whether implementing a neighboring item now would force a second migration later.

This category is mandatory whenever current work sits between:

1. a proving-lane owner still being removed; and
2. a shared owner that already exists or is partially landed.

### 8. Maximum-Realizable-Parity Remainder

State what remains on the path to the maximum honest parity that LOGOS can realistically reach for the touched surface.

This is not a generic "future work" bucket. It must answer:

1. what parity surface is still missing;
2. whether the remainder is intentionally out of scope for the current slice or blocked by current substrate;
3. whether the remainder is a deliberate product decision, a dependency gap, or a genuine architectural blocker.

## Required Questions

Every bounded slice artifact should answer these questions directly when applicable:

1. What exact surface is being closed?
2. What adjacent surface is still open after this slice?
3. What stale owner logic must be removed before the neighboring item can be called closed?
4. What transitional residue remains, and why is it not final owner truth?
5. Did this slice create or extend a Codex-base seam? If yes, why was an outer wrapper insufficient?
6. What semantics remain weaker than the canonical OpenAI path after this slice?
7. What future misreading would be dangerous if someone skimmed only the report summary?
8. What remainder still exists toward maximum realizable parity, and why is it not in this slice?

## Artifact Requirements

### Strict Specs

A strict spec should include:

1. explicit scope boundary;
2. adjacent items or surfaces still open;
3. transitional residue allowed during the slice;
4. stale or invalid code targeted for removal or downgrade;
5. seam notes if the slice may touch `codex-core` or another preserved Codex seam;
6. a closure gate that states what must be true before the item can be called honestly closed.

### External-Agent Tasks

An execution task should include:

1. explicit non-goals;
2. stale-code targets;
3. residual-risk prompts when the agent could overclaim closure;
4. required explicit answers for owner truth, residue, and adjacent-open surfaces.

### Sprint Packages

A sprint package should record:

1. which checkpoint re-evaluates adjacent items after substrate work lands;
2. which items are intentionally not included in the sprint;
3. which risks are expected to collapse only after specific earlier tasks close;
4. which false-discovery paths must be avoided during mid-sprint reassessment.

### Implementation Reports

An implementation report is incomplete without:

1. `Stale Code Removed`;
2. `Stale Code Still Remaining`;
3. `Residual Risks`;
4. `Parity Remainder` that separates the closed surface from the remaining path to maximum realizable parity;
5. `Explicit Answers` that separate the closed surface from neighboring still-open surfaces.

### Review Passes

A review pass must check:

1. whether the bounded slice actually closed the named surface;
2. whether adjacent-open surfaces were left explicit;
3. whether compatibility residue is honestly described;
4. whether a new seam now requires ongoing governance attention.

## Worked Example: Item 05 Versus Item 11

`Item 05` is the model case for why this guide exists.

1. The closed surface was one-shot `thread/shellCommand` parity.
2. The adjacent-open surface was `Item 11` background terminals.
3. Shared `host_exec` ownership moved execution lifecycle out of the OpenRouter lane, but that did not create a full PTY or long-lived terminal substrate.
4. `thread/backgroundTerminals/clean` only became a narrow cancellation bridge for the touched one-shot surface.
5. OpenRouter shell semantics remained honestly weaker than canonical because the touched path stayed `unsandboxed` with `skip approval`.
6. The new `codex-core` seam was justified, but it also became a future seam-expansion risk if later work starts piling unrelated provider glue into `host_exec`.
7. The parity remainder after Item 05 was full background-terminal/process lifecycle ownership, which stayed explicitly open in Item 11.

The correct closeout was therefore:

1. `Item 05` closed for its bounded shell-command parity slice;
2. `Item 11` still open;
3. residual risks mandatory to record, not optional commentary.

## Hard Rules

1. Do not close an adjacent item by implication.
2. Do not describe compatibility residue as steady-state owner truth.
3. Do not hide stale owner logic behind wording like "legacy path" unless the code truly no longer owns the behavior.
4. Do not treat a shared mechanic as shared security posture unless the security semantics were actually implemented and proven.
5. Do not extend a Codex-base seam without recording why the outer additive route was insufficient.
6. If residual risks dominate the slice, route back to discussion/spec instead of widening implementation blindly.
7. Do not confuse a bounded closure with maximum realizable parity; record the remainder explicitly.

## Evidence

- path: `2026-04-14-orchestrated-agent-loop.md`
  symbol: `Default Loop`
  lines: 1-220
- path: `2026-04-14-glm-5.1-agent-profile.md`
  symbol: `Tasking Rules For This Agent`
  lines: 1-220
- path: `../../../AGENTS.md`
  symbol: `Documentation Workflow`
  lines: 1-120
- path: `../../07-logos-phase/implementation/2026-04-18-item-05-shell-execution-strict-spec.md`
  symbol: `Locked decisions`
  lines: 1-260
- path: `../../07-logos-phase/implementation/2026-04-18-item-05-shell-execution-implementation-report.md`
  symbol: `Residual Risks`
  lines: 1-220
- path: `../../07-logos-phase/implementation/2026-04-17-item-11-background-terminals-strict-spec.md`
  symbol: `Extraction Completion Rule`
  lines: 1-260
- path: `../../07-logos-phase/implementation/2026-04-03-codex-core-seam-index.md`
  symbol: `LOGOS Codex-Core Seam Index`
  lines: 1-240

## Links

- [[2026-04-14-orchestrated-agent-loop.md]]
- [[2026-04-14-glm-5.1-agent-profile.md]]
- [[../agent-navigation.md]]
- [[../../../AGENTS.md]]
- [[../../07-logos-phase/implementation/2026-04-03-codex-core-seam-index.md]]
- [[../../07-logos-phase/implementation/2026-04-18-item-05-shell-execution-strict-spec.md]]
- [[../../07-logos-phase/implementation/2026-04-18-item-05-shell-execution-implementation-report.md]]
- [[../../07-logos-phase/implementation/2026-04-17-item-11-background-terminals-strict-spec.md]]
