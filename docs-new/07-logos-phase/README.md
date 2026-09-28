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
    [analysis/2026-04-18-implemented-items-parity-scope-analysis.md],
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
    [architecture/2026-04-03-logos-compaction-strategy-discussion.md],
    [architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md],
    [architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md],
    [architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md],
    [architecture/2026-04-11-phase-4-persistence-resume-discussion.md],
    [architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [architecture/2026-04-11-logos-structured-output-capability-discussion.md],
    [architecture/2026-04-11-code-main-fork-phase-6-triage-discussion.md],
    [architecture/2026-04-11-logos-philosophy-and-invariants.md],
    [
      architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [architecture/2026-04-17-session-state-foundation-sprint-discussion.md],
    [architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
     [architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md],
     [architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md],
     [architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md],
     [architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md],
      [architecture/2026-04-18-item-07-archive-metadata-migration-discussion.md],
     [architecture/2026-04-16-item-11-background-terminals-draft-implementation-discussion.md],
    [architecture/2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
    [
      architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md,
    ],
    [architecture/2026-04-14-openrouter-thread-resume-support-discussion.md],
    [architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md],
    [
      architecture/2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md,
    ],
    [architecture/2026-04-14-openrouter-thread-archive-support-discussion.md],
    [architecture/2026-04-14-openrouter-thread-unarchive-support-discussion.md],
    [architecture/2026-04-14-openrouter-thread-rollback-discussion.md],
    [architecture/2026-04-14-openrouter-thread-compaction-support-discussion.md],
    [architecture/2026-04-14-openrouter-turn-interrupt-discussion.md],
    [architecture/2026-04-14-openrouter-turn-steering-discussion.md],
    [architecture/2026-04-14-openrouter-realtime-conversation-discussion.md],
    [architecture/2026-04-11-local-validation-post-mvp-milestone-discussion.md],
    [architecture/2026-04-11-glm-zai-post-mvp-milestone-discussion.md],
    [architecture/2026-04-11-gemini-post-mvp-milestone-discussion.md],
    [architecture/2026-04-02-logos-tool-loop-architecture-spec.md],
    [architecture/2026-04-02-logos-capability-model-architecture-spec.md],
    [architecture/2026-04-02-logos-approval-control-architecture-spec.md],
    [architecture/2026-04-02-logos-host-daemon-transport-spec.md],
    [architecture/2026-04-08-logos-plugin-system-future-discussion.md],
    [architecture/2026-04-08-logos-plugin-system-architecture-direction.md],
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
    [
      implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md,
    ],
    [
      implementation/2026-04-03-phase-2-openai-canonical-turn-loop-strict-spec.md,
    ],
    [
      implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md,
    ],
    [implementation/2026-04-08-phase-3-tool-loop-boundary-strict-spec.md],
    [implementation/2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md],
    [implementation/2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md],
    [implementation/2026-04-11-phase-4-persistence-resume-strict-spec.md],
    [implementation/2026-04-11-phase-4-persistence-resume-acceptance-report.md],
    [implementation/2026-04-02-openrouter-validation-adapter-strict-spec.md],
    [
      implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md,
    ],
    [
      implementation/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md,
    ],
    [implementation/2026-04-14-phase-6-session-state-layer-extraction-strict-spec.md],
    [implementation/2026-04-16-item-01-implementation-reference.md],
    [implementation/2026-04-17-item-02-implementation-reference.md],
    [implementation/2026-04-17-item-03-implementation-reference.md],
    [implementation/2026-04-17-item-04-implementation-reference.md],
    [implementation/2026-04-18-item-05-implementation-reference.md],
    [implementation/2026-04-18-item-05-shell-execution-strict-spec.md],
    [implementation/2026-04-17-session-state-foundation-sprint-strict-spec.md],
    [implementation/2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md],
    [implementation/2026-04-17-session-state-foundation-task-02-session-persistence-strict-spec.md],
    [implementation/2026-04-17-session-state-foundation-task-03-turn-lifecycle-strict-spec.md],
    [implementation/2026-04-17-session-state-foundation-task-04-fork-resume-strict-spec.md],
    [implementation/2026-04-17-session-state-foundation-slice-implementation-report-template.md],
    [implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md],
    [implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md],
    [implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md],
    [implementation/2026-04-18-session-state-foundation-task-03-implementation-report.md],
     [implementation/2026-04-18-session-state-foundation-task-04-implementation-report.md],
     [implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md],
      [implementation/2026-04-18-item-07-archive-metadata-migration-strict-spec.md],
      [implementation/2026-04-18-item-07-archive-metadata-migration-implementation-report.md],
     [implementation/2026-04-17-item-11-background-terminals-strict-spec.md],
    [implementation/2026-04-02-codex-base-repository-strategy.md],
    [implementation/2026-04-02-upstream-sync-policy.md],
    [implementation/2026-04-02-branch-and-pr-workflow.md],
    [research/2026-03-30-logos-research-requirements.md],
    [research/2026-03-30-logos-provider-research-report.md],
    [research/2026-04-02-opencode-goose-architecture-reference.md],
    [research/2026-04-16-session-layer-research-transfer-ledger.md],
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
2. [[analysis/2026-04-18-implemented-items-parity-scope-analysis.md]]
3. [[architecture/2026-03-30-adapter-first-architecture-discussion.md]]
4. [[architecture/2026-03-30-logos-adapter-boundary-spec.md]]
5. [[architecture/2026-03-30-logos-crate-map.md]]
6. [[architecture/2026-03-30-logos-runtime-types-v0.md]]
7. [[architecture/2026-04-02-logos-adapter-project-docs-spec.md]]
8. [[architecture/2026-04-02-logos-adapter-kernel-architecture.md]]
9. [[architecture/2026-04-02-logos-provider-adapter-profiles.md]]
9. [[architecture/2026-04-02-logos-provider-openai-adapter-spec.md]]
10. [[architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md]]
11. [[architecture/2026-04-02-logos-provider-gemini-adapter-spec.md]]
12. [[architecture/2026-04-02-logos-provider-anthropic-adapter-spec.md]]
13. [[architecture/2026-04-02-logos-provider-zai-adapter-spec.md]]
14. [[architecture/2026-04-02-logos-provider-lmstudio-adapter-spec.md]]
15. [[architecture/2026-04-02-logos-persistence-resume-architecture-spec.md]]
16. [[architecture/2026-04-03-logos-compaction-strategy-discussion.md]]
17. [[architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md]]
18. [[architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md]]
19. [[architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md]]
20. [[architecture/2026-04-11-phase-4-persistence-resume-discussion.md]]
21. [[architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md]]
22. [[architecture/2026-04-11-logos-structured-output-capability-discussion.md]]
23. [[architecture/2026-04-11-code-main-fork-phase-6-triage-discussion.md]]
24. [[architecture/2026-04-11-logos-philosophy-and-invariants.md]]
25. [[architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
26. [[architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md]]
27. [[architecture/2026-04-17-session-state-foundation-sprint-discussion.md]]
28. [[architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md]]
29. [[architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md]]
30. [[architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md]]
31. [[architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md]]
32. [[architecture/2026-04-18-item-07-archive-metadata-migration-discussion.md]]
33. [[architecture/2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]]
34. [[architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
35. [[architecture/2026-04-11-local-validation-post-mvp-milestone-discussion.md]]
36. [[architecture/2026-04-11-glm-zai-post-mvp-milestone-discussion.md]]
37. [[architecture/2026-04-11-gemini-post-mvp-milestone-discussion.md]]
33. [[architecture/2026-04-02-logos-tool-loop-architecture-spec.md]]
34. [[architecture/2026-04-02-logos-capability-model-architecture-spec.md]]
35. [[architecture/2026-04-02-logos-approval-control-architecture-spec.md]]
36. [[architecture/2026-04-02-logos-host-daemon-transport-spec.md]]
37. [[architecture/2026-04-08-logos-plugin-system-future-discussion.md]]
38. [[architecture/2026-04-08-logos-plugin-system-architecture-direction.md]]
39. [[architecture/2026-04-02-logos-runtime-contract-spec.md]]
40. [[architecture/2026-04-02-logos-runtime-tier-contract-spec.md]]
41. [[architecture/2026-04-02-logos-tier-0-semantic-surfaces-spec.md]]
42. [[architecture/2026-04-02-logos-adapter-health-and-drift-policy.md]]
43. [[architecture/2026-04-02-logos-minimal-survivable-agent-baseline-spec.md]]
44. [[architecture/2026-04-02-logos-semantics-and-guarantees-matrix.md]]
45. [[architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md]]
46. [[research/2026-03-30-logos-provider-research-report.md]]
47. [[research/2026-04-02-opencode-goose-architecture-reference.md]]
48. [[research/2026-04-16-session-layer-research-transfer-ledger.md]]
49. [[implementation/2026-04-02-codex-base-repository-strategy.md]]
50. [[implementation/2026-04-02-upstream-sync-policy.md]]
51. [[implementation/2026-04-02-branch-and-pr-workflow.md]]
52. [[implementation/2026-03-30-logos-global-roadmap.md]]
53. [[implementation/2026-03-30-logos-phase-1-implementation-slice.md]]
54. [[implementation/2026-04-02-phase-1-boundary-extraction-strict-spec.md]]
55. [[implementation/2026-04-03-codex-core-seam-index.md]]
56. [[implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md]]
57. [[implementation/2026-04-03-phase-2-openai-canonical-turn-loop-strict-spec.md]]
58. [[implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md]]
59. [[implementation/2026-04-08-phase-3-tool-loop-boundary-strict-spec.md]]
60. [[implementation/2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md]]
61. [[implementation/2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md]]
62. [[implementation/2026-04-11-phase-4-persistence-resume-strict-spec.md]]
63. [[implementation/2026-04-11-phase-4-persistence-resume-acceptance-report.md]]
64. [[implementation/2026-04-02-openrouter-validation-adapter-strict-spec.md]]
65. [[implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md]]
66. [[implementation/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md]]
67. [[implementation/2026-04-14-phase-6-session-state-layer-extraction-strict-spec.md]]
68. [[implementation/2026-04-16-item-01-implementation-reference.md]]
69. [[implementation/2026-04-17-item-02-implementation-reference.md]]
70. [[implementation/2026-04-17-item-03-implementation-reference.md]]
71. [[implementation/2026-04-17-item-04-implementation-reference.md]]
72. [[implementation/2026-04-17-session-state-foundation-sprint-strict-spec.md]]
73. [[implementation/2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md]]
74. [[implementation/2026-04-17-session-state-foundation-task-02-session-persistence-strict-spec.md]]
75. [[implementation/2026-04-17-session-state-foundation-task-03-turn-lifecycle-strict-spec.md]]
76. [[implementation/2026-04-17-session-state-foundation-task-04-fork-resume-strict-spec.md]]
77. [[implementation/2026-04-17-session-state-foundation-slice-implementation-report-template.md]]
78. [[implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md]]
79. [[implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md]]
80. [[implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md]]
81. [[implementation/2026-04-18-session-state-foundation-task-03-implementation-report.md]]
82. [[implementation/2026-04-18-session-state-foundation-task-04-implementation-report.md]]
83. [[implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md]]
84. [[implementation/2026-04-18-item-07-archive-metadata-migration-strict-spec.md]]
85. [[implementation/2026-04-18-item-07-archive-metadata-migration-implementation-report.md]]
86. [[research/2026-03-30-logos-research-requirements.md]]

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

## OpenRouter Phase-6 Control Surfaces

The remaining OpenRouter phase-6 support questions now live as separate discussion docs instead of one blended TODO list.

Ownership baseline for all of these docs:

1. [[architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
2. [[architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md]]

1. [[architecture/2026-04-14-openrouter-thread-resume-support-discussion.md]]
2. [[architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md]]
3. [[architecture/2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md]]
4. [[architecture/2026-04-18-item-07-archive-metadata-migration-discussion.md]]
5. [[architecture/2026-04-14-openrouter-thread-archive-support-discussion.md]]
6. [[architecture/2026-04-14-openrouter-thread-unarchive-support-discussion.md]]
7. [[architecture/2026-04-14-openrouter-thread-rollback-discussion.md]]
8. [[architecture/2026-04-14-openrouter-thread-compaction-support-discussion.md]]
9. [[architecture/2026-04-14-openrouter-turn-interrupt-discussion.md]]
10. [[architecture/2026-04-14-openrouter-turn-steering-discussion.md]]
10. [[architecture/2026-04-14-openrouter-realtime-conversation-discussion.md]]

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
- [[architecture/2026-04-03-logos-compaction-strategy-discussion.md]]
- [[architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md]]
- [[architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md]]
- [[architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md]]
- [[architecture/2026-04-11-phase-4-persistence-resume-discussion.md]]
- [[architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md]]
- [[architecture/2026-04-11-logos-structured-output-capability-discussion.md]]
- [[architecture/2026-04-11-code-main-fork-phase-6-triage-discussion.md]]
- [[architecture/2026-04-11-logos-philosophy-and-invariants.md]]
- [[architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[architecture/2026-04-17-session-state-foundation-sprint-discussion.md]]
- [[architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md]]
- [[architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md]]
- [[architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md]]
- [[architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md]]
- [[architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md]]
- [[architecture/2026-04-16-item-11-background-terminals-draft-implementation-discussion.md]]
- [[implementation/2026-04-17-item-11-background-terminals-strict-spec.md]]
- [[architecture/2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]]
- [[architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
- [[architecture/2026-04-14-openrouter-thread-resume-support-discussion.md]]
- [[architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md]]
- [[architecture/2026-04-14-openrouter-thread-background-terminals-cleanup-discussion.md]]
- [[architecture/2026-04-14-openrouter-thread-archive-support-discussion.md]]
- [[architecture/2026-04-14-openrouter-thread-unarchive-support-discussion.md]]
- [[architecture/2026-04-14-openrouter-thread-rollback-discussion.md]]
- [[architecture/2026-04-14-openrouter-thread-compaction-support-discussion.md]]
- [[architecture/2026-04-14-openrouter-turn-interrupt-discussion.md]]
- [[architecture/2026-04-14-openrouter-turn-steering-discussion.md]]
- [[architecture/2026-04-14-openrouter-realtime-conversation-discussion.md]]
- [[architecture/2026-04-11-local-validation-post-mvp-milestone-discussion.md]]
- [[architecture/2026-04-11-glm-zai-post-mvp-milestone-discussion.md]]
- [[architecture/2026-04-11-gemini-post-mvp-milestone-discussion.md]]
- [[architecture/2026-04-02-logos-tool-loop-architecture-spec.md]]
- [[architecture/2026-04-02-logos-capability-model-architecture-spec.md]]
- [[architecture/2026-04-02-logos-approval-control-architecture-spec.md]]
- [[architecture/2026-04-02-logos-host-daemon-transport-spec.md]]
- [[architecture/2026-04-08-logos-plugin-system-future-discussion.md]]
- [[architecture/2026-04-08-logos-plugin-system-architecture-direction.md]]
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
- [[implementation/2026-04-03-phase-2-openai-canonical-turn-loop-strict-spec.md]]
- [[implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md]]
- [[implementation/2026-04-08-phase-3-tool-loop-boundary-strict-spec.md]]
- [[implementation/2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md]]
- [[implementation/2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md]]
- [[implementation/2026-04-11-phase-4-persistence-resume-strict-spec.md]]
- [[implementation/2026-04-11-phase-4-persistence-resume-acceptance-report.md]]
- [[implementation/2026-04-02-openrouter-validation-adapter-strict-spec.md]]
- [[implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md]]
- [[implementation/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md]]
- [[implementation/2026-04-14-phase-6-session-state-layer-extraction-strict-spec.md]]
- [[implementation/2026-04-16-item-01-implementation-reference.md]]
- [[implementation/2026-04-17-item-02-implementation-reference.md]]
- [[implementation/2026-04-17-item-03-implementation-reference.md]]
- [[implementation/2026-04-17-item-04-implementation-reference.md]]
- [[implementation/2026-04-18-item-05-implementation-reference.md]]
- [[implementation/2026-04-18-item-05-shell-execution-strict-spec.md]]
- [[implementation/2026-04-17-session-state-foundation-sprint-strict-spec.md]]
- [[implementation/2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md]]
- [[implementation/2026-04-17-session-state-foundation-task-02-session-persistence-strict-spec.md]]
- [[implementation/2026-04-17-session-state-foundation-task-03-turn-lifecycle-strict-spec.md]]
- [[implementation/2026-04-17-session-state-foundation-task-04-fork-resume-strict-spec.md]]
- [[implementation/2026-04-17-session-state-foundation-slice-implementation-report-template.md]]
- [[implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md]]
- [[implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md]]
- [[implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md]]
- [[implementation/2026-04-18-session-state-foundation-task-03-implementation-report.md]]
- [[implementation/2026-04-18-session-state-foundation-task-04-implementation-report.md]]
- [[implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md]]
- [[implementation/2026-04-17-item-11-background-terminals-strict-spec.md]]
- [[implementation/2026-04-02-codex-base-repository-strategy.md]]
- [[implementation/2026-04-02-upstream-sync-policy.md]]
- [[implementation/2026-04-02-branch-and-pr-workflow.md]]
- [[research/2026-03-30-logos-provider-research-report.md]]
- [[research/2026-04-02-opencode-goose-architecture-reference.md]]
- [[research/2026-04-16-session-layer-research-transfer-ledger.md]]
- [[architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md]]
- [[research/2026-03-30-logos-research-requirements.md]]
