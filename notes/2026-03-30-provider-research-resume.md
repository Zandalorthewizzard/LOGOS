# 2026-03-30 provider research resume

## Current state

- The source inventory is normalized.
- Provider cache folders exist under `input-sources-temp/`.
- `index.md` files are in place for each target folder.
- Canonical docs and repos are partially captured, with real checkout folders now present for:
  - `input-sources-temp/zai-python-sdk/repo-src`
  - `input-sources-temp/openrouter-ts-sdk/repo-src`
  - `input-sources-temp/openrouter-python-sdk/repo-src`
  - `input-sources-temp/google-js-genai/repo-src`

## Safe resume point

Next agent should:

1. fetch canonical snapshots into `input-sources-temp/*`;
2. save repo READMEs or minimal checkouts for official SDKs that are still missing;
3. update `provider-source-inventory.md` with confirmed statuses;
4. promote durable findings into `docs-new/07-logos-phase/research/`.

## Open items

- OpenAI docs still need local snapshots.
- OpenRouter canonical docs URLs are now partly confirmed from HTML, but additional pages can still be added.
- Z.AI JS/TS SDK remains `UNKNOWN`.
- LM Studio should be snapshotted in HTML because it is the main source of truth.
