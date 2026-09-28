# RA-1.6 — Review Orchestration

## LOGOS Current Implementation

### OR Path

**Where:** `codex-rs/app-server/src/openrouter_beta_threads.rs:1180-1769`

Flow:

1. `start_review()` — inline or detached mode
2. Resolve review target (uncommitted changes, branch diff, commit, custom)
3. For inline: run review turn on the same thread through SessionEngine
4. For detached: fork thread → run review on fork → report back
5. Review prompt generated from target
6. Review output parsed from model response

### Codex Path

**Where:** `codex-rs/core/src/tasks/review.rs`

Flow:

1. `Op::Review` → `spawn_review_thread()`
2. Creates sub-agent CodexThread with review-specific config (no web_search, no spawn, approval=Never)
3. Sub-agent runs as `SessionSource::SubAgent(SubAgentSource::Review)`
4. Own SQ/EQ pair, independent model client
5. Events forwarded to parent thread
6. Review output parsed from JSON in last agent message
7. `ExitedReviewMode` event with findings, correctness, explanation, confidence

**Known weaknesses:**

- OR review re-implements review logic through SessionEngine
- Codex review spawns a full CodexThread — heavyweight
- Neither shares review prompt generation

## OpenCode Analysis

### No Dedicated Review Feature

OpenCode does not have a built-in code review feature. There is no `review/start` command or equivalent.

**What exists instead:**

- Agents can be configured with different modes and permissions
- The `plan` agent provides read-only analysis
- Users can prompt the model to review code through normal conversation
- Subtask mechanism (`subtask` part type) allows delegating analysis to a sub-agent

### Subtask System

**File:** `src/session/message-v2.ts` (subtask part type)

Parts can be of type `subtask` with fields: prompt, description, agent, model. This is a generic sub-agent mechanism that could be used for review-like tasks.

### Comparison

| Aspect                 | LOGOS (OR)          | LOGOS (Codex)                  | OpenCode          |
| ---------------------- | ------------------- | ------------------------------ | ----------------- |
| Dedicated review       | Yes                 | Yes                            | No                |
| Sub-agent model        | SessionEngine turn  | Separate CodexThread           | Subtask part type |
| Inline review          | Yes (same thread)   | Yes (same CodexThread)         | N/A               |
| Detached review        | Yes (fork + review) | Yes (fork + CodexThread)       | N/A               |
| Review output parsing  | Custom              | JSON parsing                   | N/A               |
| Review-specific config | Manual prompt       | Disabled tools, approval=Never | N/A               |

### Reusable Patterns

1. **No dedicated review feature**: OpenCode's approach — use normal conversation + agent configuration — is simpler. Review is just a prompt, not a separate system. LOGOS could simplify by making review a specialized agent rather than a separate operation.

2. **Subtask as generic sub-agent**: The `subtask` part type is a generic way to delegate work to a sub-agent. This could replace the specialized review thread pattern.

## Goose Analysis

### Review / Permission System

Goose has a **multi-layered tool inspection and permission system** rather than a dedicated review feature.

**Key files:**

- `crates/goose/src/tool_inspection.rs` — ToolInspector trait, ToolInspectionManager, InspectionResult
- `crates/goose/src/tool_monitor.rs` — RepetitionInspector (doom loop detection)
- `crates/goose/src/action_required_manager.rs` — User action request/response with oneshot channels
- `crates/goose/src/providers/base.rs:498-501, 770-784` — PermissionRouting enum, permission_confirmation
- `crates/goose/src/permission/` — **MISSING** (PermissionInspector, PermissionJudge)

### Tool Inspector Framework

**`ToolInspector` trait** (`tool_inspection.rs:34-54`):

```rust
trait ToolInspector: Send + Sync {
    fn name(&self) -> &'static str;
    async fn inspect(&self, session_id, tool_requests, messages, goose_mode) -> Vec<InspectionResult>;
    fn is_enabled(&self) -> bool { true }
    fn as_any(&self) -> &dyn Any;
}
```

**`ToolInspectionManager`** coordinates multiple inspectors:

- Runs all inspectors sequentially
- Aggregates results
- `apply_inspection_results_to_permissions()` merges into PermissionCheckResult
- Inspector priority: Deny > RequireApproval > Allow

**`InspectionAction`**:

- `Allow` — proceed without user intervention
- `Deny` — block execution entirely
- `RequireApproval(Option<String>)` — ask user, optional warning message

### Permission Routing

**`PermissionRouting`** (`base.rs:498-501`):

- `ActionRequired` — provider handles permission UI (e.g., ACP protocol)
- `Noop` — goose handles permissions internally

**`PermissionCheckResult`** (from missing `permission/` module):

```rust
struct PermissionCheckResult {
    approved: Vec<ToolRequest>,
    needs_approval: Vec<ToolRequest>,
    denied: Vec<ToolRequest>,
}
```

### User Action Flow

**`ActionRequiredManager`** (`action_required_manager.rs`):

1. Tool inspection determines `RequireApproval`
2. `request_and_wait(message, schema, timeout)` creates pending request
3. `MessageContent::ActionRequired` sent with elicitation ID
4. ACP server forwards to client as permission request
5. Client responds (AllowAlways, AllowOnce, RejectOnce, RejectAlways)
6. Response matched by request ID via oneshot channel
7. Timeout if no response within duration

### Session Name Review (Auto-Generated)

`maybe_update_name()` (`session_manager.rs:340-363`): Uses the LLM to generate session names. This is a form of "review" — the model reviews the conversation and produces a description. Uses fast model for cost efficiency.

### Comparison

| Aspect            | Goose                                               |
| ----------------- | --------------------------------------------------- |
| Dedicated review  | No (no separate review thread/operation)            |
| Permission system | Pluggable ToolInspector framework                   |
| Inspector types   | RepetitionInspector + PermissionInspector (missing) |
| User action flow  | ActionRequiredManager with oneshot + timeout        |
| Provider routing  | PermissionRouting (provider-managed vs internal)    |
| Finding IDs       | Structured (REP-001 for repetition, etc.)           |
| Confidence scores | Per-inspection (0.0-1.0)                            |

## Cross-Project Comparison Matrix

| Aspect          | LOGOS (OR)          | LOGOS (Codex)        | OpenCode                | Goose                             |
| --------------- | ------------------- | -------------------- | ----------------------- | --------------------------------- |
| Review feature  | Yes                 | Yes                  | No (use normal prompts) | No (no dedicated review)          |
| Permission sys  | Elicitation counter | Per-command approval | Ruleset-based           | Pluggable ToolInspector framework |
| Sub-agent model | SessionEngine turn  | Separate CodexThread | Subtask parts           | No (inferred from agents/ module) |
| Inline/detached | Both                | Both                 | N/A                     | N/A                               |
| Finding IDs     | None                | None                 | None                    | Structured (REP-001, etc.)        |
| Confidence      | None                | None                 | None                    | Per-inspection (0.0-1.0)          |

## Recommendations for LOGOS

### What to Borrow

1. **Simplify review to agent-level**: Instead of a dedicated review system, make review a specialized agent configuration. Reduces code, shares infrastructure.

2. **Generic sub-agent mechanism**: Consider a subtask/sub-agent pattern like OpenCode's that can be used for review and other delegated tasks.

### What to Avoid

1. **Don't keep two separate review implementations**: The OR and Codex review paths are completely different code. The shared layer should have one review orchestration that works for both.

### Risks and Open Questions

1. **Review output format**: Codex expects structured JSON output from review. If review becomes "just a prompt," how to ensure consistent output format?

2. **Review isolation**: Review should not modify the session state. Inline review on the same thread risks polluting history. Detached review (fork) is safer.
