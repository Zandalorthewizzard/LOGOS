# RA-3.1 — Turn Steering (Mid-turn Input Injection)

## LOGOS Current Implementation

### Codex Path

**Where:** `codex-rs/core/src/codex.rs:752`

`CodexThread::steer_input()`:

- Places user input into `pending_input` queue
- On next iteration of sampling loop (`codex.rs:5948`), merges with primary input
- Non-steerable turns: Review and Compact reject steering
- Works because CodexThread owns the iterative sampling loop

### OR Path

**Where:** Rejected — "Turn steering is not yet supported for OpenRouter beta threads."

**Why extremely complex:**

1. OR turn runs in spawned tokio task making HTTP request to provider
2. To inject input mid-turn: abort current request → save partial → merge new input → new request
3. OpenRouter may not support clean partial cancellation
4. If tool calls already returned and executing — state contradiction risk

## OpenCode Analysis

### No Turn Steering

OpenCode does not implement mid-turn input injection. There is no `turn/steer` equivalent.

**What exists instead:**

- Users can submit new messages, which queue after current turn completes
- The `Runner` pattern prevents concurrent turns but allows sequential queuing
- No mechanism to modify an in-flight LLM request

### Why OpenCode Doesn't Need It

OpenCode's architecture makes steering less necessary:

1. Each turn is a single `streamText()` call — no iterative sampling loop
2. Tool calls are executed within the same stream, not as separate requests
3. The AI SDK handles tool call loops internally
4. There's no server-side state to maintain — each request is independent

### Comparison

| Aspect                   | LOGOS (Codex)             | LOGOS (OR) | OpenCode                 |
| ------------------------ | ------------------------- | ---------- | ------------------------ |
| Turn steering            | Yes (pending_input queue) | No         | No                       |
| Architecture requirement | Iterative sampling loop   | N/A        | Single streamText() call |
| Non-steerable turns      | Review, Compact           | N/A        | N/A                      |
| Fallback                 | None                      | N/A        | Queue next message       |

## Goose Analysis

### Turn Steering

**Not implemented.** Goose has no mid-turn steering or input injection mechanism.

**Evidence:**

- `SessionConfig { id, schedule_id, max_turns, retry_config }` — `max_turns` field exists but is always set to `None` in all observed usage (`scheduler.rs:876`, ACP server)
- `GOOSE_MAX_TURNS` env var referenced only in telemetry (`posthog.rs:420`), not enforced
- No `pending_input` queue or equivalent
- No mid-turn message injection API visible

The agent loop (`Agent::reply()`) takes a single user message and streams responses. No mechanism exists to inject additional input during the stream.

### Why Goose Doesn't Need It

Same reasoning as OpenCode:

1. Each `Agent::reply()` call processes one user message through to completion
2. Tool calls are handled within the agent loop internally
3. No iterative sampling loop visible that could accept mid-turn input
4. Cancellation exists (`CancellationToken`) but no re-injection

### Comparison

| Aspect             | Goose                                     |
| ------------------ | ----------------------------------------- |
| Turn steering      | No                                        |
| max_turns field    | Declared in SessionConfig, never enforced |
| Mid-turn injection | No                                        |
| Architecture       | Single reply() → Stream<AgentEvent>       |

## Cross-Project Comparison Matrix

| Aspect             | LOGOS (Codex)       | LOGOS (OR) | OpenCode      | Goose                              |
| ------------------ | ------------------- | ---------- | ------------- | ---------------------------------- |
| Turn steering      | Yes                 | No         | No            | No (max_turns declared but unused) |
| Mid-turn injection | pending_input queue | N/A        | N/A           | N/A                                |
| Fallback           | None                | N/A        | Queue message | N/A                                |

## Recommendations for LOGOS

### What to Borrow

1. **Don't implement steering for MVP**: OpenCode proves that a functional coding agent doesn't need mid-turn steering. Users can wait for the turn to complete and then provide new input.

2. **Queue-based fallback**: If steering is desired in the future, implement as: receive input → queue → inject at next sampling iteration (if iterative loop exists) or at next turn start.

### Implementation Considerations (Post-MVP)

If steering is eventually needed:

1. Only possible if LOGOS controls the sampling loop (not if AI SDK handles tool loops internally)
2. Requires SessionEngine to support input injection at iteration boundaries
3. Abort current HTTP request cleanly
4. Merge new input with existing context
5. Re-emit current turn with updated input

### Risks

1. **High complexity, low ROI**: Steering is a rare feature that adds significant complexity. OpenCode doesn't have it and functions well.

2. **Provider dependency**: Whether steering is possible depends on whether the provider supports request cancellation mid-stream.
