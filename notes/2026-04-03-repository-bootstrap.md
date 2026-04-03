# 2026-04-03 Repository Bootstrap

## Context

- Created the live public fork `https://github.com/Zandalorthewizzard/LOGOS` from `openai/codex`.
- Rebased the local `C:\Work\proj\LOGOS` workspace onto the real fork instead of the local `input-sources-temp/codex-upstream` reference checkout.

## Confirmed Facts

- `origin` -> `https://github.com/Zandalorthewizzard/LOGOS.git`
- `upstream` -> `https://github.com/openai/codex.git`
- clean upstream baseline tag: `logos-baseline-2026-04-03-af8a9d2`
- baseline commit: `af8a9d2d2ba84a1dafad6fa51fe3d63e96aceb75`
- `main` branch protection currently blocks force-push and deletion and requires conversation resolution
- upstream sync helper files:
  - `scripts/logos-sync-upstream.ps1`
  - `notes/logos-upstream-sync.workflow-disabled.yml` (disabled workflow draft; not part of active baseline CI)

## Fork-Safe CI Baseline

- default push/PR CI is now Linux-first and GitHub-hosted
- private OpenAI runner group usage was removed from the active baseline workflows
- expensive macOS large-runner lanes were removed from active default workflows
- `rust-ci-full.yml` no longer runs on every push; it is manual-only until LOGOS has matching infra
- npm package staging in `ci.yml` is disabled by default and only runs when `LOGOS_ENABLE_NPM_STAGING=true`
- `sdk.yml` and `bazel.yml` are now manual-only heavy workflows; upstream runs them quickly with remote Bazel infra, but the fork currently falls back to local-only Bazel and was hitting the 10m/30m job timeouts
- default LOGOS baseline is therefore the fast signal set: `ci`, `Codespell`, `cargo-deny`, and `rust-ci` for PR fast-path coverage
- the first GitHub-based upstream-sync workflow draft was removed from active workflows after repeated file-level validation failures on push

## Local-Only Decision

- `input-sources-temp/` remains the local reference/evidence cache and is not part of the first live fork bootstrap commit.

## Next Safe Resume Point

- Start phase-1 work from `feat/phase1-*` branches on top of the LOGOS bootstrap commit.
- Use `upstream-sync/*` branches plus the sync helper when `openai/codex` advances.
