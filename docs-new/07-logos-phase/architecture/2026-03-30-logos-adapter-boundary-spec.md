# Logos Adapter Boundary Spec

## 1. Start Here

- This is the working engineering spec for the adapter-first Codex/Logos integration path.
- It turns `docs-temp/arch_adapters.md` into a narrower implementation contract.
- It is intentionally not a "rewrite Codex into a universal LLM core" plan.

## 2. Intent + Non-goals

### Intent

- Preserve the strongest parts of Codex while creating a narrow Logos-owned runtime boundary.
- Make OpenAI/Responses the canonical upstream path without making raw Responses JSON the only internal truth.
- Enable additional providers through adapters without flattening provider-specific semantics into lies.

### Non-goals

- No direct rewrite of Codex core into a fully abstract provider-neutral universe.
- No mutation of Codex/OpenAI mechanics that would make upstream sync materially harder.
- No requirement that all providers expose identical reasoning, server-state, tool, or built-in capability semantics.

## 3. Locked Decisions

### Hard Rule 1

- We do not intervene in or degrade the OpenAI Responses path or core Codex mechanics just to achieve abstraction.
- The OpenAI/Codex path remains the highest-fidelity route for upstream compatibility, best OpenAI subscription value, and fastest access to new Codex capabilities.

### Hard Rule 2

- We take as much as possible from Codex for adapter architecture, tool runtime, and orchestration patterns, but never at the cost of erasing provider-specific capabilities or forcing other providers to pretend they are OpenAI.

### Consequences

- `Responses` is the canonical upstream protocol, not the universal internal ontology.
- Logos owns a small internal runtime contract.
- Provider crates are allowed to surface provider-specific extensions through explicit extension channels.
- Runtime-owned tools stay portable; provider-native tools remain optional fast paths.

## 4. Target Outcome

- Logos has a narrow runtime contract that the session engine and tool loop consume.
- OpenAI is integrated first through a canonical adapter that preserves Codex semantics.
- A second provider can be added without touching the core session loop in broad, cross-cutting ways.
- Session, resume, and tool execution stay stable even when provider state semantics differ.

## 5. Current Codex Modules That Stay As-Is

These are the modules to preserve as much as possible in phase 1.

### Keep As-Is or Near As-Is

- `codex-rs/core/src/tools/`:
  - tool registry
  - tool router
  - tool handlers
  - orchestration, approval, sandbox, retry flow
- `codex-rs/core/src/agent/`:
  - agent lifecycle patterns
  - spawn/wait control concepts
- `codex-rs/mcp-server/` and `codex-rs/rmcp-client/`:
  - MCP integration model
- `codex-rs/codex-client/`:
  - low-level HTTP transport and retry building blocks
- `codex-rs/lmstudio/` and `codex-rs/ollama/` bootstrap logic:
  - model discovery
  - health checks
  - local model bootstrap behavior

### Wrap, Do Not Rewrite In Phase 1

- `codex-rs/core/src/client.rs`
- `codex-rs/codex-api/`
- `codex-rs/core/src/model_provider_info.rs`

These should initially be wrapped by the OpenAI adapter path rather than surgically generalized.

### Do Not Treat As Stable Internal Truth

- `codex-rs/protocol/src/models.rs` `ResponseItem`
- `codex-rs/core/src/client_common.rs` `ToolSpec`
- `codex-rs/codex-api/src/endpoint/common.rs` `ResponsesApiRequest`
- Responses-specific SSE/WS event shapes

These remain valid wire/protocol types for the OpenAI adapter, but not the long-term Logos runtime contract.

## 6. Types That Become `logos-runtime-types`

This crate must stay small. It should contain only what the session engine and tool loop actually need.

### Required Types

- `InternalTurnRequest`
- `InternalMessage`
- `InternalToolDef`
- `InternalEvent`
- `ContinuationHandle`
- `CapabilitySet`
- `ModelRef`
- `ModelMeta`
- `OutputMode`
- `ReasoningPolicy`
- `TurnLimits`
- `ProviderError`

### Required Refinements

- `InternalMessage` must support explicit assistant tool calls, not only user, assistant, and tool result messages.
- `InternalEvent` must support stable ordering and lifecycle framing for streamed output.
- `CapabilitySet` should split:
  - provider-level capabilities
  - model-level capabilities

### Minimum Shape

```rust
pub struct InternalTurnRequest {
    pub model_ref: ModelRef,
    pub system: Option<String>,
    pub messages: Vec<InternalMessage>,
    pub tools: Vec<InternalToolDef>,
    pub output_mode: OutputMode,
    pub reasoning_policy: ReasoningPolicy,
    pub continuation: Option<ContinuationHandle>,
    pub limits: TurnLimits,
}

pub enum InternalMessage {
    User { parts: Vec<ContentPart> },
    Assistant { parts: Vec<ContentPart> },
    AssistantToolCall {
        call_id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        call_id: String,
        output: serde_json::Value,
    },
}

pub enum InternalEvent {
    MessageStarted { id: Option<String> },
    TextDelta { text: String },
    MessageCompleted { id: Option<String> },
    ToolCallStarted { call_id: String, name: String },
    ToolCallArgumentsDelta { call_id: String, delta: String },
    ToolCallFinished { call_id: String },
    ReasoningDelta { text: String },
    Usage { input_tokens: u64, output_tokens: u64 },
    ContinuationUpdated { handle: ContinuationHandle },
    ProviderExtension { key: String, payload: serde_json::Value },
    RawProviderEvent { provider: String, payload: serde_json::Value },
    Completed,
    Failed(ProviderError),
}
```

## 7. Adapters Required In Phase 1

Phase 1 is about proving the boundary, not about provider breadth.

### Mandatory

- `logos-provider-api`
  - declares the adapter trait
  - owns capability reporting contract
  - owns provider error contract
- `logos-provider-openai`
  - canonical adapter
  - wraps current Codex/OpenAI path
  - preserves `Responses`, `previous_response_id`, provider-native built-in tools, and OpenAI-specific extensions

### Strongly Recommended Next

- `logos-provider-openrouter`
  - thin variant over openai-like mapping where possible
  - hardcodes stateless continuation policy
  - forces the session engine to support `ClientManagedHistory`

### Explicitly Deferred

- `logos-provider-gemini`
- `logos-provider-zai`
- `logos-provider-anthropic`

These should come only after the boundary survives OpenAI plus one non-stateful adapter.

## 8. Provider API Contract

```rust
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn provider_capabilities(&self) -> ProviderCapabilitySet;

    fn model_capabilities(&self, model: &ModelRef) -> ModelCapabilitySet;

    async fn list_models(&self) -> Result<Vec<ModelMeta>, ProviderError>;

    async fn stream_turn(
        &self,
        req: InternalTurnRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = InternalEvent> + Send>>, ProviderError>;
}
```

### Notes

- `logos-provider-openai` may internally delegate to existing Codex client and protocol machinery.
- Other adapters may be native-first and use provider-specific HTTP/streaming code.
- The session engine must never assume server-side state unless capabilities and continuation mode allow it.

## 9. Invariants Required For Session, Resume, and Tool Loop

These are the non-negotiable invariants.

### Session Loop Invariants

- The session engine consumes `InternalEvent`, not raw provider wire events.
- A provider may emit extension payloads, but core control flow cannot depend on opaque provider-only shapes.
- Provider differences in server-side state must be expressed through `ContinuationHandle` and capability checks, not hidden branching spread across the engine.

### Resume Invariants

- Persisted session history must be reconstructible from `InternalMessage` and `ContinuationHandle`.
- Resume must work both for:
  - server-stateful providers like OpenAI
  - client-history providers like OpenRouter-style adapters
- If provider-native continuation becomes invalid, the engine must fall back to client-managed history when allowed by capability policy.

### Tool Loop Invariants

- Tool execution remains runtime-owned by default.
- Provider-native built-in tools are optional optimized paths and must not become the only execution path.
- Tool call execution must preserve stable call IDs across stream, dispatch, result injection, and persistence.
- Tool results re-enter the conversation as `InternalMessage::ToolResult`, not provider-specific output shapes.

### Ordering Invariants

- `ToolCallArgumentsDelta` events for a given `call_id` must be delivered in order.
- `Completed` or `Failed` is terminal for a turn stream.
- `ContinuationUpdated` must only publish handles safe to persist for the current provider policy.

## 10. Architecture Fit

- This path fits the existing daemon-first roadmap because it preserves the decision to take Codex as-is where it is strongest.
- It also fits the stated need to avoid making raw OpenAI semantics the architecture of the entire Chaemera host.
- It reduces the blast radius of provider expansion to adapter crates plus a narrow runtime contract.

## 11. Migration Phases

### Phase 1: Boundary Extraction

- Add `logos-runtime-types`.
- Add `logos-provider-api`.
- Add `logos-provider-openai` as a wrapper around current Codex/OpenAI logic.
- Do not change the underlying Responses mechanics beyond what is required to adapt event and message shapes.

### Phase 2: Tool Loop Decoupling

- Change the session engine/tool seam to operate on `InternalEvent` and `InternalMessage`.
- Preserve existing Codex tool runtime modules.
- Ensure persistence and resume no longer require raw `ResponseItem` outside the OpenAI adapter.

### Phase 3: Second Adapter Validation

- Add `logos-provider-openrouter`.
- Prove stateless continuation works without contaminating the OpenAI path.
- Lock fallback behavior for client-managed history.

### Phase 4: Native Adapters

- Add provider-native adapters where compatibility facades become insufficient.
- Start with providers whose semantics materially differ from Responses.

## 12. Requirement -> Task -> Test -> Gate

### Requirement 1

- Requirement: OpenAI/Codex path must remain upstream-friendly.
- Task: implement OpenAI through a wrapper adapter, not a core rewrite.
- Test: OpenAI adapter still supports existing Responses features and stateful continuation.
- Gate: no phase-1 task requires changing Codex wire/protocol semantics as a prerequisite.

### Requirement 2

- Requirement: new providers must not force false OpenAI semantics.
- Task: add `ContinuationHandle`, capabilities, and extension channels.
- Test: a stateless adapter can run without fake `previous_response_id` behavior.
- Gate: session engine works with both server-state and client-history modes.

### Requirement 3

- Requirement: tool runtime remains stable.
- Task: keep Codex tool runtime modules as-is and only change the model/tool boundary.
- Test: tool call dispatch and result injection still work after internal event normalization.
- Gate: adapters do not directly own tool execution except for explicit provider-native built-in tools.

## 13. Acceptance Criteria

- Two hard rules are documented and treated as non-negotiable.
- `logos-runtime-types` remains small and does not mirror all OpenAI wire types.
- `logos-provider-openai` exists as the canonical adapter path.
- A second adapter can be added without broad edits across session, persistence, and tool runtime code.
- Session, resume, and tool loop invariants are explicit before implementation starts.

## 14. Risks and Rollback

### Risks

- The internal contract becomes too thin and everything leaks through `ProviderExtension`.
- The internal contract becomes too broad and recreates a giant fake-neutral ontology.
- Persistence and resume remain secretly tied to `ResponseItem` semantics.
- Provider-native tools create hidden branching that bypasses runtime-owned tool safety.

### Rollback Strategy

- If boundary extraction starts forcing large Codex core surgery, stop and revert to OpenAI-only daemon integration for the current milestone.
- If adapter semantics destabilize session resume, keep provider support limited to OpenAI until persistence invariants are re-locked.

## 15. Immediate Next Artifacts

- Map current Codex modules to `keep`, `wrap`, `adapter-owned`, and `defer` buckets.
- Draft the first `logos-runtime-types` Rust file skeleton.
- Draft the `logos-provider-api` trait crate skeleton.
- Define the persistence/resume storage contract for `InternalMessage` plus `ContinuationHandle`.
