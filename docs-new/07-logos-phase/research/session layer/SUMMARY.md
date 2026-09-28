# Session Layer Research — Cross-Project Comparative Summary

**Date:** 2026-04-16
**Scope:** LOGOS vs OpenCode vs Goose — 16 mechanic areas

---

## Master Comparison Table

| Mechanic                   | LOGOS (OR adapter)        | LOGOS (Codex)                  | OpenCode                              | Goose                               | LOGOS Recommendation                           |
| -------------------------- | ------------------------- | ------------------------------ | ------------------------------------- | ----------------------------------- | ---------------------------------------------- |
| **Thread/Session Manager** | HashMap always loaded     | HashMap always loaded          | SQLite on demand                      | SQLite on demand                    | SQLite on demand + TTL eviction for active     |
| **Persistence**            | Monolithic JSON           | JSONL + SQLite                 | SQLite + event sourcing               | SQLite (WAL, 9 migrations)          | SQLite for metadata + JSONL for history        |
| **Concurrency Guard**      | None                      | Single executor                | Runner + BusyError                    | None (SQLite tx only)               | Runner-based guard (from OpenCode)             |
| **Fork/Resume**            | Array clone, full history | JSONL copy, turn boundary      | SQL INSERT...SELECT, message boundary | SQL copy_session, full history      | SQL fork + message-level granularity           |
| **Turn Lifecycle**         | Spawned tokio task        | CodexThread Op loop            | Runner + runLoop                      | Agent::reply() → Stream             | Typed event stream + dedicated processor       |
| **Notifications**          | Manual construction       | EventMsg mapper                | SyncEvent + Bus                       | AgentEvent stream                   | Derive from state changes                      |
| **Shell Execution**        | Custom spawn              | Sandbox + tool handler         | AI SDK tool                           | MCP extension (external)            | Shared tool handler + sandbox option           |
| **Review/Permission**      | Elicitation counter       | Per-command approval           | Ruleset-based                         | Pluggable ToolInspector framework   | ToolInspector framework + per-command          |
| **Archive**                | File move                 | File move                      | Soft delete (timestamp)               | Hard delete only                    | Soft delete (timestamp field)                  |
| **Rollback**               | Not implemented           | In-memory array truncation     | Revert + filesystem (git)             | Not implemented                     | Message-level + git worktree snapshots         |
| **Compaction**             | Not implemented           | Server-side /responses/compact | Local (prune + summarize)             | Local (summarize via compaction.md) | Two-phase: prune + summarize (OpenCode)        |
| **Turn Interrupt**         | Not implemented           | CancellationToken              | AbortController + partial save        | CancellationToken (no partial save) | CancellationToken + partial state preservation |
| **Background Terminals**   | Not implemented           | UnifiedExecProcessManager      | PTY with ring buffer + WebSocket      | Not implemented                     | Instance-scoped + ring buffer                  |
| **Turn Steering**          | Not implemented           | pending_input queue            | Not implemented                       | Not implemented (max_turns unused)  | Post-MVP: queue-based injection                |
| **Advanced Compaction**    | N/A                       | N/A                            | Tool output pruning only              | Provider opt-out flag               | Two-phase + tool-type-aware pruning            |
| **Streaming**              | Full replay               | Server-side continuation       | Full replay + Anthropic cache         | Full replay + OpenRouter headers    | Full replay + provider-specific caching        |
| **Multi-Provider**         | Bound at creation         | Bound at creation              | Per-message deferred                  | Per-session, switchable             | Per-session deferred + per-message tracking    |

---

## Feature Presence Matrix

| Feature                     | LOGOS (OR) | LOGOS (Codex)      | OpenCode        | Goose              |
| --------------------------- | ---------- | ------------------ | --------------- | ------------------ |
| SQLite persistence          | No         | Partial (state DB) | Yes             | Yes                |
| Schema migrations           | No         | No                 | Yes (Drizzle)   | Yes (9 versions)   |
| Event sourcing              | No         | No                 | Yes             | No                 |
| Concurrency guard           | No         | Yes                | Yes             | No                 |
| Soft-delete archive         | No         | No                 | Yes             | No                 |
| Rollback/revert             | No         | Partial            | Yes             | No                 |
| Compaction                  | No         | Server-side        | Local (2-phase) | Local (1-phase)    |
| Tool output pruning         | No         | No                 | Yes             | No                 |
| Doom loop detection         | No         | No                 | Yes (3x)        | Yes (configurable) |
| Partial interrupt save      | No         | No                 | Yes             | No                 |
| Background terminals        | No         | Yes                | Yes             | No                 |
| Turn steering               | No         | Yes                | No              | No                 |
| Per-message provider        | No         | No                 | Yes             | No                 |
| Mid-session provider switch | No         | No                 | Yes             | Yes                |
| Pluggable tool inspection   | No         | No                 | No              | Yes                |
| Canonical model registry    | No         | No                 | No              | Yes                |
| LLM session naming          | No         | No                 | No              | Yes                |
| Chat history search         | No         | No                 | No              | Yes (SQL LIKE)     |
| Scheduled sessions          | No         | No                 | No              | Yes (cron)         |
| Export/import               | No         | No                 | No              | Yes (JSON)         |

---

## Pattern Recommendations by Priority

### Tier 1 — Adopt for MVP (Highest Impact)

| Pattern                            | Source        | Mechanic | Impact                               |
| ---------------------------------- | ------------- | -------- | ------------------------------------ |
| SQLite persistence with migrations | Goose         | RA-1.2   | Replaces fragile JSON files          |
| Builder-pattern updates            | Goose         | RA-1.7   | Clean, extensible metadata mutations |
| Runner-based concurrency guard     | OpenCode      | RA-1.4   | Prevents concurrent turn corruption  |
| Soft-delete archive                | OpenCode      | RA-1.7   | Reversible, queryable, simple        |
| CancellationToken per turn         | Goose + Codex | RA-2.3   | Clean interrupt mechanism            |
| RepetitionInspector                | Goose         | RA-1.5   | Configurable doom loop detection     |
| Canonical model registry           | Goose         | RA-2.2   | Accurate context limits per model    |

### Tier 2 — Build for MVP (Required for Stateless)

| Pattern                                 | Source   | Mechanic | Impact                               |
| --------------------------------------- | -------- | -------- | ------------------------------------ |
| Two-phase compaction                    | OpenCode | RA-2.2   | Prune (free) + summarize (expensive) |
| Partial state preservation on interrupt | OpenCode | RA-2.3   | Don't discard interrupted turns      |
| Derive notifications from state         | OpenCode | RA-1.4   | Eliminate manual construction bugs   |
| Pluggable ToolInspector framework       | Goose    | RA-1.6   | Extensible permission pipeline       |
| manages_own_context() flag              | Goose    | RA-3.2   | Provider opt-out from compaction     |

### Tier 3 — Post-MVP (Nice to Have)

| Pattern                       | Source   | Mechanic | Impact                          |
| ----------------------------- | -------- | -------- | ------------------------------- |
| Per-message provider tracking | OpenCode | RA-3.3   | Enables future provider handoff |
| Git worktree snapshots        | OpenCode | RA-2.1   | Filesystem rollback             |
| Tool-type-aware pruning       | New      | RA-3.2   | Smarter compaction              |
| Provider-specific tokenizers  | New      | RA-3.4   | Accurate overflow detection     |
| Instance-scoped PTY           | OpenCode | RA-2.4   | Background terminal management  |
| Chat history search           | Goose    | RA-1.2   | Cross-session search            |

---

## What LOGOS Should NOT Do

1. **Don't adopt Goose's lack of concurrency guard** — both Goose and LOGOS OR lack this. Must add.
2. **Don't adopt Goose's hard delete** — OpenCode's soft delete is strictly better.
3. **Don't adopt Goose's single tokenizer** — use provider-specific tokenizers where available.
4. **Don't adopt OpenCode's per-delta event sourcing** — only persist final turn state.
5. **Don't adopt any project's "no rollback" approach** — at minimum, implement message-level truncation.
6. **Don't adopt Goose's no-partial-save on interrupt** — OpenCode's partial finalization is better UX.

---

## Goose-Specific Notes

### Missing Source Modules

Several critical Goose modules were **not available** in the provided source checkout:

- `agents/` — Agent loop, AgentEvent stream, AgentConfig
- `context_mgmt/` — Compaction algorithm
- `conversation/` — Message, MessageContent, Conversation model
- `execution/` — Tool execution engine
- `permission/` — PermissionInspector, PermissionJudge
- `security/` — Security inspector

Analysis was inferred from usage sites in available code (scheduler.rs, ACP server references, provider base.rs). Some conclusions may be incomplete.

### Goose Architecture Strengths

1. **30+ provider support** via factory pattern — broadest provider coverage of any analyzed project
2. **SQLite with 9 migrations** — most mature schema evolution of analyzed projects
3. **Pluggable inspector framework** — most extensible permission/inspection system
4. **Canonical model registry** — JSON-based model metadata, auto-populates context limits
5. **Session type taxonomy** (7 types) — supports user, scheduled, sub-agent, hidden, terminal, gateway, ACP
6. **Scheduled sessions via cron** — unique feature not present in other projects

### Goose Architecture Weaknesses

1. **No rollback/revert** — closest is timestamp-based truncation
2. **No concurrency guard** — relies on SQLite tx isolation alone
3. **No background terminal/PTY** — process_group(0) is the extent of subprocess management
4. **No partial state on interrupt** — cancelled turns appear to be lost
5. **No tool output pruning** — single-phase summarization only
6. **No per-message provider tracking** — provider stored at session level
7. **Missing core modules** — agent loop, compaction, permissions not inspectable

---

## Source Files Analyzed

### Goose (C:\Work\proj\LOGOS\input-sources-temp\goose-upstream\crates\goose\src\)

| File                             | Lines | Content                                     |
| -------------------------------- | ----- | ------------------------------------------- |
| `lib.rs`                         | 44    | Module declarations                         |
| `session/session_manager.rs`     | 1700+ | SessionManager, SessionStorage, SQLite CRUD |
| `session/legacy.rs`              | 141   | Legacy JSONL import                         |
| `session/chat_history_search.rs` | 304   | Cross-session keyword search                |
| `session/extension_data.rs`      | 302   | Extension state storage                     |
| `session/diagnostics.rs`         | 145   | System diagnostics                          |
| `providers/base.rs`              | 1160  | Provider trait, MessageStream, Usage        |
| `providers/init.rs`              | 273   | Provider registry/factory                   |
| `providers/errors.rs`            | 161   | ProviderError enum                          |
| `providers/retry.rs`             | 302   | Exponential backoff                         |
| `providers/api_client.rs`        | 677   | Shared HTTP client                          |
| `providers/openrouter.rs`        | ~200  | OpenRouter adapter                          |
| `providers/toolshim.rs`          | 446   | Text-to-tool-call extraction                |
| `model.rs`                       | 555   | ModelConfig, context limits                 |
| `token_counter.rs`               | 305   | Token counting with tiktoken                |
| `tool_inspection.rs`             | 302   | ToolInspector trait framework               |
| `tool_monitor.rs`                | 135   | RepetitionInspector                         |
| `action_required_manager.rs`     | 99    | Permission request/response flow            |
| `scheduler.rs`                   | 1128  | Cron job scheduler                          |
| `scheduler_trait.rs`             | 41    | Scheduler trait                             |
| `session_context.rs`             | 81    | Task-local session ID                       |
| `subprocess.rs`                  | 38    | Process group isolation                     |
| `prompt_template.rs`             | 242   | Template rendering with minijinja           |
| `utils.rs`                       | 128   | Unicode sanitization, cancellation helper   |
