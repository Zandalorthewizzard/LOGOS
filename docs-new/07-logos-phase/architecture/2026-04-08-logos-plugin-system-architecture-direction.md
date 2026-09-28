---
id: logos-plugin-system-architecture-direction
title: LOGOS Plugin System Architecture Direction
type: spec
status: draft
tags: [architecture, spec, plugins, extensibility, future, logos]
related:
  [
    [README.md],
    [2026-04-08-logos-plugin-system-future-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
  ]
depends_on:
  [
    [2026-04-08-logos-plugin-system-future-discussion.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Plugin System Architecture Direction

## Status

- This is a future-facing architecture direction doc.
- It is not a strict implementation spec.
- It should guide later discussion once LOGOS core runtime phases are stable enough to support extension hosting.

## Core Decision

- LOGOS should target a plugin system around the preserved Codex kernel.
- The plugin host belongs to the LOGOS runtime and daemon layer, not to `codex-core`.
- Plugins extend runtime behavior, integrations, and policy surfaces without redefining canonical OpenAI/Codex transport semantics.

## Architectural Layering

### Layer 0: Preserved Codex Kernel

- `codex-core` and the canonical OpenAI path remain the reference execution kernel.
- Narrow seams may still exist, but plugin logic should not live here.

### Layer 1: LOGOS Runtime And Daemon

- The daemon owns session authority, execution journal, approvals, persistence, and plugin lifecycle.
- The host/UI remains a consumer of daemon commands and event streams.

### Layer 2: First-Party Core Plugins

- LOGOS-owned extensions shipped in-tree.
- These should prove the public plugin contracts before community exposure.

### Layer 3: Community Plugins

- Out-of-tree extensions using the same versioned contracts.
- This layer should come only after manifests, permissions, compatibility, and audit posture are mature.

## Plugin Families

The minimum future family set should stay small and explicit.

### 1. Provider Plugins

- add provider adapters or provider bridges
- may wrap external runtimes, ACP agents, or provider-native bridges
- must remain capability-declared and honesty-first

### 2. Tool Plugins

- expose runtime-owned tools, MCP-backed tools, and integration tool packs
- execute under runtime policy and approval control

### 3. Policy Plugins

- contribute approval, routing, or execution-policy advice
- runtime remains final authority for safety-critical decisions

### 4. Context Plugins

- contribute memory sources, retrieval surfaces, or project-context enrichment
- must not become canonical instruction truth

### 5. Observer Plugins

- telemetry sinks, audit exporters, logging bridges, external notifications
- read-only or append-only by default

## Plugin Boundary Model

### Public Boundary

- The public plugin contract should be protocol-based.
- Plugins should be loaded as external processes or equivalent isolated workers.

### Internal Bootstrap

- First-party plugins may temporarily be implemented in-process while contracts stabilize.
- Even then, they should conform to the same manifest and lifecycle model where practical.

### Explicit Non-Decision

- LOGOS should not promise stable Rust dynamic-library ABI plugins.
- `.dll` / `.so` loading may exist as an experiment later, but must not be the canonical public plugin story.

## Manifest Direction

Every plugin should eventually declare at least:

1. `id`
2. `displayName`
3. `version`
4. `pluginApiVersion`
5. `kind`
6. `entrypoint`
7. `runtimeCompatibility`
8. `requestedPermissions`
9. `declaredCapabilities`
10. `author`, `license`, and optional trust metadata

## Lifecycle Direction

The baseline lifecycle should be:

1. discover
2. validate manifest
3. negotiate plugin API version
4. resolve permissions and policy gates
5. activate
6. register capabilities and surfaces
7. serve requests or events
8. suspend, resume, or reload when needed
9. shutdown cleanly

## Runtime Authority Boundaries

### Runtime Must Keep Authority Over

1. canonical OpenAI continuation and turn semantics
2. durable journal ownership
3. approval state machine
4. canonical compaction and reinjection authority
5. final event ordering and delivery
6. trust policy for plugin execution

### Plugins May Contribute To

1. provider integration
2. tool execution surfaces
3. policy suggestions
4. context enrichment
5. observability and exports

### Plugins Must Not Do

1. silently bypass runtime approvals
2. silently mutate canonical journal truth
3. claim canonical parity by configuration alone
4. redefine preserved Codex/OpenAI turn semantics from outside the runtime

## Execution Model By Family

### Provider Plugins

- register one or more provider profiles
- expose capabilities explicitly
- run behind the same adapter-health and downgrade rules as in-tree adapters

### Tool Plugins

- present declared tools to the runtime
- receive tool calls only through runtime-owned dispatch
- return normalized outputs rather than provider-native transport frames

### Policy Plugins

- receive bounded policy context
- return policy advice or decisions according to declared authority level
- default to advisory mode unless explicitly promoted by runtime policy

### Context Plugins

- provide context items or retrieval results
- remain separate from canonical instruction layers

### Observer Plugins

- subscribe to selected event classes
- default to read-only posture

## Security And Trust Direction

The plugin host should eventually support:

1. per-plugin permissions
2. secret scoping separate from the main runtime
3. audit logging of activation and sensitive actions
4. explicit allowlist or trust policy for community plugins
5. optional signature or publisher verification later

## First-Party Core Plugin Rule

- First-party "core plugins" should be real plugins, not hidden privileged forks of the plugin system.
- They may receive temporary bootstrap exceptions, but the long-term goal is one extension model.
- Good likely first-party plugin candidates are:
  1. future non-canonical provider adapters
  2. MCP and ACP bridges
  3. telemetry and audit integrations
  4. context and retrieval modules

## Phase Placement

### Current Phase

- discussion only
- no implementation commitment

### Earliest Practical Start

- after phase 4 establishes durable runtime authority around persistence, resume, and compaction

### First Serious Use

- after phase 5 or phase 6, once LOGOS has both:
  1. a non-canonical adapter path
  2. a stable enough runtime authority surface to host extensions safely

### Community Exposure

- after post-MVP hardening, likely after the provider and persistence lanes are no longer shifting rapidly

## Non-goals

1. no plugin rewrite of `codex-core`
2. no marketplace design now
3. no stable Rust ABI promise
4. no assumption that every plugin type must exist in v1 of the host
5. no broad implementation authorization in current phases

## Open Questions

1. Should provider plugins and tool plugins share one transport protocol or use separate contracts?
2. Should policy plugins be advisory-only at first?
3. How much of plugin distribution should be repo-local versus registry-based?
4. Does LOGOS want one unified plugin manifest for daemon, host, and workflow extensions, or separate manifests by class?
5. Should WASM be a real near-term option or only a later sandboxing path?

## Evidence

- path: `2026-04-08-logos-plugin-system-future-discussion.md`
  symbol: `Preserved Design Answer`, `Phase Placement`
  lines: `36-127`
- path: `2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Core Decision`, `Daemon Owns`, `Required Transport Surfaces`
  lines: `34-123`
- path: `2026-04-02-logos-runtime-contract-spec.md`
  symbol: `Core Position`, `Logos Runtime Guarantees`
  lines: `38-128`
- path: `2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Core Decision`
  lines: `33-37`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Canonical Storage Boundaries`
  lines: `82-97`
- path: `../../../codex-rs/logos-provider-api/src/provider.rs`
  symbol: `ProviderSession`, `ProviderAdapter`
  lines: `23-50`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `RuntimeToolExecutor`, `ToolApprover`, `ToolRuntime`
  lines: `88-151`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `33-128`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `MVP Definition`, `Phase 4: Persistence And Resume`, `Phase 7: Native Provider Expansion`
  lines: `37-69`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 4: Persistence And Resume`, `Phase 7: Native Provider Expansion`
  lines: `151-227`
- path: `../research/2026-04-02-opencode-goose-architecture-reference.md`
  symbol: `Extension And Tool Boundary`, `ACP Boundary`, `Parity Decision For LOGOS`
  lines: `157-183`
- path: `../research/2026-04-02-opencode-goose-architecture-reference.md`
  symbol: `Parity Decision For LOGOS`
  lines: `261-299`

## Links

- [[README.md]]
- [[2026-04-08-logos-plugin-system-future-discussion.md]]
- [[2026-04-02-logos-host-daemon-transport-spec.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
- [[2026-04-02-logos-tool-loop-architecture-spec.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
