---
id: logos-item-11-background-terminals-draft-implementation-discussion
title: Item 11 Background Terminals Draft Implementation Discussion
type: discussion
status: active
tags:
  [
    logos,
    architecture,
    phase-6,
    session-state,
    background-terminals,
    host-ops,
    research-transfer,
  ]
related:
  [
    [README.md],
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-18-item-05-shell-execution-draft-implementation-discussion.md],
    [../implementation/2026-04-17-item-11-background-terminals-strict-spec.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
    [../research/session layer/RA-2.4-background-terminals.md],
  ]
depends_on:
  [
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md],
    [2026-04-18-item-05-shell-execution-draft-implementation-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 11: Background Terminals Draft Implementation Discussion

## Purpose

This document turns the validated `Item 11: Background Terminal Management` verdict into a practical LOGOS implementation draft.

## Current Transfer Status

1. `Item 11` verdict is `CUSTOM` with `DUAL_CONSENSUS_VALID`.
2. The `CUSTOM` arises because the key local substrate already exists in the preserved canonical Codex path: process registry, process-id allocation, stdin interaction, cleanup, and terminal interaction plumbing are already implemented locally.
3. Correct target variant: extract the common local substrate from the canonical path into a shared LOGOS layer.
4. External research is still useful here, but mainly as evidence for ownership shape, lifecycle rules, and future refinements such as bounded scrollback and process lifetime policy.

## Why `CUSTOM` Is The Correct Verdict

### Existing Local Substrate

The canonical path already owns a substantial local background-terminal stack:

1. `UnifiedExecProcessManager` in `core/src/unified_exec/`
2. session-owned process manager wiring in `state/service.rs`
3. `close_unified_exec_processes()` cleanup path
4. `WriteStdinRequest` support
5. `TerminalInteractionEvent` emission

This means the problem is not "how do we import background-terminal support from competitors?".

The problem is:

1. the useful substrate is trapped inside the canonical Codex session owner
2. non-canonical lanes cannot reuse it honestly without extraction
3. the OpenRouter proving lane therefore created a small parallel owner in `openrouter_beta_threads.rs`

### Why `ACCEPT` Is Misleading

`ACCEPT` would imply the research pattern itself is the main implementation source.

That is not true here. The main implementation source is already local:

1. preserved canonical Codex process management
2. preserved canonical cleanup semantics
3. preserved canonical terminal interaction plumbing

### Why `ADAPT` Is Also Misleading

`ADAPT` would imply the main work is reinterpreting the imported competitor pattern into LOGOS.

That is also not the main truth for Item 11. The main work is:

1. extracting or re-hosting the existing local substrate
2. correcting ownership
3. reducing duplicated process bookkeeping in provider lanes

Research still helps, but it is secondary to local extraction.

## What This Draft Covers

1. Coupling diagnosis for the current canonical background-terminal substrate.
2. The minimum shared process-lifecycle layer needed by non-canonical provider lanes.
3. Cleanup ownership for `thread/backgroundTerminals/clean`.
4. Process registry and thread/runtime association.
5. What should move now versus what should remain future work.

## What This Draft Does Not Cover

These remain separate items and must not be silently folded into Item 11:

1. Shell command execution orchestration (Item 05).
2. Turn interrupt semantics (Item 10).
3. Turn steering semantics (Item 12).
4. Persistence of process metadata across app restart.
5. Full PTY feature design such as resize, reconnect, and rich terminal UI.
6. Session persistence schema (Item 02).
7. Thread registry extraction (Item 01).

## Parity Remainder Toward Maximum Realizable Codex/OpenAI-Like Behavior

Item 11 currently closes only the first layer of the background-terminal story:

1. shared cleanup owner
2. thread-scoped registry
3. `thread/backgroundTerminals/clean` delegation

That is the bounded extraction we just proved.

If we want the touched background-terminal surface to work as nearly like Codex/OpenAI as the current architecture can honestly support, the remaining parity surface is:

1. `stdin` / `write_stdin`
2. `resize` / TTY control, if that is part of the current terminal contract
3. normal process visibility, including `thread_processes()` / `ps`-like inspection
4. output buffering and streaming, not only cancellation
5. attach / detach-like behavior, if the UI exposes it
6. a terminal-interaction event pipeline so the TUI sees a live background terminal instead of only a cancellable handle
7. restart handling, if the goal is recoverable state rather than cleanup only

This is the maximum-realizable parity remainder for Item 11. It must stay explicit in future specs, tasks, and reports so we do not accidentally treat a closed cleanup slice as a closed terminal product surface.

## Objective Limits: What Cannot Be Exactly the Same

Some differences are objective, not just implementation gaps:

1. **Live process recovery after app-server restart**
   - This will not be identical without an explicit checkpoint/restart or daemon-supervision model.
   - A local OS process does not become durable provider state on its own.
2. **Terminal emulator / platform behavior**
   - Resize, alternate screen, scrollback, PTY quirks, and stdin handling depend on the OS and terminal emulator.
   - We can approximate the behavior, but not honestly promise byte-for-byte identical UX.
3. **Provider-side semantics**
   - For OpenRouter, the provider is not the source of truth for terminal lifecycle.
   - The source of truth must remain the shared LOGOS-owned process manager, which makes the behavior similar but not provider-native identical.
4. **Security posture for the touched OpenRouter path**
   - The current shell path remains intentionally weaker (`unsandboxed`, `skip approval`).
   - Item 11 does not upgrade that posture, and it should remain explicitly documented as weaker than the canonical path.

In other words: "almost Codex parity" here means the shared process owner is capable of a fuller interactive terminal lifecycle, not just cleanup. It does not mean we can promise exact parity for restart-safe PTY recovery or platform-identical terminal semantics.

## Current Code Diagnosis

### Canonical Background-Terminal Substrate

The canonical path already has a real local process manager:

1. `SessionServices` owns `UnifiedExecProcessManager` in `core/src/state/service.rs`.
2. `UnifiedExecProcessManager` owns process allocation and runtime tracking in `core/src/unified_exec/`.
3. `UnifiedExecHandler` uses that manager for interactive exec processes and `WriteStdinRequest`.
4. `TerminalInteractionEvent` already exists as the canonical interaction event.
5. `close_unified_exec_processes()` already implements the canonical cleanup path.

This is the substrate future providers should reuse.

### OpenRouter Proving Lane

The OpenRouter proving lane currently owns a much smaller parallel stack:

1. `OpenRouterBetaThread` owns `running_commands: Mutex<HashMap<String, Arc<ProcessHandle>>>`.
2. `background_terminals_clean()` only iterates handles and calls `request_terminate()`.
3. There is no shared process registry outside the OpenRouter thread manager.
4. There is no reusable stdin interaction path.
5. There is no durable visibility model beyond the thread-local map.

This proves only the minimum cleanup capability, not the correct final owner.

### Why Direct Reuse Fails Today

The canonical substrate cannot be reused as-is because it is still canonically coupled:

1. process ownership is session-scoped through `SessionServices`
2. interaction and cleanup live on the canonical `Session` lifecycle
3. event emission assumes canonical event channels
4. provider lanes such as OpenRouter do not own `Session` or `TurnContext`

So the right move is not to create another provider-owned process stack.

The right move is to extract the useful local substrate into a shared owner.

## Draft Target Model

### 1. Shared LOGOS Process Owner

Introduce a LOGOS-owned process manager above provider wires and outside adapter lanes.

Working target shape:

```rust
struct LogosProcessManager {
    processes: Mutex<HashMap<LogosProcessId, Arc<ManagedProcess>>>,
    by_thread: Mutex<HashMap<ThreadId, BTreeSet<LogosProcessId>>>,
}
```

Responsibilities:

1. register a process handle
2. associate it with a LOGOS thread/runtime identity
3. terminate all processes for a thread
4. remove exited processes automatically
5. expose narrow hooks for future stdin or terminal interaction

### 2. Canonical Bridge, Not Canonical Replacement

The preserved OpenAI/Codex path should not be rewritten for symmetry.

Instead:

1. canonical path keeps working through its current session model
2. a narrow bridge adapts canonical `UnifiedExecProcessManager` behavior into the shared LOGOS process owner
3. non-canonical lanes target the shared owner directly

This keeps the preserved path stable while still collapsing duplicate ownership.

### 3. OpenRouter Stops Owning Cleanup

After extraction:

1. `openrouter_beta_threads.rs` no longer owns `running_commands`
2. OR shell execution registers processes in the shared LOGOS process owner
3. `thread/backgroundTerminals/clean` targets the shared owner by `thread_id`
4. OR lane becomes only a caller and event consumer, not the lifecycle owner

### Extraction Completion Rule

This removal is not optional cleanup work after the fact. It is part of the item itself.

Item 11 is not closed until:

1. `running_commands` is deleted from `OpenRouterBetaThread`
2. `background_terminals_clean()` in `openrouter_beta_threads.rs` stops owning the termination loop
3. provider-owned process bookkeeping is removed as the final owner for OpenRouter background-terminal cleanup
4. any transitional duplication is explicitly temporary and does not survive the closing implementation commit

### 4. Keep PTY Expansion Out Of The First Wave

The first implementation wave should not broaden into:

1. ring buffer scrollback
2. websocket transport
3. crash recovery by PID
4. persistent terminal registry across restart

### 5. First-Wave External Contract Stability

The first extraction wave should preserve the existing app-server and TUI contract:

1. `thread/backgroundTerminals/clean` remains the cleanup entrypoint by `thread_id`
2. the method still returns the existing empty response object
3. the existing `TerminalInteractionNotification` shape remains valid for canonical behavior and future non-canonical reuse
4. no TUI intervention is required for the first extraction wave unless later work broadens the terminal feature surface beyond the current contract

Those are future refinements, not prerequisites for correcting ownership now.

## Move Now

The first extraction wave for Item 11 should move:

1. process registration API
2. thread-scoped cleanup API
3. exit-driven auto-prune
4. minimal process metadata keyed by thread/process id

These are enough to remove the current OpenRouter-owned cleanup map.

## Move Later

The second wave can revisit:

1. stdin/write support for non-canonical lanes
2. bounded output buffering
3. richer terminal interaction surfaces
4. crash/restart recovery

## Reuse vs Duplication

The target must not duplicate the full canonical background-terminal stack.

The intended split is:

### Reuse

1. process manager semantics
2. cleanup semantics
3. terminal interaction substrate
4. local process lifetime logic

### Re-host

1. owner identity (`Session` -> shared LOGOS layer)
2. thread/runtime association
3. adapter-lane access path

### Avoid

1. a second independent provider-owned process manager
2. a permanent OpenRouter-only cleanup implementation
3. forced degradation of the canonical path

## Why This Still Supports `CUSTOM`

1. The substrate already exists locally and is worth preserving.
2. The imported research mainly validates the need for shared ownership and future refinements such as bounded scrollback and better lifetime policy.
3. The implementation task is extraction and re-hosting, not direct import and not large-scale canonical rewrite.

## Revisit Triggers

This `CUSTOM` verdict should be re-opened only if real implementation proves one of these:

1. `UnifiedExecProcessManager` is too deeply entangled with canonical session internals to extract even through a narrow bridge.
2. Non-canonical lanes require fundamentally different process-lifecycle semantics than the canonical local substrate can support.
3. Phase-6 support needs force immediate persistent process recovery, making the first-wave extraction too narrow.

If those findings appear, Item 11 may need to move from `CUSTOM` to `ADAPT`.

## Closeout Expectations

1. OpenRouter no longer owns `running_commands`.
2. `thread/backgroundTerminals/clean` targets a shared LOGOS-owned process owner.
3. The preserved canonical path remains stable.
4. Future non-canonical lanes can reuse the same local process substrate without cloning OR bookkeeping.

## Evidence

- path: `../../../codex-rs/core/src/state/service.rs`
  symbol: `SessionServices`
  lines: `24-33`
- path: `../../../codex-rs/core/src/unified_exec/mod.rs`
  symbol: `UnifiedExecProcessManager`
  lines: `125-140`
- path: `../../../codex-rs/core/src/tools/handlers/unified_exec.rs`
  symbol: `UnifiedExecHandler`
  lines: `179-195`
- path: `../../../codex-rs/core/src/tasks/mod.rs`
  symbol: `close_unified_exec_processes`
  lines: `535-537`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThread`
  lines: `81-89`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `background_terminals_clean`
  lines: `626-639`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::thread_background_terminals_clean`
  lines: `3506-3521`
- path: `../research/2026-04-16-session-layer-research-transfer-ledger.md`
  symbol: `Item 11`
  lines: `528-567`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md]]
- [[2026-04-02-logos-host-daemon-transport-spec.md]]
- [[2026-04-18-item-05-shell-execution-draft-implementation-discussion.md]]
- [[../implementation/2026-04-17-item-11-background-terminals-strict-spec.md]]
- [[../research/2026-04-16-session-layer-research-transfer-ledger.md]]
- [[../research/session layer/RA-2.4-background-terminals.md]]
