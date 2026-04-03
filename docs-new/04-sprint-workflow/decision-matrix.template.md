---
id: sprint-decision-matrix-template
title: Decision Matrix Template
type: template
status: stable
tags: [decision, matrix, template]
related: [[spec-template.md], [runbook.md], [../03-templates/strict-spec.template.md]]
depends_on: [[../01-concepts/discussion-first.md]]
generated: false
source_of_truth: governance
outline: []
---

# Decision Matrix Template

## Matrix

| Decision ID | Scope | Decision | Rationale | Status | Source |
| --- | --- | --- | --- | --- | --- |
| D-001 | <domain> | <decision> | <why> | locked | <discussion/task doc> |

## Usage Rules

1. Strict specs may reference only locked decisions.
2. Draft decisions cannot be used for autonomous implementation.
3. Superseded decisions remain in history with explicit status.

## Evidence

- path: `../01-concepts/discussion-first.md`
  symbol: `Sources Allowed For Strict Spec`
  lines: 1-220
- path: `spec-template.md`
  symbol: `Locked Decisions`
  lines: 1-260

## Links

- [[spec-template.md]]
- [[runbook.md]]
- [[../03-templates/strict-spec.template.md]]
