---
id: logos-upstream-sync-policy
title: LOGOS Upstream Sync Policy
type: spec
status: active
tags: [implementation, git, upstream, sync, codex, logos]
related:
  [
    [README.md],
    [2026-04-02-codex-base-repository-strategy.md],
    [2026-04-02-branch-and-pr-workflow.md],
    [../architecture/2026-03-30-logos-adapter-boundary-spec.md]
  ]
depends_on:
  [
    [2026-04-02-codex-base-repository-strategy.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Upstream Sync Policy

## Purpose

- Define how LOGOS stays aligned with Codex upstream.
- Prevent sync work from being mixed with LOGOS feature work.
- Keep future Codex adoption feasible while LOGOS grows its own runtime layers.

## Core Decision

- Upstream sync is a first-class maintenance workflow.
- Sync work must be isolated from LOGOS feature changes.
- The project should prefer frequent, understandable syncs over rare massive rebases.

## Default Sync Model

- official Codex is `upstream`
- LOGOS fork is `origin`
- upstream updates land through dedicated sync branches and dedicated PRs
- feature PRs must not bundle upstream-sync noise unless absolutely unavoidable

## Sync Goals

- preserve Codex parity where LOGOS intentionally wraps existing behavior
- adopt upstream fixes and new capabilities quickly where they do not violate LOGOS architecture rules
- keep conflict surface small by minimizing unnecessary Codex-base divergence

## Allowed Sync Operations

- fetch upstream
- merge or rebase upstream changes into dedicated sync branches according to repo policy
- resolve conflicts in a way that preserves documented LOGOS invariants
- add parity or regression notes when upstream changes affect runtime behavior

## Required Sync Discipline

- every sync PR must state the upstream commit range
- every sync PR must state whether canonical OpenAI behavior changed upstream
- every sync PR must state whether any LOGOS wrapper seam needs adjustment
- every sync PR must explicitly call out conflicts in:
  - transport
  - session
  - tool runtime
  - MCP boundary
  - approvals

## Responses Reuse Evaluation

### What This Policy Reuses From Responses

- preserving the strongest upstream `Responses` path is the main reason upstream sync discipline matters

### Advantages

- keeps OpenAI/Codex parity realistic over time
- reduces risk of a stale fork drifting into an unmaintainable product

### Disadvantages And Limits

- sync work adds regular operational cost
- if LOGOS edits too much inside Codex base, sync cost rises sharply

### Decision

- Keep upstream sync frequent and isolated.
- Optimize repository structure so Codex-base conflicts remain rare and understandable.

## Sync PR Acceptance Gates

1. Upstream range is documented.
2. Canonical OpenAI path is re-checked.
3. LOGOS-specific wrappers still compile or are explicitly updated.
4. No unrelated feature work is bundled into the sync PR.

## Open Questions

- whether the project should prefer merge-based sync or rebase-based sync as the default public history style
- whether some upstream updates should be grouped into periodic integration windows

## Evidence

- path: `../architecture/2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Hard Rule 1`
  lines: `25-29`
- path: `../architecture/2026-04-02-logos-provider-openai-adapter-spec.md`
  symbol: `Non-goals`
  lines: `135-139`
- path: `2026-04-02-codex-base-repository-strategy.md`
  symbol: `Working Philosophy`
  lines: `31-36`

## Links

- [[README.md]]
- [[2026-04-02-codex-base-repository-strategy.md]]
- [[2026-04-02-branch-and-pr-workflow.md]]
