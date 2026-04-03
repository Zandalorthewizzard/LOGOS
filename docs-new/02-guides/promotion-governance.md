---
id: guide-promotion-governance
title: Promotion Governance Principles
type: guide
status: stable
tags: [promotion, governance, release]
related: [[operations/git-burst-workflow.md], [operations/acceptance-testing.md], [operations/slice-audit.md], [../01-concepts/strict-specs.md]]
depends_on: []
generated: false
source_of_truth: governance
outline: []
---

# Promotion Governance Principles

## General Principles

1. Promotion is gated and evidence-based.
2. Rollback path is mandatory and testable.
3. Promotion artifacts are explicit and versioned.
4. Decision matrix controls pass, conditional-pass, fail, deferred outcomes.

## Required Artifact Set

1. Release descriptor.
2. Gate summary.
3. Acceptance report.
4. KPI report (when performance gate exists).
5. Rollback report.

## Rollback Principles

1. Keep previous stable descriptor pointer.
2. Test mandatory smoke cases after rollback.
3. Record residual risks.

## Example Section (Adapt To Your Project)

1. Example branches: `main`, `dev`, `iv`, `burst/YYYYMMDD-topic`.
2. Example release ID: `iv-XX.Y.Z`.
3. Example gate IDs: `G1..G8` and acceptance IDs `A1..A12`.

## Evidence

- path: `../../Documentation System Pack/guides/dev-iv-promotion-playbook-v1.md`
  symbol: `Promotion Flow`
  lines: 1-240
- path: `../../Documentation System Pack/guides/dev-iv-git-burst-workflow-v1.md`
  symbol: `Core Rule Set`
  lines: 1-280

## Links

- [[operations/git-burst-workflow.md]]
- [[operations/acceptance-testing.md]]
- [[operations/slice-audit.md]]
- [[../01-concepts/strict-specs.md]]
