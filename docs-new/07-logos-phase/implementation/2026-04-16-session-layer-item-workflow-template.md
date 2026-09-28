---
id: logos-session-layer-item-workflow-template
title: Session Layer Item Workflow Template
type: template
status: active
tags: [logos, architecture, phase-6, session-state, template, workflow]
related:
  [
    [../implementation/2026-04-16-item-01-implementation-reference.md],
    [
      ../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md,
    ],
    [../research/session layer/SUMMARY.md],
  ]
generated: false
source_of_truth: canonical
---

# Session Layer Item Workflow Template

## Purpose

This document defines the standard workflow for processing each Item from the session-layer research ledger into an implementation-ready state. Item 01 (Thread Manager) established this workflow; future items must follow it.

---

## Pipeline Stages

Each item passes through four stages. A stage gate must be passed before the next stage begins.

```
Research Analysis → Verdict Discussion → Draft Implementation → Implementation Reference
        (RA-*)              (Ledger)          (Discussion)           (Reference)
```

---

## Stage 1: Research Analysis (Completed for all 16 items)

**Input:** Source code of LOGOS, OpenCode, Goose.
**Output:** `RA-X.X-mechanic-name.md` in `docs-new/07-logos-phase/research/session layer/`.

**Already done.** All 16 RA documents have:

- LOGOS Current Implementation (with file:lines evidence)
- OpenCode Analysis
- Goose Analysis
- Cross-Project Comparison Matrix
- Recommendations for LOGOS

**Stage gate:** RA document has no `TBD` in comparison matrix.

---

## Stage 2: Verdict Discussion

**Input:** RA document for the item.
**Output:** Verdict entry in the research transfer ledger + discussion document.

### 2a. Verdict Options

| Verdict    | Meaning                                                                        | Implication                                                                           |
| ---------- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------- |
| **ACCEPT** | Take the pattern as-is. Core mechanic maps directly.                           | Implementation refactors competitor code with minimal changes.                        |
| **ADAPT**  | Direction correct, but LOGOS constraints require significant reinterpretation. | Implementation uses competitor pattern as skeleton but rewrites substantial portions. |
| **CUSTOM** | Core substrate already exists locally, but under the wrong owner or coupling.  | Implementation extracts or re-hosts local LOGOS/Codex mechanics into a shared layer. |
| **DEFER**  | Not needed for current MVP scope.                                              | No implementation work. Revisit when scope expands.                                   |
| **REJECT** | Pattern does not fit LOGOS at all.                                             | LOGOS must build its own solution from scratch.                                       |

### 2b. Verdict Criteria

The verdict must be argued on **fundamental constraints**, not surface differences:

- ACCEPT requires: the pattern works for LOGOS without structural changes
- ADAPT requires: the pattern's direction is right but LOGOS has a fundamental constraint that prevents direct adoption
- CUSTOM requires: the needed substrate already exists locally and the main task is correcting ownership/coupling rather than importing the external pattern directly
- DEFER requires: the item is explicitly out of MVP scope
- REJECT requires: the pattern contradicts LOGOS philosophy or architecture

Surface-level differences (different crate names, different error types, different field names) are NOT grounds for ADAPT. They are normal refactoring.

### 2c. Discussion Document Template

File: `docs-new/07-logos-phase/architecture/2026-04-XX-item-NN-mechanic-name-draft-implementation-discussion.md`

```markdown
---
id: logos-item-NN-mechanic-name-draft-implementation-discussion
title: Item NN Mechanic Name Draft Implementation Discussion
type: discussion
status: active
tags: [logos, architecture, phase-6, session-state, mechanic-name]
related: [...]
depends_on: [...]
generated: false
source_of_truth: working-doc
---

# Item NN: Mechanic Name Draft Implementation Discussion

## Purpose

This document turns the validated Item NN verdict into a practical
implementation draft.

## Current Transfer Status

1. Item NN verdict is [ACCEPT/ADAPT/CUSTOM/DEFER/REJECT].
2. [Summary of what was accepted/adapted/deferred/rejected.]

## Accepted Import For Item NN

[The specific pattern(s) being imported, in 1-3 bullet points.]

## What This Draft Covers

[Numbered list of scope items — be narrow.]

## What This Draft Does Not Cover

[Numbered list of explicitly excluded items — these are separate decisions.]

## Current Code Diagnosis

### [Section per relevant current LOGOS component]

- Where: file:lines
- What it does
- Why it needs to change (or not)

## Draft Target Model

### [Sub-models as numbered sections]

## Practical Draft Shape

### Stage A: [First increment]

### Stage B: [Second increment]

### Stage C: [Third increment if needed]

## Minimal Draft Data Model

[Only if the item involves data schema changes.]

## Why This Still Supports The [VERDICT]

[Argument that the draft does not expand scope beyond the verdict.]

## Revisit Triggers

The current verdict should be re-opened only if:

1. [Concrete falsifiable condition]
2. [Concrete falsifiable condition]
3. [Concrete falsifiable condition]

## Closeout Expectation

The later strict spec should be considered successful only if:

1. [Concrete success criterion]
2. [Concrete success criterion]

## Evidence

- path: [file path]
  symbol: [symbol name]
  lines: [line range]

## Links
```

### 2d. Scope Control Rules

1. **No scope creep.** If the draft starts including items from the "Does Not Cover" list, stop and re-check the verdict.
2. **Adjacent mechanics are separate items.** If the draft needs ProcessManager, notification bus, or provider switching — reference them as dependencies, do not design them here.
3. **OpenAI canonical path stays separate** unless the item explicitly requires migration.

### 2e. Dual Consensus

When two agents independently analyze the item and reach the same verdict through different reasoning paths, the verdict is `DUAL_CONSENSUS_VALID`. This is the strongest validation level.

If agents disagree, the discussion must resolve the disagreement before proceeding to Stage 3.

---

## Stage 3: Implementation Reference

**Input:** Draft implementation discussion + competitor source code.
**Output:** `docs-new/07-logos-phase/implementation/2026-04-XX-item-NN-implementation-reference.md`

### 3a. Purpose

The implementation reference is a **code-level mapping** from competitor patterns to LOGOS Rust code. It answers: "which lines of competitor code do we refactor, which do we adapt, and what do we write from scratch?"

### 3b. Document Template

````markdown
---
id: logos-item-NN-mechanic-name-implementation-reference
title: Item NN Mechanic Name Implementation Reference
type: implementation-reference
status: active
tags:
  [logos, architecture, phase-6, session-state, mechanic-name, implementation]
related: [...]
depends_on: [...]
generated: false
source_of_truth: working-doc
---

# Item NN: Mechanic Name Implementation Reference

## Purpose

Concrete code patterns extracted from Goose (Rust) and OpenCode (TypeScript)
for each stage of the Item NN draft implementation.

## Source Projects

| Project  | Language   | Key Files                     |
| -------- | ---------- | ----------------------------- |
| Goose    | Rust       | [list files with line counts] |
| OpenCode | TypeScript | [list files with line counts] |

---

## Stage A: [First Stage Name]

### A.1 [Pattern Name]

**[Project]** (`file:lines`) — [Directly refactorable / Adapt / Write from scratch]:

```rust
// Competitor code
```
````

**LOGOS action:** [What to do with this code]

[Repeat A.2, A.3, ... for each pattern in this stage]

### A.N What LOGOS Must Write From Scratch

| Item | Why not refactorable |
| ---- | -------------------- |
| ...  | ...                  |

---

## Stage B: [Second Stage Name]

[Same structure as Stage A]

---

## Stage C: [Third Stage Name if applicable]

[Same structure as Stage A]

---

## Cross-Cutting: What [Project] Gets Right

| Pattern | Source     | Value for LOGOS |
| ------- | ---------- | --------------- |
| ...     | file:lines | ...             |

## Cross-Cutting: What Both Get Wrong

| Pattern | Problem | LOGOS Better Approach |
| ------- | ------- | --------------------- |
| ...     | ...     | ...                   |

## Implementation Priority

| Step      | Action | Source                     | Estimated Effort |
| --------- | ------ | -------------------------- | ---------------- |
| 1         | ...    | Goose refactor / New / ... | X days           |
| **Total** |        |                            | **~N days**      |

## Evidence

- path: [file path]
  symbol: [symbol name]
  lines: [line range]

```

### 3c. Source Code Reading Rules

1. **Goose is read at code level.** It's Rust. Extract actual code blocks that can be refactored.
2. **OpenCode is read at pattern level.** It's TypeScript. Extract the design pattern and translate to Rust mentally.
3. **Both must be read with line numbers.** Every code reference must have `file:lines` evidence.
4. **Mark every pattern** as one of:
   - **Directly refactorable** — can copy the Rust code with cosmetic changes
   - **Adapt** — pattern is right, LOGOS-specific changes needed
   - **Write from scratch** — no competitor equivalent exists

### 3d. Effort Estimation Rules

- "Refactor" = 0.5-1 day per pattern (copy, adjust names, add tests)
- "Adapt" = 1-2 days per pattern (understand, redesign partially, implement, test)
- "Write from scratch" = 2-3 days per pattern (design, implement, test, debug)
- Add 20% buffer for integration with existing LOGOS code

---

## Stage 4: Strict Spec (Post-Reference)

**Not covered by this template.** The strict spec is derived from the implementation reference and is the final document before coding begins. It follows the strict-spec template in `docs-new/03-templates/strict-spec.template.md`.

---

## File Naming Convention

| Stage | Pattern | Example |
|-------|---------|---------|
| Research Analysis | `RA-X.X-mechanic-name.md` | `RA-1.1-thread-manager.md` |
| Verdict Discussion | `2026-04-XX-item-NN-mechanic-name-draft-implementation-discussion.md` | `2026-04-16-item-01-thread-manager-draft-implementation-discussion.md` |
| Implementation Reference | `2026-04-XX-item-NN-implementation-reference.md` | `2026-04-16-item-01-implementation-reference.md` |
| Strict Spec | `2026-04-XX-item-NN-mechanic-name-strict-spec.md` | (future) |

All files live under `docs-new/07-logos-phase/`:
- Research: `research/session layer/`
- Discussions: `architecture/`
- References: `implementation/`

---

## Item Queue (Priority Order)

Based on the discussion plan and Item 01 experience:

| Priority | Item | RA Doc | Verdict | Discussion | Reference |
|----------|------|--------|---------|------------|-----------|
| 1 | Thread Manager | RA-1.1 | ACCEPT | Done | Done |
| 2 | Session Persistence | RA-1.2 | ADAPT | Done | Done |
| 3 | Archive/Metadata | RA-1.7 | Pending | — | — |
| 4 | Turn Lifecycle | RA-1.4 | ACCEPT | Done | Done |
| 5 | Fork/Resume | RA-1.3 | ADAPT | Done | Done |
| 6 | Shell Execution | RA-1.5 | ADAPT | Done | — |
| 7 | Review Orchestration | RA-1.6 | Pending | — | — |
| 8 | Turn Interrupt | RA-2.3 | Pending | — | — |
| 9 | Rollback | RA-2.1 | Pending | — | — |
| 10 | Background Terminals | RA-2.4 | Pending | — | — |
| 11 | Compaction | RA-2.2 | Pending | — | — |
| 12 | Turn Steering | RA-3.1 | Pending | — | — |
| 13 | Advanced Compaction | RA-3.2 | Pending | — | — |
| 14 | Multi-Provider Handoff | RA-3.3 | Pending | — | — |
| 15 | Streaming Optimization | RA-3.4 | Pending | — | — |

---

## Agent Coordination

When processing an item through this pipeline:

1. **Research agent** — reads source code, fills RA documents (already done)
2. **Discussion agent** — reads RA doc, argues verdict, writes discussion (needs human checkpoint for verdict)
3. **Reference agent** — reads discussion + competitor source, writes implementation reference
4. **Human** — reviews at verdict checkpoint and reference completion

The human checkpoint is mandatory at Stage 2 (verdict) and optional at Stage 3 (reference).
```
