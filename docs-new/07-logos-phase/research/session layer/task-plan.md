# Comparative Analysis Task Plan

**Project:** Stateless Session Layer Research
**Date:** 2026-04-16
**Scope:** Systematic comparison of LOGOS session mechanics against OpenCode and Goose

---

## Purpose

For each mechanic listed in the discussion plan (`2026-04-16-stateless-session-layer-discussion-plan.md`), produce a structured comparative analysis document. The goal is to identify reusable patterns, avoid known pitfalls, and strengthen LOGOS design decisions before implementation.

---

## Task List

### Tier 1 — Migration + Audit

| Task ID | Mechanic                               | OpenCode | Goose       | Document                                 |
| ------- | -------------------------------------- | -------- | ----------- | ---------------------------------------- |
| T1.1    | Stateless Thread Manager               | Analyze  | Placeholder | `RA-1.1-thread-manager.md`               |
| T1.2    | Session Persistence Layer              | Analyze  | Placeholder | `RA-1.2-session-persistence.md`          |
| T1.3    | Fork / Resume Orchestration            | Analyze  | Placeholder | `RA-1.3-fork-resume.md`                  |
| T1.4    | Turn Lifecycle & Notification Dispatch | Analyze  | Placeholder | `RA-1.4-turn-lifecycle-notifications.md` |
| T1.5    | Shell Command Execution                | Analyze  | Placeholder | `RA-1.5-shell-execution.md`              |
| T1.6    | Review Orchestration                   | Analyze  | Placeholder | `RA-1.6-review-orchestration.md`         |
| T1.7    | Archive/Unarchive + Metadata           | Analyze  | Placeholder | `RA-1.7-archive-metadata.md`             |

### Tier 2 — Build from Scratch

| Task ID | Mechanic                       | OpenCode | Goose       | Document                         |
| ------- | ------------------------------ | -------- | ----------- | -------------------------------- |
| T2.1    | Rollback Engine                | Analyze  | Placeholder | `RA-2.1-rollback.md`             |
| T2.2    | Local Compaction Engine        | Analyze  | Placeholder | `RA-2.2-compaction.md`           |
| T2.3    | Turn Interrupt                 | Analyze  | Placeholder | `RA-2.3-turn-interrupt.md`       |
| T2.4    | Background Terminal Management | Analyze  | Placeholder | `RA-2.4-background-terminals.md` |

### Tier 3 — High Complexity

| Task ID | Mechanic                       | OpenCode | Goose       | Document                           |
| ------- | ------------------------------ | -------- | ----------- | ---------------------------------- |
| T3.1    | Turn Steering                  | Analyze  | Placeholder | `RA-3.1-turn-steering.md`          |
| T3.2    | Advanced Compaction Strategies | Analyze  | Placeholder | `RA-3.2-advanced-compaction.md`    |
| T3.3    | Multi-Provider Session Handoff | Analyze  | Placeholder | `RA-3.3-multi-provider-handoff.md` |
| T3.4    | Streaming Optimization         | Analyze  | Placeholder | `RA-3.4-streaming-optimization.md` |

---

## Document Template

Each analysis document must follow this structure:

```markdown
# RA-X.X — [Mechanic Name]

## LOGOS Current Implementation

- Where: file:lines
- What it does (brief)
- Known weaknesses

## OpenCode Analysis

- How OpenCode solves the same problem (or doesn't)
- Key source files and patterns
- Reusable patterns for LOGOS
- What to avoid

## Goose Analysis

<!-- PLACEHOLDER: To be filled -->

## Cross-Project Comparison Matrix

| Aspect | LOGOS | OpenCode | Goose |
| ------ | ----- | -------- | ----- |
| ...    | ...   | ...      | ...   |

## Recommendations for LOGOS

- What to borrow
- What to avoid
- What LOGOS can do better
- Risks and open questions
```

---

## Final Document Composition (after all analyses complete)

```
research/session-layer/
├── INDEX.md                              ← progress tracker
├── task-plan.md                          ← this file
├── RA-1.1-thread-manager.md
├── RA-1.2-session-persistence.md
├── RA-1.3-fork-resume.md
├── RA-1.4-turn-lifecycle-notifications.md
├── RA-1.5-shell-execution.md
├── RA-1.6-review-orchestration.md
├── RA-1.7-archive-metadata.md
├── RA-2.1-rollback.md
├── RA-2.2-compaction.md
├── RA-2.3-turn-interrupt.md
├── RA-2.4-background-terminals.md
├── RA-3.1-turn-steering.md
├── RA-3.2-advanced-compaction.md
├── RA-3.3-multi-provider-handoff.md
├── RA-3.4-streaming-optimization.md
└── SUMMARY.md                            ← synthesized findings and recommendations
```

---

## Execution Order

First pass: OpenCode analysis only, Goose placeholders.
Execution follows the priority order from the discussion plan:

**Wave 1 (foundation):**

1. T1.1 Thread Manager
2. T1.2 Session Persistence
3. T1.7 Archive/Metadata (simplest, quick win)

**Wave 2 (orchestration):** 4. T1.3 Fork/Resume 5. T1.4 Turn Lifecycle & Notifications 6. T1.5 Shell Execution

**Wave 3 (features):** 7. T2.3 Turn Interrupt 8. T2.1 Rollback 9. T1.6 Review 10. T2.4 Background Terminals

**Wave 4 (complex):** 11. T2.2 Compaction 12. T3.1 Turn Steering 13. T3.2 Advanced Compaction 14. T3.4 Streaming Optimization 15. T3.3 Multi-Provider Handoff
