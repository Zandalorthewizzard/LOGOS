---
id: logos-phase-2-openai-canonical-turn-loop-acceptance-report
title: LOGOS Phase 2 OpenAI Canonical Turn Loop Acceptance Report
type: report
status: active
tags: [implementation, report, phase-2, openai, turn-loop, logos]
related:
  [
    [README.md],
    [2026-04-03-phase-2-openai-canonical-turn-loop-strict-spec.md],
    [
      ../architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md,
    ],
    [../architecture/2026-04-03-logos-compaction-strategy-discussion.md],
    [2026-04-03-codex-core-seam-index.md],
  ]
depends_on:
  [
    [2026-04-03-phase-2-openai-canonical-turn-loop-strict-spec.md],
    [
      ../architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md,
    ],
    [../architecture/2026-04-03-logos-compaction-strategy-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 2 OpenAI Canonical Turn Loop Acceptance Report

## Scope

This report records the phase-2 preservation work for the canonical OpenAI turn loop after phase-1 boundary extraction was stabilized.

## Delivered Preservation Work

1. Explicit continuation remains the only cross-turn continuation mechanism on the LOGOS OpenAI path.
2. Fresh request-scoped `ModelClientSession` instances now preserve `x-codex-turn-state` within one logical turn and reset it between turns, including after follow-up request failures.
3. Adapter normalization preserves stream semantics without duplicate tool or reasoning emissions.
4. OpenAI compaction is modeled honestly as a native remote rewrite backend.
5. `logos-session-engine` now exposes a provider-native compaction handoff that preserves the canonical OpenAI remote compact fast path without introducing portable compaction early.
6. Compacted history normalization now matches canonical `compact_remote` filtering by dropping provider-produced developer, tool, and reasoning artifacts.

## Acceptance Mapping

### A1. Canonical continuation remains explicit and lifecycle-correct

- Met.
- `responses_websocket_v2_uses_explicit_previous_response_id_without_incremental_carryover` proves explicit continuation survives request construction.
- `continuation_handle_is_preserved_for_follow_up_turn` proves the normalized OpenAI boundary reuses only the explicit continuation handle across turns.

### A2. Turn-state preservation remains canonical within logical turns

- Met.
- `turn_state_persists_within_runtime_tool_turn_and_resets_after` proves sticky-routing state survives fresh request-scoped sessions within one logical turn and clears after the turn completes.
- `turn_state_resets_after_follow_up_error` proves stale turn-state does not leak into the next user turn after a failed follow-up request.

### A3. Stream ordering and terminal semantics remain canonical

- Met for the current preservation scope.
- `output_item_added_and_done_do_not_duplicate_tool_start` proves tool framing is not duplicated.
- `reasoning_item_added_and_done_do_not_duplicate_reasoning_text` proves reasoning text is not duplicated.

### A4. OpenAI compaction path is preserved without shared-architecture distortion

- Met for the current phase-2 scope.
- `openai_session_exposes_honest_phase_two_capabilities` proves the adapter now advertises native remote compaction honestly.
- `compact_history_preserves_openai_native_fast_path_and_resets_continuation` proves the session engine can invoke the canonical OpenAI remote compact path, replace normalized history with a compaction checkpoint, and clear stale continuation afterward.
- `compacted_history_normalizes_compaction_and_messages` proves compacted history filtering drops provider-produced tool and reasoning artifacts while preserving the canonical summary/message shape.

### A5. Later persistence/resume work has explicit handoff hooks

- Met for the current preservation scope.
- `ProviderCapabilitySet.compaction_backend` provides an honest adapter-native backend classification.
- `SessionEngine::compact_history()` is an explicit phase-2 handoff point without broad journal/resume redesign.

## Phase-2 Completion State

The phase-2 canonical OpenAI turn-loop preservation scope is logically complete for the current strict spec.

Completed in this phase:

1. explicit continuation and continuation priority are preserved;
2. turn-state remains canonical within one logical turn;
3. stale turn-state is cleared on error exits;
4. normalized event ordering stays honest;
5. OpenAI remote compaction remains adapter-native and preserved;
6. compacted history filtering matches canonical preservation rules;
7. phase-4 persistence/resume work now has explicit handoff surfaces instead of hidden coupling.

Still out of scope for this phase:

1. portable multi-provider compaction implementation;
2. durable journal/resume persistence logic;
3. non-OpenAI provider preservation or validation work.

## Codex-Base Seams Used

The phase-2 preservation lane depends on three narrow registered Codex-base seams.

1. Prompt tool injection seam
2. explicit continuation carryover seam
3. turn-state transfer hook seam

Each seam stays additive and separately registered in `2026-04-03-codex-core-seam-index.md`.

## Tests Run

1. `cargo test -p codex-core prompt_stores_explicit_previous_response_id --lib`
2. `cargo test -p codex-core --test all suite::turn_state::websocket_turn_state_persists_within_turn_and_resets_after`
3. `cargo test -p logos-provider-openai --lib`
4. `cargo test -p logos-session-engine`
5. `just fix -p codex-core`
6. `just fix -p logos-runtime-types`
7. `just fix -p logos-provider-openai`
8. `just fix -p logos-session-engine`
9. `just fmt`

## Deferred Follow-Ups

1. phase-4 durable journal and resume implementation
2. portable compaction policy and backend selection beyond OpenAI native remote rewrite
3. later provider validation phases

## Evidence

- path: `../../../codex-rs/core/src/client.rs`
  symbol: `ModelClientSession::set_turn_state`
  lines: `681-689`
- path: `../../../codex-rs/core/src/client.rs`
  symbol: `ModelClientSession::current_turn_state`
  lines: `690-692`
- path: `../../../codex-rs/logos-provider-openai/src/adapter.rs`
  symbol: `OpenAiAdapter`
  lines: `1-203`
- path: `../../../codex-rs/logos-provider-openai/src/map_history.rs`
  symbol: `map_compacted_history`
  lines: `1-114`
- path: `../../../codex-rs/logos-runtime-types/src/capabilities.rs`
  symbol: `CompactionBackendKind`
  lines: `1-29`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine::compact_history`
  lines: `92-119`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `compact_history_preserves_openai_native_fast_path_and_resets_continuation`
  lines: `182-233`
- path: `2026-04-03-phase-2-openai-canonical-turn-loop-strict-spec.md`
  symbol: `Acceptance and Tests`
  lines: `110-116`

## Links

- [[README.md]]
- [[2026-04-03-phase-2-openai-canonical-turn-loop-strict-spec.md]]
- [[../architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md]]
- [[../architecture/2026-04-03-logos-compaction-strategy-discussion.md]]
- [[2026-04-03-codex-core-seam-index.md]]
