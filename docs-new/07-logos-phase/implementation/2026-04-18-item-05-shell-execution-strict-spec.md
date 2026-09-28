---
id: logos-item-05-shell-execution-strict-spec
title: Item 05 Shell Execution Strict Spec
type: spec
status: active
tags:
  [logos, phase-6, strict-spec, shell-execution, host-ops, session-state]
related:
  [
    [README.md],
    [../architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md],
    [../architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md],
    [
      ../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [../implementation/2026-04-18-item-05-implementation-reference.md],
    [../implementation/2026-04-17-item-11-background-terminals-strict-spec.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
depends_on:
  [
    [../architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md],
    [../architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md],
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

# Item 05 Shell Execution Strict Spec

## Start Here

1. `AGENTS.md`
2. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
3. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md`
4. `docs-new/07-logos-phase/architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md`
5. `docs-new/07-logos-phase/architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md`
6. `docs-new/07-logos-phase/implementation/2026-04-18-item-05-implementation-reference.md`
7. `docs-new/07-logos-phase/research/2026-04-16-session-layer-research-transfer-ledger.md`
8. `codex-rs/app-server-protocol/src/protocol/v2.rs`
9. `codex-rs/app-server/README.md`
10. `codex-rs/app-server/src/codex_message_processor.rs`
11. `codex-rs/app-server/tests/suite/v2/thread_shell_command.rs`
12. `codex-rs/core/src/codex.rs`
13. `codex-rs/core/src/tasks/user_shell.rs`
14. `codex-rs/core/src/exec.rs`
15. `codex-rs/core/src/unified_exec/mod.rs`
16. `codex-rs/core/src/tools/orchestrator.rs`
17. `codex-rs/core/src/state/service.rs`
18. `codex-rs/app-server/src/openrouter_beta_threads.rs`
19. `docs-new/07-logos-phase/implementation/2026-04-17-item-11-background-terminals-strict-spec.md`

## Intent

Implement `thread/shellCommand` parity for the OpenRouter beta lane through a shared LOGOS-owned host-exec substrate that reuses the mature local Codex execution mechanics instead of preserving a second provider-lane shell stack.

Non-goals:

1. no broad `codex-core` rewrite for abstraction purity
2. no degradation of the preserved OpenAI canonical path
3. no full background-terminal extraction or PTY lifecycle work in this slice
4. no general tool-loop shell unification beyond what is strictly required for `thread/shellCommand`
5. no protocol or TUI surface redesign
6. no fake claim that non-canonical shell execution suddenly gains product semantics that the existing contract does not provide

## Target Outcome

1. `thread/shellCommand` works for OpenRouter beta threads through the existing app-server product path.
2. The touched shell-command flow is driven by one shared LOGOS host-exec implementation, not by a permanent OpenRouter-only spawn/stream/capture loop.
3. The canonical `thread/shellCommand` path keeps its current observable behavior while reusing the shared extraction target or a narrow bridge into it.
4. The OpenRouter lane stops owning `drive_openrouter_shell_command()` and the provider-lane-specific spawn/stream/capture lifecycle for this surface.
5. The existing app-server contract remains stable, so the TUI `!` workflow should not need changes for this slice.
6. The resulting substrate leaves a clean extension point for `Item 11`, but does not claim to close background-terminal management itself.

## Locked Decisions

1. Shell execution is a shared LOGOS host/runtime concern, not a provider feature and not a permanent OpenRouter-lane responsibility.
2. The primary implementation source for this item is the existing local Codex shell/exec substrate, not competitor code.
3. OpenCode and Goose remain guardrail references only. They do not become the implementation owner for this slice.
4. The first implementation slice is about `thread/shellCommand` parity, not about every shell-like execution surface in the product.
5. The first slice must preserve the existing external `thread/shellCommand` contract and notification shapes.
6. The current user-facing `thread/shellCommand` semantics remain the contract for this slice. Do not silently widen them. If the implementation keeps explicit `NoSandbox` or `SkipApproval` semantics for this product surface, report that honestly.
7. If a narrow Codex-base seam is needed, it must be additive, tightly scoped, and registered in `2026-04-03-codex-core-seam-index.md`.
8. `Item 11` remains a separate item. Do not claim that shell-command parity also closes shared background-terminal cleanup.

## Architecture Fit

1. A shared LOGOS host-exec layer sits above provider wires and below product-lane orchestration.
2. The canonical path remains the reference behavior owner, but it stops being the only place where the local shell-command substrate is usable.
3. The OpenRouter beta lane becomes a caller of the shared host-exec layer instead of a second owner of process spawning, output streaming, and command-completion choreography.
4. The touched path may keep provider-specific validation and thread-state checks in the lane, but not provider-owned shell lifecycle mechanics.
5. The shared host-exec layer must not depend on `SessionEngine` or `OpenRouterBetaThread`; those remain caller-side orchestration types.

## External Contract Compatibility

### App-Server Contract Lock

The first slice must preserve:

1. `thread/shellCommand`
2. `ThreadShellCommandParams`
3. `ThreadShellCommandResponse`
4. `CommandExecutionOutputDeltaNotification`
5. the existing turn/item notification vocabulary already consumed by the current product path

### Current Product Semantics Lock

The first slice must preserve the current product posture for `thread/shellCommand`:

1. the request returns immediately with `{}` while progress streams through notifications
2. command output is persisted back into thread history on completion
3. no undocumented new approval or sandbox behavior may be implied by the refactor itself

### TUI Compatibility Expectation

Because the protocol surface stays unchanged, no TUI changes are expected for this slice. If implementation work discovers that TUI changes are required, stop and route back to discussion. That would mean the current boundary diagnosis was incomplete.

## Implementation Tasks

### T1. Shared HostExec Context Boundary

- package: additive LOGOS-owned host-exec layer under `codex-rs/`; exact crate or module name may remain open
- output:
  1. execution context shape that no longer requires `Session` or `OpenRouterBetaThread` as the owner type
  2. explicit policy values for sandbox and approval instead of hidden coupling
  3. explicit sink boundary for command begin, delta, and end notifications

### T2. Shared One-Shot Shell Runner

- package: shared LOGOS host-exec layer
- output:
  1. one-shot user shell-command execution path
  2. stdout/stderr streaming plus final output capture
  3. cancellation hook for the running command
  4. no third independent spawn/stream/capture implementation beyond the shared runner and any narrow bridge needed from canonical Codex code

### T3. Canonical Bridge For `thread/shellCommand`

- package: preserved canonical path plus a narrow bridge into the shared runner
- output:
  1. canonical `thread/shellCommand` still behaves the same externally
  2. touched canonical code reuses the shared runner or the shared extracted core for this product surface
  3. if a seam is required, it is narrow and separately documented

### T4. OpenRouter Rewire

- package: app-server OpenRouter lane
- output:
  1. `shell_command()` delegates to the shared host-exec runner
  2. `drive_openrouter_shell_command()` is removed
  3. provider-lane manual stdout/stderr loop is removed
  4. provider-lane manual command-completion choreography for this surface is removed
  5. any retained compatibility hook for `Item 11` is explicitly transitional and not the owner of shell-command execution

### T5. Contract Proof And Documentation

- package: protocol, app-server tests, docs, seam register if needed
- output:
  1. protocol surface unchanged
  2. OpenRouter parity tests added
  3. implementation report created with explicit stale-code accounting
  4. seam index updated if this slice introduces a Codex-base seam

## Stale Code Targets

The implementation report must explicitly account for these code paths:

1. `OpenRouterBetaThread::running_commands` in `openrouter_beta_threads.rs` as it relates to `thread/shellCommand`
2. `OpenRouterBetaThreadManager::shell_command`
3. `drive_openrouter_shell_command`
4. direct provider-lane use of `spawn_pipe_process_no_stdin()` for steady-state `thread/shellCommand` execution
5. any provider-lane-owned notification choreography that exists only because OpenRouter cannot yet call the shared host-exec layer

The slice may leave adjacent `Item 11` cleanup residue only if:

1. the retained code is clearly marked transitional
2. it is no longer the owner of shell-command execution semantics
3. the implementation report explains why it remains

## Requirement -> Task -> Test -> Gate

| Requirement | Task IDs | Tests | Acceptance | Gates |
| --- | --- | --- | --- | --- |
| Shared host-exec boundary exists for the touched shell-command surface | `T1`, `T2` | focused tests in the new or touched host-exec package | `A1` | no direct `Session` or OpenRouter-lane ownership in the shared runner |
| Canonical `thread/shellCommand` remains stable while reusing the shared extraction target | `T3` | existing canonical `thread_shell_command` tests plus any focused no-regression additions | `A2` | no observable protocol drift |
| OpenRouter no longer owns its manual shell driver loop | `T4` | focused `codex-app-server` OpenRouter shell parity tests | `A3` | `drive_openrouter_shell_command()` removed |
| Protocol and product contract remain stable | `T5` | `codex-app-server-protocol` tests plus touched app-server tests | `A4` | no API drift and no unsupported semantics claims |

## Acceptance And Tests

### Acceptance Targets

#### A1. Shared Runner Proof

1. The touched `thread/shellCommand` flow is implemented through one shared host-exec runner.
2. The shared runner is outside provider lanes.

#### A2. Canonical No-Regression Proof

1. The canonical `thread/shellCommand` tests continue to pass.
2. Canonical behavior remains unchanged for the touched path.

#### A3. OpenRouter Ownership-Removal Proof

1. `drive_openrouter_shell_command()` is deleted.
2. The OpenRouter lane no longer owns the steady-state spawn/stream/capture lifecycle for `thread/shellCommand`.
3. OpenRouter parity works in both standalone-turn and active-turn attachment modes.

#### A4. External Contract And Honesty Proof

1. `ThreadShellCommandParams`, `ThreadShellCommandResponse`, and `CommandExecutionOutputDeltaNotification` stay unchanged.
2. The implementation report explicitly states whether the OpenRouter shell-command path in this slice is `NoSandbox` / `SkipApproval`, or something else.
3. No background-terminal or PTY feature is claimed closed by this slice.

### Expected Test Commands

1. `cargo test -p codex-app-server-protocol`
2. run focused `cargo test -p codex-app-server` coverage for `thread_shell_command`
3. run focused `cargo test -p codex-app-server` coverage for the new OpenRouter shell parity tests
4. run focused tests for the new or touched shared host-exec package or module
5. if core files are touched, run the narrowest honest `cargo test -p codex-core` coverage for the touched shell-command path
6. `just fmt`
7. `just fix -p <changed crate>` for the touched crates when possible

## Promotion Artifacts

1. implementation report for the `Item 05` slice
2. explicit stale-code removal list for the OpenRouter lane
3. explicit statement on whether this slice needed a Codex-base seam
4. explicit statement on whether any `Item 11` compatibility residue remains

## Risks And Rollback

1. Risk: the canonical shell-command path may be too tightly coupled to `Session` and `TurnContext` to expose cleanly.
2. Mitigation: prefer a narrow bridge or seam over broad rewrites; if the seam is larger than expected, stop and document the blocker.
3. Risk: the slice may accidentally widen into background terminals or tool-exec redesign.
4. Mitigation: keep scope locked to `thread/shellCommand` parity plus the smallest reusable substrate required for that path.
5. Risk: the implementation may overclaim sandbox or approval support for the OpenRouter path.
6. Mitigation: keep the external product contract explicit and require the implementation report to state the actual policy used by the touched path.
7. Rollback: if the shared runner cannot be introduced without destabilizing canonical behavior, stop with a blocked report rather than landing a second permanent provider-owned implementation.

## Agent Guardrails

1. Do not widen this slice into `Item 11`.
2. Do not change the app-server protocol for `thread/shellCommand`.
3. Do not degrade the preserved OpenAI path.
4. Do not claim non-canonical sandbox or approval parity unless the code and tests actually prove it.
5. Do not add a broad `codex-core` rewrite.
6. Do not leave `drive_openrouter_shell_command()` in place and still call the slice closed.
7. If a seam is required, register it and keep the seam change separate when reasonably possible.

## Evidence

- path: `../research/2026-04-16-session-layer-research-transfer-ledger.md`
  symbol: `Item 05`
  lines: `271-307`
- path: `../architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md`
  symbol: `Practical Draft Shape`
  lines: `289-344`
- path: `../architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md`
  symbol: `Ownership Baseline`
  lines: `18-24`
- path: `../../../codex-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `ThreadShellCommandParams`, `CommandExecutionOutputDeltaNotification`
  lines: `2971-2987, 5274-5292`
- path: `../../../codex-rs/app-server/README.md`
  symbol: `thread/shellCommand`
  lines: `148-148, 434-453`
- path: `../../../codex-rs/app-server/tests/suite/v2/thread_shell_command.rs`
  symbol: `thread_shell_command_runs_as_standalone_turn_and_persists_history`, `thread_shell_command_uses_existing_active_turn`
  lines: `41-143, 158-320`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `thread_shell_command`
  lines: `3519-3573`
- path: `../../../codex-rs/core/src/codex.rs`
  symbol: `run_user_shell_command`
  lines: `4851-4876`
- path: `../../../codex-rs/core/src/tasks/user_shell.rs`
  symbol: `execute_user_shell_command`
  lines: `92-312`
- path: `../../../codex-rs/core/src/unified_exec/mod.rs`
  symbol: `UnifiedExecContext`, `UnifiedExecProcessManager`
  lines: `72-140`
- path: `../../../codex-rs/core/src/tools/orchestrator.rs`
  symbol: `ToolOrchestrator::run`
  lines: `101-199`
- path: `../../../codex-rs/core/src/state/service.rs`
  symbol: `SessionServices`
  lines: `30-61`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `running_commands`, `shell_command`, `drive_openrouter_shell_command`
  lines: `96-96, 1063-1170, 2443-2619`

## Links

- [[README.md]]
- [[../architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md]]
- [[../architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md]]
- [[../implementation/2026-04-18-item-05-implementation-reference.md]]
- [[../implementation/2026-04-17-item-11-background-terminals-strict-spec.md]]
- [[../research/2026-04-16-session-layer-research-transfer-ledger.md]]
