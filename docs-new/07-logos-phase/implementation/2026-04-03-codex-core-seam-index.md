---
id: logos-codex-core-seam-index
title: LOGOS Codex-Core Seam Index
type: index
status: active
tags: [implementation, codex-core, seams, upstream, logos]
related:
  [
    [README.md],
    [2026-04-02-codex-base-repository-strategy.md],
    [2026-04-02-upstream-sync-policy.md],
    [2026-04-03-phase-1-boundary-extraction-acceptance-report.md]
  ]
depends_on:
  [
    [2026-04-02-codex-base-repository-strategy.md],
    [2026-04-02-upstream-sync-policy.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Codex-Core Seam Index

## Purpose

This index records every intentional LOGOS intervention in `codex-rs/core/` or equivalent Codex-base seams that exist only to open a narrow additive integration surface for LOGOS.

## Rules

1. One seam intervention maps to one dedicated commit.
2. The seam commit should stay separate from larger consumer changes when reasonably possible.
3. Every seam entry must explain why a pure wrapper/additive seam outside `codex-core` was insufficient.
4. Prefer helper, builder, or re-export seams over moving orchestration into `codex-core`.
5. If upstream later grows an equivalent public seam, mark the LOGOS seam `replaced` and migrate consumers.

## Entries

### SEAM-2026-04-03-01 Prompt Tool Injection

- status: `active`
- introduced_by_commit: `983795862`
- area: `codex-rs/core/src/client_common.rs`
- why_needed: external LOGOS adapters could already reuse `ModelClientSession`, but `Prompt.tools` and `Prompt.parallel_tool_calls` were crate-private, so `request.tools` could not flow into the canonical OpenAI/Codex request path without duplicating internal request wiring or exposing deeper core internals.
- minimal_scope:
  1. add `Prompt::set_configured_tools`
  2. add `Prompt::configured_tools`
  3. add `Prompt::parallel_tool_calls_enabled`
- affected_files:
  1. `codex-rs/core/src/client_common.rs`
  2. `codex-rs/core/src/client_common_tests.rs`
- public_surface_added:
  1. `Prompt::set_configured_tools`
  2. `Prompt::configured_tools`
  3. `Prompt::parallel_tool_calls_enabled`
- consuming_logos_crates:
  1. `codex-rs/logos-provider-openai`
- non_goals:
  1. no tool execution orchestration moved into `codex-core`
  2. no raw Codex/OpenAI wire types exported through the LOGOS shared runtime
  3. no changes to canonical Codex handler ownership or transport flow
- upstream_conflict_risk: `low-to-medium`; `Prompt` is central, but the seam is localized and additive.
- rollback_or_replace_path: if upstream later exposes an equivalent public prompt-tool injection API, migrate LOGOS consumers to that upstream API and mark this seam `replaced`.

## Evidence

- equivalent: `git show --stat --summary 983795862`
- path: `../../../codex-rs/core/src/client_common.rs`
  symbol: `Prompt`
  lines: `26-85`
- path: `../../../codex-rs/core/src/client_common_tests.rs`
  symbol: `prompt_accepts_external_configured_tools`
  lines: `203-228`

## Links

- [[README.md]]
- [[2026-04-02-codex-base-repository-strategy.md]]
- [[2026-04-02-upstream-sync-policy.md]]
- [[2026-04-03-phase-1-boundary-extraction-acceptance-report.md]]
