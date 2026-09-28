---
id: logos-item-01-thread-manager-draft-implementation-discussion
title: Item 01 Thread Manager Draft Implementation Discussion
type: discussion
status: active
tags: [logos, architecture, phase-6, session-state, thread-manager, research-transfer]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [../implementation/2026-04-16-item-01-implementation-reference.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
    [../implementation/2026-04-14-phase-6-session-state-layer-extraction-strict-spec.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [../implementation/2026-04-16-item-01-implementation-reference.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 01 Thread Manager Draft Implementation Discussion

## Purpose

This document turns the validated `Item 01: Thread Manager` research verdict into a practical LOGOS implementation draft.

It is intentionally narrower than a strict implementation spec.

The goal is to make the accepted direction concrete against current code so LOGOS can later re-check whether `ACCEPT` still holds once real implementation work starts.

## Current Transfer Status

1. `Item 01` from the session-layer research ledger is now `DUAL_CONSENSUS_VALID`.
2. The accepted import is the core thread-manager pattern itself, not the full surrounding infrastructure from the compared systems.
3. This draft exists so future spec work can start from code-aware implementation shape instead of only from theory.

## Accepted Import For Item 01

The accepted pattern is:

1. storage-backed registry
2. load on demand
3. no permanent in-memory thread ownership between turns for stateless-provider lanes

This means the LOGOS target for stateless-provider lanes is not the current OpenRouter proving-lane shape where one manager permanently owns an in-memory `HashMap<ThreadId, Arc<...>>`.

## What This Draft Covers

This draft covers only the thread-manager core:

1. durable registry ownership
2. loaded-vs-active boundary
3. active-turn runtime boundary
4. concurrency-guard boundary
5. app-server dispatch simplification target

## What This Draft Does Not Cover

These remain separate items and must not be silently folded into `Item 01`:

1. full session persistence schema and migrations
2. background shell process ownership and lifecycle design
3. notification derivation and event bus design
4. review and fork orchestration
5. OpenAI canonical-path migration
6. multi-provider handoff

## Current Code Diagnosis

### OpenRouter Proving Lane Owns Too Much

Today the OpenRouter lane owns a permanent in-memory registry:

1. `OpenRouterBetaThreadManager` stores `threads: Arc<Mutex<HashMap<ThreadId, Arc<OpenRouterBetaThread>>>>`
2. each `OpenRouterBetaThread` owns:
   - `runtime: Mutex<OpenRouterBetaSession>`
   - `state: Mutex<OpenRouterThreadState>`
   - `running_commands: Mutex<HashMap<String, Arc<ProcessHandle>>>`

That makes the proving lane the owner of both thread identity and runtime residency.

### Non-canonical Persistence Is File-Snapshot Based Today

The current persistence layer stores non-canonical thread state under:

1. `logos-openrouter-beta/active-threads`
2. `logos-openrouter-beta/archived-threads`

This already proves that thread data is serializable enough to survive outside process memory, but it is still file-oriented and lane-owned.

### ThreadStateManager Already Lives Separately

`ThreadStateManager` already owns connection and subscription state outside `OpenRouterBetaThreadManager`.

That is important because it means thread registry ownership does not need to stay coupled to subscription ownership.

### SessionEngine Is Re-creatable

`SessionEngine::new(...)` creates a fresh provider session, and `seed_history(...)` restores history plus continuation.

That means the LOGOS runtime already has the minimum mechanics needed for an ephemeral runtime model:

1. create runtime on demand
2. seed persisted history
3. run one turn
4. persist updated state
5. drop runtime

### App-server Dispatch Is Branching On Lane-owned Presence Checks

`codex_message_processor.rs` currently repeats OpenRouter-vs-Codex branching by asking whether the OpenRouter lane currently contains a given thread.

That is the practical symptom that registry ownership is sitting at the wrong layer.

## Draft Target Model

### 1. Durable Non-canonical Thread Registry

Introduce a LOGOS-owned non-canonical thread registry layer under `codex-rs/`.

Working direction:

1. a new additive shared crate such as `logos-session-state`
2. SQLite-backed metadata registry for non-canonical threads
3. one durable source of truth for:
   - thread id
   - provider lane / provider id
   - display name
   - archived status
   - coarse lifecycle status
   - config snapshot reference
   - created / updated timestamps

This registry is the accepted imported core.

### 2. Ephemeral Runtime During Active Turn

For stateless-provider lanes, thread runtime should exist only while it is actively needed.

Practical meaning:

1. `thread/read`, `thread/list`, `archive`, and `unarchive` should not require a permanently resident thread object
2. `turn/start` loads durable state, constructs a fresh `SessionEngine`, runs the turn, persists the result, and drops the runtime
3. between turns, the registry owns identity and metadata; runtime does not remain the owner

This is the key operational translation of the accepted verdict.

### 3. Instance-scoped Services Stay Separate

The draft model assumes separate ownership for adjacent runtime surfaces:

1. `ThreadStateManager` remains the owner of connection/subscription state
2. a future `ProcessManager` or equivalent owns long-lived shell/background process state
3. a future turn/concurrency guard owns busy/idle exclusion

This prevents `Item 01` from re-growing into a monolithic thread object just because nearby services exist.

### 4. OpenAI Canonical Path Stays Separate

The draft does not require migration of the canonical OpenAI path.

The target boundary is:

1. `ThreadManager` continues to own the canonical Codex/OpenAI lane
2. the new LOGOS non-canonical registry serves stateless-provider lanes
3. app-server dispatch becomes less provider-lane-specific over time, but phase-6 does not require a forced unification of runtime semantics

## Practical Draft Shape

### Stage A: Registry First

First introduce durable registry ownership without trying to solve every adjacent item at once.

Stage-A target:

1. non-canonical thread metadata moves to SQLite-backed registry ownership
2. `read/list/archive/unarchive` stop depending on the OpenRouter in-memory map as the durable identity source
3. current JSON snapshot persistence can remain temporarily for history and continuation payloads until `Item 02: session persistence` is locked
4. legacy JSON sessions must be importable into the new registry on first run instead of being silently orphaned; Goose's `import_legacy()` posture is the reference pattern for this migration shape
5. during this temporary stage, `thread/read` with turns included may still load turn history from the existing JSON snapshot path while durable metadata moves first

This keeps `Item 01` bounded.

### Stage B: Runtime Residency Shrinks

Once registry ownership exists:

1. `start_turn` and related paths should construct runtime on demand
2. history and continuation load from persistence
3. runtime is dropped after persistence write completes
4. shell-command and background-terminal residency are not solved by this stage and continue to depend on the current `running_commands` ownership until a separate `ProcessManager` extraction item is implemented

At this point, the OpenRouter proving lane no longer owns permanent thread residency between turns.

### Stage C: Dispatch Simplification

After the new registry becomes authoritative for non-canonical lanes:

1. `codex_message_processor.rs` should stop repeating `if openrouter_beta_threads.contains(thread_id)` as the main ownership test
2. app-server should instead resolve the thread through durable lane-aware session-state metadata
3. lane-specific runtime bootstrap remains provider-sensitive, but registry ownership is no longer provider-lane-owned

## Minimal Draft Data Model

The draft intentionally keeps the registry record narrow.

One acceptable direction is:

1. `thread_records`
   - `thread_id`
   - `provider_lane`
   - `provider_id`
   - `name`
   - `archived_at`
   - `created_at`
   - `updated_at`
   - `session_state_ref`
   - `config_snapshot_ref`

The exact schema belongs to later persistence work.

For this draft, `session_state_ref` and `config_snapshot_ref` mean references to separately stored history/config payloads; the exact backing form is intentionally deferred until `Item 02: session persistence` locks the concrete schema.

The point of this draft is narrower:

1. metadata registry must be durable
2. active runtime must not be the long-term owner of thread identity
3. transient busy or active-turn state belongs to a separate runtime/concurrency surface, not to the durable registry record

## Why This Still Supports The `ACCEPT` Verdict

The accepted imported core remains intact:

1. storage-backed registry
2. load-on-demand ownership
3. no permanent in-memory thread residency between turns for stateless-provider lanes

The draft does not turn `ACCEPT` into `ADAPT` by quietly importing neighboring problems.

Instead it keeps the imported core strict and pushes adjacent mechanics back into their own decision surfaces.

## Revisit Triggers

The current `ACCEPT` verdict should be re-opened only if real implementation produces evidence that one of these is false:

1. non-canonical threads cannot preserve correct semantics without permanent in-memory residency between turns
2. `SessionEngine` reconstruction from durable state is insufficient for stateless-provider lanes
3. thread identity must remain coupled to process ownership for correctness rather than only for convenience
4. app-server subscription behavior proves that thread identity must stay tied to runtime residency

If those findings appear, `Item 01` may need to move from `ACCEPT` to `ADAPT`.

Absent that evidence, the correct current direction remains:

1. durable LOGOS-owned non-canonical registry
2. ephemeral runtime during active turn
3. separate instance-scoped services for subscriptions and processes

## Closeout Expectation For The Future Strict Spec

The later strict spec derived from this draft should be considered successful only if it proves:

1. non-canonical `thread/read` and `thread/list` no longer require permanent lane-owned in-memory thread residency
2. `archive/unarchive` act on durable registry state
3. `turn/start` reconstructs runtime from durable state for stateless-provider lanes
4. OpenAI canonical thread ownership remains preserved
5. adjacent items such as process ownership and notification derivation stay explicitly separate

## Evidence

- path: `../research/2026-04-16-session-layer-research-transfer-ledger.md`
  symbol: `Item 01`
  lines: `78-109`
- path: `2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Problem Statement`
  lines: `33-40`
- path: `2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Over-owned By The OpenRouter Lane Today`
  lines: `55-61`
- path: `2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Current Consensus`
  lines: `77-82`
- path: `2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Move Now`
  lines: `85-92`
- path: `2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Current Consensus`
  lines: `49-57`
- path: `2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Ownership Layers`
  lines: `59-103`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThreadManager`
  lines: `73-108`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThread`
  lines: `78-90`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `PersistedOpenRouterThread`
  lines: `24-47`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `active_thread_path`
  lines: `85-117`
- path: `../../../codex-rs/app-server/src/thread_state.rs`
  symbol: `ThreadStateManager`
  lines: `134-173`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `45-88`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `seed_history`
  lines: `135-154`
- path: `../../../codex-rs/core/src/thread_manager.rs`
  symbol: `ThreadManagerState`
  lines: `200-223`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `OpenRouter lane dispatch checks`
  lines: `2751, 2880, 5878, 7014`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `OpenRouter lane-aware listing`
  lines: `3648, 3656, 3797`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[../implementation/2026-04-16-item-01-implementation-reference.md]]
- [[../research/2026-04-16-session-layer-research-transfer-ledger.md]]
- [[../implementation/2026-04-14-phase-6-session-state-layer-extraction-strict-spec.md]]
