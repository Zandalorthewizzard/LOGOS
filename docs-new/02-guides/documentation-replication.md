---
id: guide-doc-replication
title: Documentation System Replication
type: guide
status: stable
tags: [bootstrap, migration, replication]
related: [[../README.md], [../03-templates/README.md], [../03-templates/migration-checklist.template.md], [agent-navigation.md], [../01-concepts/discussion-first.md]]
depends_on: []
generated: false
source_of_truth: process
outline: []
---

# Documentation System Replication

## Scenarios

1. Bootstrap empty project.
2. Migrate existing project.

## Bootstrap Workflow

1. Copy templates from `03-templates/`.
2. Instantiate `INDEX`, `inventory`, `progress`, `validation`.
3. Create minimal canonical docs and link them in `INDEX`.
4. Insert AGENTS rules template into project `AGENTS.md`.

## Migration Workflow

1. Build document inventory.
2. Classify docs by type and status.
3. Move docs into target layered structure.
4. Mark superseded docs explicitly (`deprecated` / `superseded by`).
5. Sync meta-files and validate links.
6. Execute and archive results from `../03-templates/migration-checklist.template.md`.

## Done Criteria

1. Entry point exists and is discoverable.
2. Canonical claims are evidence-backed.
3. Meta-files are synchronized.
4. Unknown/deferred zones are explicit.

## Evidence

- path: `../docs-refactoring.md`
  symbol: `Priority 3: Restructure for Consistency`
  lines: 1-260
- path: `../../Documentation System Pack/documentation-system-replication-guide-ru.md`
  symbol: `Acceptance Checklist`
  lines: 1-320

## Links

- [[../README.md]]
- [[../03-templates/README.md]]
- [[../03-templates/migration-checklist.template.md]]
- [[agent-navigation.md]]
- [[../01-concepts/discussion-first.md]]
