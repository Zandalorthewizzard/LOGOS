# Code-Main Fork Research Task

## Status

This is a non-canonical research task for an external agent.

Source under review:

- `C:\Work\proj\LOGOS\input-sources-temp\code-main`

This pass is read-only. Do not modify the fork and do not modify LOGOS while doing this research.

## Goal

Inspect the `code-main` fork and extract:

1. architecture or implementation choices that may help LOGOS phase 6
2. multi-provider or provider-foundation decisions worth borrowing
3. mistakes, dead ends, or abstraction drift that LOGOS should avoid
4. findings directly relevant to:
   - OpenRouter agent-beta work
   - provider foundation stabilization
   - Codex coexistence and auth isolation
   - runtime-owned tools, MCP, approvals, replay, and transport contracts

This is not a general repo tour. The output must be filtered for LOGOS relevance.

## Required Output

Produce one evidence-backed report with these sections:

1. `Top 5 borrowable ideas`
2. `Top 5 avoidable mistakes`
3. `Direct implications for LOGOS phase 6`
4. `Direct implications for post-MVP provider work`
5. `UNKNOWN / needs deeper follow-up`

For each finding include:

1. finding title
2. why it matters for LOGOS
3. concrete recommendation:
   - `Adopt`
   - `Adapt`
   - `Reject`
   - `Watch`
4. evidence:
   - absolute path
   - symbol or file-level area
   - line numbers where practical

If a claim is not supported by code or docs, mark it `UNKNOWN`.

## Research Priorities

### 1. Provider Foundation And Multi-Provider Modeling

Look for:

1. provider and model metadata contracts
2. how providers are identified, selected, and exposed to users
3. capability modeling and any drift between advertised and real behavior
4. any clean or dirty approach to multi-provider Responses parity

Start here:

1. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\model-provider-info\src\lib.rs`
2. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\responses-api-proxy\src\lib.rs`
3. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\responses-api-proxy\src\main.rs`
4. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\cli\src\main.rs`
5. `C:\Work\proj\LOGOS\input-sources-temp\code-main\config.toml.example`
6. `C:\Work\proj\LOGOS\input-sources-temp\code-main\docs\config.md`
7. `C:\Work\proj\LOGOS\input-sources-temp\code-main\docs\authentication.md`

Questions:

1. Do they have a better provider metadata surface than current LOGOS?
2. Do they collapse provider differences honestly or dishonestly?
3. Is there anything useful for future adapter conformance in LOGOS?

### 2. Tool Loop, MCP, And Approval Surfaces

Look for:

1. runtime-owned tools or tool-call contracts
2. MCP call transport and correlation
3. approval correlation and turn ownership
4. protocol or app-server surfaces that expose useful operational patterns

Start here:

1. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\mcp-server\src\codex_tool_runner.rs`
2. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\mcp-server\src\message_processor.rs`
3. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\mcp-server\src\exec_approval.rs`
4. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\mcp-server\src\patch_approval.rs`
5. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\rmcp-client\src\lib.rs`
6. `C:\Work\proj\LOGOS\input-sources-temp\code-main\code-rs\app-server\src\code_message_processor.rs`
7. `C:\Work\proj\LOGOS\input-sources-temp\code-main\code-rs\app-server-protocol\src\protocol\v2.rs`

Questions:

1. Do they have a cleaner MCP or approval correlation story than LOGOS?
2. Are there patterns worth borrowing for app-server or future host integration?
3. Are there anti-patterns where protocol breadth produced architectural noise?

### 3. Auth Isolation, Config Isolation, And Coexistence

Look for:

1. config home separation
2. auth storage strategy
3. env filtering and provider-key import rules
4. whether multiple agent tools can coexist cleanly on one machine

Start here:

1. `C:\Work\proj\LOGOS\input-sources-temp\code-main\code-rs\arg0\src\lib.rs`
2. `C:\Work\proj\LOGOS\input-sources-temp\code-main\code-rs\core\src\config.rs`
3. `C:\Work\proj\LOGOS\input-sources-temp\code-main\code-rs\core\src\config_loader\mod.rs`
4. `C:\Work\proj\LOGOS\input-sources-temp\code-main\docs\authentication.md`
5. `C:\Work\proj\LOGOS\input-sources-temp\code-main\docs\install.md`

Questions:

1. Do they solve side-by-side installation or auth isolation better than LOGOS currently plans to?
2. Are there mistakes that would create Codex/LOGOS collisions if copied?
3. Is there anything useful for phase-6 coexistence testing?

### 4. Runtime, Resume, And Thread/Conversation Surfaces

Look for:

1. restart or resume patterns
2. thread or conversation lifecycle design
3. compaction or checkpointing signals
4. how much of the system depends on raw provider payloads vs normalized state

Start here:

1. `C:\Work\proj\LOGOS\input-sources-temp\code-main\code-rs\app-server\src\code_message_processor.rs`
2. `C:\Work\proj\LOGOS\input-sources-temp\code-main\code-rs\app-server-protocol\src\protocol\v2.rs`
3. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\core\src\codex.rs`
4. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-rs\core\src\client.rs`

Questions:

1. Do they have a useful normalized turn or thread surface?
2. Did they create any provider-coupled or app-server-coupled dead ends that LOGOS should avoid?
3. Is there anything worth studying for future journal or host integration work?

### 5. Product And Portability Lessons

Look for:

1. binary/runtime footprint implications
2. JS/TS-heavy surfaces that add complexity without strong runtime value
3. platform support compromises
4. anything that would conflict with the LOGOS philosophy of a lightweight Rust-first CLI agent

Start here:

1. `C:\Work\proj\LOGOS\input-sources-temp\code-main\README.md`
2. `C:\Work\proj\LOGOS\input-sources-temp\code-main\docs\index.md`
3. `C:\Work\proj\LOGOS\input-sources-temp\code-main\docs\install.md`
4. `C:\Work\proj\LOGOS\input-sources-temp\code-main\code-rs\app-server\`
5. `C:\Work\proj\LOGOS\input-sources-temp\code-main\codex-cli\`

Questions:

1. What parts look architecturally useful despite the added size?
2. What parts look like product noise or operational drag?
3. What should LOGOS explicitly avoid as it grows past beta?

## Hard Constraints

1. Do not propose copying broad architecture blindly.
2. Do not praise or criticize without evidence.
3. Do not spend time on unrelated UI polish unless it reveals provider/runtime lessons.
4. Keep focus on LOGOS-relevant runtime, provider, tool, auth, transport, and portability concerns.
5. If you find a useful pattern, explain whether it belongs in:
   - phase 6
   - post-beta platform work
   - post-MVP provider milestones

## Desired Deliverable Tone

Be concrete and selective.

Bad output:

1. a broad summary of the entire fork
2. generic statements like `it has many providers`
3. praise without evidence

Good output:

1. `This provider metadata contract is cleaner than LOGOS because ...`
2. `This app-server protocol adds noise and should not be copied because ...`
3. `This auth isolation decision is directly relevant to phase 6 coexistence because ...`
