---
id: template-migration-checklist
title: Migration Checklist Template
type: template
status: stable
tags: [migration, checklist, template]
related: [[README.md], [../02-guides/documentation-replication.md]]
depends_on: [[metadata.json]]
generated: false
source_of_truth: process
outline: []
---

# Migration Checklist Template

## 0. Preparation

1. Backup existing docs.
2. Record migration date and owner.
3. Define target docs root.

## 1. Inventory Pass

1. List current documents.
2. Classify each doc by type.
3. Assign status (`stable`, `draft`, `deprecated`, `closed`, `active`).

## 2. Structure Migration

1. Move docs into target structure.
2. Mark superseded docs explicitly.

## 3. Canonical Baseline

1. Create or update index and meta-files.
2. Ensure canonical claims reflect real implementation, not assumptions.

## 4. AGENTS Integration

1. Merge governance rules into project AGENTS file.
2. Add required reading chain.
3. Verify source-of-truth rule is explicit.

## 5. Strict Task Layer

1. Adopt strict task spec template for AI coding.
2. Enforce mandatory sections in project governance.
3. Add external reference policy when third-party patterns are used.

## 6. Discoverability

1. Add all canonical docs to index.
2. Add reciprocal links from docs back to hubs.
3. Ensure discussion hubs link to topic documents.

## 7. Validation Pass

1. Update verified claims in validation report.
2. Add evidence blocks (`path`, `symbol`, `lines`).
3. Mark unknown areas as `UNKNOWN` or `deferred`.

## 8. Done Criteria

1. Index covers active documentation.
2. Inventory is valid and current.
3. Progress records migration work.
4. Validation contains claims, evidence, and open risks.
5. AGENTS enforces documentation governance.

## Evidence

- path: `../../Documentation System Pack/templates/documentation-system/migration-checklist-ru.md`
  symbol: `0..8 checklist`
  lines: 1-260
- path: `../02-guides/documentation-replication.md`
  symbol: `Migration Workflow`
  lines: 1-220

## Links

- [[README.md]]
- [[../02-guides/documentation-replication.md]]
