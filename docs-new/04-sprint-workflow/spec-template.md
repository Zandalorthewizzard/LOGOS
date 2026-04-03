---
id: sprint-spec-template
title: Sprint Spec Template
type: template
status: stable
tags: [sprint, spec, template]
related: [[runbook.md], [../03-templates/strict-spec.template.md], [decision-matrix.template.md]]
depends_on: [[../01-concepts/strict-specs.md]]
generated: false
source_of_truth: governance
outline: []
---

# Sprint Spec Template

```md
---
id: dev-task-<stream>-s<idx>-<slug>
title: <STREAM> S<idx> <Title>
type: guide
status: active
tags: [<stream>, sprint-s<idx>, <domain-tags>]
related: [[DEV/tasks/active/<stream>/sprint-plan.md]]
depends_on: [[DEV/tasks/active/<stream>/decision-matrix.md]]
generated: false
source_of_truth: sprint task planning
outline: []
---

# <STREAM> S<idx> <Title>

## Start Here
1. <doc-link-1>
2. <doc-link-2>

## Intent
<goal>

Non-goals:
1. <non-goal-1>
2. <non-goal-2>

## Target Outcome
1. <outcome-1>
2. <outcome-2>

## Locked Decisions
1. <decision-id-1>
2. <decision-id-2>

## Architecture Fit
1. <package-1>: <ownership>
2. <package-2>: <ownership>

## Implementation Tasks (ordered)
1. T1: <task>
2. T2: <task>

## Requirement -> Task -> Test -> Gate
| Requirement | Task IDs | Tests | Acceptance | Gates |
| --- | --- | --- | --- | --- |
| <req-1> | T1 | <cmd> | <A*> | <G*> |

## Acceptance and Tests
1. <build>
2. <type-check>
3. <tests>

## Promotion Artifacts
1. <artifact-1>
2. <artifact-2>

## Risks and Rollback
1. Risk: <risk>
2. Mitigation: <mitigation>
3. Rollback: <steps>

## Agent Guardrails
1. Do not <forbidden>
```

## Evidence

- path: `../../Documentation System Pack/guides/sprint-authoring/sprint-spec-base-template-v1.md`
  symbol: `Sprint Spec Base Template`
  lines: 1-320
- path: `../03-templates/strict-spec.template.md`
  symbol: `Mandatory Sections`
  lines: 1-320

## Links

- [[runbook.md]]
- [[../03-templates/strict-spec.template.md]]
- [[decision-matrix.template.md]]
