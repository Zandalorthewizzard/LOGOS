---
id: logos-guide-op-research-transfer-and-dual-consensus-validation
title: Research Transfer And Dual-Consensus Validation
type: guide
status: active
tags: [operations, research, decision-surfaces, validation, consensus]
related:
  [
    [2026-04-14-orchestrated-agent-loop.md],
    [../working-notes.md],
    [../../../AGENTS.md],
    [../../07-logos-phase/architecture/2026-04-11-logos-philosophy-and-invariants.md],
    [
      ../../07-logos-phase/architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
  ]
depends_on: []
generated: false
source_of_truth: process
outline: []
---

# Research Transfer And Dual-Consensus Validation

## Purpose

Define how LOGOS imports ideas, mechanics, and design patterns from large external research packages into local decision surfaces without relying on chat memory.

This guide exists so research transfer becomes:

1. indexed
2. exhaustive
3. verdict-driven
4. easy to audit later

## When To Use This

Use this workflow when:

1. the user brings a large research package with an index document
2. the research spans multiple mechanics or decision surfaces
3. the goal is not just to read the research, but to decide what LOGOS should adopt, adapt, defer, or reject

## Core Principle

External research is input, not authority.

LOGOS does not adopt an idea only because:

1. it exists in a mature OSS project
2. it is battle-tested elsewhere
3. it looks similar to a current LOGOS problem

Every imported idea must still be judged against:

1. LOGOS philosophy and invariants
2. current ownership model
3. current roadmap and phase scope
4. actual code and runtime behavior

## Default Workflow

1. Read the research index document first.
2. Enumerate every major aspect, mechanic, concept, and implementation pattern from that index.
3. Map each one to the relevant LOGOS decision surface or discussion doc.
4. Write an assistant transfer verdict for each item with an explicit rationale.
5. Update the relevant discussion docs with the accepted or contested observations.
6. Present each item and verdict to the user for a second explicit verdict.
7. Mark an item as valid only after dual consensus exists.
8. Only after validation, use the result to write or update strict specs, implementation slices, or tasks.

## Required Artifacts

For a real research intake, create two documents:

1. a research-specific transfer ledger
2. updates to the affected decision-surface discussions

This guide is the workflow definition.
The ledger is the execution artifact for one concrete research package.

## Assistant Transfer Verdicts

Use exactly one of these verdicts per item:

### 1. `ACCEPT`

Meaning:

1. the idea is compatible with current LOGOS philosophy
2. the ownership model is sound
3. the mechanism is mature enough to reinterpret directly into LOGOS

Required explanation:

1. why it fits
2. which LOGOS layer should own it
3. which discussion doc now treats it as accepted input

### 2. `ADAPT`

Meaning:

1. the idea is valuable
2. but the original shape is not directly acceptable for LOGOS
3. a bounded reinterpretation is needed

Required explanation:

1. what is good in the source idea
2. what must change for LOGOS
3. where the adapted form should be fixed

### 3. `CUSTOM`

Meaning:

1. the core substrate already exists locally in LOGOS or preserved Codex
2. the real work is extraction, re-hosting, or ownership correction rather than importing the external pattern directly
3. external research is still useful, but mainly as validation of owner shape, lifecycle expectations, or surrounding refinements

Required explanation:

1. which local substrate already exists
2. why its current owner or coupling is wrong
3. what shared LOGOS layer should become the new owner

### 4. `DEFER`

Meaning:

1. the idea may be useful
2. but the current phase, ownership model, or unresolved decisions make acceptance premature

Required explanation:

1. what blocks adoption now
2. what future discussion or phase should revisit it

### 5. `REJECT`

Meaning:

1. the idea conflicts with LOGOS philosophy, ownership, semantics, or roadmap
2. or it would create architectural distortion

Required explanation:

1. why it conflicts
2. which current invariant, boundary, or phase rule it violates

## User Consensus Verdicts

The user should answer each item with one of the same verdict families:

1. `ACCEPT`
2. `ADAPT`
3. `CUSTOM`
4. `DEFER`
5. `REJECT`

The point is not cosmetic agreement.
The point is to produce a second explicit judgment that can be traced later.

## Consensus State Rules

Each item in the research transfer ledger must also have a `consensus_state`.

Use exactly one:

### `PENDING`

The item has not yet received an assistant verdict.

### `ASSISTANT_REVIEWED`

The assistant verdict exists, but the user verdict does not.

### `DUAL_CONSENSUS_VALID`

Both sides gave the same verdict family and the item is now valid input for:

1. strict specs
2. implementation slices
3. task writing

### `CONTESTED`

Assistant and user verdicts diverged, or the rationale still conflicts.

These items must stay in discussion and cannot be used as settled implementation input.

### `DEFERRED`

Both sides agree the item is out of scope for now.

### `REJECTED`

Both sides agree the item should not be adopted.

## Research Transfer Ledger Shape

Create the research-specific ledger using this item format:

```text
1. Research item
- source_ref: <research doc / section / symbol>
- topic: <mechanic or concept>
- summary: <one compact statement>

2. LOGOS target surface
- target_discussions:
  - <discussion doc>
  - <discussion doc>
- target_layer: <adapter / session-state / provider-native backend / app-server / host ops / etc.>

3. Assistant transfer verdict
- verdict: <ACCEPT | ADAPT | CUSTOM | DEFER | REJECT>
- rationale:
  - <why>
  - <why>
- required reinterpretation:
  - <only if ADAPT>

4. User verdict
- verdict: <ACCEPT | ADAPT | CUSTOM | DEFER | REJECT>
- rationale:
  - <user rationale>

5. Consensus state
- <PENDING | ASSISTANT_REVIEWED | DUAL_CONSENSUS_VALID | CONTESTED | DEFERRED | REJECTED>

6. Documentation updates
- updated_discussions:
  - <doc>
  - <doc>
- follow_up_specs:
  - <spec or none yet>
```

## Sweep Discipline

The ledger is not complete until every major item from the research index is accounted for.

Use a top-level sweep checklist:

1. all index sections enumerated
2. all mechanics mapped to decision surfaces
3. all assistant verdicts written
4. all user verdicts captured
5. all consensus states assigned
6. all accepted or adapted items reflected in discussion docs

## What Counts As Ready For Specs

A research-derived item is ready to drive specs, sprints, or tasks only if:

1. it is `DUAL_CONSENSUS_VALID`
2. its target layer is explicit
3. the relevant discussion docs were updated
4. its conflicts with current LOGOS invariants are either absent or resolved explicitly

## What Must Not Happen

1. no importing ideas straight from research into tasks without ledger review
2. no relying on "we already discussed this in chat"
3. no mixing accepted and contested items in one implementation spec
4. no silent adoption of ideas that were only partially examined

## Recommended Orchestrator Language

For each item, the orchestrator should say one of:

1. `I agree and would ACCEPT this because ...`
2. `I agree with the core idea, but would ADAPT it because ...`
3. `I would mark this CUSTOM because the substrate already exists locally, but the owner is wrong and needs extraction ...`
4. `I would DEFER this because ...`
5. `I would REJECT this because ...`

This keeps the transfer readable and makes later audit simple.

## Relationship To The Agent Loop

This workflow sits before implementation tasking.

The normal sequence is:

1. external or manual research
2. research transfer ledger and discussion updates
3. dual-consensus validation
4. strict specs
5. bounded implementation slices
6. independent review

## Evidence

- path: `2026-04-14-orchestrated-agent-loop.md`
  symbol: `Default Loop`
  lines: `20-66`
- path: `../../../AGENTS.md`
  symbol: `Documentation Workflow`
  lines: `58-72`
- path: `../../07-logos-phase/architecture/2026-04-11-logos-philosophy-and-invariants.md`
  symbol: `Acceptance Use`
  lines: `117-126`
- path: `../../07-logos-phase/architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Current Consensus`
  lines: `22-37`

## Links

- [[2026-04-14-orchestrated-agent-loop.md]]
- [[../working-notes.md]]
- [[../../../AGENTS.md]]
- [[../../07-logos-phase/architecture/2026-04-11-logos-philosophy-and-invariants.md]]
- [[../../07-logos-phase/architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
