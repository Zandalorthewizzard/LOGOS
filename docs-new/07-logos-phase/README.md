---
id: logos-phase-hub
title: LOGOS Architecture Phase Hub
type: hub
status: active
tags: [hub, logos, codex, adapters, architecture]
related:
  [
    [../00-navigation/INDEX.md],
    [analysis/2026-03-30-codex-model-agnostic-analysis.md],
    [architecture/2026-03-30-adapter-first-architecture-discussion.md],
    [architecture/2026-03-30-logos-adapter-boundary-spec.md],
    [architecture/2026-03-30-logos-crate-map.md],
    [architecture/2026-03-30-logos-runtime-types-v0.md],
    [architecture/2026-04-02-logos-adapter-project-docs-spec.md],
    [architecture/2026-04-02-logos-adapter-kernel-architecture.md],
    [architecture/2026-04-02-logos-provider-adapter-profiles.md],
    [architecture/2026-04-02-logos-provider-openai-adapter-spec.md],
    [architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md],
    [architecture/2026-04-02-logos-provider-gemini-adapter-spec.md],
    [architecture/2026-04-02-logos-provider-anthropic-adapter-spec.md],
    [architecture/2026-04-02-logos-provider-zai-adapter-spec.md],
    [architecture/2026-04-02-logos-provider-lmstudio-adapter-spec.md],
    [architecture/2026-04-02-logos-persistence-resume-architecture-spec.md],
    [architecture/2026-04-02-logos-tool-loop-architecture-spec.md],
    [architecture/2026-04-02-logos-capability-model-architecture-spec.md],
    [architecture/2026-04-02-logos-approval-control-architecture-spec.md],
    [architecture/2026-04-02-logos-host-daemon-transport-spec.md],
    [architecture/2026-04-02-logos-runtime-tier-contract-spec.md],
    [architecture/2026-04-02-logos-tier-0-semantic-surfaces-spec.md],
    [architecture/2026-04-02-logos-adapter-health-and-drift-policy.md],
    [architecture/2026-04-02-logos-minimal-survivable-agent-baseline-spec.md],
    [architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md],
    [architecture/2026-04-02-logos-runtime-contract-spec.md],
    [architecture/2026-04-02-logos-semantics-and-guarantees-matrix.md],
    [implementation/2026-03-30-logos-global-roadmap.md],
    [implementation/2026-03-30-logos-phase-1-implementation-slice.md],
    [implementation/2026-04-02-phase-1-boundary-extraction-strict-spec.md],
    [implementation/2026-04-03-codex-core-seam-index.md],
    [implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md],
    [implementation/2026-04-02-openrouter-validation-adapter-strict-spec.md],
    [implementation/2026-04-02-codex-base-repository-strategy.md],
    [implementation/2026-04-02-upstream-sync-policy.md],
    [implementation/2026-04-02-branch-and-pr-workflow.md],
    [research/2026-03-30-logos-research-requirements.md],
    [research/2026-03-30-logos-provider-research-report.md],
    [research/2026-04-02-opencode-goose-architecture-reference.md]
  ]
depends_on: [[../01-concepts/discussion-first.md]]
generated: false
source_of_truth: governance
outline: []
---

# LOGOS Architecture Phase Hub

This folder is the main architecture lane for the current LOGOS bootstrapping phase.

## Canonical Reading Order

1. [[analysis/2026-03-30-codex-model-agnostic-analysis.md]]
2. [[architecture/2026-03-30-adapter-first-architecture-discussion.md]]
3. [[architecture/2026-03-30-logos-adapter-boundary-spec.md]]
4. [[architecture/2026-03-30-logos-crate-map.md]]
5. [[architecture/2026-03-30-logos-runtime-types-v0.md]]
6. [[architecture/2026-04-02-logos-adapter-project-docs-spec.md]]
7. [[architecture/2026-04-02-logos-adapter-kernel-architecture.md]]
8. [[architecture/2026-04-02-logos-provider-adapter-profiles.md]]
9. [[architecture/2026-04-02-logos-provider-openai-adapter-spec.md]]
10. [[architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md]]
11. [[architecture/2026-04-02-logos-provider-gemini-adapter-spec.md]]
12. [[architecture/2026-04-02-logos-provider-anthropic-adapter-spec.md]]
13. [[architecture/2026-04-02-logos-provider-zai-adapter-spec.md]]
14. [[architecture/2026-04-02-logos-provider-lmstudio-adapter-spec.md]]
15. [[architecture/2026-04-02-logos-persistence-resume-architecture-spec.md]]
16. [[architecture/2026-04-02-logos-tool-loop-architecture-spec.md]]
17. [[architecture/2026-04-02-logos-capability-model-architecture-spec.md]]
18. [[architecture/2026-04-02-logos-approval-control-architecture-spec.md]]
19. [[architecture/2026-04-02-logos-host-daemon-transport-spec.md]]
20. [[architecture/2026-04-02-logos-runtime-contract-spec.md]]
21. [[architecture/2026-04-02-logos-runtime-tier-contract-spec.md]]
22. [[architecture/2026-04-02-logos-tier-0-semantic-surfaces-spec.md]]
23. [[architecture/2026-04-02-logos-adapter-health-and-drift-policy.md]]
24. [[architecture/2026-04-02-logos-minimal-survivable-agent-baseline-spec.md]]
25. [[architecture/2026-04-02-logos-semantics-and-guarantees-matrix.md]]
26. [[architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md]]
27. [[research/2026-03-30-logos-provider-research-report.md]]
28. [[research/2026-04-02-opencode-goose-architecture-reference.md]]
29. [[implementation/2026-04-02-codex-base-repository-strategy.md]]
30. [[implementation/2026-04-02-upstream-sync-policy.md]]
31. [[implementation/2026-04-02-branch-and-pr-workflow.md]]
32. [[implementation/2026-03-30-logos-global-roadmap.md]]
33. [[implementation/2026-03-30-logos-phase-1-implementation-slice.md]]
34. [[implementation/2026-04-02-phase-1-boundary-extraction-strict-spec.md]]
35. [[implementation/2026-04-03-codex-core-seam-index.md]]
36. [[implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md]]
37. [[implementation/2026-04-02-openrouter-validation-adapter-strict-spec.md]]
38. [[research/2026-03-30-logos-research-requirements.md]]

## Folder Layout

1. `analysis/` - audits and architectural analysis.
2. `architecture/` - discussions, boundary specs, crate maps, type contracts.
3. `implementation/` - phased execution and implementation slices.
4. `research/` - research tasking, reports, and provider comparison work.

## Purpose

1. Lock the current adapter-first architecture direction.
2. Preserve imported working context from earlier Chaemera-side exploration.
3. Make empty-context onboarding possible for agents starting inside LOGOS.
4. Provide the first stable landing zone for future canonical specs and research outputs.

## Current Placement Rule

1. All currently imported Logos strategy and architecture docs live here because they are part of the same current architecture phase.
2. Future narrowing can split them into more specialized folders if the project grows enough to justify it.

## Evidence

- path: `../../input-docs-temp/logos-adapter-boundary-spec.md`
  symbol: `Imported boundary spec`
  lines: 1-400
- path: `../../input-docs-temp/logos-research-requirements.md`
  symbol: `Imported research task brief`
  lines: 1-400

## Links

- [[../00-navigation/INDEX.md]]
- [[architecture/2026-03-30-logos-adapter-boundary-spec.md]]
- [[architecture/2026-03-30-logos-crate-map.md]]
- [[architecture/2026-03-30-logos-runtime-types-v0.md]]
- [[architecture/2026-04-02-logos-adapter-project-docs-spec.md]]
- [[architecture/2026-04-02-logos-adapter-kernel-architecture.md]]
- [[architecture/2026-04-02-logos-provider-adapter-profiles.md]]
- [[architecture/2026-04-02-logos-provider-openai-adapter-spec.md]]
- [[architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md]]
- [[architecture/2026-04-02-logos-provider-gemini-adapter-spec.md]]
- [[architecture/2026-04-02-logos-provider-anthropic-adapter-spec.md]]
- [[architecture/2026-04-02-logos-provider-zai-adapter-spec.md]]
- [[architecture/2026-04-02-logos-provider-lmstudio-adapter-spec.md]]
- [[architecture/2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[architecture/2026-04-02-logos-tool-loop-architecture-spec.md]]
- [[architecture/2026-04-02-logos-capability-model-architecture-spec.md]]
- [[architecture/2026-04-02-logos-approval-control-architecture-spec.md]]
- [[architecture/2026-04-02-logos-host-daemon-transport-spec.md]]
- [[architecture/2026-04-02-logos-runtime-contract-spec.md]]
- [[architecture/2026-04-02-logos-runtime-tier-contract-spec.md]]
- [[architecture/2026-04-02-logos-tier-0-semantic-surfaces-spec.md]]
- [[architecture/2026-04-02-logos-adapter-health-and-drift-policy.md]]
- [[architecture/2026-04-02-logos-minimal-survivable-agent-baseline-spec.md]]
- [[architecture/2026-04-02-logos-semantics-and-guarantees-matrix.md]]
- [[architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md]]
- [[implementation/2026-03-30-logos-global-roadmap.md]]
- [[implementation/2026-03-30-logos-phase-1-implementation-slice.md]]
- [[implementation/2026-04-02-phase-1-boundary-extraction-strict-spec.md]]
- [[implementation/2026-04-03-codex-core-seam-index.md]]
- [[implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md]]
- [[implementation/2026-04-02-openrouter-validation-adapter-strict-spec.md]]
- [[implementation/2026-04-02-codex-base-repository-strategy.md]]
- [[implementation/2026-04-02-upstream-sync-policy.md]]
- [[implementation/2026-04-02-branch-and-pr-workflow.md]]
- [[research/2026-03-30-logos-provider-research-report.md]]
- [[research/2026-04-02-opencode-goose-architecture-reference.md]]
- [[research/2026-03-30-logos-research-requirements.md]]
