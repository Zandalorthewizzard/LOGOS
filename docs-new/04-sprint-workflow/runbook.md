---
id: sprint-workflow-runbook
title: Sprint Pack Runbook
type: guide
status: stable
tags: [sprint, runbook, generation]
related: [[README.md], [spec-template.md], [../03-templates/strict-spec.template.md]]
depends_on: [[../01-concepts/discussion-first.md]]
generated: false
source_of_truth: process
outline: []
---

# Sprint Pack Runbook

## Phase 1: Intake

1. Collect discussion docs and decision sources.
2. Build scope table: in-scope, out-of-scope, deferred, high-risk.

## Phase 2: Base Documents

1. Create stream folder.
2. Create stream README.
3. Create sprint plan.
4. Create decision matrix if needed.
5. Create sprint specs.

## Phase 3: Slice Partitioning

1. Partition sprint scope into exactly three long implementation slices.
2. Create ownership and exclude boundaries so slices have zero file overlap.
3. Assign one agent per slice and designate orchestrator (default: Agent 1).
4. Keep documentation updates out of slice scope; orchestrator handles acceptance docs.

## Phase 4: Spec Quality

1. Fill all mandatory strict-spec sections.
2. Ensure package/file targets are concrete.
3. Map requirements to tests and gates.
4. Add explicit "tests authored in-slice, executed post-sprint" rule.

## Phase 5: Parallel Implementation

1. Execute all three slices simultaneously.
2. Allow test code additions inside slices, but do not run acceptance tests per slice.
3. Collect delivery evidence for each slice.

## Phase 6: Full-Sprint Acceptance

1. Merge slice outputs.
2. Run acceptance and test commands once for full sprint.
3. Produce sprint-level pass/fail decision and rollback note.

## Phase 7: Synchronization

1. Orchestrator validates slice deliveries and updates sprint docs.
2. Update project `INDEX`.
3. Update project `inventory`.
4. Update project `progress`.
5. Update project `validation`.

## Evidence

- path: `../../Documentation System Pack/guides/sprint-authoring/sprint-pack-generation-runbook-v1.md`
  symbol: `Phase 1..5`
  lines: 1-260
- path: `../03-templates/strict-spec.template.md`
  symbol: `Requirement -> Task -> Test -> Gate`
  lines: 1-320

## Links

- [[README.md]]
- [[spec-template.md]]
- [[../03-templates/strict-spec.template.md]]
