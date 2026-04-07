# PR Summary Draft

## Title

Add phase-1 Logos boundary extraction and complete the first runtime-owned tool loop slice

## Summary

- add the phase-1 LOGOS boundary extraction slice with additive runtime, provider, session, and tool crates
- register a narrow `codex-core` Prompt seam for external tool injection and keep it isolated in one dedicated seam commit
- register a second narrow continuation seam so explicit `previous_response_id` survives Prompt -> API request -> websocket conversion without hidden cross-turn session reuse
- forward runtime-owned tool declarations through the canonical OpenAI/Codex request path and complete runtime-owned tool execution plus `ToolResult` reinjection in the normalized session loop
- add explicit approval events and capability-gated provider-native tool rejection so the phase-1 loop matches the architecture specs more honestly
- stabilize the OpenAI adapter by creating a fresh `ModelClientSession` per turn and deduplicating normalized `output_item.added`/`output_item.done` semantics
- add an architecture-first phase-2 discussion/spec pair so the next OpenAI canonical turn-loop phase starts from explicit ownership boundaries instead of ad hoc refactors

## Key Proofs

- text-only OpenAI turn through the new boundary
- continuation handle preservation via `previous_response_id`
- normalized tool-call boundary with stable `call_id`
- outbound runtime tool declaration forwarding
- runtime tool execution roundtrip with reinjection
- runtime tool failure reinjection with `is_error = true`
- approval-denied reinjection branch with explicit normalized approval events
- provider-native declaration capability gating

## Known Limits

- provider-native tool declarations are still unsupported from the normalized OpenAI phase-1 path
- no non-OpenAI provider work is included in this branch
