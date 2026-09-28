---
id: logos-openrouter-thread-shell-command-support-discussion
title: LOGOS OpenRouter Thread Shell Command Support Discussion
type: discussion
status: active
tags: [logos, openrouter, phase-6, shell-command, host-ops]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Thread Shell Command Support Discussion

## Purpose

This document isolates `thread/shellCommand` as its own OpenRouter parity surface.

The question is not whether OpenRouter itself supports shell commands. The question is whether the OpenRouter beta thread lane can reuse the existing LOGOS host/runtime command path honestly without falling back to the canonical Codex thread runtime.

## Current State

1. `thread/shellCommand` is still explicitly unsupported for OpenRouter beta threads.
2. The canonical path already has app-server proofs for standalone shell-command turns and shell commands attached to an active turn.
3. OpenRouter beta threads already have thread and turn state, tool events, and app-server lifecycle handling, but they do not yet participate in this host operation.

## Ownership Baseline

1. `thread/shellCommand` is a LOGOS-owned host/runtime operation, not a provider-specific adapter feature.
2. OpenRouter should reuse the same session-state and host-operation layer as later non-canonical providers rather than owning its own permanent shell stack.
3. Provider-specific behavior should be limited to policy hooks or capability truth, not process ownership.

## Why Shell Command Needs Separate Attention

1. It is a user-visible host/runtime surface, not only a provider adapter concern.
2. It touches command execution items, notifications, turn attachment rules, and persisted thread history.
3. A fake implementation would create the illusion of parity while bypassing the real OpenRouter beta lane.

## Decision Questions

1. Can OpenRouter threads reuse the existing host execution machinery without routing through a generic core thread path?
2. How should standalone shell commands be represented on OpenRouter threads when no agent turn is already active?
3. How should OpenRouter shell commands interact with active turns so command execution stays on the same logical turn where appropriate?

## Current Likely Path

1. Reuse the existing host/runtime command execution path rather than inventing OpenRouter-specific process machinery.
2. Keep command execution as a host-owned operation above provider wires.
3. Treat the implementation target as a LOGOS-owned session-state layer, even if OpenRouter remains the first proving lane.
4. Preserve the same visible command execution items and notifications already used on the canonical path.

## Closeout Expectations

1. `thread/shellCommand` should work on OpenRouter beta threads through the app-server product path.
2. Standalone-turn and active-turn shell command modes should both be tested.
3. Persisted OpenRouter thread history should reflect the command execution result honestly.

## Evidence

- path: `../../../codex-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `ThreadShellCommandParams`
  lines: `2971-2987`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `CodexMessageProcessor::thread_shell_command`
  lines: `3519-3573`
- path: `../../../codex-rs/app-server/tests/suite/v2/thread_shell_command.rs`
  symbol: `thread_shell_command_runs_as_standalone_turn_and_persists_history`
  lines: `41-143`
- path: `../../../codex-rs/app-server/tests/suite/v2/thread_shell_command.rs`
  symbol: `thread_shell_command_uses_existing_active_turn`
  lines: `146-320`
- path: `2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Host Orchestration`
  lines: `1-220`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-02-logos-host-daemon-transport-spec.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
