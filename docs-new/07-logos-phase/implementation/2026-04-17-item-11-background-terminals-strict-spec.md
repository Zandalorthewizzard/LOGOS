---
id: logos-item-11-background-terminals-strict-spec
title: Item 11 Background Terminals Strict Spec
type: spec
status: active
tags:
  [logos, phase-6, strict-spec, session-state, background-terminals, host-ops]
related:
  [
    [README.md],
    [
      ../architecture/2026-04-16-item-11-background-terminals-draft-implementation-discussion.md,
    ],
    [
      ../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [../architecture/2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
depends_on:
  [
    [
      ../architecture/2026-04-16-item-11-background-terminals-draft-implementation-discussion.md,
    ],
    [
      ../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 11 Background Terminals Strict Spec

## Start Here

1. `AGENTS.md`
2. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
3. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md`
4. `docs-new/07-logos-phase/architecture/2026-04-16-item-11-background-terminals-draft-implementation-discussion.md`
5. `docs-new/07-logos-phase/research/2026-04-16-session-layer-research-transfer-ledger.md`
6. `codex-rs/core/src/state/service.rs`
7. `codex-rs/core/src/unified_exec/mod.rs`
8. `codex-rs/core/src/tools/handlers/unified_exec.rs`
9. `codex-rs/core/src/tasks/mod.rs`
10. `codex-rs/app-server/src/openrouter_beta_threads.rs`
11. `codex-rs/app-server/src/codex_message_processor.rs`
12. `codex-rs/app-server-protocol/src/protocol/v2.rs`
13. `codex-rs/app-server/README.md`
14. `codex-rs/tui/src/app_server_session.rs`
15. `codex-rs/tui/src/app.rs`
16. `codex-rs/tui/src/app/app_server_adapter.rs`
17. `codex-rs/app-server/src/bespoke_event_handling.rs`

## Intent

Extract the common local background-terminal and process-cleanup substrate out of the canonical owner shape into a shared LOGOS layer so non-canonical provider lanes can reuse it without permanent provider-owned bookkeeping.

Non-goals:

1. no broad `codex-core` rewrite for abstraction purity
2. no replacement of the preserved OpenAI canonical path
3. no full PTY feature expansion in the first wave
4. no persistent process recovery across restart in the first wave
5. no new TUI surface or app-server protocol surface in the first wave
6. no permanent OpenRouter-owned `running_commands` registry after closure

## Target Outcome

1. A shared LOGOS-owned process owner exists for thread-scoped process registration and cleanup.
2. The canonical local process substrate is reused through extraction or a narrow bridge instead of being copied into provider lanes.
3. OpenRouter no longer owns `running_commands` or the cleanup termination loop.
4. `thread/backgroundTerminals/clean` still works by `thread_id` through the existing app-server contract.
5. The existing TUI contract remains valid without TUI code changes in the first extraction wave.

## Parity Remainder

Item 11 closes the cleanup-owner / thread-scoped registry layer only. The remaining surface toward maximum realizable Codex/OpenAI-like background-terminal parity is:

1. `stdin` / `write_stdin`
2. `resize` / TTY control if the current terminal contract includes it
3. process visibility, including `thread_processes()` / `ps`-like inspection
4. output buffering and streaming, not only cancellation
5. attach / detach-like behavior if the UI exposes it
6. a terminal-interaction event pipeline that makes the TUI see a live background terminal rather than only a cancellable handle
7. restart handling if recoverable state is required instead of cleanup only

This remainder must stay explicit in the spec and in later implementation reports. It is not part of the first extraction wave, but it is also not hidden or implied away.

## Locked Decisions

1. Background-terminal and cleanup ownership belongs to LOGOS Layer 2, not to provider adapters or provider-specific lanes.
2. The main substrate for this item is already local in preserved Codex code: process management, stdin interaction, cleanup, and terminal interaction plumbing.
3. The correct implementation posture is extraction and re-hosting of that local substrate into a shared LOGOS layer, not a second provider-owned process stack.
4. The preserved OpenAI path must remain stable and must not be forced onto a weaker least-common-denominator implementation.
5. The first extraction wave keeps the existing app-server request, response, and notification contract stable.
6. Removal of OpenRouter-owned cleanup logic is part of closure for this item, not optional cleanup for later.
7. If a narrow Codex-base seam is required to expose reusable process-lifecycle substrate, it must be registered in the seam index as a dedicated seam change.

## Architecture Fit

1. The shared LOGOS session-state or host-exec layer owns thread-scoped process registration, cleanup, and minimal process bookkeeping.
2. The canonical path keeps its current session model, but bridges the relevant local process-lifecycle substrate into the shared owner instead of remaining the only owner shape.
3. OpenRouter and future non-canonical lanes become callers and event consumers; they no longer own process registries or cleanup loops.
4. TUI remains an app-server client. It should continue to speak only the existing protocol surface and must not care who owns the internal process manager.

## External Contract Compatibility

### App-Server Contract Lock

The first extraction wave must preserve:

1. `thread/backgroundTerminals/clean` as the cleanup method
2. `ThreadBackgroundTerminalsCleanParams { thread_id }`
3. `ThreadBackgroundTerminalsCleanResponse {}`
4. `TerminalInteractionNotification` as the existing notification shape for terminal interaction events

### TUI Compatibility Verdict

Based on current code evidence, the TUI does not depend on OpenRouter-owned `running_commands` or on any OpenRouter-specific cleanup implementation.

The TUI currently depends only on:

1. the existing app-server cleanup request/response
2. the existing `TerminalInteractionNotification`
3. thread-based routing of those notifications

So the first-wave extraction should not require TUI changes.

If implementation work discovers that TUI changes are required for this first wave, stop and route back to discussion. That would contradict the current evidence-backed boundary.

## Implementation Tasks

### T1. Shared Process Owner

- package: additive LOGOS-owned shared layer under `codex-rs/`; exact crate or module name may stay open, but final ownership must not remain in a provider lane
- files: new shared owner plus minimal integration surfaces
- output:
  1. register process handle
  2. associate process with `thread_id` and process id
  3. terminate all processes for one thread
  4. auto-prune exited processes
  5. expose narrow hooks for future stdin or terminal interaction reuse

### T2. Canonical Bridge

- package: preserved canonical path plus a narrow LOGOS bridge
- files: only the narrow surfaces needed to reuse the local process-lifecycle substrate
- output:
  1. preserved canonical behavior still works
  2. no second independent process-lifecycle implementation is introduced
  3. if a Codex-base seam is required, it is narrow, dedicated, and registered

### T3. OpenRouter Rewire

- package: app-server OpenRouter lane
- files: `codex-rs/app-server/src/openrouter_beta_threads.rs` and any adjacent glue
- output:
  1. shell execution registers processes in the shared LOGOS owner
  2. `thread/backgroundTerminals/clean` targets the shared owner by `thread_id`
  3. `running_commands` is deleted from `OpenRouterBetaThread`
  4. provider-owned cleanup termination loop is deleted from the OpenRouter lane

### T4. External Contract Proof

- package: app-server protocol, app-server, and TUI compatibility proof
- files: protocol and TUI files should remain untouched unless a real blocker is discovered
- output:
  1. protocol surface remains unchanged
  2. app-server request routing still works
  3. TUI compatibility remains intact without intervention

### T5. Documentation And Closure

- package: docs and seam register if needed
- files:
  1. the Item 11 draft discussion
  2. this strict spec
  3. seam index if a new Codex-base seam is introduced
- output:
  1. explicit proof that OpenRouter-owned cleanup logic was removed
  2. explicit note whether extraction needed a seam or only additive glue

## Requirement -> Task -> Test -> Gate

| Requirement | Task IDs | Tests | Acceptance | Gates |
| --- | --- | --- | --- | --- |
| Shared LOGOS owner exists for thread-scoped process cleanup | `T1` | focused tests in the shared owner crate or module | `A1` | one shared owner only; no provider-owned duplicate owner |
| Canonical local process substrate is reused without broad rewrite | `T2` | focused canonical no-regression test for touched process-cleanup path | `A2` | preserved OpenAI path stays stable |
| OpenRouter no longer owns cleanup bookkeeping | `T3` | focused `codex-app-server` tests for OpenRouter shell registration and background-terminal cleanup | `A3` | `running_commands` and provider-owned cleanup loop are removed |
| App-server and TUI contract stay stable | `T4` | `codex-app-server-protocol` tests plus focused app-server proof; no TUI code changes expected | `A4` | no protocol drift; no TUI intervention |

## Acceptance And Tests

### Acceptance Targets

#### A1. Shared Owner Proof

1. The first-wave implementation has one shared LOGOS owner for thread-scoped background-terminal cleanup.
2. That owner is outside provider lanes.

#### A2. Canonical No-Regression Proof

1. The preserved canonical path still uses the local process-lifecycle substrate successfully after the bridge or extraction.
2. No OpenAI degradation is introduced.

#### A3. OpenRouter Ownership-Removal Proof

1. `OpenRouterBetaThread` no longer owns `running_commands`.
2. `background_terminals_clean()` in the OpenRouter lane no longer owns the termination loop.
3. OpenRouter cleanup now targets the shared owner by `thread_id`.

#### A4. App-Server And TUI Boundary Proof

1. `ThreadBackgroundTerminalsCleanParams` and `ThreadBackgroundTerminalsCleanResponse` stay unchanged.
2. `TerminalInteractionNotification` stays unchanged for the first wave.
3. No TUI code changes are required.

### Expected Test Commands

1. run focused tests for the shared owner crate or module
2. `cargo test -p codex-app-server-protocol`
3. run focused `cargo test -p codex-app-server` coverage for `thread/backgroundTerminals/clean`
4. run one focused canonical cleanup or unified-exec no-regression test in the touched crate
5. if implementation unexpectedly touches TUI files, run focused `cargo test -p codex-tui` and treat that as a boundary exception that must be documented

## Promotion Artifacts

1. implementation report that states whether extraction used a bridge only or required a Codex-base seam
2. explicit closure note that lists the OpenRouter-owned cleanup fields and functions removed
3. targeted no-regression note for the app-server and TUI contract boundary

## Risks And Rollback

1. Risk: the useful local substrate may be too deeply entangled with canonical `SessionServices` to expose cleanly.
2. Mitigation: prefer a narrow bridge or seam over a broad rewrite; if a seam is needed, register it and keep it dedicated.
3. Risk: Item 11 may accidentally widen into full shell-command or PTY re-architecture.
4. Mitigation: keep the first wave limited to registration, cleanup, and minimal process bookkeeping.
5. Risk: implementation may silently preserve OpenRouter-owned cleanup logic alongside the shared owner.
6. Mitigation: treat explicit deletion of `running_commands` ownership as a hard acceptance condition.
7. Rollback: if extraction proves blocked, stop with `blocked by canonical process-lifecycle coupling` instead of landing a second permanent provider-owned process manager.

## Agent Guardrails

1. Do not introduce a second independent provider-owned process manager.
2. Do not widen the first wave into full PTY, terminal resize, persistent recovery, or richer terminal UI.
3. Do not change the existing app-server protocol or TUI surface in the first wave.
4. Do not degrade the preserved OpenAI path.
5. Do not treat OpenRouter-owned cleanup deletion as optional follow-up work.
6. Do not add a broad `codex-core` rewrite. If a seam is required, keep it narrow, explicit, and separately registered.

## Evidence

- path: `../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `The LOGOS-owned layer above provider wires should own`
  lines: `81-89`
- path: `../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Immediate Implications For The Current OpenRouter Beta Lane`
  lines: `124-131`
- path: `../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Move Now`
  lines: `85-93`
- path: `../architecture/2026-04-16-item-11-background-terminals-draft-implementation-discussion.md`
  symbol: `Correct target variant`
  lines: `54-54`
- path: `../architecture/2026-04-16-item-11-background-terminals-draft-implementation-discussion.md`
  symbol: `OpenRouter Stops Owning Cleanup`
  lines: `193-215`
- path: `../../../codex-rs/core/src/state/service.rs`
  symbol: `SessionServices`
  lines: `30-33`
- path: `../../../codex-rs/core/src/unified_exec/mod.rs`
  symbol: `UnifiedExecProcessManager`
  lines: `125-140`
- path: `../../../codex-rs/core/src/tools/handlers/unified_exec.rs`
  symbol: `WriteStdinRequest / TerminalInteractionEvent`
  lines: `334-351`
- path: `../../../codex-rs/core/src/tasks/mod.rs`
  symbol: `close_unified_exec_processes`
  lines: `535-537`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThread`
  lines: `81-89`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `background_terminals_clean`
  lines: `626-639`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `shell_command`
  lines: `646-769`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `thread_background_terminals_clean`
  lines: `3506-3538`
- path: `../../../codex-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `ThreadBackgroundTerminalsCleanParams`
  lines: `2988-2995`
- path: `../../../codex-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `TerminalInteractionNotification`
  lines: `5262-5268`
- path: `../../../codex-rs/app-server/README.md`
  symbol: `background terminals`
  lines: `580-586`
- path: `../../../codex-rs/tui/src/app_server_session.rs`
  symbol: `thread_background_terminals_clean`
  lines: `546-559`
- path: `../../../codex-rs/tui/src/app.rs`
  symbol: `AppCommandView::CleanBackgroundTerminals`
  lines: `2358-2363`
- path: `../../../codex-rs/tui/src/app/app_server_adapter.rs`
  symbol: `ServerNotification::TerminalInteraction`
  lines: `359-360`
- path: `../../../codex-rs/app-server/src/bespoke_event_handling.rs`
  symbol: `EventMsg::TerminalInteraction`
  lines: `1682-1693`

## Links

- [[README.md]]
- [[../architecture/2026-04-16-item-11-background-terminals-draft-implementation-discussion.md]]
- [[../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[../architecture/2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md]]
- [[../research/2026-04-16-session-layer-research-transfer-ledger.md]]
