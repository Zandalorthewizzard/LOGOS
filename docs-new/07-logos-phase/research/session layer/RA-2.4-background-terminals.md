# RA-2.4 — Background Terminal Management

## LOGOS Current Implementation

### Codex Path

**Where:** `codex-rs/core/src/unified_exec/`

`UnifiedExecProcessManager`:

- Created at session init (`codex.rs:1879`)
- Manages background processes spawned by model's `container.exec` tool
- Configurable `background_terminal_max_timeout`
- Process tracking by `process_id`
- Stdin interaction via `TerminalInteraction` events
- `Op::CleanBackgroundTerminals` terminates all running processes
- Auto-cleanup on session shutdown

### OR Path

**Where:** `codex-rs/app-server/src/codex_message_processor.rs:3512-3521`

Currently **delegates to OR thread manager** but implementation is minimal:

- `background_terminals_clean()` — kills tracked processes
- No full background terminal lifecycle in OR

**Known weaknesses:**

- OR path has minimal background terminal support
- Process management tied to CodexThread lifecycle
- No persistence of background terminal state across resume

## OpenCode Analysis

### PTY System

**File:** `src/pty/index.ts`

Architecture:

- In-memory `Map<PtyID, Active>` per instance (not per session)
- Each `Active` PTY has:
  - Process handle
  - Ring buffer (2MB max) for scrollback
  - WebSocket connections for streaming
- Operations:
  - `create()` — spawn PTY process
  - `write(id, data)` — send input to process
  - `resize(id, size)` — resize terminal
  - `connect(id)` — attach WebSocket for streaming
  - `disconnect(id, ws)` — detach WebSocket
  - `kill(id)` — terminate process
- Auto-cleanup on process exit
- Environment: `TERM=xterm-256color`, `OPENCODE_TERMINAL=1`

### Key Design Decisions

1. **Instance-scoped, not session-scoped**: PTYs live at the application instance level, not per session. This means they survive across session switches.

2. **Ring buffer for scrollback**: 2MB max ring buffer prevents unbounded memory growth. Clients can read buffer on connect.

3. **WebSocket streaming**: Real-time output via WebSocket, not SSE. Enables bidirectional communication (stdin + stdout).

4. **Auto-cleanup**: Process exit triggers cleanup. No manual tracking needed.

### Comparison

| Aspect           | LOGOS (Codex)              | LOGOS (OR)           | OpenCode                 |
| ---------------- | -------------------------- | -------------------- | ------------------------ |
| Scope            | Per CodexThread            | Per thread (minimal) | Per application instance |
| Process tracking | By process_id              | Minimal              | Map<PtyID, Active>       |
| Scrollback       | Not specified              | None                 | Ring buffer (2MB)        |
| Streaming        | TerminalInteraction events | None                 | WebSocket                |
| Stdin support    | Yes                        | No                   | Yes                      |
| Persistence      | None (lost on shutdown)    | None                 | None                     |
| Cleanup          | Manual + session shutdown  | Manual               | Auto on process exit     |

### Reusable Patterns

1. **Instance-scoped PTY management**: Decoupling PTYs from session scope is more practical. Background processes often outlive the session that started them.

2. **Ring buffer for scrollback**: Simple, bounded memory usage. New connections can read historical output.

3. **WebSocket for bidirectional streaming**: More natural for terminal interaction than SSE (which is server-to-client only).

4. **Auto-cleanup on process exit**: Reduces manual lifecycle management.

### What to Avoid

1. **Don't tie PTYs to session lifecycle**: OpenCode proves that instance-scoped is better. If a session is archived or evicted, background terminals should survive.

## Goose Analysis

### Background Terminals / PTY

**Not implemented in the core crate.**

Goose has no PTY, terminal multiplexer, or background process management in `crates/goose/src/`.

**What exists:**

- `crates/goose/src/subprocess.rs` — Process group isolation (38 lines, no PTY)
- ACP protocol has `client_capabilities.terminal` flag — suggests terminal support in the protocol layer
- `SessionType::Terminal` variant — suggests terminal session type exists but may be for ACP routing

**What's missing:**

- No PTY allocation
- No terminal session management
- No ring buffer or scrollback
- No process persistence
- No WebSocket streaming for terminal output
- Shell execution delegated to MCP extensions (not in core crate)

### Comparison

| Aspect       | Goose                 |
| ------------ | --------------------- |
| Scope        | Not implemented       |
| Scrollback   | Not implemented       |
| Streaming    | Not implemented       |
| Persistence  | Not implemented       |
| Auto-cleanup | Not implemented       |
| Protocol     | ACP has terminal flag |

## Cross-Project Comparison Matrix

| Aspect       | LOGOS (Codex)    | LOGOS (OR)           | OpenCode        | Goose           |
| ------------ | ---------------- | -------------------- | --------------- | --------------- |
| Scope        | Per thread       | Per thread (minimal) | Per instance    | Not implemented |
| Scrollback   | N/A              | None                 | Ring buffer 2MB | Not implemented |
| Streaming    | Events           | None                 | WebSocket       | Not implemented |
| Persistence  | None             | None                 | None            | Not implemented |
| Auto-cleanup | Session shutdown | None                 | Process exit    | Not implemented |

## Recommendations for LOGOS

### What to Borrow

1. **Instance-scoped process manager**: Create a shared `ProcessManager` at app-server level, not per thread. Processes survive session lifecycle.

2. **Ring buffer for output**: Bounded memory for terminal output history. Practical and prevents OOM.

3. **Auto-cleanup**: Register process exit handlers for automatic cleanup.

### What LOGOS Can Do Better

1. **Persistent process tracking**: Neither Codex nor OpenCode persists background terminal state. LOGOS could store process metadata (PID, command, cwd) in SQLite for recovery after crash/restart.

2. **Process reconnection**: If app-server restarts, ability to reconnect to still-running processes (by PID).

### Risks and Open Questions

1. **Process orphan management**: If app-server crashes, background processes become orphans. Need strategy for detecting and cleaning up orphans on restart.

2. **Cross-platform process management**: PTY behavior differs significantly between Windows and Unix. OpenCode uses `bun-pty` which abstracts this, but LOGOS needs a Rust equivalent.
