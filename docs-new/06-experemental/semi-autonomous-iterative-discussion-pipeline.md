---
id: experimental-semi-autonomous-iterative-pipeline
title: Semi-Autonomous Iterative Discussion Pipeline
type: guide
status: draft
tags: [experimental, semi-autonomous, iteration, gates, human-in-the-loop]
related: [[README.md], [iteration-card.template.md], [boundary-question.template.md], [ad-gate-checklist.template.md], [../01-concepts/discussion-first.md], [../03-templates/strict-spec.template.md]]
depends_on: [[../01-concepts/discussion-first.md], [../01-concepts/strict-specs.md]]
generated: false
source_of_truth: discussion
outline: []
---

# Semi-Autonomous Iterative Discussion Pipeline

## Status

Draft concept for pilot use. Do not treat as mandatory baseline governance.

## Intent

Formalize pre-coding reasoning as iterative decision cycles with explicit human checkpoints only for boundary questions.

## Entry Contract

Each iteration requires three input packages:
1. Task package: task spec, decision matrix, acceptance constraints.
2. Evidence package: code anchors, docs, reports, runtime constraints.
3. Governance package: index, inventory, progress, validation docs.

## Iteration Stages

1. Stage 0 - Intake Lock: scope one decision ID and expected output.
2. Stage 1 - Fact Pass: separate facts from assumptions with evidence anchors.
3. Stage 2 - Option Pass: produce 2-3 viable options with impact notes.
4. Stage 3 - Reference Parity Pass: classify reference patterns as `Adopt`, `Adapt`, `Reject`.
5. Stage 4 - Risk and Rollback Pass: define failure modes, rollback unit, stop-conditions.
6. Stage 5 - Decision Draft: recommend one option and map to tests and gates.
7. Stage 6 - Boundary Escalation: ask human only unresolved boundary questions.
8. Stage 7 - Artifact Sync: sync discussion, task docs, and meta-files.
9. Stage 8 - Readiness Gate: mark `implementation-ready` or `blocked`.

## AD Gate Model

1. AD-G1: Scope lock confirmed.
2. AD-G2: Fact pass evidence complete.
3. AD-G3: Option matrix complete.
4. AD-G4: Parity decision (`Adopt/Adapt/Reject`) fixed.
5. AD-G5: Risk and rollback package complete.
6. AD-G6: Boundary questions resolved or explicitly deferred.
7. AD-G7: Artifacts synchronized.
8. AD-G8: Readiness status assigned.

## Iteration Outputs

1. Iteration card per decision ID.
2. Optional boundary-question packet.
3. Updated decision state (`ready`, `blocked`, `deferred`).

## Non-Goals

1. Replacing strict task specs.
2. Removing evidence requirements.
3. Bypassing human approval where policy requires it.

## Pilot Adoption

1. Start on one active stream.
2. Track late reversals and unresolved boundary questions.
3. Promote to core only after repeatable pilot success.

## Evidence

- path: `../../Pensieve/docs-new/DEV/discussions/autonomous-development-iterative-discussion-pipeline-v1-ru.md`
  symbol: `Stage 0..8, boundary escalation, AD-G1..AD-G8`
  lines: 1-280
- path: `../01-concepts/discussion-first.md`
  symbol: `Discussion -> Spec -> Implementation`
  lines: 1-220

## Links

- [[README.md]]
- [[iteration-card.template.md]]
- [[boundary-question.template.md]]
- [[ad-gate-checklist.template.md]]
- [[../01-concepts/discussion-first.md]]
- [[../03-templates/strict-spec.template.md]]
