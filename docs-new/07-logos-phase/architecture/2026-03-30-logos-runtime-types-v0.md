# Logos Runtime Types v0

## 1. Purpose

- This document defines the first practical Rust API surface for the adapter boundary.
- It is intentionally small and incomplete.
- It is not a provider-neutral grand theory; it is the minimum contract the session engine, provider adapters, and tool runtime need to share.

## 2. Design Constraints

- Preserve OpenAI/Responses fidelity through `logos-provider-openai`.
- Do not make raw OpenAI wire structs part of the shared runtime API.
- Allow provider-specific extensions without forcing fake equivalence.
- Make resume and tool-loop behavior explicit in the type system.

## 3. Proposed Module Layout

```rust
logos_runtime_types/
  src/
    lib.rs
    model.rs
    content.rs
    message.rs
    tools.rs
    events.rs
    continuation.rs
    capabilities.rs
    limits.rs
    errors.rs
```

## 4. v0 Rust API

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub type ProviderId = String;
pub type ModelId = String;
pub type CallId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRef {
    pub provider_id: ProviderId,
    pub model_id: ModelId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelMeta {
    pub model_ref: ModelRef,
    pub display_name: String,
    pub provider_label: String,
    pub context_window: Option<u64>,
    pub default_output_mode: OutputMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalTurnRequest {
    pub model_ref: ModelRef,
    pub system: Option<String>,
    pub messages: Vec<InternalMessage>,
    pub tools: Vec<InternalToolDef>,
    pub output_mode: OutputMode,
    pub reasoning_policy: ReasoningPolicy,
    pub continuation: Option<ContinuationHandle>,
    pub limits: TurnLimits,
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InternalMessage {
    User { parts: Vec<ContentPart> },
    Assistant { parts: Vec<ContentPart> },
    AssistantToolCall {
        call_id: CallId,
        name: String,
        arguments: Value,
    },
    ToolResult {
        call_id: CallId,
        output: Value,
        is_error: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentPart {
    Text { text: String },
    ImageRef { uri: String, mime_type: Option<String> },
    FileRef { uri: String, mime_type: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub execution: ToolExecutionKind,
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolExecutionKind {
    RuntimeOwned,
    ProviderNative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputMode {
    Text,
    Structured,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningPolicy {
    Off,
    Default,
    Low,
    Medium,
    High,
    ProviderDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuationHandle {
    OpenAiPreviousResponseId(String),
    ProviderSessionId(String),
    ClientManagedHistory,
    Opaque(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuationMode {
    ServerManaged,
    ClientManaged,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilitySet {
    pub supports_server_side_state: bool,
    pub supports_client_side_history: bool,
    pub supports_structured_output: bool,
    pub supports_context_cache: bool,
    pub supports_provider_builtin_tools: bool,
    pub supports_compaction: bool,
    pub continuation_mode: ContinuationMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapabilitySet {
    pub supports_tool_calling: bool,
    pub supports_parallel_tools: bool,
    pub supports_tool_arg_streaming: bool,
    pub supports_reasoning_deltas: bool,
    pub supports_opaque_reasoning_carryover: bool,
    pub supports_images: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnLimits {
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InternalEvent {
    MessageStarted {
        message_id: Option<String>,
    },
    TextDelta {
        text: String,
    },
    MessageCompleted {
        message_id: Option<String>,
    },
    ToolCallStarted {
        call_id: CallId,
        name: String,
    },
    ToolCallArgumentsDelta {
        call_id: CallId,
        delta: String,
    },
    ToolCallFinished {
        call_id: CallId,
    },
    ReasoningDelta {
        text: String,
    },
    Usage {
        input_tokens: u64,
        output_tokens: u64,
    },
    ContinuationUpdated {
        handle: ContinuationHandle,
    },
    ProviderExtension {
        key: String,
        payload: Value,
    },
    RawProviderEvent {
        provider_id: ProviderId,
        payload: Value,
    },
    Completed,
    Failed(ProviderError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderErrorKind {
    Authentication,
    RateLimited,
    ContextLimit,
    InvalidRequest,
    Transport,
    ProviderUnavailable,
    UnsupportedCapability,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderError {
    pub provider_id: ProviderId,
    pub kind: ProviderErrorKind,
    pub message: String,
    pub retryable: bool,
    pub metadata: BTreeMap<String, Value>,
}
```

## 5. Companion Provider Trait v0

```rust
use async_trait::async_trait;
use futures_core::Stream;
use std::pin::Pin;

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

## 6. Behavioral Notes

### Why `AssistantToolCall` Exists In `InternalMessage`

- Resume and persistence cannot rely only on streamed `InternalEvent` values.
- The system needs a durable conversational representation of tool calls that completed or partially completed during a turn.

### Why `ProviderExtension` And `RawProviderEvent` Both Exist

- `ProviderExtension` is the supported escape hatch for meaningful provider-specific features.
- `RawProviderEvent` exists for debugging, observability, and controlled compatibility work.
- Core control flow must not depend on `RawProviderEvent`.

### Why `ToolExecutionKind` Exists

- It keeps the portable runtime-owned tool path explicit.
- It also allows an adapter like OpenAI to expose provider-native built-in tools without pretending that all providers share them.

## 7. v0 Non-goals

- No persistence schema yet.
- No host/daemon wire protocol yet.
- No agent tree types yet.
- No baked-in approval model yet.
- No attempt to normalize every provider-specific reasoning artifact.

## 8. Acceptance Checks

- OpenAI adapter can map Responses request/response flow into these types without degrading Codex fidelity.
- OpenRouter-style adapter can operate with `ClientManagedHistory` and the same session loop.
- Tool loop can consume streamed tool calls and reinject `ToolResult` messages without provider wire types leaking into shared APIs.
- Resume design can be built on `InternalMessage` plus `ContinuationHandle` without requiring `ResponseItem` as a stored canonical record.
