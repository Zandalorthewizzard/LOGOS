---
id: guide-op-acceptance-testing
title: Acceptance Testing Example
type: guide
status: stable
tags: [operations, acceptance, gates]
related: [[../promotion-governance.md], [changeset-runner.md], [slice-audit.md], [../../03-templates/slice-audit.template.md]]
depends_on: []
generated: false
source_of_truth: governance
outline: []
---

# Acceptance Testing Example

## Scope

Example acceptance pack for gated promotion cycles. Adapt IDs and thresholds.

## Execution Order

1. Verify all sprint slices are complete and integrated.
2. Determinism/profile resolution checks.
3. Stability core cases.
4. Performance/KPI checks.
5. Rollback probe.

## Required Outputs

1. Acceptance report.
2. KPI report.
3. Rollback report.

## Blocking Rules

1. Critical correctness failure blocks promotion.
2. Missing explainability payload blocks promotion if required by policy.
3. KPI threshold miss follows gate policy (`fail` or `conditional-pass`).
4. Missing or incomplete slice audit report blocks sign-off for slice-level promotion.
5. Missing full-sprint acceptance execution evidence blocks promotion.

## Evidence

- path: `../../Documentation System Pack/guides/search-acceptance-runbook-v1.md`
  symbol: `Case Execution Order`
  lines: 1-260
- path: `../promotion-governance.md`
  symbol: `Required Artifact Set`
  lines: 1-240

## Links

- [[../promotion-governance.md]]
- [[changeset-runner.md]]
- [[slice-audit.md]]
- [[../../03-templates/slice-audit.template.md]]
