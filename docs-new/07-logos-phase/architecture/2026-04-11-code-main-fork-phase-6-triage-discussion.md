---
id: logos-code-main-fork-phase-6-triage-discussion
title: LOGOS Code-Main Fork Phase 6 Triage Discussion
type: discussion
status: active
tags: [logos, phase-6, openrouter, coexistence, provider-foundation, triage]
related:
  [
    [README.md],
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [../implementation/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md],
  ]
depends_on:
  [
    [2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
    [../implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Code-Main Fork Phase 6 Triage Discussion

## Purpose

This document records which ideas from the `code-main` fork research are relevant to the current phase-6 lane, which ones should wait until after the beta, and which ones should not shape the current LOGOS architecture.

It exists to avoid two failure modes:

1. widening phase 6 with interesting but non-blocking ideas
2. re-importing architecture that the current LOGOS lane already rejects

## Decision Summary

### Take Now In Phase 6

1. close Codex coexistence and auth isolation as a beta blocker
2. wire existing provider tuning knobs into the OpenRouter path instead of inventing new ones
3. extend reserved built-in provider ID protection to cover LOGOS-owned built-ins such as `openrouter`
4. lock support-tier language around a validated OpenRouter model or route set rather than provider-wide marketing claims

### Defer Until After Beta

1. project `.env` provider-key filtering
2. MCP elicitation and request-to-thread correlation follow-up
3. enterprise-style config layering precedence
4. `[[agents]]`-style sub-agent configuration
5. proxy-style request inspection utilities for testing or diagnostics

### Reject For The Current Architecture

1. `CODE_HOME` to `CODEX_HOME` runtime bridging
2. widening arg0 dispatch around executable-name multiplexing as a product strategy
3. internal proxy binaries as part of the normal runtime path

## Why These Are The Current Phase-6 Priorities

### 1. Coexistence Is Already A Phase-6 Contract, Not A Nice-To-Have

The current phase-6 docs already require no implicit reuse of `~/.codex`, but the current inherited startup path still documents loading `.env` from `~/.codex/.env`.

That means coexistence is not a hypothetical polish issue. It is a direct beta blocker.

Decision:

1. solve home, config, cache, and auth isolation now
2. do not add runtime bridging between LOGOS and Codex homes

### 2. OpenRouter Should Reuse Existing Provider Knobs, Not Trigger New Foundation Work

The fork report is directionally right that OpenRouter benefits from provider-level retry and timeout tuning, but this is not a new architectural requirement. The existing Codex-base provider config already contains these fields.

Decision:

1. phase 6 should wire and document existing provider knobs for OpenRouter
2. phase 6 should not invent new retry-capability abstractions just because OpenRouter is the first live non-canonical beta path

### 3. Reserved Provider-ID Protection Should Cover LOGOS Built-Ins

The fork report is useful here because it highlights a real foundation gap in the current LOGOS tree: built-in provider IDs are already reserved in Codex-base config validation, but the current reserved set does not yet include the built-in OpenRouter adapter.

Decision:

1. extend reserved built-in provider ID protection in the current phase-6 provider-foundation slice
2. treat this as a small hardening change, not a new architecture lane

### 4. OpenRouter Claims Must Stay Tiered And Validated

The report is also directionally correct that OpenRouter support cannot be claimed as a flat provider-wide parity promise. The current phase-6 tool-gate and OpenRouter risk docs already push in that direction.

Decision:

1. phase 6 should document validated OpenRouter model or route sets explicitly
2. capability truth should be reflected through support-tier language and acceptance wording, not through speculative new bool fields

## Why The Deferred Items Are Not Phase-6 Work

### Project `.env` Filtering

The `code-main` project-level `.env` filtering is a good production-safety pattern, but it solves a different problem than the current LOGOS beta blocker.

Current blocker:

1. inherited Codex-home collision

Deferred problem:

1. future project-level dotenv ingestion policy for multi-provider setups

Until LOGOS has an explicit project `.env` loading policy of its own, this should not distract phase 6.

### MCP Elicitation And Correlation Follow-Up

This is relevant to later provider and tool expansion, but phase 6 is already bounded around OpenRouter beta hardening and release readiness. The current phase-6 gate is about proving the OpenRouter tool path through existing seams, not about opening new MCP protocol surface.

### Enterprise Config Layering

The `ConfigLayerSource` precedence model is useful, but it targets a later enterprise or managed-environment lane rather than the current beta release lane.

### `[[agents]]` Blocks And Sub-Agent Configuration

This belongs to a future orchestration or plugin direction, not the current OpenRouter beta lane.

### Proxy Utilities

A request-inspection proxy may become useful as a dev-only or testing tool, but phase 6 should not adopt proxy binaries as part of the release-critical runtime surface.

## Why The Rejected Items Stay Rejected

### `CODE_HOME` to `CODEX_HOME` Bridging

This directly conflicts with the current LOGOS phase-6 coexistence goal. Runtime bridging solves migration discomfort by reintroducing hidden shared state, which is exactly the behavior the beta must avoid.

### Expanded Arg0 Multiplexing

Executable-name dispatch may be a practical internal implementation detail for inherited helpers, but it is not a good foundation for LOGOS product identity or coexistence guarantees.

### Internal Proxy Binary In The Main Runtime

This adds attack surface and deployment complexity while solving a problem that is not core to the current beta release.

## Report Caveats

The search report is useful, but some of its signals need reinterpretation:

1. provider retry and websocket knobs are not a new idea borrowed from the fork; they already exist in the current Codex-base provider model
2. early provider validation is also already present in the current LOGOS/Codex-base config path
3. the actionable value of the report is therefore triage, not wholesale architectural import

## Phase-6 Consequence

Phase 6 should treat the `code-main` fork research as a sharpening input, not as a second architecture source of truth.

The practical execution consequence is:

1. prioritize coexistence and OpenRouter hardening items that align with the current beta gate
2. park useful but non-blocking ideas explicitly
3. reject patterns that reintroduce hidden shared state or deployment complexity

## Evidence

- path: `../../../codex-rs/arg0/src/lib.rs`
  symbol: `arg0_dispatch_or_else`
  lines: `140-152`
- path: `2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md`
  symbol: `Coexistence Requirements`
  lines: `87-99`
- path: `../../../codex-rs/model-provider-info/src/lib.rs`
  symbol: `ModelProviderInfo`
  lines: `75-123`
- path: `../../../codex-rs/core/src/config/mod.rs`
  symbol: `RESERVED_MODEL_PROVIDER_IDS`
  lines: `150-156`
- path: `../../../codex-rs/core/src/config/mod.rs`
  symbol: `validate_model_providers`
  lines: `1867-1876`
- path: `../../../codex-rs/logos-provider-openrouter/src/adapter.rs`
  symbol: `OPENROUTER_PROVIDER_ID`
  lines: `25-25`
- path: `../../../input-sources-temp/code-main/code-rs/arg0/src/lib.rs`
  symbol: `load_dotenv`
  lines: `124-172`
- path: `../../../input-sources-temp/code-main/codex-rs/model-provider-info/src/lib.rs`
  symbol: `ModelProviderInfo`
  lines: `104-123`
- path: `../../../input-sources-temp/code-main/code-rs/app-server-protocol/src/protocol/v2.rs`
  symbol: `ConfigLayerSource`
  lines: `279-347`
- path: `../../../input-sources-temp/code-main/codex-rs/responses-api-proxy/src/lib.rs`
  symbol: `run_main`
  lines: `73-136`

## Links

- [[README.md]]
- [[2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
- [[2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]]
- [[../implementation/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md]]
