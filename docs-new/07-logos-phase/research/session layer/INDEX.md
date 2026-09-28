# Session Layer Research — Progress Index

**Project:** Stateless Session Layer Comparative Analysis
**Started:** 2026-04-16
**Sources:**

- LOGOS: `C:\Work\proj\LOGOS`
- OpenCode: `C:\Work\proj\LOGOS\input-sources-temp\opencode-upstream\packages\opencode`
- Goose: `C:\Work\proj\LOGOS\input-sources-temp\goose-upstream\crates\goose\src`

---

## Execution Waves

### Wave 1 — Foundation

| Task ID | Mechanic                     | Document                        | OpenCode | Goose | Status |
| ------- | ---------------------------- | ------------------------------- | -------- | ----- | ------ |
| T1.1    | Stateless Thread Manager     | `RA-1.1-thread-manager.md`      | Done     | Done  | done   |
| T1.2    | Session Persistence Layer    | `RA-1.2-session-persistence.md` | Done     | Done  | done   |
| T1.7    | Archive/Unarchive + Metadata | `RA-1.7-archive-metadata.md`    | Done     | Done  | done   |

### Wave 2 — Orchestration

| Task ID | Mechanic                       | Document                                 | OpenCode | Goose | Status |
| ------- | ------------------------------ | ---------------------------------------- | -------- | ----- | ------ |
| T1.3    | Fork/Resume Orchestration      | `RA-1.3-fork-resume.md`                  | Done     | Done  | done   |
| T1.4    | Turn Lifecycle & Notifications | `RA-1.4-turn-lifecycle-notifications.md` | Done     | Done  | done   |
| T1.5    | Shell Command Execution        | `RA-1.5-shell-execution.md`              | Done     | Done  | done   |

### Wave 3 — Features

| Task ID | Mechanic                 | Document                         | OpenCode | Goose | Status |
| ------- | ------------------------ | -------------------------------- | -------- | ----- | ------ |
| T2.3    | Turn Interrupt           | `RA-2.3-turn-interrupt.md`       | Done     | Done  | done   |
| T2.1    | Rollback Engine          | `RA-2.1-rollback.md`             | Done     | Done  | done   |
| T1.6    | Review Orchestration     | `RA-1.6-review-orchestration.md` | Done     | Done  | done   |
| T2.4    | Background Terminal Mgmt | `RA-2.4-background-terminals.md` | Done     | Done  | done   |

### Wave 4 — Complex

| Task ID | Mechanic                | Document                           | OpenCode | Goose | Status |
| ------- | ----------------------- | ---------------------------------- | -------- | ----- | ------ |
| T2.2    | Local Compaction Engine | `RA-2.2-compaction.md`             | Done     | Done  | done   |
| T3.1    | Turn Steering           | `RA-3.1-turn-steering.md`          | Done     | Done  | done   |
| T3.2    | Advanced Compaction     | `RA-3.2-advanced-compaction.md`    | Done     | Done  | done   |
| T3.4    | Streaming Optimization  | `RA-3.4-streaming-optimization.md` | Done     | Done  | done   |
| T3.3    | Multi-Provider Handoff  | `RA-3.3-multi-provider-handoff.md` | Done     | Done  | done   |

---

## Summary Statistics

- Total tasks: 16
- Completed: 16
- In progress: 0
- Pending: 0
- OpenCode analyzed: 16/16
- Goose analyzed: 16/16

---

## Key Cross-Cutting Findings

### OpenCode Patterns Worth Adopting

1. **Runner-based concurrency guard** (prevent concurrent turns)
2. **Two-phase compaction** (tool output pruning + summarization)
3. **Event-sourced mutations** (SyncEvent for state changes)
4. **Soft-delete archive** (timestamp field, not file move)
5. **Per-message provider/model tracking** (enables future handoff)
6. **Doom loop detection** (3x identical tool calls)
7. **Partial state preservation on interrupt** (don't discard interrupted turns)

### Goose Patterns Worth Adopting

1. **SQLite persistence with schema migrations** (9 versioned migrations, WAL mode)
2. **Builder-pattern updates** (SessionUpdateBuilder, compile-time checked)
3. **Pluggable ToolInspector framework** (extensible permission/inspection pipeline)
4. **RepetitionInspector** (configurable doom loop detection with finding IDs)
5. **Canonical model registry** (accurate context limits per model)
6. **manages_own_context() flag** (provider opt-out from client-side compaction)
7. **Date-prefixed sequential IDs** (YYYYMMDD_N, natural ordering)
8. **LLM-generated session names** (auto-naming via fast model)
9. **30+ provider factory** (dynamic provider registration)

### Patterns to Avoid

1. Manual notification construction (derive from state changes)
2. Monolithic JSON persistence (use SQLite or append-only)
3. File-move archive (use soft delete)
4. Discarding interrupted turns (save partial content)
5. Single tokenizer for all providers (use provider-specific where available)
6. No concurrency guard (Goose doesn't have one either — LOGOS must add it)

### OpenCode Limitations (LOGOS Can Do Better)

1. No sandbox support for shell execution
2. No dedicated review feature
3. No background terminal persistence across restart
4. No turn steering capability
5. No advanced compaction (hierarchical, incremental)

### Goose Limitations (LOGOS Can Do Better)

1. No rollback/revert mechanism at all
2. No background terminal/PTY management
3. No concurrency guard for concurrent turns
4. No partial state preservation on interrupt
5. No tool output pruning or two-phase compaction
6. No per-message provider tracking (only session-level)
7. Key modules missing from analysis (agents/, context_mgmt/, permission/, execution/)
