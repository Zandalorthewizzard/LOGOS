---
id: logos-item-03-fork-resume-implementation-reference
title: Item 03 Fork / Resume Orchestration Implementation Reference
type: implementation-reference
status: active
tags:
  [logos, architecture, phase-6, session-state, fork, resume, implementation]
related:
  [
    [
      ../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md,
    ],
    [../research/session layer/RA-1.3-fork-resume.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
    [../implementation/2026-04-16-item-01-implementation-reference.md],
    [../implementation/2026-04-17-item-02-implementation-reference.md],
  ]
depends_on:
  [
    [
      ../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md,
    ],
    [../research/session layer/RA-1.3-fork-resume.md],
    [
      ../implementation/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md,
    ],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 03: Fork / Resume Orchestration Implementation Reference

## Purpose

Concrete code patterns extracted from Goose (Rust), OpenCode (TypeScript), and Codex (Rust) for each stage of the Item 03 draft implementation. This document maps production patterns to LOGOS equivalents, identifying what can be refactored, adapted, or must be written from scratch.

## Source Projects

| Project  | Language   | Key Files                                                                          | Lines |
| -------- | ---------- | ---------------------------------------------------------------------------------- | ----- |
| Goose    | Rust       | `crates/goose/src/session/session_manager.rs` — copy_session, truncate, CRUD       | 2032  |
| OpenCode | TypeScript | `src/session/index.ts` — fork with message ID remapping, parent tracking           | 887   |
| OpenCode | TypeScript | `src/session/session.sql.ts` — parent_id field, cascade deletes                    | 103   |
| Codex    | Rust       | `core/src/thread_manager.rs` — fork_thread with rollout truncation                 | 1037  |
| Codex    | Rust       | `protocol/src/protocol.rs` — InitialHistory enum                                   | 4762  |
| Codex    | Rust       | `state/src/runtime/threads.rs` — thread_spawn_edges with recursive CTE             | 1445  |
| LOGOS OR | Rust       | `app-server/src/openrouter_beta_threads.rs` — fork_thread, resume_persisted_thread | 2426  |

---

## Stage A: Fork Over Durable State

### A.1 Fork Flow: Copy Session + History

**Goose** (`session_manager.rs:1415-1458`) — Adapt:

```rust
async fn copy_session(
    &self,
    session_manager: &SessionManager,
    session_id: &str,
    new_name: String,
) -> Result<Session> {
    let original_session = self.get_session(session_id, true).await?;
    let new_session = self.create_session(
        original_session.working_dir.clone(),
        new_name,
        original_session.session_type,
        original_session.goose_mode,
    ).await?;
    let mut builder = session_manager.update(&new_session.id)
        .extension_data(original_session.extension_data)
        .schedule_id(original_session.schedule_id)
        .recipe(original_session.recipe)
        .user_recipe_values(original_session.user_recipe_values);
    if let Some(provider_name) = original_session.provider_name {
        builder = builder.provider_name(provider_name);
    }
    if let Some(model_config) = original_session.model_config {
        builder = builder.model_config(model_config);
    }
    builder = builder.goose_mode(original_session.goose_mode);
    builder.apply().await?;
    if let Some(conversation) = original_session.conversation {
        self.replace_conversation(&new_session.id, &conversation).await?;
    }
    self.get_session(&new_session.id, true).await
}
```

**LOGOS action:** Adapt. LOGOS fork must:

1. Resolve source from registry (not in-memory `Arc<>`) — uses Item 01 registry
2. Load history from JSONL/journal (not in-memory runtime) — uses Item 02 persistence
3. Copy metadata via partial update builder — uses Item 02 builder
4. Copy history to new JSONL file — new code
5. Record edge in `thread_edges` — new table (see A.3)

Goose's pattern is the closest Rust equivalent. The key adaptation: Goose loads from SQLite, LOGOS loads from registry + JSONL/journal.

### A.2 Fork Granularity: Message-Level Cutoff

**OpenCode** (`index.ts:511-546`) — Pattern-level reference:

```typescript
const fork = Effect.fn("Session.fork")(function* (input: {
  sessionID: SessionID;
  messageID?: MessageID;
}) {
  const directory = yield* InstanceState.directory;
  const original = yield* get(input.sessionID);
  const title = getForkedTitle(original.title);
  const session = yield* createNext({
    directory,
    workspaceID: original.workspaceID,
    title,
  });
  const msgs = yield* messages({ sessionID: input.sessionID });
  const idMap = new Map<string, MessageID>();
  for (const msg of msgs) {
    if (input.messageID && msg.info.id >= input.messageID) break;
    const newID = MessageID.ascending();
    idMap.set(msg.info.id, newID);
    const parentID =
      msg.info.role === "assistant" && msg.info.parentID
        ? idMap.get(msg.info.parentID)
        : undefined;
    const cloned = yield* updateMessage({
      ...msg.info,
      sessionID: session.id,
      id: newID,
      ...(parentID && { parentID }),
    });
    for (const part of msg.parts) {
      yield* updatePart({
        ...part,
        id: PartID.ascending(),
        messageID: cloned.id,
        sessionID: session.id,
      });
    }
  }
  return session;
});
```

Key properties:

- Message-level granularity: `if (input.messageID && msg.info.id >= input.messageID) break`
- ID remapping: `idMap` maps old → new message IDs
- Parent-child preservation: assistant messages keep their parent reference through remapped IDs
- No SessionEngine to rebuild: messages are just data rows

**Codex** (`thread_manager.rs:596-639`) — Directly relevant:

```rust
pub async fn fork_thread<S>(&self, snapshot: S, config: Config, path: PathBuf, ...) -> CodexResult<NewThread>
where S: Into<ForkSnapshot>
{
    let history = RolloutRecorder::get_rollout_history(&path).await?;
    let history = match snapshot {
        ForkSnapshot::TruncateBeforeNthUserMessage(nth) => {
            truncate_before_nth_user_message(history, nth, &snapshot_state)
        }
        ForkSnapshot::Interrupted => { /* handle mid-turn boundary */ }
    };
    self.state.spawn_thread(config, history, ...).await
}
```

**LOGOS action:** MVP: whole-history fork only. Post-MVP: adopt Codex `TruncateBeforeNthUserMessage` pattern for turn-boundary fork. The Codex approach is more aligned with LOGOS because both use history arrays (Codex: `Vec<RolloutItem>`, LOGOS: `Vec<InternalMessage>`), while OpenCode uses SQL rows.

For message-ID remapping (OpenCode pattern): LOGOS `InternalMessage` does not have inter-message parent references, so remapping is simpler — just copy the array. This is an advantage of LOGOS's flatter message model.

### A.3 Lineage Tracking: Edge Records

**Codex** (`state/src/runtime/threads.rs:84-108`) — Adapt:

```rust
pub async fn upsert_thread_spawn_edge(
    &self,
    parent_thread_id: ThreadId,
    child_thread_id: ThreadId,
    status: DirectionalThreadSpawnEdgeStatus,
) -> Result<()> {
    sqlx::query(r#"
        INSERT INTO thread_spawn_edges (parent_thread_id, child_thread_id, status)
        VALUES (?, ?, ?)
        ON CONFLICT(child_thread_id) DO UPDATE SET
            parent_thread_id = excluded.parent_thread_id,
            status = excluded.status
    "#)
    .bind(parent_thread_id.to_string())
    .bind(child_thread_id.to_string())
    .bind(status.as_ref())
    .execute(self.pool.as_ref()).await?;
    Ok(())
}
```

Recursive descendant queries (`threads.rs:134-189`):

```rust
pub async fn list_thread_spawn_descendants_with_status(
    &self,
    root_thread_id: ThreadId,
    status: DirectionalThreadSpawnEdgeStatus,
) -> Result<Vec<ThreadId>> {
    // Uses WITH RECURSIVE CTE for tree traversal
}
```

**OpenCode** (`session.sql.ts:23,42`) — Pattern-level reference:

```typescript
parent_id: text().$type<SessionID>(),
// ...
index("session_parent_idx").on(table.parent_id),
```

**LOGOS action:** Adapt Codex pattern. LOGOS `thread_edges` table (defined in Item 03 draft) adds `fork_type` column that Codex does not have (Codex edge status only). This supports distinguishing `fork`, `review`, `scheduled` origins.

The `ON CONFLICT DO UPDATE` upsert pattern from Codex is directly reusable. The recursive CTE pattern for descendant queries is directly reusable for future review orchestration (Item 06).

LOGOS also has `Thread.forked_from_id: Option<String>` already in the wire protocol. This is a simpler lineage surface for single-parent lookup. Edge records support richer queries.

### A.4 What LOGOS Must Write From Scratch

| Item                                               | Why not refactorable                                                                                 |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Fork from durable state (registry + JSONL)         | No competitor has LOGOS's two-layer persistence (SQLite metadata + JSONL history). Must bridge both. |
| Fork over ephemeral runtime (source not in memory) | Competitors always load from DB. LOGOS must handle "source thread was evicted from memory."          |
| Config validation on fork                          | No competitor validates model/tool availability on fork.                                             |

---

## Stage B: Provider-Policy-Aware Resume

### B.1 Resume From Durable State

**Current LOGOS OR** (`openrouter_beta_threads.rs:413-479`) — Current implementation to replace:

```rust
pub(crate) async fn resume_persisted_thread(&self, thread_id: ThreadId, ...) {
    let Some(persisted) = openrouter_beta_persistence::load_active_thread(
        self.codex_home.as_path(), &thread_id.to_string()
    ).await.map_err(|error| ...)? else { return Ok(None); };
    let config_snapshot = persisted.config_snapshot.to_core();
    let runtime = OpenRouterBetaSession::forked_from(
        persisted.history, persisted.continuation, settings
    ).await.map_err(|error| ...)?;
    let thread = Arc::new(OpenRouterBetaThread { runtime: Mutex::new(runtime), state: ..., running_commands: ... });
    self.threads.lock().await.insert(thread_id, thread.clone());
    let _ = thread_state_manager.try_add_connection_to_thread(thread_id, connection_id).await;
    self.persist_thread(thread).await.map_err(|error| ...)?;
    Ok(Some(ThreadResumeResponse { ... }))
}
```

Problems this reference must solve:

1. Loads from monolithic JSON → must load from SQLite metadata + JSONL history
2. Rebuilds `OpenRouterBetaSession::forked_from` → must use provider-policy-aware dispatch
3. Inserts into in-memory map → Item 01 ephemeral model may not keep thread resident
4. No validation of config/model/tools

**Codex** (`thread_manager.rs:453-470`) — Pattern reference for resume:

```rust
pub async fn resume_thread_from_rollout(
    &self, config: Config, path: PathBuf, ...
) -> CodexResult<NewThread> {
    self.state.spawn_thread(
        config, InitialHistory::Resumed(ResumedHistory {
            conversation_id, history, rollout_path,
        }),
        ...,
    ).await
}
```

Codex resume = `spawn_thread` with `InitialHistory::Resumed`. The `InitialHistory` enum carries the history payload and lets the thread construction code handle the details.

**LOGOS action:** Adapt Codex pattern. LOGOS equivalent of `InitialHistory`:

```rust
enum SessionHistorySource {
    Fresh,
    Resumed { history: Vec<InternalMessage>, continuation: Option<ContinuationHandle> },
    Forked { history: Vec<InternalMessage>, parent_id: ThreadId },
}
```

This enum is the LOGOS equivalent of Codex's `InitialHistory` but uses `InternalMessage` instead of `RolloutItem` and `ContinuationHandle` instead of server-managed state.

### B.2 Continuation Mode Dispatch

**No competitor equivalent.** Neither Goose nor OpenCode has continuation mode dispatch because:

- Goose: no continuation concept, always loads from SQLite
- OpenCode: no continuation concept, always loads from SQLite
- Codex: single continuation mode (server-managed `previous_response_id`)

LOGOS uniquely has multiple continuation modes (`continuation.rs:12-17`):

```rust
pub enum ContinuationMode {
    ServerManaged,
    ClientManaged,
    Hybrid,
}
```

**LOGOS action:** Write from scratch. Dispatch logic:

```rust
fn resolve_resume_continuation(
    mode: ContinuationMode,
    persisted_handle: Option<ContinuationHandle>,
    history: &[InternalMessage],
) -> ResumeContinuation {
    match mode {
        ContinuationMode::ServerManaged => {
            match persisted_handle {
                Some(h @ ContinuationHandle::OpenAiPreviousResponseId(_)) => {
                    ResumeContinuation::Native(h)
                }
                _ => ResumeContinuation::Replay(history.to_vec())
            }
        }
        ContinuationMode::ClientManaged => {
            ResumeContinuation::Replay(history.to_vec())
        }
        ContinuationMode::Hybrid => {
            match persisted_handle {
                Some(h @ ContinuationHandle::OpenAiPreviousResponseId(_)) => {
                    ResumeContinuation::NativeWithReplayFallback(h, history.to_vec())
                }
                _ => ResumeContinuation::Replay(history.to_vec())
            }
        }
    }
}
```

This follows the execution policy from the ownership doc: provider-native first → LOGOS-managed fallback.

### B.3 Validation Step

**No competitor equivalent.** RA-1.3 confirms: "None of the projects do this."

**LOGOS action:** Write from scratch. Validation checks:

1. **Model availability:** resolve model through provider adapter → check if reachable
2. **Tool compatibility:** check tool names in history against current tool registry
3. **Config freshness:** compare snapshot policy with current global policy
4. **Provider capability:** check if provider supports the continuation mode stored in config

Result is a `ResumeValidation` struct with per-check booleans and warnings list. Hard failures block resume. Soft failures produce warnings but allow continuation.

### B.4 Listener Reattachment

**Current LOGOS OR** already does this correctly (`openrouter_beta_threads.rs:459-461`):

```rust
let _ = thread_state_manager
    .try_add_connection_to_thread(thread_id, connection_id)
    .await;
```

**LOGOS action:** Preserve existing pattern. No changes needed. `ThreadStateManager::try_add_connection_to_thread` already owns the subscription/connection layer independently of thread ownership.

### B.5 What LOGOS Must Write From Scratch

| Item                                    | Why not refactorable                                |
| --------------------------------------- | --------------------------------------------------- |
| Continuation mode dispatch              | No competitor has multiple continuation modes       |
| Validation on resume                    | No competitor validates model/tool/config on resume |
| Resume from LOGOS two-layer persistence | No competitor has SQLite metadata + JSONL split     |

---

## Stage C: Review Integration Surface

### C.1 Review as Fork Consumer

**Current LOGOS OR** (`openrouter_beta_threads.rs:1180-1769`) — 589 lines of review logic. Review is already built on fork/resume in OR, but ad hoc.

**Codex** (`guardian/review_session.rs:170-176`) — Pattern reference:

```rust
async fn fork_initial_history(&self) -> Option<InitialHistory> {
    let items = self.recorder.get_rollout_items().await;
    if items.is_empty() { return None; }
    Some(InitialHistory::Forked(items))
}
```

Codex review forks rollout items as `InitialHistory::Forked`. Simple and clean.

**LOGOS action:** Item 06 will design this in detail. For Item 03, the only requirement is that `fork_type` in `thread_edges` supports the value `"review"`, and that fork/resume orchestration does not assume fork always means user-initiated branch.

---

## Cross-Cutting: What Codex Gets Right

| Pattern                                  | Source                      | Value for LOGOS                                     |
| ---------------------------------------- | --------------------------- | --------------------------------------------------- |
| `InitialHistory` enum for session origin | `protocol.rs:2259-2263`     | Clean discrimination of New/Resumed/Forked sessions |
| `thread_spawn_edges` with upsert         | `threads.rs:84-108`         | Robust lineage with ON CONFLICT handling            |
| Recursive CTE for descendants            | `threads.rs:134-189`        | Tree queries for review orchestration               |
| Turn-boundary truncation in fork         | `thread_manager.rs:610-613` | Precise fork granularity                            |
| `forked_from_id()` from history items    | `protocol.rs:2265-2280`     | Lineage extraction without separate table lookup    |

## Cross-Cutting: What Goose Gets Right

| Pattern                                        | Source                         | Value for LOGOS                    |
| ---------------------------------------------- | ------------------------------ | ---------------------------------- |
| `copy_session` preserves provider/model config | `session_manager.rs:1440-1448` | Forked session keeps same provider |
| Builder pattern for forked session metadata    | `session_manager.rs:1432-1450` | Clean partial construction         |
| Full conversation copy on fork                 | `session_manager.rs:1452-1454` | History follows fork               |

## Cross-Cutting: What OpenCode Gets Right

| Pattern                                 | Source              | Value for LOGOS                                |
| --------------------------------------- | ------------------- | ---------------------------------------------- |
| Message-level fork granularity          | `index.ts:523-524`  | Fork to any message, not just turn boundary    |
| ID remapping for parent-child messages  | `index.ts:521-533`  | Preserves message relationships across fork    |
| `parent_id` as first-class schema field | `session.sql.ts:23` | Simple single-parent lookup without edge table |
| Soft-archive with `time_archived`       | `session.sql.ts:37` | No file moves for state transitions            |

## Cross-Cutting: What All Get Wrong

| Pattern                       | Problem                                                | LOGOS Better Approach                       |
| ----------------------------- | ------------------------------------------------------ | ------------------------------------------- |
| No validation on resume/fork  | None check model/tool availability                     | Validation step before runtime construction |
| No continuation mode dispatch | All assume single resume semantic                      | Provider-policy-aware dispatch              |
| Fork requires live session    | Goose loads from DB but LOGOS OR requires live `Arc<>` | Fork from durable state via registry        |

## Implementation Priority

| Step      | Action                                                  | Source             | Estimated Effort |
| --------- | ------------------------------------------------------- | ------------------ | ---------------- |
| 1         | `SessionHistorySource` enum                             | Codex adapt        | 0.5 day          |
| 2         | Fork from registry + JSONL (replace `Arc<>` dependency) | Goose adapt + new  | 2 days           |
| 3         | `thread_edges` table + edge CRUD (migration 2)          | Codex refactor     | 1 day            |
| 4         | Continuation mode dispatch                              | Write from scratch | 1.5 days         |
| 5         | Validation step (model/tool/config)                     | Write from scratch | 1.5 days         |
| 6         | Resume from two-layer persistence                       | Adapt OR + new     | 2 days           |
| 7         | Listener reattachment preservation                      | Preserve existing  | 0 days           |
| 8         | Fork type support in edge records                       | New                | 0.5 day          |
| **Total** |                                                         |                    | **~9 days**      |

## Dependencies

| Dependency                    | What Item 03 Needs                                         | Status                       |
| ----------------------------- | ---------------------------------------------------------- | ---------------------------- |
| Item 01 (Thread Manager)      | Registry to resolve threads without live `Arc<>`           | Draft done, reference done   |
| Item 02 (Session Persistence) | SQLite metadata + JSONL history to load from durable state | Draft done, reference done   |
| Phase-4 Journal               | Canonical history/continuation records                     | Spec locked, not implemented |

Item 03 Stages A and B cannot fully ship until Items 01 and 02 provide the registry and persistence substrate. However:

- `SessionHistorySource` enum (Step 1) can be designed now
- `thread_edges` schema (Step 3) can be added as migration 2 in Item 02's framework
- Validation logic (Step 5) can be designed and tested against mocks
- Continuation dispatch (Step 4) can be designed and unit-tested

## Evidence

- path: `../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md`
  symbol: `Draft Target Model`
  lines: `88-160`
- path: `../research/session layer/RA-1.3-fork-resume.md`
  symbol: `Cross-Project Comparison Matrix`
  lines: `171-182`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `copy_session`
  lines: `1415-1458`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/session/index.ts`
  symbol: `fork`
  lines: `511-546`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/session/session.sql.ts`
  symbol: `SessionTable`
  lines: `14-44`
- path: `../../../codex-rs/core/src/thread_manager.rs`
  symbol: `fork_thread`
  lines: `596-639`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `InitialHistory`
  lines: `2259-2296`
- path: `../../../codex-rs/state/src/runtime/threads.rs`
  symbol: `upsert_thread_spawn_edge`
  lines: `84-108`
- path: `../../../codex-rs/state/src/runtime/threads.rs`
  symbol: `list_thread_spawn_descendants_with_status`
  lines: `134-189`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `fork_thread`
  lines: `1075-1178`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `resume_persisted_thread`
  lines: `413-479`
- path: `../../../codex-rs/logos-runtime-types/src/continuation.rs`
  symbol: `ContinuationMode`
  lines: `12-17`
