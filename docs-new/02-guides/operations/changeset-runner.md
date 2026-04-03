---
id: guide-op-changeset-runner
title: ChangeSet Runner Guide
type: guide
status: stable
tags: [operations, runner, validation]
related: [[pipeline-recovery.md], [acceptance-testing.md], [../promotion-governance.md]]
depends_on: []
generated: false
source_of_truth: mixed
outline: []
---

# ChangeSet Runner Guide

## Purpose

Operational pattern for validate/run/run-stage and candidate artifact generation.

## Generic Procedure

1. Validate input contract first.
2. Run full pipeline.
3. Use stage-targeted execution for debugging.
4. Build promotion artifact bundle from candidate and rollback descriptors.

## Adaptation Notes

Command names and package filters are project-specific and must be adapted.

## Example Commands (Project-Specific)

```powershell
pnpm --filter @project/changeset-runner start -- validate .\changeset.json
pnpm --filter @project/changeset-runner start -- run .\changeset.json --verbose
pnpm --filter @project/changeset-runner start -- run-stage graph .\changeset.json --verbose
```

## Evidence

- path: `../../Documentation System Pack/guides/changeset-runner-workflow.md`
  symbol: `Validate Input Contract`
  lines: 1-260
- path: `pipeline-recovery.md`
  symbol: `Stage Recovery Matrix`
  lines: 1-220

## Links

- [[pipeline-recovery.md]]
- [[acceptance-testing.md]]
- [[../promotion-governance.md]]
