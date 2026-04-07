---
id: logos-compaction-strategy-discussion
title: LOGOS Compaction Strategy Discussion
type: discussion
status: active
tags: [architecture, compaction, context, providers, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-provider-openai-adapter-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-02-logos-capability-model-architecture-spec.md],
    [../implementation/2026-03-30-logos-global-roadmap.md]
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-provider-openai-adapter-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Compaction Strategy Discussion

## Purpose

- Fix the current design direction for context compaction before later phases widen provider coverage.
- Preserve the canonical Codex/OpenAI compaction path without making it the universal kernel ontology.
- Record which phase should own the real implementation work so the topic does not disappear between phase-1 and later runtime work.

## Problem

- Codex already has working pre-turn, manual, and mid-turn compaction behavior.
- That behavior currently mixes three different concerns:
  1. turn-level compaction policy
  2. provider-specific compaction backend selection
  3. history replacement and context reinjection
- In current Codex base, remote compaction backend choice is effectively hard-coded to OpenAI provider detection.
- In current LOGOS runtime types, `supports_compaction: bool` is too weak to represent the real difference between:
  1. provider-native remote history rewrite
  2. portable Logos-side summary checkpoint compaction
  3. no compaction support

## Current Codex Behavior

### Trigger Surfaces

- Pre-turn auto-compaction runs before the next sampling request if token usage crosses the configured threshold.
- Mid-turn auto-compaction runs only when a follow-up request is still needed after sampling and the token limit has been reached.
- Manual `/compact` uses the same compaction machinery but starts from an explicit standalone task path.

### Current Backends

- Local compaction creates a handoff-style summary from the existing compaction prompt and rebuilds compacted history locally.
- Remote compaction delegates history rewriting to the OpenAI `/responses/compact` path and then filters the returned transcript before reinjection.

### Current Instruction-Safety Mechanisms

- `InitialContextInjection::DoNotInject` is used for pre-turn and manual compaction so the next real user turn fully reinjects canonical initial context.
- `InitialContextInjection::BeforeLastUserMessage` is used for mid-turn compaction so the model sees the compaction item in the expected place while canonical context is reinserted above the last real user message.
- Remote compaction output is not trusted as canonical instruction truth: stale `developer` messages are dropped and only selected message kinds survive.
- `reference_context_item` and `build_initial_context()` remain the authoritative baseline for later context reinjection.

## Design Position

- Compaction is primarily a LOGOS session-engine policy surface.
- Provider adapters own compaction backends, not the top-level compaction workflow.
- OpenAI remote compaction remains the canonical adapter-native fast path.
- Portable non-OpenAI compaction must be modeled honestly as Logos-owned summary checkpointing, not as fake OpenAI-style remote compaction.

## Ownership Split

### Session Engine Owns

1. Manual `/compact`.
2. Auto-compaction trigger policy.
3. Pre-turn versus mid-turn selection.
4. `InitialContextInjection` policy.
5. History replacement semantics.
6. Durable checkpoint semantics and reinjection rules.
7. Safety rules around approvals, tool loops, and follow-up turns.

### Provider Adapter Owns

1. Whether a provider-native compaction backend exists at all.
2. Provider-specific request mapping for native compaction.
3. Provider-specific response filtering and normalization.
4. Any provider-native extension payloads needed for observability or later compatibility work.

## Safety Rules To Keep

1. Compaction must never become the canonical source of developer instructions, policy instructions, realtime baseline, or model-switch instructions.
2. Compaction output is a model checkpoint, not the durable truth of the conversation.
3. Initial context must continue to be reconstructed from canonical runtime/session state, not copied back from provider-produced compacted history.
4. Tool correlation, approval state, ghost snapshots, and continuation handles must remain reconstructible after compaction.
5. Auto-compaction must only run at stable turn boundaries and not in the middle of unresolved tool-call or approval control flow.

## Recommended Capability Direction

- The current provider capability flag `supports_compaction` should not be treated as the final model.
- Later strict spec work should replace or extend it with an explicit compaction backend classification.
- The minimum honest classification needed by LOGOS is:
  1. native remote history rewrite
  2. portable summary checkpoint
  3. disabled

## Recommended Runtime Direction

- Keep OpenAI remote compaction adapter-owned and preserved.
- Move compaction policy selection into the session-engine rather than encoding it as provider-name branching.
- Keep portable compaction separate from provider-native compaction even if both produce a similar user-visible result.
- Add threshold policy that aims below the trigger watermark after compaction so the agent does not immediately compact again on the next follow-up request.

## Phase Placement

### Design Lock Timing

- This design should be treated as locked now, during the late phase-1 and phase-2 planning window.
- The reason is simple: continuation, tool loop, and persistence work will otherwise keep baking in the wrong assumption that compaction is only a provider boolean or an OpenAI-only side path.

### Earliest Required Preservation Work

- Phase 2 must preserve the canonical OpenAI compaction path and avoid further hard-coding provider-name selection as architectural truth.
- That is preservation work, not the full multi-provider implementation.

### Primary Implementation Phase

- Phase 4 is the main implementation home for this design.
- Compaction replaces history, interacts with resume, and creates checkpoint-like artifacts, so it belongs with durable journal and resume policy work rather than as an isolated phase-2 transport tweak.

### Follow-On Validation

- Phase 5 and later provider phases should validate that non-OpenAI adapters can use portable compaction honestly without contaminating the canonical OpenAI path.

## Non-goals

1. No fake-neutral rewrite of OpenAI remote compaction.
2. No requirement that every provider expose an OpenAI-like remote compact endpoint.
3. No promotion of provider-produced compacted history into canonical instruction truth.
4. No immediate wide refactor of `codex-rs/core/compact*` during phase-1 stabilization.

## Open Questions

1. Whether compaction backend classification belongs inside `ProviderCapabilitySet` or needs its own dedicated policy surface.
2. Whether portable summary checkpointing should remain adapter-independent or allow provider-tuned prompt variants later.
3. What exact low-watermark policy should be used after auto-compaction in long tool-heavy turns.
4. Whether approval and tool-runtime checkpoints need dedicated compaction metadata in the future durable journal.

## Evidence

- path: `../../../codex-rs/core/src/compact.rs`
  symbol: `InitialContextInjection`
  lines: `35-51`
- path: `../../../codex-rs/core/src/compact.rs`
  symbol: `run_compact_task`
  lines: `70-83`
- path: `../../../codex-rs/core/src/compact.rs`
  symbol: `run_compact_task_inner`
  lines: `85-231`
- path: `../../../codex-rs/core/src/compact.rs`
  symbol: `insert_initial_context_before_last_real_user_or_summary`
  lines: `283-322`
- path: `../../../codex-rs/core/src/compact_remote.rs`
  symbol: `run_remote_compact_task`
  lines: `37-48`
- path: `../../../codex-rs/core/src/compact_remote.rs`
  symbol: `process_compacted_history`
  lines: `169-188`
- path: `../../../codex-rs/core/src/compact_remote.rs`
  symbol: `should_keep_compacted_history_item`
  lines: `206-233`
- path: `../../../codex-rs/core/src/codex.rs`
  symbol: `run_pre_sampling_compact`
  lines: `6210-6229`
- path: `../../../codex-rs/core/src/codex.rs`
  symbol: `run_auto_compact`
  lines: `6278-6293`
- path: `../../../codex-rs/core/src/codex.rs`
  symbol: `build_initial_context`
  lines: `3550-3621`
- path: `../../../codex-rs/core/src/codex.rs`
  symbol: `record_context_updates_and_set_reference_context_item`
  lines: `3760-3789`
- path: `../../../codex-rs/core/src/config/mod.rs`
  symbol: `model_auto_compact_token_limit`
  lines: `239-239`
- path: `../../../codex-rs/logos-runtime-types/src/capabilities.rs`
  symbol: `ProviderCapabilitySet`
  lines: `7-16`
- path: `../../../codex-rs/logos-provider-openai/src/adapter.rs`
  symbol: `provider_capabilities`
  lines: `64-75`
- path: `2026-04-02-logos-adapter-kernel-architecture.md`
  symbol: `Rule 2: Continuation Is A Policy Surface`
  lines: `85-88`
- path: `2026-04-02-logos-provider-openai-adapter-spec.md`
  symbol: `Open Questions`
  lines: `149-152`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Canonical Storage Boundaries`
  lines: `82-97`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 2: OpenAI Canonical Turn Loop`
  lines: `106-123`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 4: Persistence And Resume`
  lines: `144-161`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-04-02-logos-provider-openai-adapter-spec.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[2026-04-02-logos-capability-model-architecture-spec.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
