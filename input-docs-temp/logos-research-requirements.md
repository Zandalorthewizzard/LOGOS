# Logos 0.1 Research Requirements

## 1. Purpose

- This document is the tasking brief for a dedicated research agent.
- Its goal is to front-load discovery for Logos 0.1, adapter architecture, and provider support so later implementation work spends less time re-reading source and vendor docs.
- The output should be evidence-heavy, link-rich, and organized so it can directly influence architecture and implementation planning.

## 2. Research Objective

The research agent must build a complete context pack for:

1. Codex internals that matter to adapter extraction
2. Chaemera/Logos host architecture constraints
3. provider API semantics and incompatibilities
4. persistence, session, resume, and tool-loop implications
5. practical implementation risks for Logos 0.1

## 3. Non-goals

1. no code changes
2. no speculative architecture without evidence
3. no provider marketing summary without concrete API semantics
4. no documentation that paraphrases code without file references

## 4. Hard Rules For The Research

### Rule 1

- Do not recommend architecture that degrades or rewrites the OpenAI Responses/Codex path merely for abstraction purity.

### Rule 2

- Do not treat non-OpenAI providers as if they must be force-fit into OpenAI semantics when their native capabilities differ materially.

## 5. Required Research Deliverables

The research agent should produce one consolidated report or a small set of linked reports covering all sections below.

### Deliverable A: Codex Extraction Surface Audit

Must answer:

- Which Codex modules are safe to preserve intact?
- Which modules must be wrapped first?
- Which types are pure OpenAI wire types and must not leak into shared Logos runtime?
- Which modules affect session, resume, persistence, tool loop, approvals, MCP, and agent orchestration?

Required evidence format for code claims:

- `path`
- `symbol` when available
- `lines`
- short explanation of why the file matters

### Deliverable B: Logos Host Constraint Audit

Must answer:

- What parts of Chaemera already exist that should remain the host shell?
- What runtime, transport, IPC, or daemon boundaries already exist or are being planned?
- Which current docs already constrain the daemon-first architecture?

Required evidence:

- repo docs references with file paths and lines
- code references where host/runtime boundaries already exist

### Deliverable C: Provider Semantics Matrix

Must compare at minimum:

- OpenAI Responses
- OpenRouter Responses facade
- Anthropic Messages / tool use
- Gemini OpenAI compatibility and native APIs
- Z.AI OpenAI-compatible plus native capability docs
- Ollama Responses/OpenAI-compatible behavior
- LM Studio OpenAI-compatible behavior

For each provider or mode, capture:

- state model
- streaming model
- tool calling model
- argument streaming behavior
- structured output support
- reasoning/thinking semantics
- context caching or continuation semantics
- built-in/native tool support
- image/file input support
- error model and rate-limit semantics

Required source quality:

- official docs first
- SDK docs second
- source or issue threads only when official docs are insufficient

### Deliverable D: Resume / Persistence Research

Must answer:

- What exact data needs to be persisted for OpenAI stateful continuation?
- What fallback is required for stateless providers?
- What internal record model is sufficient for resume without storing raw provider wire types as canonical truth?
- Which Codex files currently own rollout/session persistence semantics?

### Deliverable E: Tool Loop Research

Must answer:

- Which parts of Codex tool runtime are reusable with minimal surgery?
- Where exactly do provider wire tool calls become executable runtime actions?
- Which tool concepts are portable and which are provider-native?
- What invariants around call IDs, ordering, and result injection are required?

### Deliverable F: Risks And Unknowns Register

Must explicitly mark:

- confirmed facts
- likely-but-unconfirmed assumptions
- unknowns that require prototyping
- decisions blocked on missing evidence

## 6. Required Context The Research Agent Must Read

### Existing Chaemera/Logos docs

- `docs-new/07-codex-logos-phase/README.md`
- `docs-new/07-codex-logos-phase/2026-03-29-codex-logos-daemon-first-roadmap.md`
- `docs-new/07-codex-logos-phase/2026-03-29-chat-ui-post-xml-surface-inventory.md`
- `docs-new/04-sprint-workflow/specs/2026-03-29-tauri-chat-runtime-phase-1-foundation-spec.md`
- `docs-temp/codex-model-agnostic-analysis.md`
- `docs-temp/arch_adapters.md`
- `docs-temp/logos-adapter-boundary-spec.md`
- `docs-temp/logos-crate-map.md`
- `docs-temp/logos-runtime-types-v0.md`
- `AGENTS.md`

### Codex source areas

- `C:\Work\codex-main\codex-rs\core\src\codex.rs`
- `C:\Work\codex-main\codex-rs\core\src\client.rs`
- `C:\Work\codex-main\codex-rs\core\src\client_common.rs`
- `C:\Work\codex-main\codex-rs\core\src\model_provider_info.rs`
- `C:\Work\codex-main\codex-rs\core\src\tools\`
- `C:\Work\codex-main\codex-rs\core\src\agent\`
- `C:\Work\codex-main\codex-rs\core\src\stream_events_utils.rs`
- `C:\Work\codex-main\codex-rs\core\src\message_history.rs`
- `C:\Work\codex-main\codex-rs\protocol\src\models.rs`
- `C:\Work\codex-main\codex-rs\protocol\src\protocol.rs`
- `C:\Work\codex-main\codex-rs\protocol\src\openai_models.rs`
- `C:\Work\codex-main\codex-rs\codex-api\`
- `C:\Work\codex-main\codex-rs\codex-client\`
- `C:\Work\codex-main\codex-rs\rmcp-client\`
- `C:\Work\codex-main\codex-rs\mcp-server\`
- `C:\Work\codex-main\codex-rs\lmstudio\`
- `C:\Work\codex-main\codex-rs\ollama\`
- `C:\Work\codex-main\codex-rs\core\models.json`
- `C:\Work\codex-main\codex-rs\core\config.schema.json`

### External provider docs to research

- OpenAI Responses API docs
- OpenAI Responses streaming docs
- OpenRouter Responses docs
- Anthropic Messages API docs
- Anthropic tool use docs
- Gemini OpenAI compatibility docs
- Gemini native interactions or stateful orchestration docs
- Z.AI OpenAI-compatible docs
- Z.AI native HTTP/tool/thinking docs
- Ollama OpenAI compatibility and Responses support docs
- LM Studio OpenAI compatibility docs

## 7. Required Output Structure

The research report should contain these sections in order:

1. Executive summary
2. Confirmed architecture facts
3. Codex extraction surface audit
4. Host/daemon constraint audit
5. Provider semantics matrix
6. Persistence/resume findings
7. Tool-loop findings
8. Recommended decisions for Logos 0.1
9. Unknowns and prototype targets
10. Appendix of evidence

## 8. Evidence Requirements

### For code-backed claims

Every substantial claim about repository code must include:

- `path: ...`
- `symbol: ...` when possible
- `lines: ...`

Example:

- path: `C:\Work\codex-main\codex-rs\core\src\model_provider_info.rs`
- symbol: `ModelProviderInfo`
- lines: `72-130`

### For external API claims

Every substantial claim about provider behavior must include:

- canonical doc URL
- exact feature area referenced
- whether the source is official, SDK, or secondary
- date accessed if the report format allows it

### Unknowns policy

- If the agent cannot verify a behavior, mark it `UNKNOWN`.
- If two sources conflict, record the conflict explicitly.
- Do not silently resolve ambiguity in favor of a cleaner architecture story.

## 9. Questions The Research Must Answer Explicitly

The report must explicitly answer yes/no or short-form answers for all of these:

1. Can Logos 0.1 ship with OpenAI-first plus one validation adapter without overcommitting the architecture?
2. Is OpenRouter still the best second adapter for boundary validation?
3. What exact state must be persisted to resume OpenAI turns faithfully?
4. Can tool calls be normalized without losing Codex tool-runtime strengths?
5. Which provider-native capabilities are too semantically different to normalize in v0?
6. Which existing Chaemera host surfaces reduce implementation scope immediately?
7. Which areas are most likely to cause architecture regret if not researched first?

## 10. Preferred Research Depth

- Be exhaustive for Codex boundary, persistence, tool loop, and OpenAI/OpenRouter semantics.
- Be medium-to-deep for Anthropic, Gemini, Z.AI, Ollama, and LM Studio.
- Prefer completeness over speed; this research exists to reduce later discovery cost.

## 11. Suggested Final Artifacts

At the end of the research, the agent should ideally leave behind:

- one primary research report
- one provider comparison matrix table
- one Codex module evidence appendix
- one unknowns/prototype checklist

## 12. Success Criteria

- A future implementation agent can begin Logos 0.1 adapter work with materially less discovery.
- The report clearly separates confirmed facts from assumptions.
- Architecture decisions can reference explicit evidence instead of recollection.
- The research is deep enough to change the plan if the facts demand it.
