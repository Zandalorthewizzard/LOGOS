---
id: logos-guide-op-orchestrated-agent-loop
title: Orchestrated Agent Loop
type: guide
status: active
tags: [operations, agents, orchestration, review]
related:
  [
    [../agent-navigation.md],
    [../working-notes.md],
    [2026-04-18-slice-risk-register-and-boundary-discipline.md],
    [2026-04-14-glm-5.1-agent-profile.md],
    [../../../AGENTS.md],
    [../../07-logos-phase/implementation/2026-03-30-logos-global-roadmap.md]
  ]
depends_on: []
generated: false
source_of_truth: process
outline: []
---

# Orchestrated Agent Loop

## Purpose

Define the default LOGOS workflow for using an external execution agent on bounded implementation slices without relying on that agent to judge completeness.

## Source Note

This guide is the project-local adaptation of the workflow captured in `C:\Work\SYNC\MIND8\70_PROJECTS\doc_sys\research\logos-orchestrated-agent-loop-template.md`.

## Why This Exists

1. External implementation agents are useful workers, but they are not always reliable judges of contract completeness.
2. LOGOS phase work frequently spans protocol contracts, runtime semantics, and capability truth, so narrow implementation proofs are not enough.
3. A stable loop of `task -> implementation -> independent review -> narrowed next task` reduces false-green outcomes.

## Default Loop

1. Orchestrator reads the current roadmap, discussion doc, strict spec, and relevant code paths.
2. Orchestrator locks one bounded slice with explicit goals, non-goals, proofs, and return format.
3. External agent implements that slice and returns a structured report.
4. Orchestrator independently verifies the claim against code, tests, and the actual contract.
5. Orchestrator returns a verdict in the fixed form:
   `Findings -> Verification -> Verdict`
6. If gaps remain, orchestrator writes the next narrower task from those findings.
7. The loop repeats until the full phase closes honestly.

## Core Rules

1. One external-agent pass should own one closely related slice, not a mixed bag of surfaces.
2. The task must state explicit non-goals so the agent does not widen scope opportunistically.
3. The return format must force the agent to separate implemented work, explicit unsupported modes, and blockers.
4. The orchestrator must not trust a closure claim without independent verification.
5. If the code only supports a narrow happy path, the next task must tighten contract truth before widening breadth.
6. Every bounded slice must carry explicit residual-risk, stale-owner, and adjacent-open-surface accounting from `2026-04-18-slice-risk-register-and-boundary-discipline.md`.
7. Every bounded slice must also state the remainder toward maximum realizable parity for the touched surface.

## Task Writing Standard

Every task for an external execution agent should include:

1. Branch and phase context.
2. Locked decisions that are no longer open to redesign.
3. Exact scope for this pass.
4. Explicit non-goals.
5. Required proofs and test targets.
6. Guardrails about seams, fallbacks, and unsupported behavior.
7. A fixed return format.
8. A bounded risk register that names adjacent-open surfaces, transitional residue, stale owner logic, seam implications, and weaker semantics that must remain explicit.
9. A parity remainder that names what still remains toward maximum realizable parity and why it is not in this slice.

## Execution Task Template

```text
Next pass: <bounded slice name>.

Work in the same branch:
`<branch-name>`

This is a strict bounded implementation pass.
Do not widen scope.

Locked decisions:
1. <decision 1>
2. <decision 2>
3. <decision 3>

Read first:
1. <doc-or-file-1>
2. <doc-or-file-2>
3. <doc-or-file-3>

Primary objective:
<one-sentence statement of the exact slice to close>

What to do:

A. <sub-slice A>
- <required behavior>
- <required behavior>

B. <sub-slice B>
- <required behavior>
- <required behavior>

Explicit behavior rules:
- <what must now work>
- <what must remain explicit if unsupported>

Non-goals:
- <non-goal 1>
- <non-goal 2>
- <non-goal 3>

Risk register:
- Adjacent still-open surfaces: <surface 1>, <surface 2>
- Transitional residue allowed in this slice: <residue>
- Stale owner logic targeted for removal or downgrade: <logic>
- Seam implications: <seam or "none">
- Weaker semantics that must remain explicit: <semantics>

Guardrails:
1. <guardrail 1>
2. <guardrail 2>
3. <guardrail 3>

Required tests and proofs:
1. <proof test 1>
2. <proof test 2>
3. <proof test 3>

Testing instructions:
- Run targeted tests for every changed crate.
- Run only specific relevant targets, not the full workspace.
- Run `just fix -p ...` for changed Rust crates.
- Run `just fmt`.
- Do not rerun tests after final `fix` / `fmt`.

Return format:
1. `Closed`
2. `Still Explicitly Unsupported`
3. `Blocked`
4. `Changed crates/files`
5. `Tests/checks run`
6. `Verdict`
7. `Explicit answers`
8. `Residual risks`
```

## Agent Report Template

```text
1. Closed
- <what was actually implemented>

2. Still Explicitly Unsupported
- <exact modes or operations that remain unsupported after this pass>

3. Blocked
- <what could not be implemented honestly, with exact reason>

4. Changed crates/files
- <crate>
  - <file>
  - <file>

5. Tests/checks run
- <command>
- <command>

6. Verdict
- `slice closed`
- or `still blocked by <exact blocker>`

7. Explicit answers
- <contract question 1>
- <contract question 2>
- <contract question 3>

8. Residual risks
- <adjacent-open surface 1>
- <transitional residue>
- <stale owner or seam risk>

9. Parity remainder
- <maximum parity surface still open>
- <why it is not in this slice>
- <follow-up slice or blocker>
```

## Review Verdict Template

```text
Findings
1. <severity>: <gap, overclaim, or risk with file references>
2. <severity>: <gap, overclaim, or risk with file references>

Verification
- <test or command rerun>
- <test or command rerun>

Verdict
<whether the slice is cleanly closed, partially closed, or still blocked>
<what the next bounded slice must be>
```

## When To Tighten The Loop

Tighten the slice size further when:

1. The agent repeatedly overclaims closure.
2. Request modes or config fields are being silently ignored.
3. The contract is broader than the tests.
4. Product/runtime semantics matter more than local happy-path code.

## Evidence

- path: `../agent-navigation.md`
  symbol: `Task Chain`
  lines: 1-120
- path: `2026-04-18-slice-risk-register-and-boundary-discipline.md`
  symbol: `Artifact Requirements`
  lines: 1-240
- path: `../working-notes.md`
  symbol: `What To Capture`
  lines: 1-200
- path: `../../../AGENTS.md`
  symbol: `Documentation Workflow`
  lines: 1-140
- path: `../../07-logos-phase/implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `LOGOS 0.1 beta`
  lines: 180-280

## Links

- [[../agent-navigation.md]]
- [[../working-notes.md]]
- [[2026-04-18-slice-risk-register-and-boundary-discipline.md]]
- [[2026-04-14-glm-5.1-agent-profile.md]]
- [[../../../AGENTS.md]]
- [[../../07-logos-phase/implementation/2026-03-30-logos-global-roadmap.md]]
