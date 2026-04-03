---
id: guide-sprint-authoring
title: Sprint Authoring Guide
type: guide
status: stable
tags: [sprint, authoring, task-spec]
related: [[../04-sprint-workflow/README.md], [../03-templates/strict-spec.template.md], [operations/acceptance-testing.md], [operations/slice-audit.md]]
depends_on: [[../01-concepts/discussion-first.md], [../01-concepts/strict-specs.md]]
generated: false
source_of_truth: process
outline: []
---

# Sprint Authoring Guide

## Goal

Create sprint packs that are strict, testable, and ready for autonomous execution.

## Workflow

1. Intake discussion/direct source.
2. Define scope, non-goals, and deferred boundaries.
3. Instantiate sprint folder and base docs from `04-sprint-workflow/`.
4. Author sprint specs from strict template contract.
5. Partition scope into three long code slices with explicit ownership/exclude boundaries.
6. Execute all three slices in parallel.
7. Run tests once after full sprint integration.
8. Orchestrator accepts slices and syncs docs/meta-files.

## Quality Gates For Sprint Specs

1. No architecture invention in coding phase.
2. Concrete package/file targets for each implementation task.
3. Explicit rollback path.
4. Explicit acceptance commands.
5. Exactly three long parallel slices with zero file overlap.
6. No documentation-only slice.
7. Slice audit report for each sprint slice via `operations/slice-audit.md`.
8. Acceptance execution evidence exists only at full-sprint level.

## Deferred Streams

If decisions are not locked, produce only preliminary docs and keep implementation blocked.

## Evidence

- path: `../../Documentation System Pack/guides/sprint-authoring/sprint-pack-generation-runbook-v1.md`
  symbol: `Phase 1..5`
  lines: 1-260
- path: `../04-sprint-workflow/spec-template.md`
  symbol: `Mandatory Contract`
  lines: 1-240

## Links

- [[../04-sprint-workflow/README.md]]
- [[../03-templates/strict-spec.template.md]]
- [[operations/acceptance-testing.md]]
- [[operations/slice-audit.md]]
