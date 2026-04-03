# PR Summary Draft

## Title

Add phase-1 Logos boundary extraction and complete the first runtime-owned tool loop slice

## Summary

- add the phase-1 LOGOS boundary extraction slice with additive runtime, provider, session, and tool crates
- register a narrow `codex-core` Prompt seam for external tool injection and keep it isolated in one dedicated seam commit
- forward runtime-owned tool declarations through the canonical OpenAI/Codex request path and complete runtime-owned tool execution plus `ToolResult` reinjection in the normalized session loop

## Key Proofs

- text-only OpenAI turn through the new boundary
- continuation handle preservation via `previous_response_id`
- normalized tool-call boundary with stable `call_id`
- outbound runtime tool declaration forwarding
- runtime tool execution roundtrip with reinjection
- runtime tool failure reinjection with `is_error = true`

## Known Limits

- provider-native tool declarations are still unsupported from the normalized boundary
- approval/policy-aware runtime execution is not wired yet
- no non-OpenAI provider work is included in this branch
