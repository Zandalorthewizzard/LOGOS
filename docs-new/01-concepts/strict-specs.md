---
id: concept-strict-specs
title: Strict Task Specifications
type: concept
status: stable
tags: [spec, strictness, contract]
related: [[ai-driven-development.md], [discussion-first.md], [../03-templates/strict-spec.template.md]]
depends_on: []
generated: false
source_of_truth: process
outline: []
---

# Strict Task Specifications

## Principle

Task specification is normative. It must remove architecture guessing during implementation.

## Mandatory Sections

1. Start Here
2. Intent
3. Target Outcome
4. Locked Decisions
5. Architecture Fit
6. External References (optional)
7. Implementation Tasks (ordered)
8. Requirement -> Task -> Test -> Gate
9. Acceptance and Tests
10. Promotion Artifacts
11. Risks and Rollback
12. Agent Guardrails

## Definition Of Ready

Task is ready only if package ownership, file targets, tests, gates, compatibility mode, and rollback unit are explicit.

## Definition Of Done

Task is done only if mapped tests pass, required artifacts exist, and gate decision is recorded.

## Evidence Rule

Canonical docs must include evidence blocks. Missing evidence must be marked `UNKNOWN`.

## Evidence

- path: `../docs-refactoring.md`
  symbol: `Hard-Enforce Evidence Requirement`
  lines: 1-260
- path: `../03-templates/strict-spec.template.md`
  symbol: `Mandatory Sections`
  lines: 1-320

## Links

- [[ai-driven-development.md]]
- [[discussion-first.md]]
- [[../03-templates/strict-spec.template.md]]
