---
id: guide-op-pipeline-recovery
title: Pipeline Recovery Guide
type: guide
status: stable
tags: [operations, recovery, checkpoints]
related: [[changeset-runner.md], [git-burst-workflow.md], [../promotion-governance.md]]
depends_on: []
generated: false
source_of_truth: process
outline: []
---

# Pipeline Recovery Guide

## Goal

Decide whether to resume or rerun failed/incomplete pipeline execution.

## Recovery Checklist

1. Check journal status by change set ID.
2. If status is committed, do not rerun same unit.
3. Validate stage-specific skip conditions.
4. Verify concurrency lock behavior when parallel writers exist.

## Stage Recovery Matrix

| Stage | Skip On Recovery | Condition |
| --- | --- | --- |
| ingest | yes | checkpoint exists |
| parse | no | always replay |
| graph | conditional | checkpoint and delta exist |
| embeddings | no | replay |
| publish | conditional | checkpoint and snapshot exist |

## Evidence

- path: `../../Documentation System Pack/guides/pipeline-recovery-checklist.md`
  symbol: `Checklist`
  lines: 1-220
- path: `changeset-runner.md`
  symbol: `Generic Procedure`
  lines: 1-220

## Links

- [[changeset-runner.md]]
- [[git-burst-workflow.md]]
- [[../promotion-governance.md]]
