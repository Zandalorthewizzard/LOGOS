---
id: sprint-workflow-hub
title: Sprint Workflow Hub
type: hub
status: stable
tags: [sprint, workflow, hub]
related: [[runbook.md], [spec-template.md], [decision-matrix.template.md], [../02-guides/sprint-authoring.md]]
depends_on: [[../01-concepts/discussion-first.md], [../01-concepts/strict-specs.md]]
generated: false
source_of_truth: process
outline: []
---

# Sprint Workflow Hub

## Use Cases

1. Generate sprint pack from discussion docs.
2. Generate sprint pack from direct chat/context source.
3. Generate mixed source sprint pack.

## Included Assets

1. `runbook.md`
2. `spec-template.md`
3. `decision-matrix.template.md`

## Output Expectations

1. Sprint pack has strict sections and test mapping.
2. Deferred unknowns are explicit.
3. Meta-files are synchronized after changes.

## Hard Slice Partitioning Rules

1. Each sprint is partitioned into exactly three long, terminal slices.
2. All three slices must be executable simultaneously without file overlap.
3. Slices are implementation-only; documentation-only slices are forbidden.
4. Orchestrator accepts slice deliveries and updates documentation/meta-files (default role: Agent 1).
5. Tests are executed once after full sprint completion; slice work may add tests but must defer execution.

## Evidence

- path: `../../Documentation System Pack/guides/sprint-authoring/README.md`
  symbol: `Mandatory Output`
  lines: 1-260
- path: `runbook.md`
  symbol: `Phase 1..5`
  lines: 1-220

## Links

- [[runbook.md]]
- [[spec-template.md]]
- [[decision-matrix.template.md]]
- [[../02-guides/sprint-authoring.md]]
