# RA-2.1 — Rollback Engine

## LOGOS Current Implementation

### Codex Path

**Where:** `codex-rs/core/src/codex.rs` → `handlers::thread_rollback`

`Op::ThreadRollback { num_turns }`:

- Drops last N user turns from in-memory `ResponseItem` array
- Does NOT revert filesystem changes
- Emits `EventMsg::ThreadRolledBack`
- After rollback, `previous_response_id` continues pointing to server state (unchanged)

### OR Path

**Where:** Rejected — "Thread rollback is not yet supported for OpenRouter beta threads."

**Why not implemented:** Requires:

1. Truncate history array (drop last N turns)
2. Rebuild SessionEngine with truncated history
3. Update continuation handle
4. Persist updated state
5. Optionally revert filesystem changes

### Related: Undo

**Where:** `codex-rs/core/src/tasks/undo.rs`

`Op::Undo`:

- Pops last turn from history
- Restores filesystem from ghost snapshot (git-based)
- More aggressive than rollback — includes file changes

## OpenCode Analysis

### Revert (Undo/Rollback)

**File:** `src/session/revert.ts:42-91`

Flow:

1. Assert session not busy
2. Walk all messages to find revert target (specific message or part)
3. Capture current filesystem snapshot if not already captured (`snapshot.track()`)
4. Apply patch reversion via `snapshot.revert(patches)`
5. Compute diff summary for the reverted range
6. Store diff in Storage and update session revert state
7. **On next prompt**: remove all messages >= revert target

### Unrevert (Redo)

**File:** `src/session/revert.ts:93-101`

1. Restore from saved snapshot via `snapshot.restore()`
2. Clear revert state

### Snapshot Mechanism

**File:** `src/snapshot/index.ts`

Uses separate git worktree:

- `track()`: `git add --sparse .` + `git write-tree` → tree hash
- `patch(hash)`: `git diff --cached --name-only <hash>` → changed files
- `restore(hash)`: `git read-tree <hash>` + `git checkout-index -a -f`
- `revert(patches)`: Batch `git checkout <hash> -- <file>` per patch
- Files > 2MB excluded via sparse checkout
- Auto-cleanup via scheduled GC

### Key Differences

| Aspect             | LOGOS (Codex)                                 | LOGOS (OR)      | OpenCode                               |
| ------------------ | --------------------------------------------- | --------------- | -------------------------------------- |
| Rollback mechanism | In-memory array truncation                    | Not implemented | Revert to message/part + filesystem    |
| Filesystem revert  | No (rollback) / Yes (undo via ghost snapshot) | N/A             | Yes (git worktree snapshots)           |
| Granularity        | N turns                                       | N/A             | Specific message or part               |
| Undo/redo          | Undo via ghost snapshot                       | N/A             | Revert + unrevert via saved snapshot   |
| Snapshot model     | Ghost commit per turn                         | N/A             | Separate git worktree with tree hashes |
| Persistence        | In-memory only (no disk update)               | N/A             | Snapshot stored, diff persisted        |
| After revert       | Continue with truncated history               | N/A             | Messages removed on next prompt        |

### Reusable Patterns

1. **Message-level granularity** (`revert.ts:42-91`): OpenCode can revert to a specific message or even a specific part within a message. This is more precise than "last N turns." Users can point to exactly where they want to go back to.

2. **Git worktree snapshots** (`snapshot/index.ts`): Using a separate git worktree for filesystem snapshots is robust:
   - Tree hashes are immutable and cheap to compute
   - No need to copy files — git manages it
   - Patch computation via `git diff`
   - Revert via `git checkout`
   - Files > 2MB excluded (practical)

3. **Deferred message deletion** (`revert.ts:103-144`): OpenCode doesn't delete messages immediately on revert. It stores the revert state and removes messages on next prompt. This avoids mutating history during revert — cleaner separation.

4. **Revert + unrevert**: Full undo/redo cycle. If user reverts but changes mind, unrevert restores everything. Neither Codex nor OR has redo.

5. **Diff summary on revert**: OpenCode computes and displays what files changed during the revert. Helpful for user confidence.

### What to Avoid

1. **Don't skip filesystem revert**: Codex rollback only truncates context, not files. This is confusing — user rolls back but files stay changed. OpenCode's approach (revert both context and files) is strictly better for UX.

2. **Don't delete messages immediately**: OpenCode's deferred deletion avoids partial state issues.

## Goose Analysis

### Rollback / Revert

**Not implemented.** Goose has no rollback, undo, or revert mechanism.

The closest operations are:

1. **`truncate_conversation(session_id, timestamp)`** (`session_manager.rs:1460-1469`):
   - `DELETE FROM messages WHERE session_id = ? AND created_timestamp >= ?`
   - Hard delete, no snapshot, no undo, no filesystem revert

2. **`delete_session(session_id)`** (`session_manager.rs:1305-1331`):
   - Full session deletion (messages + session row)
   - No recovery

3. **`export_session(id)`** / **`import_session(json)`**:
   - Manual backup/restore via JSON serialization
   - User can export before risky operations as a poor man's rollback

### What's Missing

- No checkpoint/snapshot mechanism
- No undo/redo stack
- No git-based filesystem revert
- No message-level rollback
- No diff computation for changes made during a session
- No "ghost snapshot" like Codex

### Comparison

| Aspect            | Goose                                   |
| ----------------- | --------------------------------------- |
| Rollback support  | Not implemented                         |
| Filesystem revert | Not implemented                         |
| Granularity       | Timestamp-based truncation only         |
| Undo/redo         | Not implemented                         |
| Deferred deletion | Not implemented (immediate hard DELETE) |
| Diff display      | Not implemented                         |
| Manual backup     | Export/import JSON (user-managed)       |

## Cross-Project Comparison Matrix

| Aspect            | LOGOS (Codex)              | LOGOS (OR) | OpenCode                  | Goose                     |
| ----------------- | -------------------------- | ---------- | ------------------------- | ------------------------- |
| Rollback support  | Yes (in-memory only)       | No         | Yes (revert + filesystem) | No                        |
| Filesystem revert | Ghost snapshot (undo only) | N/A        | Git worktree snapshots    | No                        |
| Granularity       | N turns                    | N/A        | Specific message/part     | Timestamp truncation only |
| Undo/redo         | Undo only                  | N/A        | Revert + unrevert         | No                        |
| Deferred deletion | No                         | N/A        | Yes (on next prompt)      | No (immediate DELETE)     |
| Diff display      | No                         | N/A        | Yes                       | No                        |
| Manual backup     | No                         | N/A        | No                        | Export/import JSON        |

## Recommendations for LOGOS

### What to Borrow

1. **Message-level granularity**: Allow rollback to a specific message, not just "last N turns." More intuitive for users.

2. **Git-based filesystem snapshots**: If filesystem revert is offered, use git worktree approach. Immutable, cheap, well-tested. Codex already uses git for ghost snapshots — the concept exists.

3. **Deferred deletion**: Don't mutate history immediately. Store rollback target, apply on next turn. Cleaner state management.

4. **Revert + unrevert**: Offer full undo/redo. If user rolls back, they should be able to go forward again.

5. **Diff summary**: Show user what files will change before/during rollback.

### Implementation Considerations

1. **Two rollback modes**:
   - "Context only" (like Codex) — truncate history, don't touch files
   - "Full revert" (like OpenCode) — truncate history + revert filesystem

   Let user choose based on confidence level.

2. **Turn boundary definition**: For "last N turns" mode, a turn = user message + all tool calls + assistant response. For message-level mode, any message.

3. **SessionEngine rebuild**: After truncating history, rebuild SessionEngine with `seed_history()`. Same pattern as fork/resume.

### Risks and Open Questions

1. **Snapshot cost**: Git-based snapshots per step (OpenCode) vs per turn (Codex ghost snapshot). Per-step is more granular but more expensive. What granularity does LOGOS need?

2. **Large file exclusion**: OpenCode excludes files > 2MB. What's the right threshold for LOGOS?

3. **Concurrent access**: If another process modifies files between snapshot and revert, revert may conflict. Need conflict resolution strategy.
