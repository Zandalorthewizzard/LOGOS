# Branch Summary

## Branch

- `feat/phase1-boundary-extraction`

## Completed Milestones

- added the phase-1 LOGOS additive crates for runtime, provider, session, and tool ownership
- introduced one registered `codex-core` seam for Prompt tool injection
- introduced a second narrow `codex-core`/`codex-api` seam for explicit `previous_response_id` carryover
- forwarded runtime-owned tool declarations into the canonical OpenAI/Codex request path
- completed runtime-owned tool execution plus `ToolResult` reinjection in `logos-session-engine`
- added explicit approval request/outcome events plus capability-gated provider-native tool rejection
- stabilized OpenAI continuation around a fresh per-turn `ModelClientSession` plus explicit continuation handle propagation
- removed duplicate normalized tool/reasoning emissions from adapter event mapping
- proved text turns, continuation reuse, stable tool-call IDs, outbound tool forwarding, runtime tool roundtrip, and tool error reinjection

## Current Shape

- `logos-provider-openai` is a thin request/event adapter over the canonical Codex/OpenAI path
- `logos-session-engine` owns the normalized turn loop
- `logos-tool-runtime` owns runtime dispatch and execution glue
- `codex-core` only exposes the narrow Prompt helper seam registered in the seam index

## Phase-1 Status

- phase-1 is logically complete for the current strict-spec scope
- remaining deferred work belongs to later persistence/resume and provider-native fast-path phases, not to the core boundary slice itself

## Phase-2 Foundation

- created an architecture-first discussion/spec pair for `Phase 2: OpenAI Canonical Turn Loop`
- phase-2 is now framed as preservation of canonical OpenAI turn/session/compaction behavior, not as a pretext for broad refactoring
- started implementation with honest compaction-backend classification and explicit turn-state transfer for fresh request-scoped sessions inside one logical turn
- preserved the canonical OpenAI remote compaction fast path through a normalized session-engine handoff and added a phase-2 acceptance report

## Phase-2 Status

- phase-2 is logically complete for the current canonical OpenAI turn-loop preservation scope
- later persistence/resume and multi-provider compaction work remain explicitly deferred to later phases

## Resume Point

- next safe focus is either a later-phase strict spec or the future durable approval/resume layer
