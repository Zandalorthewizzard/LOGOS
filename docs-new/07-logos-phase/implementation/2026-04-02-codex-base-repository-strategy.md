---
id: logos-codex-base-repository-strategy
title: LOGOS Codex Base Repository Strategy
type: spec
status: active
tags: [implementation, repository, codex, upstream, fork, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-global-roadmap.md],
    [../architecture/2026-03-30-logos-adapter-boundary-spec.md],
    [../architecture/2026-04-02-logos-provider-openai-adapter-spec.md],
    [2026-04-02-upstream-sync-policy.md],
    [2026-04-02-branch-and-pr-workflow.md]
  ]
depends_on:
  [
    [../architecture/2026-03-30-logos-adapter-boundary-spec.md],
    [../architecture/2026-04-02-logos-provider-openai-adapter-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Codex Base Repository Strategy

## Purpose

- Define the repository baseline for LOGOS before implementation starts.
- Prevent accidental divergence from Codex while the adapter architecture is still being established.
- Make the engineering meaning of `Codex full upstream` explicit.

## Core Decision

- Codex is the canonical upstream base for LOGOS.
- LOGOS should begin from a real fork or a faithfully mirrored upstream snapshot tied to a specific Codex commit.
- The initial implementation strategy is wrapper-first and adapter-first, not Codex-core rewrite-first.

## Working Philosophy

- Preserve the full upstream Codex base as much as possible.
- Do not modify Codex core mechanics merely to make multi-provider abstraction cleaner.
- Build LOGOS-owned runtime seams, adapter crates, and orchestration layers around the preserved Codex path.
- Treat OpenAI/Codex behavior as the canonical parity target.

## What `Full Upstream` Means

In the current LOGOS phase, `full upstream` means:

- use the official Codex repository as the baseline source
- pin a concrete baseline commit or tag
- keep Codex-origin behavior intact unless a change is strictly necessary for wrapper integration or extraction safety
- prefer additive LOGOS crates and wrapper seams over invasive edits inside Codex base modules

It does not mean:

- pretending LOGOS has no code of its own
- blocking all future Codex changes forever
- forbidding clearly justified boundary hooks when the project eventually proves they are necessary

## Initial Repository Shape

Recommended starting shape:

1. one GitHub fork of Codex as the canonical LOGOS code repository
2. one documented upstream remote pointing to official Codex
3. LOGOS-specific work in additive crates, folders, and integration seams
4. a pinned baseline reference recorded in docs before the first implementation PR lands

## Responses Reuse Evaluation

### What This Strategy Reuses From Responses

- the canonical OpenAI `Responses` path already present in Codex remains the reference implementation LOGOS must preserve

### Advantages

- strongest possible parity baseline
- easiest way to verify that OpenAI path remains identical in behavior
- easiest way to benefit from future Codex/OpenAI upstream improvements

### Disadvantages And Limits

- repository discipline becomes critical
- careless changes inside Codex base can create long-term merge pain
- wrapper seams may feel slower at first than aggressive refactoring

### Decision

- Preserve Codex as the base.
- Prefer wrapper and additive extraction first.
- Delay invasive Codex-base edits until the adapter boundary is proven and the need is evidence-backed.

## Allowed Early Change Types

- additive LOGOS crates
- additive integration layers around preserved Codex modules
- minimally invasive hooks only when needed to connect the wrapper seam
- tests and harnesses that prove parity or regression

## Disallowed Early Change Types

- broad rewrites of Codex transport or session behavior
- changes that weaken upstream sync feasibility
- refactors whose only purpose is aesthetic provider-neutrality
- edits that degrade the canonical OpenAI path to accommodate weaker providers

## Baseline Recording Requirement

Before real implementation starts, record all of these in the repository and docs:

- upstream repository URL
- baseline commit or tag
- fork URL
- date of baseline adoption
- any already-present local deviations from upstream

## Operational Status As Of 2026-04-03

- upstream repository URL: `https://github.com/openai/codex.git`
- fork URL: `https://github.com/Zandalorthewizzard/LOGOS.git`
- baseline adoption date: `2026-04-03`
- baseline commit: `af8a9d2d2ba84a1dafad6fa51fe3d63e96aceb75`
- baseline tag: `logos-baseline-2026-04-03-af8a9d2`
- already-present local LOGOS materials carried into the fork after baseline adoption:
  - `docs-new/`
  - `input-docs-temp/`
  - `notes/`
  - repo-level LOGOS instructions merged into `AGENTS.md`

## Live Repository Controls

- `origin` points to the public LOGOS fork.
- `upstream` points to the official Codex repository.
- `main` is branch-protected against force-push and deletion, with conversation resolution required before merge.
- upstream sync starts from dedicated `upstream-sync/*` branches and uses the local helper script instead of being bundled into feature work.
- the first GitHub workflow draft for upstream sync was moved out of active workflows after fork-side validation failures and remains staged only as a disabled reference copy.

## Acceptance Gates

1. One real Codex baseline commit is selected and documented.
2. LOGOS fork strategy is explicit.
3. Early work can proceed without ambiguous ownership of Codex base files.
4. OpenAI parity remains the governing acceptance criterion for any Codex-touching change.

## Evidence

- path: `../architecture/2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Hard Rule 1`
  lines: `25-39`
- path: `../architecture/2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Current Codex Modules That Stay As-Is`
  lines: `48-87`
- path: `../architecture/2026-04-02-logos-provider-openai-adapter-spec.md`
  symbol: `Decision`
  lines: `74-79`
- path: `2026-04-02-phase-1-boundary-extraction-strict-spec.md`
  symbol: `External References`
  lines: `68-71`
- path: `../../../../.github/pull_request_template.md`
  symbol: `LOGOS Pull Request`
  lines: `1-200`
- path: `../../../../scripts/logos-sync-upstream.ps1`
  symbol: `LOGOS Upstream Sync Helper`
  lines: `1-240`
- path: `../../../../notes/logos-upstream-sync.workflow-disabled.yml`
  symbol: `LOGOS Upstream Sync`
  lines: `1-240`
- equivalent: `git remote get-url origin` -> `https://github.com/Zandalorthewizzard/LOGOS.git`
- equivalent: `git remote get-url upstream` -> `https://github.com/openai/codex.git`
- equivalent: `git rev-parse origin/main` and `git rev-parse upstream/main` -> `af8a9d2d2ba84a1dafad6fa51fe3d63e96aceb75`
- equivalent: `git rev-parse logos-baseline-2026-04-03-af8a9d2`
- equivalent: `gh api repos/Zandalorthewizzard/LOGOS/branches/main/protection`

## Links

- [[README.md]]
- [[2026-04-02-upstream-sync-policy.md]]
- [[2026-04-02-branch-and-pr-workflow.md]]
