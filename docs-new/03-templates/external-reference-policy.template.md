---
id: template-external-ref-policy
title: External Reference Policy Template
type: template
status: stable
tags: [external-references, portability, governance]
related: [[strict-spec.template.md], [AGENTS.template.md], [../01-concepts/strict-specs.md]]
depends_on: [[metadata.json]]
generated: false
source_of_truth: governance
outline: []
---

# External Reference Policy Template

## Purpose

Define controlled reuse of external repository patterns without violating local architecture.

## Source Registry

| ID | Repository | Path or URL | Purpose | Status |
| --- | --- | --- | --- | --- |
| EXT-01 | `<repo>` | `<path-or-url>` | `<purpose>` | active |

## Portability Matrix

| External Pattern | Local Decision IDs | Adopt/Adapt/Reject | Notes |
| --- | --- | --- | --- |
| `<pattern>` | `<ID list>` | Adapt | `<notes>` |

## Constraints

1. External logic cannot override local contracts.
2. Provider-specific details remain optional unless approved.
3. Port behavior, not foreign architecture assumptions.

## Evidence Mapping

1. external path or URL: `<...>`
2. relevant symbols: `<...>`
3. mapped local files: `<...>`

## Change Control

1. Re-evaluate matrix when local contracts change.
2. Re-evaluate matrix when external source changes materially.
3. Record updates in progress and validation docs.

## Evidence

- path: `../../Documentation System Pack/templates/documentation-system/EXTERNAL-REFERENCE-POLICY.template.md`
  symbol: `Portability Matrix`
  lines: 1-220
- path: `AGENTS.template.md`
  symbol: `External Reference Rules`
  lines: 1-220

## Links

- [[strict-spec.template.md]]
- [[AGENTS.template.md]]
- [[../01-concepts/strict-specs.md]]
