---
id: logos-provider-lmstudio-adapter-spec
title: LOGOS LM Studio Adapter Architecture Spec
type: spec
status: active
tags: [architecture, adapters, lmstudio, local, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-provider-adapter-profiles.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [../research/2026-03-30-logos-provider-research-report.md]
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-03-30-logos-adapter-boundary-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS LM Studio Adapter Architecture Spec

## Purpose

- Define LM Studio as a dev/local validation adapter, not a canonical provider.
- Capture how Logos should use LM Studio for local prototyping without letting it redefine cloud-provider architecture.
- Separate OpenAI-compatible validation value from native local API experimentation.

## Adapter Role

- dev/local validation target
- split-mode provider
- useful for offline and low-cost prototyping

## Why LM Studio Matters

- It offers a local `Responses`-compatible endpoint with continuation support.
- It also exposes native local APIs, so it is a useful test of mixed-mode local runtime design.
- It can reduce cloud dependency during early prototype work, especially for continuation and MCP experiments.

## Adapter Modes

### Mode A: OpenAI-Compatible `/v1/responses`

- main mode for Logos validation value
- useful for continuation and tool-loop prototype work

### Mode B: Native Local REST/Chat

- optional local experiment mode
- not the canonical compatibility baseline for Logos

## Responses Reuse Evaluation

### Surface Type

- OpenAI-compatible `Responses` endpoint plus native local API surfaces.

### What Logos Reuses

- OpenAI-compatible request and continuation patterns for local validation
- stream and tool-loop prototype behavior where docs indicate parity is good enough
- normalized kernel concepts for persistence, tools, and capabilities

### Advantages

- local validation of `previous_response_id`
- lower cloud cost and easier offline experimentation
- useful MCP and tool-loop prototyping path

### Disadvantages

- behavior still depends on local model/runtime specifics
- compatibility can change with LM Studio releases
- should not be mistaken for canonical cloud-provider semantics
- native local mode and compat mode must not be conflated

### Decision

- Use `Responses` compat mode for dev validation value.
- Do not let LM Studio define canonical shared semantics.
- Treat native local APIs as optional experimental mode.

## Required Adapter Behavior

### Validation Role

- This adapter exists to validate architecture seams, not to redefine them.
- Any success on LM Studio must still be checked against canonical OpenAI assumptions before being promoted into shared rules.

### Continuation

- `previous_response_id` support makes LM Studio useful for local continuation testing.
- Native local state behavior remains mode-specific and must not automatically widen shared abstractions.

### Tools And MCP

- Compat-mode tools and MCP support are useful for local testing.
- Differences between native and compat MCP exposure must remain explicit.

## Non-goals

- no promotion of LM Studio into canonical architecture authority
- no assumption that local model behavior equals cloud-provider behavior
- no silent merge of native local APIs with compat-mode semantics

## Acceptance Gates

1. LM Studio remains clearly marked dev-only/non-canonical in project docs.
2. Compat and native modes remain separate in persisted mode/capability records.
3. Local continuation tests are treated as validation evidence, not final semantic proof.
4. Release churn and model dependence remain documented risks.

## Open Questions

- whether LM Studio local continuation is stable enough to become a routine regression target
- which local-only capabilities are worth exposing through Logos at all
- how much native local API work should happen before broader cloud-provider implementation

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 6: Local Validation Targets`
  lines: `182-199`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Provider Semantics Matrix`
  lines: `135-136`
- path: `../research/2026-03-30-provider-comparison-matrix.md`
  symbol: `Matrix`
  lines: `38-39`
- path: `../research/2026-03-30-preliminary-provider-research-scan.md`
  symbol: `Провайдер/режим`
  lines: `107-107`
- path: `../../input-sources-temp/provider-source-inventory.md`
  symbol: `LM Studio`
  lines: `143-159`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-04-02-logos-provider-adapter-profiles.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
