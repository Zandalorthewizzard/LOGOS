---
id: logos-approval-control-architecture-spec
title: LOGOS Approval Control Architecture Spec
type: spec
status: active
tags: [architecture, approvals, control-flow, tools, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [../research/2026-03-30-logos-provider-research-report.md],
    [../research/2026-03-30-logos-codex-evidence-appendix.md]
  ]
depends_on:
  [
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Approval Control Architecture Spec

## Purpose

- Lock approval handling as an explicit control surface before coding.
- Prevent approval state from disappearing into generic tool execution logic.
- Preserve Codex strengths around separate approval flows, rejection branches, and resumability.

## Core Decision

- Approvals are first-class control-flow artifacts.
- Approval state must be representable in runtime memory, stream processing, and durable journal.
- Provider adapters may surface approval-relevant events, but approval policy belongs to Logos runtime.

## Why This Needs Its Own Spec

- The research pack explicitly separates resume and approval surfaces in current Codex behavior.
- Tool execution safety depends on preserving approval state and branch outcomes.
- Hidden approval branching would create severe architecture regret once multiple adapters exist.

## Approval Objects

The architecture must support explicit approval objects for, at minimum:

1. command execution approval
2. patch or file mutation approval
3. MCP or remote-tool execution approval
4. branch outcome recording: approved, denied, canceled, expired, interrupted

## Responses Reuse Evaluation

### What This Spec Reuses From Responses

- very little directly from `Responses`
- mainly the lesson that model output and executable side effects must stay distinct
- tool-call framing remains useful, but approval itself is Logos-owned rather than provider-owned

### Advantages Of Reusing Responses Concepts

- compatible with the canonical OpenAI path where model output leads into tool action
- preserves a clean seam between model intent and runtime execution

### Disadvantages And Limits

- `Responses` does not give Logos a complete approval model
- overreliance on provider tool surfaces would hide the runtime-owned safety policy
- approval semantics must survive providers that expose tools very differently or not at all

### Decision

- Reuse only the model-to-tool transition lesson from `Responses`.
- Keep approval architecture firmly runtime-owned and provider-agnostic at the control-flow level.

## Approval Flow Rules

### Rule 1: Explicit Pause Points

- When an action requires approval, the session loop must enter an explicit pending-approval state.

### Rule 2: Stable Correlation

- The approval object must correlate to the exact requested action and call ID.

### Rule 3: Durable Branching

- Approval result must be durably recorded and replayable.
- Resume logic must know whether the action was approved, denied, or still pending.

### Rule 4: No Hidden Auto-Execution

- Provider-native convenience features must not bypass approval policy unless the product explicitly permits it.

## Provider Interaction Rules

- Providers may trigger tool intents.
- Providers do not own the final approval decision for runtime-owned tools.
- Provider-native tools that can execute side effects require explicit policy on whether they bypass, mirror, or integrate with Logos approval logic.

## Non-goals

- no UI-specific approval presentation design yet
- no final wire protocol for host/daemon approval transport yet
- no silent conflation of approval denial with normal tool failure

## Acceptance Gates

1. Approval state is represented as explicit control-flow, not just logs.
2. Tool execution cannot lose correlation with its approval decision.
3. Resume can reconstruct pending, approved, and denied branches.
4. Provider-native tool shortcuts cannot silently bypass runtime policy.

## Open Questions

- which provider-native tool categories, if any, may bypass normal approval in 0.1
- whether approval policy needs separate classes for local vs remote tool effects
- what minimum approval record shape should be persisted in the durable journal

## Evidence

- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Confirmed Architecture Facts`
  lines: `68-82`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Tool-Loop Findings`
  lines: `167-179`
- path: `../research/2026-03-30-logos-codex-evidence-appendix.md`
  symbol: `Code-backed Evidence Blocks`
  lines: `22-120`

## Links

- [[README.md]]
- [[2026-04-02-logos-tool-loop-architecture-spec.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
