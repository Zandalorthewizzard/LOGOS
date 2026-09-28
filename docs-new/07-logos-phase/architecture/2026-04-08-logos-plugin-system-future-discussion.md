---
id: logos-plugin-system-future-discussion
title: LOGOS Plugin System Future Discussion
type: discussion
status: active
tags: [architecture, discussion, plugins, extensions, future, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-08-logos-plugin-system-architecture-direction.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [../research/2026-04-02-opencode-goose-architecture-reference.md],
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Plugin System Future Discussion

## Purpose

- Capture a future-facing plugin-system direction before it gets lost behind current phase work.
- Preserve the owner discussion from `2026-04-08` about using plugins to grow LOGOS without repeatedly invading Codex base.
- Lock the first design stance: plugin expansion must happen around the preserved Codex kernel, not by turning `codex-core` itself into a plugin host.

## Status And Scope

- This document is future-planning only.
- It does not authorize immediate implementation work in the current phase.
- It should be read as a design memory and discussion anchor for post-phase-4 or post-MVP extensibility planning.

## Captured Owner Direction

The owner proposed a future architecture where LOGOS can grow through:

1. first-party `core plugins`, meaning LOGOS-owned extensions that add capabilities above preserved Codex;
2. later community plugins, meaning third-party extensions that use the same published extension contracts;
3. a plugin strategy that reduces pressure to modify `codex-core` directly for every future feature.

That direction is consistent with current LOGOS philosophy as long as the plugin host is built around the preserved kernel rather than inside it.

## Preserved Design Answer

The current design answer from the `2026-04-08` discussion is:

1. LOGOS should not aim for a "plugin system for `codex-core`".
2. LOGOS should instead aim for a "plugin system around a preserved Codex kernel".
3. `codex-core` should remain the canonical OpenAI/Codex execution kernel.
4. LOGOS runtime should become the outer host that owns extension loading, extension policy, and extension-facing contracts.
5. Narrow evidence-backed seams into Codex base will still be needed occasionally, but a good plugin host should reduce how often that happens.

## Why This Fits The Existing Architecture

- LOGOS already treats OpenAI/Codex semantics as the canonical path instead of flattening them into fake-neutral abstractions.
- LOGOS already assigns tool runtime, approvals, orchestration, and persistence authority to the runtime/daemon layer instead of provider adapters.
- A plugin host naturally belongs in that runtime layer, because the daemon already owns session state, execution policy, and outer transport boundaries.

## Good Candidates For A Plugin Layer

These are good future plugin targets because they extend LOGOS behavior without needing to own canonical Codex transport semantics:

1. provider adapters and provider bridges that sit above the preserved kernel
2. tool packs, MCP bridges, ACP bridges, and other integration surfaces
3. approval and policy modules
4. event sinks, telemetry exporters, and audit integrations
5. context or memory sources
6. first-party workflow modules and future "core extensions" shipped by LOGOS itself

## Bad Candidates For A Plugin Layer

These are poor plugin targets because they are part of canonical runtime authority or safety-critical invariants:

1. canonical OpenAI transport and session semantics
2. continuation, websocket incrementality, and turn-state rules
3. canonical compaction truth and instruction reinjection authority
4. durable journal canonical schema while it is still being defined
5. any safety-critical path that would let a plugin silently bypass runtime approvals or execution policy

## Transport Stance

The safest future direction is not a stable Rust dynamic-library ABI.

The preferred order is:

1. versioned process-boundary plugins with an explicit protocol
2. optional WASM or WASI later if sandboxed embeddable plugins become valuable
3. no promise of a public `.dll` / `.so` Rust ABI as the main extension contract

This preserves sandboxing, versioning, and runtime authority better than native ABI plugins.

## First-Party Versus Community Plugins

### First-Party Core Plugins

- LOGOS should first prove the extension contracts with first-party plugins that ship in-tree.
- These should use the same manifest and lifecycle rules as public plugins where practical.
- This prevents the project from inventing a community surface that LOGOS itself does not trust enough to use.

### Community Plugins

- Community plugins should come later.
- They need:
  1. versioned manifests
  2. capability declarations
  3. permission requests
  4. compatibility policy
  5. lifecycle hooks
  6. trust and audit posture

Without those, a "marketplace" is just a new attack surface and maintenance burden.

## Phase Placement

The plugin-system direction should be treated as a post-core future direction.

Recommended placement:

1. discussion and architecture shaping can happen now
2. no implementation before the current core runtime phases stabilize
3. earliest plausible host work starts after persistence/resume authority is clearer in phase 4
4. first meaningful first-party pluginization is more realistic after phase 5 or phase 6, once LOGOS has at least one non-canonical provider path and one durable runtime path
5. community plugin support belongs after that, likely as post-MVP or later-phase work

## Product Value Hypothesis

If LOGOS succeeds, a plugin system could become one of the strongest ways to differentiate it:

1. preserved Codex-grade kernel quality
2. LOGOS-owned runtime authority
3. honest multi-provider and multi-tool extensibility
4. first-party and third-party growth without bloating `codex-core`

That is a much stronger long-term posture than "just another multi-provider CLI".

## Open Questions

1. Which plugin classes deserve first-class support first: provider, tool, policy, observer, or context?
2. Should provider plugins be public immediately, or remain first-party-only until adapter contracts harden further?
3. Should plugin authority always stay strictly daemon-side, or should host/UI extensions be a separate class?
4. When community plugins exist, what permission and trust model is acceptable by default?
5. How much of the future plugin protocol should reuse MCP, ACP, or a Logos-specific contract?

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Mission`, `MVP Definition`
  lines: `26-69`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 4: Persistence And Resume`, `Phase 7: Native Provider Expansion`
  lines: `151-227`
- path: `2026-04-02-logos-runtime-contract-spec.md`
  symbol: `Core Position`, `Adapter Architecture Boundary`
  lines: `38-104`
- path: `2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Core Decision`, `Daemon Owns`, `Transport Principles`
  lines: `34-123`
- path: `../../../codex-rs/logos-provider-api/src/provider.rs`
  symbol: `ProviderSession`, `ProviderAdapter`
  lines: `23-50`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `RuntimeToolExecutor`, `ToolApprover`, `ToolRuntime`
  lines: `88-151`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `33-128`
- path: `../research/2026-04-02-opencode-goose-architecture-reference.md`
  symbol: `Extension And Tool Boundary`, `Parity Decision For LOGOS`
  lines: `157-183`
- path: `../research/2026-04-02-opencode-goose-architecture-reference.md`
  symbol: `Parity Decision For LOGOS`
  lines: `261-299`

## Links

- [[README.md]]
- [[2026-04-02-logos-host-daemon-transport-spec.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
- [[2026-04-02-logos-tool-loop-architecture-spec.md]]
- [[2026-04-08-logos-plugin-system-architecture-direction.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
- [[../research/2026-04-02-opencode-goose-architecture-reference.md]]
