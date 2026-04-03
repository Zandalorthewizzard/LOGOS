---
id: logos-guide-working-notes
title: Agent Working Notes Guide
type: guide
status: active
tags: [agent, notes, continuity]
related:
  [
    [agent-navigation.md],
    [../00-navigation/INDEX.md],
    [../../notes/README.md],
    [../../AGENTS.md],
  ]
depends_on: []
generated: false
source_of_truth: process
outline: []
---

# Agent Working Notes Guide

`notes/` is the operational memory layer for agents working in `LOGOS`.

## Status

1. `notes/` is allowed and expected for substantial architecture, migration, research, debugging, and handoff work.
2. `notes/` is non-canonical. It does not override code, runtime behavior, or evidence-backed `docs-new/` artifacts.

## When To Write Notes

Write or update a note when:

1. work creates observations that may matter later;
2. research uncovers constraints, traps, or unresolved questions;
3. a migration or refactor is paused and another agent may need to resume it;
4. the session produces architecture decisions not yet promoted into canonical docs.

## What Notes Should Contain

1. current task or milestone context;
2. what changed or what was learned;
3. confirmed facts vs hypotheses;
4. open issues or blockers;
5. the next safe resume point.

## Promotion Rule

1. keep transient reasoning and breadcrumbs in `notes/`;
2. promote durable decisions, accepted findings, and stable process rules into `docs-new/`;
3. canonical promotion should include evidence.

## Recovery Flow

If an agent resumes with partial context:

1. read `AGENTS.md`;
2. read the required `docs-new/` chain;
3. read this guide;
4. read relevant files in `notes/`;
5. continue only from confirmed state, not memory alone.

## Evidence

- path: `../../AGENTS.md`
  symbol: `Required Reading Chain`
  lines: 1-200
- path: `../../notes/README.md`
  symbol: `Agent Working Notes`
  lines: 1-200

## Links

- [[agent-navigation.md]]
- [[../00-navigation/INDEX.md]]
- [[../../notes/README.md]]
- [[../../AGENTS.md]]
