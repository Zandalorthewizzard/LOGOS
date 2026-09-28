# RA-1.5 — Shell Command Execution

## LOGOS Current Implementation

### OR Path

**Where:** `codex-rs/app-server/src/openrouter_beta_threads.rs:646-769, 2107-2283`

Flow:

1. `shell_command()` validates input
2. Spawns process via platform-specific shell
3. Streams stdout/stderr
4. Captures output and persists to thread state
5. Emits notifications (CommandExecutionOutputDelta)

### Codex Path

**Where:** `codex-rs/core/src/exec.rs`, `codex-rs/core/src/tools/handlers/shell.rs`

- `Op::RunUserShellCommand` → `execute_exec_request()` in sandbox
- `Op::UserInput` → model returns function_call("shell") → shell tool handler
- Full sandbox support (seatbelt, docker, windows)
- Background terminal management via `UnifiedExecProcessManager`
- Approval flow via `ExecApprovalRequestEvent`

**Known weaknesses:**

- OR shell execution is simplified — no sandbox, no approval flow
- Duplicated code between OR and Codex paths
- No background terminal support in OR

## OpenCode Analysis

### Shell Tool

**File:** `src/shell/shell.ts`

Platform-aware shell detection:

- Respects `$SHELL` env var
- Windows: PowerShell → Git Bash → COMSPEC
- macOS: `/bin/zsh`
- Linux: `bash` → `/bin/sh`
- Login shells get `-l` flag
- Fish and Nu blacklisted for direct use

### Execution in Session

**File:** `src/session/prompt.ts:743-917`

Shell execution as a tool:

1. Model returns tool call for shell/bash command
2. Tool handler spawns via Effect's `ChildProcessSpawner`
3. Streams stdout with metadata injection
4. Handles abort: injects abort metadata into output
5. Effect-based lifecycle management (scoped AbortController)

### PTY (Interactive Terminals)

**File:** `src/pty/index.ts`

- In-memory `Map<PtyID, Active>` per instance
- Each PTY has ring buffer (2MB max) for scrollback
- WebSocket-based streaming
- Supports resize, write, connect/disconnect
- Auto-cleanup on process exit
- Environment: `TERM=xterm-256color`, `OPENCODE_TERMINAL=1`

### Comparison

| Aspect               | LOGOS (OR)            | LOGOS (Codex)                    | OpenCode                                    |
| -------------------- | --------------------- | -------------------------------- | ------------------------------------------- |
| Shell detection      | Basic                 | Platform-specific                | Platform-aware, blacklist for exotic shells |
| Sandbox              | None                  | Full (seatbelt, docker, windows) | None                                        |
| Approval flow        | None                  | Full (ExecApprovalRequest)       | Permission system (ruleset-based)           |
| Background terminals | None                  | UnifiedExecProcessManager        | PTY with ring buffer + WebSocket            |
| Output streaming     | Manual stdout capture | StdoutStream::Record             | Effect ChildProcess stream                  |
| Abort handling       | Kill process          | CancellationToken                | Effect interrupt + metadata injection       |
| Tool integration     | Separate code path    | Shell tool handler               | Shell tool via AI SDK execute()             |

### Reusable Patterns

1. **Platform-aware shell detection** (`shell.ts`): Blacklist for shells that don't work well non-interactively (Fish, Nu). Simple but practical.

2. **Permission-based tool gating** (not file-level approval): OpenCode uses a ruleset-based permission system rather than per-command approval. Tools declare required permissions, user configures allow/deny rules. Less granular but simpler than Codex's per-command approval.

3. **PTY with ring buffer** (`pty/index.ts`): Ring buffer (2MB) for scrollback is practical. WebSocket streaming for real-time output. Clean lifecycle management.

4. **Effect-scoped AbortController** (`llm.ts:51-75`): AbortController lifecycle tied to Effect scope. Automatic cleanup. LOGOS uses CancellationToken which is similar but Rust-idiomatic.

### What to Avoid

1. **No sandbox**: OpenCode runs shell commands without sandboxing. LOGOS Codex path has sandbox support. The shared layer should support sandboxing as an option, not assume no sandbox.

2. **Simplified permission model**: OpenCode's ruleset approach is simpler than Codex's per-command approval but less flexible. LOGOS should keep the per-command approval option.

## Goose Analysis

### Shell Execution

**File:** `crates/goose/src/subprocess.rs` (38 lines)

Goose's shell execution is minimal in the core crate:

```rust
pub fn configure_subprocess(command: &mut Command) {
    #[cfg(unix)]
    command.process_group(0);  // Isolate from Ctrl+C
    command.set_no_window();   // Windows: no console popup
}
```

The actual shell tool execution is in:

- `crates/goose/src/execution/` — **MISSING from checkout** (tool execution engine)
- MCP extensions (`crates/goose-mcp/`) — **not in checkout** (developer tools, shell handler)

### Subprocess Isolation

- **Unix**: `process_group(0)` — creates new process group so SIGINT doesn't propagate to children
- **Windows**: `CREATE_NO_WINDOW` flag (0x08000000) — prevents console popup windows
- No sandbox, no seccomp, no resource limits

### Permission / Approval System

**Tool Inspection Framework** (`crates/goose/src/tool_inspection.rs`, 302 lines):

```
ToolInspector trait:
  - inspect(session_id, tool_requests, messages, goose_mode) → Vec<InspectionResult>
  - name() → &str
  - is_enabled() → bool

InspectionResult:
  - tool_request_id, action, reason, confidence, inspector_name, finding_id

InspectionAction: Allow | Deny | RequireApproval(Option<String>)

ToolInspectionManager:
  - inspectors: Vec<Box<dyn ToolInspector>>
  - inspect_tools() → runs all inspectors
  - apply_tool_annotations() → modifies tool definitions
  - process_inspection_results_with_permission_inspector() → PermissionCheckResult
```

**Permission mixing** (`apply_inspection_results_to_permissions`):

- Deny overrides Allow and RequireApproval
- RequireApproval overrides auto-Allow
- Inspector order doesn't matter for final result

The `permission/` module with `PermissionInspector` and `PermissionJudge` is **MISSING** from checkout.

### GooseMode (Execution Permission Levels)

Referenced from tool_inspection.rs and model.rs:

- `GooseMode` enum controls automatic tool approval behavior
- Provider's `update_mode(session_id, mode)` can change mode per-session
- Provider's `permission_routing()` returns `ActionRequired` or `Noop` — determines if provider handles permissions

### Comparison

| Aspect               | LOGOS (OR)   | Goose                                         |
| -------------------- | ------------ | --------------------------------------------- |
| Shell tool execution | Custom spawn | MCP extension (not in core crate)             |
| Sandbox support      | None         | None visible in core                          |
| Approval model       | None         | ToolInspector + PermissionInspector framework |
| Subprocess isolation | None         | Process group isolation (Unix/Windows)        |
| Tool inspection      | None         | Pluggable framework with multiple inspectors  |

## Cross-Project Comparison Matrix

| Aspect               | LOGOS (OR)   | LOGOS (Codex)          | OpenCode              | Goose                               |
| -------------------- | ------------ | ---------------------- | --------------------- | ----------------------------------- |
| Shell tool execution | Custom spawn | Sandbox + tool handler | AI SDK tool execute() | MCP extension (external crate)      |
| Sandbox support      | None         | Full                   | None                  | None in core                        |
| Approval model       | None         | Per-command            | Ruleset-based         | ToolInspector + PermissionInspector |
| Process isolation    | None         | Sandbox-dependent      | Effect ChildProcess   | process_group(0) / CREATE_NO_WINDOW |
| Doom loop guard      | None         | None                   | 3x threshold          | RepetitionInspector (configurable)  |
| Background PTY       | None         | UnifiedExec            | PTY + ring buffer     | TBD                                 |
| Abort mechanism      | Kill         | CancellationToken      | Effect interrupt      | TBD                                 |

## Recommendations for LOGOS

### What to Borrow

1. **Ring buffer for PTY output**: Practical pattern for managing scrollback without unbounded memory growth.

2. **Tool handler as shared component**: Shell execution should be a shared tool handler, not duplicated between OR and Codex paths. The handler should accept sandbox configuration as a parameter.

### What to Avoid

1. **Don't drop sandbox support**: When migrating shell execution to shared layer, preserve Codex's sandbox capability as an option.

### What LOGOS Can Do Better

1. **Unified shell tool handler**: Define a `ShellTool` that works for both paths. Accept sandbox policy, approval policy, and output capture mode as parameters. Codex path uses it with sandbox+approval. OR path uses it with relaxed policies.

2. **Configurable output capture**: Support both streaming (for UI) and capture (for persistence) modes simultaneously.

### Risks and Open Questions

1. **Sandbox abstraction**: Codex sandbox is deeply integrated with exec.rs. Extracting it as a parameter may require significant refactoring.

2. **Cross-platform testing**: Shell behavior differs significantly across platforms. Shared implementation must handle all platforms.
