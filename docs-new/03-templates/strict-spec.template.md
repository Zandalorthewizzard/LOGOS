---
id: template-strict-spec
title: Strict AI Task Spec Template
type: template
status: stable
tags: [task-spec, ai, strict, template]
related: [[AGENTS.template.md], [external-reference-policy.template.md], [slice-audit.template.md], [../01-concepts/strict-specs.md], [../04-sprint-workflow/spec-template.md], [../06-experemental/semi-autonomous-iterative-discussion-pipeline.md]]
depends_on: [[metadata.json]]
generated: false
source_of_truth: governance
outline: []
---

# Strict AI Task Spec Template

## Start Here

1. `<primary architecture or roadmap doc>`
2. `<decision matrix>`
3. `<acceptance or gate policy>`
4. `<external parity doc if used>`

## Intent

`<exact goal>`

Non-goals:
1. `<non-goal-1>`
2. `<non-goal-2>`

## Target Outcome

1. `<observable behavior 1>`
2. `<observable behavior 2>`

## Locked Decisions

1. `<decision-id-1>`
2. `<decision-id-2>`

## Architecture Fit

1. `<package>` owns `<responsibility>`
2. `<package>` owns `<responsibility>`

## External References (optional)

1. `<repo/file> -> Adopt|Adapt|Reject -> rationale`
2. `<repo/file> -> Adopt|Adapt|Reject -> rationale`

## Implementation Tasks (ordered)

1. `T1`
- package: `<package>`
- files: `<path1>`, `<path2>`
- output: `<result>`
2. `T2`
- package: `<package>`
- files: `<path3>`
- output: `<result>`

## Requirement -> Task -> Test -> Gate

| Requirement | Task IDs | Tests | Acceptance | Gates |
| --- | --- | --- | --- | --- |
| `<req-1>` | `T1` | `<test-command>` | `<A*>` | `<G*>` |
| `<req-2>` | `T2` | `<test-command>` | `<A*>` | `<G*>` |

## Acceptance and Tests

1. `<build command>`
2. `<type-check command>`
3. `<target tests>`

## Promotion Artifacts

1. `<release-descriptor>`
2. `<gate-summary>`
3. `<acceptance-report>`
4. `<kpi-report>`
5. `<rollback-report>`

## Risks and Rollback

1. Risk: `<risk>`
2. Mitigation: `<mitigation>`
3. Rollback: `<explicit steps>`

## Agent Guardrails

1. Do not change `<forbidden area>`.
2. Do not bypass required tests and gates.

## Evidence

- path: `../../Documentation System Pack/templates/documentation-system/AI-TASK-SPEC.template.md`
  symbol: `AI Task Spec Template`
  lines: 1-240
- path: `../01-concepts/strict-specs.md`
  symbol: `Mandatory Sections`
  lines: 1-220

## Links

- [[AGENTS.template.md]]
- [[external-reference-policy.template.md]]
- [[slice-audit.template.md]]
- [[../01-concepts/strict-specs.md]]
- [[../04-sprint-workflow/spec-template.md]]
- [[../06-experemental/semi-autonomous-iterative-discussion-pipeline.md]]
