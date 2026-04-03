---
id: concept-discussion-first
title: Discussion-First Workflow
type: concept
status: stable
tags: [discussion, spec, pipeline]
related: [[ai-driven-development.md], [strict-specs.md], [../05-discussion-templates/README.md], [../02-guides/documentation-replication.md], [../06-experemental/semi-autonomous-iterative-discussion-pipeline.md]]
depends_on: []
generated: false
source_of_truth: process
outline: []
---

# Discussion-First Workflow

## Pipeline

1. Discussion
- problem framing, alternatives, constraints, and decisions.
2. Spec
- strict normative task generated from discussion or direct source.
3. Implementation
- execution only from strict spec.

## Sources Allowed For Strict Spec

1. Discussion documents.
2. Direct source input (chat context, PRD, approved decision docs).

## Blind Zone Handling

1. Create issue document when unknown appears.
2. Route issue back to discussion or strict task update.
3. Continue implementation only after ambiguity is removed.

## When To Use Strict Specs

1. Use strict specs for autonomous coding phases.
2. Use discussion templates for exploratory phases before decisions are locked.

## Evidence

- path: `../docs-refactoring.md`
  symbol: `Exploratory Phase is Already Covered`
  lines: 1-260
- path: `../05-discussion-templates/task-template.md`
  symbol: `Next Steps`
  lines: 1-220

## Links

- [[ai-driven-development.md]]
- [[strict-specs.md]]
- [[../05-discussion-templates/README.md]]
- [[../02-guides/documentation-replication.md]]
- [[../06-experemental/semi-autonomous-iterative-discussion-pipeline.md]]
