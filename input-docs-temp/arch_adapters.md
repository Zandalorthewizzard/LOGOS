Да. Если резать по-живому, то **не надо делать “вариант C” в лоб** и разламывать Codex на 17k строк ради абстракции ради абстракции. Твой анализ как раз показывает, что такая лобовая операция ударит по самому крепкому месту системы: уже существующему OpenAI/Responses runtime. 

Нормальный путь для Logos — это **adapter-first архитектура с OpenAI Responses как canonical upstream path**, но **не как единственная внутренняя истина**.

Иначе говоря:

* **не переписывать Codex против Responses**;
* **не заставлять весь Logos мыслить сырым OpenAI JSON**;
* **вставить узкий внутренний runtime boundary**, который сохраняет силу Codex и локализует грязь по адаптерам.

---

# Что бы я предложила как зрелую модульную архитектуру

## 1. Главный принцип

**OpenAI Responses = reference protocol и gold path.**
**Logos Internal Runtime = минимальный нормализующий слой.**
**Provider adapters = точки локализации несовместимостей.**

Это не “сломать отличный протокол OpenAI”, а наоборот — **сделать его первым классом, а не бетонным фундаментом всего мира**.

Почему это правильно:

* OpenAI реально развивает `Responses` как свой основной agent/runtime интерфейс: stateful continuation, tools, streaming events, previous responses, built-in tools, compact response. ([OpenAI Платформа][1])
* Но уже сейчас видно, что другие сильные провайдеры живут по-другому:

  * OpenRouter имеет Responses facade, но он **stateless** и **beta**. ([OpenRouter][2])
  * Gemini двигает свой путь: OpenAI compatibility есть, но параллельно у Google уже есть **Interactions API (beta)** как отдельный unified interface для state/tool orchestration. ([Google AI for Developers][3])
  * Z.AI имеет OpenAI-compatible вход, но собственные capability-оси: thinking, tool streaming output, context caching, structured output. ([docs.z.ai][4])

То есть рынок уже сам говорит тебе:
**единый внешний фасад полезен, но семантика внутри у всех разная.**

---

# 2. Архитектурная схема

```text
+--------------------------------------------------------------+
|                        Logos App / UI                        |
+-------------------------------+------------------------------+
                                |
                                v
+--------------------------------------------------------------+
|                    logos-session-engine                      |
|   turn loop, retries, cancellation, budgeting, tool loop     |
+-------------------------------+------------------------------+
                                |
                                v
+--------------------------------------------------------------+
|                    logos-runtime-types                       |
|  InternalTurn, InternalEvent, ToolCall, ToolResult,          |
|  ContinuationHandle, CapabilitySet, ProviderExtension        |
+-------------------------------+------------------------------+
                                |
               +----------------+----------------+
               |                                 |
               v                                 v
+-------------------------------+   +-------------------------------+
|      logos-provider-api       |   |      logos-tool-runtime       |
| ProviderAdapter traits        |   | tool registry/router/sandbox  |
| Model discovery, state, map   |   | largely reused from Codex     |
+-------------------------------+   +-------------------------------+
               |
     +---------+---------+---------+---------+
     |                   |                   |
     v                   v                   v
+-----------+     +-------------+     +---------------+
| openai    |     | openrouter  |     | gemini        |
| adapter   |     | adapter     |     | adapter       |
+-----------+     +-------------+     +---------------+
     |                   |                   |
     v                   v                   v
 Responses API      Responses facade       Native / compat API

                     +------------------+
                     | z.ai adapter      |
                     +------------------+
```

---

# 3. Что должно стать внутренним контрактом Logos

Вот тут и есть ключ: **не делай огромный provider-agnostic universe**.
Сделай **маленький и жесткий runtime contract**.

## 3.1 Internal turn model

```rust
pub struct InternalTurnRequest {
    pub model_ref: ModelRef,
    pub system: Option<String>,
    pub messages: Vec<InternalMessage>,
    pub tools: Vec<InternalToolDef>,
    pub output_mode: OutputMode,
    pub reasoning_policy: ReasoningPolicy,
    pub continuation: Option<ContinuationHandle>,
    pub limits: TurnLimits,
}
```

## 3.2 Internal message model

```rust
pub enum InternalMessage {
    User { parts: Vec<ContentPart> },
    Assistant { parts: Vec<ContentPart> },
    ToolResult { call_id: String, output: serde_json::Value },
}
```

## 3.3 Internal stream/event model

Это самая важная часть. Не `ResponseItem`, а **минимальный общий знаменатель**.

```rust
pub enum InternalEvent {
    TextDelta { text: String },
    ToolCallStarted { call_id: String, name: String },
    ToolCallArgumentsDelta { call_id: String, delta: String },
    ToolCallFinished { call_id: String },
    ReasoningDelta { text: String },
    Usage { input_tokens: u64, output_tokens: u64 },
    ContinuationUpdated { handle: ContinuationHandle },
    ProviderExtension { key: String, payload: serde_json::Value },
    Completed,
    Failed(ProviderError),
}
```

## 3.4 ContinuationHandle

Не пытайся сделать вид, что у всех один `previous_response_id`.

```rust
pub enum ContinuationHandle {
    OpenAiPreviousResponseId(String),
    ProviderSessionId(String),
    ClientManagedHistory,
    Opaque(String), // provider-owned opaque state blob
}
```

Это сразу решает ключевую проблему:

* OpenAI можно использовать нативно;
* OpenRouter честно маппится как `ClientManagedHistory`, потому что у него stateless Responses beta. ([OpenRouter][2])
* Gemini/Z.AI/другие не обязаны притворяться OpenAI.

## 3.5 CapabilitySet

Вот это обязательно. Без capability matrix ты снова скатишься в ложную универсальность.

```rust
pub struct CapabilitySet {
    pub supports_server_side_state: bool,
    pub supports_client_side_history: bool,
    pub supports_tool_calling: bool,
    pub supports_parallel_tools: bool,
    pub supports_tool_arg_streaming: bool,
    pub supports_reasoning_deltas: bool,
    pub supports_opaque_reasoning_carryover: bool,
    pub supports_structured_output: bool,
    pub supports_context_cache: bool,
    pub supports_provider_builtin_tools: bool,
    pub supports_compaction: bool,
}
```

---

# 4. Какие crates я бы ввела

## `logos-runtime-types`

Чистые типы. Без HTTP, без serde-wire-моделей провайдера.

Содержит:

* `InternalTurnRequest`
* `InternalMessage`
* `InternalEvent`
* `ContinuationHandle`
* `CapabilitySet`
* `InternalToolDef`
* `OutputMode`
* `ReasoningPolicy`

## `logos-provider-api`

Trait-слой.

```rust
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn capabilities(&self, model: &ModelRef) -> CapabilitySet;

    async fn stream_turn(
        &self,
        req: InternalTurnRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = InternalEvent> + Send>>, ProviderError>;

    async fn list_models(&self) -> Result<Vec<ModelMeta>, ProviderError>;
}
```

Отдельно можно добавить:

```rust
pub trait ProviderStatePolicy {
    fn continuation_mode(&self) -> ContinuationMode;
}
```

## `logos-provider-openai`

Это **не просто адаптер**, а **canonical adapter**.

Он должен:

* максимально прозрачно маппить Internal ↔ Responses;
* сохранять `previous_response_id`;
* сохранять OpenAI extensions через `ProviderExtension`;
* поддерживать built-in tools как provider-native path.
  OpenAI здесь остается эталонным маршрутом. ([OpenAI Платформа][1])

## `logos-provider-openrouter`

Не отдельная философия, а thin variant поверх openai-like adapter, но с другой state policy:

* continuation = `ClientManagedHistory`;
* запрет на ожидание server-side state;
* beta/stability flag. ([OpenRouter][2])

То есть это почти `openai adapter + stateless policy + quirks`.

## `logos-provider-gemini`

Не надо насильно толкать Gemini в raw Responses.

У Gemini:

* OpenAI compatibility слой есть; ([Google AI for Developers][3])
* но у него свои tool/thinking semantics;
* и Google уже двигает отдельный `Interactions API` для state/tool orchestration. ([Google AI for Developers][5])

Поэтому Gemini adapter должен быть **native-first**, а compat-mode — вторичным.

## `logos-provider-zai`

Тоже **native-first**:

* OpenAI-compatible path можно использовать для быстрого старта; ([docs.z.ai][4])
* но thinking, tool streaming output, context caching лучше маппить как provider extensions/capabilities, а не делать вид, что это “тот же Responses”. ([docs.z.ai][6])

## `logos-tool-runtime`

Его я бы старалсь **максимально переиспользовать из codex tool pipeline**.

То есть не ломать:

* tool registry
* router
* sandbox/runtime

а лишь поменять контракт между “моделью” и “исполнением инструмента”.

---

# 5. Что нельзя абстрагировать рано

Вот здесь чаще всего и совершают глупость.

## Не нормализуй до смерти:

### 1. Reasoning

У OpenAI есть свои reasoning-механики, у Gemini — свои thought-oriented semantics, у Z.AI — thinking mode, у DeepSeek — отдельный reasoning output. Это не один и тот же объект. ([Google AI for Developers][3])

Правильный путь:

* общий флаг `supports_reasoning_deltas`
* общий `ReasoningPolicy`
* provider-specific payload через extension channel

### 2. Built-in tools

OpenAI built-in tools, Gemini built-in tools и Z.AI tooling не надо насильно сливать в одну святую корову.

Правильный путь:

* **runtime-owned tools** — portable path
* **provider-native tools** — optional fast path через capability flag

### 3. Server-side state semantics

OpenAI stateful, OpenRouter stateless, Gemini вообще может жить через свой иной orchestration API. ([OpenAI Платформа][1])

Правильный путь:

* `ContinuationHandle`
* `ContinuationMode`
* session engine сам знает, когда тащить историю целиком

### 4. Compaction

OpenAI `compact` — не универсальная истина. Это provider feature, а не основа мира. ([OpenAI Платформа][1])

Правильный путь:

* `supports_compaction`
* fallback в logos-side summarization/memory compaction

---

# 6. Как это внедрять без 17k строк шторма

Не делать “большой refactor”. Делать **перевалочные этапы**.

## Этап 1. Boundary extraction

Минимум изменений.

Что меняется:

* ввести `logos-runtime-types`
* ввести `logos-provider-api`
* сделать `logos-provider-openai`
* `core/client.rs` и связанный путь не удалять, а обернуть

Это не ломает протокол. Это просто делает OpenAI первым адаптером.

## Этап 2. Tool loop decoupling

Меняешь не tools, а только модельно-инструментальный шов:

* model emits `InternalEvent::ToolCallStarted/ArgumentsDelta/Finished`
* session-engine дергает существующий tool runtime
* результат возвращается как `InternalMessage::ToolResult`

Именно тут Logos перестает быть “сырым Responses inside”.

## Этап 3. OpenRouter adapter

Это лучший второй адаптер, потому что он внешне близок к OpenAI, но сразу заставляет тебя правильно решить stateless continuation. ([OpenRouter][2])

## Этап 4. Gemini / Z.AI adapters

После этого уже можно спокойно делать native adapters:

* Gemini — под его state/tool semantics
* Z.AI — под его thinking/context cache/tool streaming

---

# 7. Что это дает именно тебе как соло-форку

Самое главное — **сдвиг точки боли**.

Без этой архитектуры:

* каждый новый провайдер = боль по всему runtime
* каждый апдейт Codex/OpenAI = риск для всей внутренней онтологии

С ней:

* апдейт OpenAI/Codex чаще всего = правки в `logos-provider-openai`
* новый провайдер = новый crate + capability mapping
* tool runtime и session loop остаются стабильнее

То есть ты не “ломаешь отличный протокол OpenAI”, а **обводишь его адаптерным кольцом**.

---

# 8. Где здесь честный компромисс

Компромисс такой:

**Logos не должен быть “provider-agnostic до последнего атома”.**
Это детская болезнь архитекторов.

Он должен быть:

* **OpenAI-first at the compatibility boundary**
* **provider-aware in the internal runtime**
* **adapter-modular in maintenance**

Это зрелая позиция.

---

# 9. Самая короткая формула архитектуры

Если уж совсем сжать:

**Оставь Responses как canonical upstream protocol.**
**Сделай маленький Internal Runtime Protocol.**
**Пусть провайдерные crate’ы переводят свой мир в этот протокол.**
**Не нормализуй reasoning, built-in tools и server-state до лжи.**

Вот тогда ты не получаешь:

* 17k строк “рефакторинга ради рефакторинга”,
* и не попадаешь в ловушку “OpenAI semantics = физика Вселенной”.

---

Могу следующим сообщением собрать это уже в сухой инженерный `.md`-спек формата:

* цели,
* non-goals,
* crate layout,
* Rust traits,
* event contract,
* migration phases,
* acceptance criteria.

[1]: https://platform.openai.com/docs/api-reference/responses-streaming?ref=medhakhurana.com&utm_source=chatgpt.com "Streaming events | OpenAI API Reference"
[2]: https://openrouter.ai/docs/api/reference/responses/overview?utm_source=chatgpt.com "OpenRouter Responses API Beta"
[3]: https://ai.google.dev/gemini-api/docs/openai?utm_source=chatgpt.com "OpenAI compatibility | Gemini API - Google AI for Developers"
[4]: https://docs.z.ai/guides/develop/openai/python?utm_source=chatgpt.com "OpenAI Python SDK - Overview"
[5]: https://ai.google.dev/gemini-api/docs/interactions?utm_source=chatgpt.com "Interactions API | Gemini API - Google AI for Developers"
[6]: https://docs.z.ai/guides/develop/http/introduction?utm_source=chatgpt.com "HTTP API Calls - Overview - Z.AI DEVELOPER DOCUMENT"
