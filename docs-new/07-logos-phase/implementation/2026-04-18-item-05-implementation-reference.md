---
id: logos-item-05-shell-execution-implementation-reference
title: Item 05 Shell Execution Implementation Reference
type: implementation-reference
status: active
tags:
  [logos, architecture, phase-6, shell-execution, host-ops, implementation]
related:
  [
    [../architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md],
    [../architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md],
    [../implementation/2026-04-18-item-05-shell-execution-strict-spec.md],
    [../research/session layer/RA-1.5-shell-execution.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
    [../implementation/2026-04-17-item-11-background-terminals-strict-spec.md],
  ]
depends_on:
  [
    [../architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md],
    [../implementation/2026-04-18-item-05-shell-execution-strict-spec.md],
    [../research/session layer/RA-1.5-shell-execution.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 05 Shell Execution Implementation Reference

## Purpose

This document maps the `Item 05` implementation slice to concrete source material.

Unlike `Item 01-04`, the primary source here is not competitor code. The main reusable substrate already exists locally in preserved Codex code. OpenCode and Goose remain guardrail sources that help sharpen shell selection, subprocess isolation, and policy vocabulary without becoming the implementation owner.

## Source Projects

| Source | Language | Key Files | Role |
| --- | --- | --- | --- |
| Local Codex canonical path | Rust | `codex-rs/app-server/src/codex_message_processor.rs`, `codex-rs/core/src/codex.rs`, `codex-rs/core/src/tasks/user_shell.rs`, `codex-rs/core/src/exec.rs`, `codex-rs/core/src/unified_exec/mod.rs`, `codex-rs/core/src/state/service.rs` | Primary substrate to reuse or extract |
| Local OpenRouter duplicate path | Rust | `codex-rs/app-server/src/openrouter_beta_threads.rs` | Duplicate owner to collapse |
| OpenCode | TypeScript | `input-sources-temp/opencode-upstream/packages/opencode/src/shell/shell.ts`, `.../session/prompt.ts`, `.../pty/index.ts` | Guardrails for shell detection and process lifecycle ergonomics |
| Goose | Rust | `input-sources-temp/goose-upstream/crates/goose/src/subprocess.rs`, `.../tool_inspection.rs` | Guardrails for subprocess isolation and approval vocabulary |

---

## Stage A: Contract Lock And Shared Context

### A.1 Preserve The Product Contract

**Local Codex** (`codex-rs/app-server-protocol/src/protocol/v2.rs:2971-2987, 5274-5292`) - Directly reusable:

- `ThreadShellCommandParams`
- `ThreadShellCommandResponse`
- `CommandExecutionOutputDeltaNotification`

**Local Codex tests** (`codex-rs/app-server/tests/suite/v2/thread_shell_command.rs:41-143, 158-320`) - Directly reusable as acceptance contract:

- standalone `thread/shellCommand`
- active-turn attachment behavior
- output persistence and notification shape

**LOGOS action:** Treat these local protocol and test artifacts as the hard contract. Do not redesign the API while extracting ownership.

### A.2 Extract A Provider-Agnostic Execution Context

**Local Codex** (`codex-rs/core/src/tasks/user_shell.rs:92-151`) - Adapt:

- `execute_user_shell_command()` currently derives execution state from `Arc<Session>` and `Arc<TurnContext>`
- shell selection, event routing, and persistence are implicit in that owner shape

**Local Codex** (`codex-rs/core/src/unified_exec/mod.rs:72-86`) - Adapt:

- `UnifiedExecContext` shows which execution-time fields matter
- the type is still canonically coupled

**Local OpenRouter** (`codex-rs/app-server/src/openrouter_beta_threads.rs:1063-1170`) - Adapt:

- OpenRouter already builds the same practical inputs from thread state, item id, working directory, and notification sender

**LOGOS action:** Write a new shared `HostExecContext` or equivalent LOGOS-native type. This is not refactorable line-for-line because neither canonical nor OpenRouter currently owns a provider-agnostic context shape.

### A.3 Notification Sink Boundary

**Local Codex** (`codex-rs/core/src/tasks/user_shell.rs:117-277`) - Adapt:

- canonical path emits command lifecycle through `session.send_event(...)`

**Local OpenRouter** (`codex-rs/app-server/src/openrouter_beta_threads.rs:1098-1129, 2565-2617`) - Adapt:

- OpenRouter emits the same lifecycle directly through `send_notification(...)`

**LOGOS action:** Write a new sink trait or boundary that both paths can target. This is shared design work, not a direct refactor of either side.

### A.4 What LOGOS Must Write From Scratch

| Item | Why not directly refactorable |
| --- | --- |
| `HostExecContext` | the shared owner shape does not exist locally |
| `HostExecNotificationSink` | canonical and OpenRouter notification owners diverge |
| explicit policy enum(s) for shell-command execution | current policy is encoded in owner-specific orchestration rather than one shared type |

---

## Stage B: Shared One-Shot Shell Runner

### B.1 Canonical Spawn, Stream, Capture, Cancel

**Local Codex** (`codex-rs/core/src/tasks/user_shell.rs:92-312`) - Primary reuse source:

- command construction
- one-shot shell execution flow
- output capture
- cancellation
- completion item persistence

**Local Codex** (`codex-rs/core/src/codex.rs:4851-4876`) - Bridge source:

- current app-server `thread/shellCommand` dispatch into canonical runtime

**LOGOS action:** Reuse or extract this local one-shot execution logic as the main runner for the touched product surface. Do not create a third permanent spawn/stream/capture implementation beside canonical and OpenRouter.

### B.2 Shell Detection Guardrails

**OpenCode** (`input-sources-temp/opencode-upstream/packages/opencode/src/shell/shell.ts:1-109`) - Selective guardrail:

- shell blacklist for `fish` and `nu`
- Windows fallback order: `pwsh` -> `powershell` -> Git Bash -> `COMSPEC`
- POSIX fallback order: `/bin/zsh` on macOS, `bash`, then `/bin/sh`

**OpenCode** (`input-sources-temp/opencode-upstream/packages/opencode/src/session/prompt.ts:797-860`) - Selective guardrail:

- shell-specific argument shaping
- explicit `-NoProfile` for PowerShell
- login-shell behavior for `bash` and `zsh`

**LOGOS action:** Borrow shell-selection and invocation guardrails where they improve robustness, but keep production naming and implementation LOGOS-native. These are input-quality improvements, not the main owner model.

### B.3 Subprocess Isolation Guardrails

**Goose** (`input-sources-temp/goose-upstream/crates/goose/src/subprocess.rs:1-37`) - Direct guardrail:

- `process_group(0)` on Unix
- `CREATE_NO_WINDOW` on Windows

**LOGOS action:** Reuse these ideas where the shared runner still needs platform-level subprocess hygiene and the canonical substrate does not already provide it more cleanly.

### B.4 What Not To Take

| Source | Pattern | Why not taken as owner |
| --- | --- | --- |
| OpenCode | full PTY service from `pty/index.ts` | belongs to `Item 11`, not this slice |
| OpenCode | no-sandbox default posture | conflicts with local Codex capability and with honest future policy evolution |
| Goose | full inspection framework from `tool_inspection.rs` | useful vocabulary, but too broad for this `thread/shellCommand` slice |

---

## Stage C: OpenRouter Rewire And Duplicate Removal

### C.1 OpenRouter Duplicate To Remove

**Local OpenRouter** (`codex-rs/app-server/src/openrouter_beta_threads.rs:2443-2619`) - Delete:

- `drive_openrouter_shell_command()`
- provider-lane stdout/stderr select loop
- provider-lane completion choreography for this surface

**Local OpenRouter** (`codex-rs/app-server/src/openrouter_beta_threads.rs:1063-1170`) - Adapt then shrink:

- `shell_command()` remains the app-server entry point
- it should become orchestration only, delegating execution to the shared runner

**LOGOS action:** Keep lane-specific validation and turn-attachment decisions in `shell_command()`, but move actual execution ownership into the shared host-exec layer.

### C.2 Transitional Process Registration

**Local OpenRouter** (`codex-rs/app-server/src/openrouter_beta_threads.rs:96-96`) - Transitional residue:

- `running_commands` exists because Item 11 is not implemented yet

**LOGOS action:** The `Item 05` slice should not pretend to close background-terminal management. If any compatibility hook remains for cleanup, report it as transitional `Item 11` residue rather than as shell-command ownership.

### C.3 Contract Proof

**Local Codex** (`codex-rs/app-server/tests/suite/v2/thread_shell_command.rs:41-143, 158-320`) - Reuse:

- canonical contract tests

**LOGOS action:** Add the narrowest OpenRouter parity tests next to these contract tests rather than inventing a separate proof style.

---

## Stage D: Future Hooks, Not Current Closure

### D.1 Approval Vocabulary

**Goose** (`input-sources-temp/goose-upstream/crates/goose/src/tool_inspection.rs:23-42, 58-171`) - Vocabulary guardrail:

- `Allow`
- `Deny`
- `RequireApproval`

**LOGOS action:** Useful vocabulary for future policy enums, but do not widen the current slice into a full inspection framework.

### D.2 Background-Terminal Relationship

**OpenCode** (`input-sources-temp/opencode-upstream/packages/opencode/src/pty/index.ts:33-34, 107-109, 296-355`) - Guardrail only:

- bounded buffer
- write/resize/connect lifecycle
- attach/detach cleanup shape

**LOGOS action:** Treat this as `Item 11` input only. Do not implement PTY or long-lived terminal features in the first shell-command parity slice.

---

## Minimal Borrow / Reuse Summary

### Primary Local Reuse

1. canonical app-server `thread/shellCommand` contract
2. canonical one-shot user shell execution path
3. canonical output capture and completion semantics
4. existing OpenRouter lane validation and turn-attachment decisions, but not its execution owner logic

### External Guardrails Only

1. OpenCode shell selection and shell-specific argv shaping
2. Goose subprocess isolation primitives
3. Goose approval vocabulary for future enums

### Write From Scratch

1. shared host-exec context
2. shared notification sink boundary
3. shared one-shot runner surface that both touched paths can consume

## Evidence

- path: `../architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md`
  symbol: `Current Code Diagnosis`, `Practical Draft Shape`
  lines: `82-188, 289-344`
- path: `../research/session layer/RA-1.5-shell-execution.md`
  symbol: `Recommendations for LOGOS`
  lines: `186-207`
- path: `../../../codex-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `ThreadShellCommandParams`, `CommandExecutionOutputDeltaNotification`
  lines: `2971-2987, 5274-5292`
- path: `../../../codex-rs/app-server/tests/suite/v2/thread_shell_command.rs`
  symbol: `thread_shell_command_runs_as_standalone_turn_and_persists_history`, `thread_shell_command_uses_existing_active_turn`
  lines: `41-143, 158-320`
- path: `../../../codex-rs/core/src/tasks/user_shell.rs`
  symbol: `execute_user_shell_command`
  lines: `92-312`
- path: `../../../codex-rs/core/src/codex.rs`
  symbol: `run_user_shell_command`
  lines: `4851-4876`
- path: `../../../codex-rs/core/src/unified_exec/mod.rs`
  symbol: `UnifiedExecContext`
  lines: `72-86`
- path: `../../../codex-rs/core/src/state/service.rs`
  symbol: `SessionServices`
  lines: `30-61`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `running_commands`, `shell_command`, `drive_openrouter_shell_command`
  lines: `96-96, 1063-1170, 2443-2619`
- path: `../../../input-sources-temp/opencode-upstream/packages/opencode/src/shell/shell.ts`
  symbol: `Shell`
  lines: `1-109`
- path: `../../../input-sources-temp/opencode-upstream/packages/opencode/src/session/prompt.ts`
  symbol: `bash tool shell invocation`
  lines: `797-860`
- path: `../../../input-sources-temp/opencode-upstream/packages/opencode/src/pty/index.ts`
  symbol: `Pty service lifecycle`
  lines: `33-34, 107-109, 296-355`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/subprocess.rs`
  symbol: `configure_subprocess`
  lines: `1-37`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/tool_inspection.rs`
  symbol: `InspectionAction`, `ToolInspectionManager`
  lines: `23-42, 58-171`

## Links

- [[README.md]]
- [[../architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md]]
- [[../implementation/2026-04-18-item-05-shell-execution-strict-spec.md]]
- [[../research/session layer/RA-1.5-shell-execution.md]]
- [[../research/2026-04-16-session-layer-research-transfer-ledger.md]]
