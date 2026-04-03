---
id: guide-op-slice-audit
title: Slice Audit Guide
type: guide
status: stable
tags: [operations, audit, quality, traceability]
related: [[acceptance-testing.md], [../promotion-governance.md], [../../03-templates/slice-audit.template.md]]
depends_on: []
generated: false
source_of_truth: governance
outline: []
---

# Slice Audit Guide

## Purpose

Standardize slice implementation audit under a parallel three-slice model so quality decisions are repeatable, traceable, and resistant to false-green outcomes.

## When To Run

1. After implementation for each slice (`Sx/Ty`) with no per-slice acceptance execution.
2. After full sprint integration and single acceptance run.
3. Before final sign-off for promotion decision.
4. After significant changes to requirement mapping or acceptance scope.

## Mandatory Inputs

1. Slice spec path.
2. Commit range and changed files.
3. Requirement to task to test to gate mapping.
4. Declared slice ownership/exclude boundaries.
5. Planned test commands for the slice.
6. Full-sprint acceptance execution artifacts.
7. Generated artifacts.

## Core Checks

1. Scope traceability is complete.
2. Slice is implementation-bearing (not documentation-only).
3. Parallel safety is preserved (no file overlap against other slices).
4. Determinism and repeatability are verified.
5. False-green guard checks are executed.
6. External freshness checks (or equivalent stack review) are recorded.
7. Risks and rollback are explicit.

## Output

Use `../../03-templates/slice-audit.template.md` and store one report per slice, plus one full-sprint acceptance summary linked from all slice reports.

## Evidence

- path: `../../Documentation System Pack/guides/slice-audit-template-v1.md`
  symbol: `Report Sections (mandatory order)`
  lines: 1-320
- path: `acceptance-testing.md`
  symbol: `Blocking Rules`
  lines: 1-200

## Links

- [[acceptance-testing.md]]
- [[../promotion-governance.md]]
- [[../../03-templates/slice-audit.template.md]]
