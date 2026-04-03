---
id: guide-op-git-burst
title: Git Burst Workflow Example
type: guide
status: stable
tags: [operations, git, burst, rollback]
related: [[../promotion-governance.md], [pipeline-recovery.md]]
depends_on: []
generated: false
source_of_truth: governance
outline: []
---

# Git Burst Workflow Example

## Scope

This is an example workflow. Adapt branch and tag names to your project policy.

## Principles

1. Create checkpoint tag before risky batch.
2. Execute scripted batch in dedicated burst branch.
3. Merge to integration branch only after validation.
4. Keep rollback points discoverable.

## Example Naming

1. Branches: `main`, `dev`, `iv`, `burst/YYYYMMDD-topic`.
2. Tags: `checkpoint/dev-pre-burst-*`, `checkpoint/dev-post-burst-*`.

## Example Command Skeleton

```powershell
git checkout dev
git pull --ff-only origin dev
git tag checkpoint/dev-pre-burst-YYYYMMDD-HHMM
git checkout -b burst/YYYYMMDD-topic
# ... commits ...
git checkout dev
git merge --no-ff burst/YYYYMMDD-topic
git tag checkpoint/dev-post-burst-YYYYMMDD-HHMM
```

## Evidence

- path: `../../Documentation System Pack/guides/dev-iv-git-burst-workflow-v1.md`
  symbol: `Daily Burst Protocol`
  lines: 1-320
- path: `../promotion-governance.md`
  symbol: `Rollback Principles`
  lines: 1-240

## Links

- [[../promotion-governance.md]]
- [[pipeline-recovery.md]]
