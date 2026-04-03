---
id: template-slice-audit
title: Slice Audit Report Template
type: template
status: stable
tags: [audit, quality, template, gates]
related: [[README.md], [strict-spec.template.md], [../02-guides/operations/slice-audit.md], [../02-guides/operations/acceptance-testing.md]]
depends_on: [[metadata.json]]
generated: false
source_of_truth: governance
outline: []
---

# Slice Audit Report Template

## Start Here

1. Fill one report per slice.
2. Keep section order unchanged for comparability.
3. Update slice tracker in the same change.

## 1) Passport

- slice_id:
- stream_id:
- date_utc:
- branch:
- commit_range:
- auditor:
- mode:

## 2) Scope Traceability

| Requirement | Task | Tests | Gate | Artifact | Status |
| --- | --- | --- | --- | --- | --- |
| | | | | | |

Rule: each scoped requirement must map to at least one test and one artifact.

## 3) Implementation Delta

1. Code delta summary:
2. Contract or type changes:
3. Documentation changes:
4. Classification (`additive` or `breaking`):
5. Rollback impact:

## 4) External Analogy Or Reference

| Area | Local Current | External Analog | Adopt/Adapt/Reject | Reasoning | Gap |
| --- | --- | --- | --- | --- | --- |
| | | | | | |

Rule: if no valid analogy exists, write `N/A` and rationale.

## 5) Freshness Check

| Topic | Package or Tool | Current Version | Current Best Practice | Decision | Notes |
| --- | --- | --- | --- | --- | --- |
| | | | | | |

Minimum:
1. At least two freshness checks.
2. One for testing toolchain relevance.
3. One for runtime or library decision relevance.

## 6) Quality and Test Audit

### 6.1 Executed Commands

```powershell
<build>
<type-check>
<target tests>
```

### 6.2 False-Green Guard

1. Confirm build before tests.
2. Re-run critical tests at least twice.
3. Run one negative control.
4. Check no residual temp artifacts affect result.

Record:
- stale_dist_guard:
- repeatability_runs:
- mutation_sanity:
- residual_artifact_check:

## 7) Metrics

| Metric | Definition | Target | Actual | Status |
| --- | --- | --- | --- | --- |
| Traceability completeness | mapped requirements / scoped requirements | 100% | | |
| Behavioral evidence coverage | requirements with behavioral proof / scoped requirements | >= 90% | | |
| Determinism stability | identical deterministic outputs on reruns | 100% | | |
| False-green risk | all guard checks pass | pass | | |
| Freshness check coverage | checks with timestamp and decision | 100% | | |

## 8) Findings and Risks

| Severity | Area | Finding | Impact | Action |
| --- | --- | --- | --- | --- |
| | | | | |

## 9) Decision and Follow-Ups

1. Decision (`pass`, `conditional-pass`, `fail`):
2. Blocking conditions:
3. Follow-up tasks:
4. Rollback plan:

## 10) Evidence Links

1. Code paths:
2. Test files:
3. Report artifacts:
4. Documentation updates:

## Naming Convention

1. Report file: `<slice-id>-audit-report-YYYY-MM-DD.md`.
2. One tracker row per slice with report path and commit hash.

## Evidence

- path: `../../Documentation System Pack/guides/slice-audit-template-v1.md`
  symbol: `Report Sections (mandatory order)`
  lines: 1-320
- path: `../02-guides/operations/slice-audit.md`
  symbol: `Core Checks`
  lines: 1-200

## Links

- [[README.md]]
- [[strict-spec.template.md]]
- [[../02-guides/operations/slice-audit.md]]
- [[../02-guides/operations/acceptance-testing.md]]
