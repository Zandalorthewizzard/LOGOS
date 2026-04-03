---
id: logos-branch-and-pr-workflow
title: LOGOS Branch And PR Workflow
type: spec
status: active
tags: [implementation, git, branches, pr, workflow, logos]
related:
  [
    [README.md],
    [2026-04-02-codex-base-repository-strategy.md],
    [2026-04-02-upstream-sync-policy.md],
    [2026-04-02-phase-1-boundary-extraction-strict-spec.md],
    [2026-04-02-openrouter-validation-adapter-strict-spec.md]
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

# LOGOS Branch And PR Workflow

## Purpose

- Define a branch and PR model that fits a Codex-upstream-based project.
- Keep upstream-sync work, LOGOS architecture work, and feature work clearly separated.

## Branch Model

### `main`

- stable LOGOS integration branch
- must remain reviewable and releasable

### `upstream-sync/*`

- dedicated branches for Codex sync work
- no unrelated LOGOS feature changes

### `feat/phase1-*`

- boundary extraction and initial runtime work

### `feat/provider-*`

- provider-specific implementation slices

### `docs/*`

- substantial documentation or planning passes

### `spike/*`

- temporary prototype branches
- must not become long-lived production branches without promotion

## PR Rules

- one concern per PR whenever possible
- upstream sync PRs stay isolated
- provider implementation PRs must cite the relevant strict spec
- architecture-changing PRs must cite the governing canonical docs
- if a PR touches Codex-base files, it must explicitly explain why wrapper or additive seams were insufficient

## Required PR Metadata

Every implementation PR should state:

- target spec(s)
- whether Codex-base files were touched
- whether OpenAI canonical path is affected
- tests run
- known limitations or deferred follow-ups

## Review Priorities

1. does this preserve canonical OpenAI/Codex behavior?
2. does this keep LOGOS-owned runtime authority explicit?
3. does this avoid unnecessary Codex-base divergence?
4. does this match the strict spec and acceptance gates?

## Responses Reuse Evaluation

### What This Workflow Reuses From Responses

- preserving the canonical OpenAI `Responses` path as the highest-fidelity route is the main reason branch discipline matters

### Advantages

- keeps parity-sensitive work reviewable
- reduces risk of mixing provider experiments into Codex-preservation work

### Disadvantages And Limits

- more branch categories add some process overhead
- discipline is only effective if PRs stay small and spec-driven

### Decision

- Use a workflow that isolates upstream sync, canonical-path preservation, and provider experiments.

## Acceptance Gates

1. Upstream sync, boundary extraction, and provider work use separate branch classes.
2. PR review can tell immediately whether Codex-base files changed.
3. Specs remain the source of truth for implementation branches.

## Evidence

- path: `2026-04-02-codex-base-repository-strategy.md`
  symbol: `Initial Repository Shape`
  lines: `53-58`
- path: `2026-04-02-upstream-sync-policy.md`
  symbol: `Required Sync Discipline`
  lines: `48-58`
- path: `2026-04-02-phase-1-boundary-extraction-strict-spec.md`
  symbol: `Locked Decisions`
  lines: `53-58`

## Links

- [[README.md]]
- [[2026-04-02-codex-base-repository-strategy.md]]
- [[2026-04-02-upstream-sync-policy.md]]
