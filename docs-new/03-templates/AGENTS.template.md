---
id: template-agents
title: Project AGENTS Template
type: template
status: stable
tags: [agents, template, governance]
related: [[../AGENTS.template.md], [strict-spec.template.md], [external-reference-policy.template.md], [INDEX.template.md]]
depends_on: [[metadata.json]]
generated: false
source_of_truth: governance
outline: []
---

# Project AGENTS.md Template

## Documentation Rules (Mandatory)

1. Source of truth is code and observable runtime behavior.
2. Any change to contracts, CLI, configuration, stages, or runtime behavior updates docs in same task.
3. Keep `INDEX`, `inventory`, `progress`, `validation` synchronized after substantial changes.
4. Unknown areas are explicit (`UNKNOWN` / `deferred`).
5. Historical reports are immutable artifacts unless policy says otherwise.

## AI Task Spec Rules (Mandatory For AI Coding)

1. Every coding task uses strict spec with full mandatory sections.
2. Task spec must include package ownership and target file paths.
3. Task spec must include requirement -> test -> gate mapping.
4. Rollback procedure is required.

## External Reference Rules (Optional, Controlled)

1. External repos are optional support, not source of authority.
2. Each external pattern is tagged `Adopt`, `Adapt`, or `Reject`.
3. Local architecture and contracts always win conflicts.

## Required Reading Chain (For Agents)

1. `01-concepts/ai-driven-development.md`
2. `01-concepts/strict-specs.md`
3. `01-concepts/discussion-first.md`
4. `02-guides/agent-navigation.md`
5. `00-navigation/INDEX.md`

## Evidence

- path: `../../Documentation System Pack/templates/documentation-system/AGENTS.docs-rules.template.md`
  symbol: `Documentation Rules`
  lines: 1-200
- path: `strict-spec.template.md`
  symbol: `Mandatory Sections`
  lines: 1-320

## Links

- [[../AGENTS.template.md]]
- [[strict-spec.template.md]]
- [[external-reference-policy.template.md]]
- [[INDEX.template.md]]
