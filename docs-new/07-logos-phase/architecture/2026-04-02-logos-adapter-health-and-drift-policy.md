---
id: logos-adapter-health-and-drift-policy
title: LOGOS Adapter Health And Drift Policy
type: spec
status: active
tags: [architecture, adapters, health, drift, telemetry, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-runtime-tier-contract-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-semantics-and-guarantees-matrix.md]
  ]
depends_on:
  [
    [2026-04-02-logos-runtime-tier-contract-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Adapter Health And Drift Policy

## Purpose

- Define how Logos keeps adapters trustworthy after implementation starts.
- Treat provider drift as semantic risk, not only docs churn.

## Health Contract

Every adapter must have a health contract with at minimum:

- canonical smoke scenarios
- golden event snapshots
- capability assertions
- error normalization assertions
- downgrade assertions
- replay/resume smoke
- tool-loop interruption recovery checks
- structured-output regression checks

## Drift Sources

- provider docs or changelog changes
- SDK changes
- new event types
- changed tool-call envelope
- changed continuation behavior
- schema compliance regressions
- rising fallback usage
- mismatch between static and observed capability state

## Detection Layers

### Doc Drift Watch

- track official API, docs, and SDK changes

### Behavior Drift Tests

- run scheduled live smoke tests against golden expectations

### Runtime Telemetry

- record unknown events, schema mismatches, replay failures, tool anomalies, and fallback spikes

## Risk States

- `stable`
- `watch`
- `degraded`
- `experimental`

Risk state must be operationally visible and must influence tier confidence.

## Failure Handling Rules

### Unknown Event Type

- emit `ProviderUnknownEvent`
- preserve raw payload or safe summary for telemetry according to privacy policy
- mark capability confidence uncertain
- evaluate downgrade

### Tool-Call Shape Drift

- disable unsafe streamed-arg assumptions
- fall back to buffered extraction if possible
- lower tool-stream confidence

### Continuation Failure

- stop claiming canonical continuation semantics
- switch to replay-driven continuation if available
- emit operational incident

### Structured Output Regression

- stop trusting native schema compliance
- switch to validate plus retry or repair mode
- lower capability confidence

## Acceptance Gates

1. Every adapter spec names health checks and watchpoints.
2. Unknown events cannot disappear silently.
3. Risk state and tier state can diverge from static claims.
4. Operations can identify why an adapter degraded.

## Evidence

- path: `C:\Users\zand\Desktop\analysis.txt`
  symbol: `Adapter Health Contract`
  lines: `559-584`
- path: `C:\Users\zand\Desktop\analysis.txt`
  symbol: `Drift Detection Model`
  lines: `586-627`

## Links

- [[README.md]]
- [[2026-04-02-logos-runtime-tier-contract-spec.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
