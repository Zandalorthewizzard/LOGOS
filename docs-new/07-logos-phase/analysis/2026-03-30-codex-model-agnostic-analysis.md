# Codex CLI: Анализ архитектуры для перехода к Model-Agnostic (Logos)

**Дата:** 2026-03-30  
**Источник:** `C:\Work\codex-main` (OpenAI Codex CLI, Apache-2.0)  
**Цель:** Оценка возможности и сложности интеграции Codex в Chaemera/Logos как model-agnostic движка

---

## 1. Краткое резюме

| Параметр | Значение |
|---|---|
| **Язык ядра** | Rust (~84 крейта в workspace) |
| **API-протокол** | OpenAI Responses API (`/v1/responses`), эксклюзивно |
| **Транспорт** | WebSocket (предпочтительно) + HTTP SSE (fallback) |
| **Провайдеры "из коробки"** | OpenAI, Ollama, LM Studio (все через Responses API) |
| **Абстракция провайдера** | Нет trait'а — только `ModelProviderInfo` (data-driven) |
| **Оценка сложности перехода к model-agnostic** | **Высокая** (объемная работа, глубокая связанность) |

**Вывод:** Codex НЕ является model-agnostic в классическом смысле. Он жёстко привязан к формату OpenAI Responses API. Поддержка Ollama/LMStudio существует, но она работает _только потому_, что эти серверы реализуют OpenAI-совместимый Responses API endpoint. Для поддержки Anthropic, Google Gemini и других провайдеров потребуется **значительная архитектурная переработка**.

---

## 2. Архитектура Codex

### 2.1 Общая схема

```
codex-cli/bin/codex.js          ← Node.js launcher (определяет платформу, запускает Rust binary)
         │
codex-rs/cli/src/main.rs        ← Rust CLI (clap, маршрутизация subcommands)
         │
         ├── tui_app_server/    ← TUI (Ratatui) — интерактивный режим
         ├── exec/              ← codex exec — неинтерактивный запуск
         ├── app-server/        ← JSON-RPC over stdio/WebSocket (для IDE)
         └── mcp-server/        ← MCP-сервер (для других инструментов)
                  │
         codex-rs/core/         ← ЯДРО: оркестрация, клиент, сессии, инструменты
                  │
         ├── codex-api/         ← API-клиент (ResponsesClient, SSE, WebSocket)
         ├── codex-client/      ← HTTP-транспорт (reqwest, retry)
         └── protocol/          ← Типы данных (ResponseItem, Event, Submission)
```

### 2.2 Слои вызова LLM

```
 codex.rs (оркестрация сессии/turn'а)
      │
 client.rs (ModelClient → выбор транспорта: WS vs HTTP)
      │
 codex-api (ResponsesClient, WebSocketClient, SSE-парсинг)
      │
 codex-client (HttpTransport trait → reqwest)
      │
 reqwest / tokio-tungstenite (сетевой уровень)
```

---

## 3. Ключевые точки связывания с OpenAI

### 3.1 Wire Protocol — `WireApi`

**Файл:** `codex-rs/core/src/model_provider_info.rs:40-44`

```rust
pub enum WireApi {
    #[default]
    Responses,  // ЕДИНСТВЕННЫЙ вариант
}
```

Раньше существовал вариант `Chat` (Chat Completions API), но он **полностью удалён** с сообщением об ошибке. Поддерживается только Responses API.

### 3.2 Формат запроса — `ResponsesApiRequest`

**Файл:** `codex-rs/codex-api/src/endpoint/common.rs`

```rust
pub struct ResponsesApiRequest {
    pub model: String,
    pub instructions: String,       // Системные инструкции
    pub input: Vec<ResponseItem>,   // История对话
    pub tools: Vec<serde_json::Value>, // Определения инструментов
    pub tool_choice: String,         // "auto"
    pub parallel_tool_calls: bool,
    pub reasoning: Option<Reasoning>, // Reasoning effort + summary
    pub store: bool,
    pub stream: bool,                // Всегда true
    pub include: Vec<String>,        // "reasoning.encrypted_content"
    pub service_tier: Option<String>,
    pub prompt_cache_key: Option<String>,
    pub text: Option<TextControls>,  // Verbosity + JSON schema output
}
```

Это **специфичный для OpenAI** формат. Ни один другой провайдер не использует:
- `reasoning` с `encrypted_content`
- `service_tier`
- `store` 
- `include: ["reasoning.encrypted_content"]`

### 3.3 Формат ответа — `ResponseItem`

**Файл:** `codex-rs/protocol/src/models.rs`

```rust
pub enum ResponseItem {
    Message { id, role, content: Vec<ContentItem>, end_turn, phase },
    Reasoning { id, summary, content, encrypted_content },
    LocalShellCall { id, call_id, status, action },
    FunctionCall { id, name, namespace, arguments, call_id },
    FunctionCallOutput { call_id, output },
    CustomToolCall { call_id, name, input },
    WebSearchCall { ... },
    ImageGenerationCall { ... },
    ToolSearchCall { ... },
    GhostSnapshot { ... },
    Compaction { encrypted_content },
    Other,  // Catch-all
}
```

Эти типы **напрямую маппятся на JSON-формат** OpenAI Responses API. Сериализация/десериализация через serde без промежуточного слоя.

### 3.4 Инструменты — `ToolSpec`

**Файл:** `codex-rs/core/src/client_common.rs`

```rust
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ToolSpec {
    #[serde(rename = "function")]
    Function(ResponsesApiTool),
    #[serde(rename = "tool_search")]
    ToolSearch { ... },
    #[serde(rename = "local_shell")]
    LocalShell {},
    #[serde(rename = "image_generation")]
    ImageGeneration { ... },
    #[serde(rename = "web_search")]
    WebSearch { ... },
    #[serde(rename = "custom")]
    Freeform(FreeformTool),
}
```

Инструменты **напрямую сериализуются** в формат OpenAI Responses API tool definitions. Нет промежуточного provider-agnostic представления.

### 3.5 SSE-события

События стриминга привязаны к Responses API:
- `response.output_item.done` — завершённый элемент
- `response.output_text.delta` — текстовый дельта
- `response.reasoning_summary_text.delta` — reasoning дельта
- `response.completed` — финальное событие с token usage
- `response.failed` — обработка ошибок (context window, quota, rate limits)

### 3.6 WebSocket-протокол

Кастомный OpenAI WebSocket поверх Responses API:
- Отправка `response.create` сообщений
- `previous_response_id` для инкрементальных запросов
- Beta-заголовок: `OpenAI-Beta: responses_websockets=2026-02-06`
- Prewarm-механизм с `generate=false`

---

## 4. Существующие "альтернативные" провайдеры

### 4.1 Как работает поддержка Ollama/LM Studio

**Оба провайдера работают ТОЛЬКО потому, что реализуют OpenAI-совместимый `/v1/responses` endpoint.**

**Файл:** `codex-rs/core/src/model_provider_info.rs:289-313`

```rust
pub fn built_in_model_providers(openai_base_url: Option<String>) -> HashMap<String, ModelProviderInfo> {
    [
        ("openai", openai_provider),           // api.openai.com/v1
        ("ollama", create_oss_provider(11434, WireApi::Responses)),  // localhost:11434/v1
        ("lmstudio", create_oss_provider(1234, WireApi::Responses)), // localhost:1234/v1
    ].into_iter().collect()
}
```

Код `create_oss_provider` создаёт провайдер с:
- `base_url: "http://localhost:{port}/v1"`
- `env_key: None` (не нужен API key)
- `requires_openai_auth: false`
- `supports_websockets: false`

**Вывод:** Ollama ≥ 0.13.4 и LM Studio должны реализовывать Responses API, чтобы работать с Codex. Если они этого не делают — они не работают.

### 4.2 Крейты lmstudio и ollama

Эти крейты (`codex-rs/lmstudio/`, `codex-rs/ollama/`) используются **только для bootstrap**:
- Проверка здоровья сервера
- Листинг моделей
- Скачивание/загрузка моделей

**Вся инференс-логика идёт через единый ResponsesClient** — нет отдельного кода для разных провайдеров.

### 4.3 Конфигурация пользовательских провайдеров

Пользователи могут добавлять провайдеры в `~/.codex/config.toml`:

```toml
[model_providers.my_provider]
name = "My Provider"
base_url = "https://my-api.example.com/v1"
env_key = "MY_API_KEY"
wire_api = "responses"  # ЕДИНСТВЕННОЕ значение
http_headers = { "X-Custom" = "value" }
```

Опять же — провайдер **обязан** говорить Responses API.

---

## 5. Провайдерные данные vs Провайдерный Trait

### 5.1 ModelProviderInfo — данные, а не поведение

```rust
pub struct ModelProviderInfo {
    pub name: String,
    pub base_url: Option<String>,
    pub env_key: Option<String>,
    pub wire_api: WireApi,
    pub http_headers: Option<HashMap<String, String>>,
    pub env_http_headers: Option<HashMap<String, String>>,
    pub query_params: Option<HashMap<String, String>>,
    pub requires_openai_auth: bool,
    pub supports_websockets: bool,
    // retry/timeout настройки...
}
```

Все провайдеры описываются **данными** (URL, заголовки, ключи), а не **поведением** (trait). Нет `Provider` trait с методами `send_request()`, `parse_response()`, `format_tools()`.

### 5.2 Единственные trait'ы в стеке

| Trait | Назначение | Уровень |
|---|---|---|
| `HttpTransport` | HTTP-запрос (reqwest) | Транспорт |
| `AuthProvider` |Bearer token | Аутентификация |
| `ToolHandler` | Обработка инструментов | Инструменты |
| `ToolOutput` | Формат результата инструмента | Инструменты |
| `ToolRuntime` | Sandbox-выполнение | Sandbox |

**Нет Provider/LLM trait'а.**

---

## 6. Что нужно для Model-Agnostic

### 6.1 Проблемные зоны (耦合)

| # | Зона | Связанность | Файлы | Объём работы |
|---|---|---|---|---|
| 1 | **Wire Protocol** | `ResponsesApiRequest` → напрямую в JSON | `codex-api/src/endpoint/common.rs`, `responses.rs`, `responses_websocket.rs` | **Высокий** |
| 2 | **Response Items** | `ResponseItem` ↔ OpenAI JSON 1:1 | `protocol/src/models.rs` (~2772 строк) | **Высокий** |
| 3 | **Tool Definitions** | `ToolSpec` сериализуется в OpenAI формат | `core/src/client_common.rs`, `tools/src/responses_api.rs` | **Высокий** |
| 4 | **SSE Events** | Парсинг Responses API событий | `codex-api/src/sse/responses.rs` | **Средний** |
| 5 | **WebSocket Protocol** | Кастомный WS Responses API | `codex-api/src/endpoint/responses_websocket.rs` | **Средний** |
| 6 | **Reasoning** | `encrypted_content`, reasoning summary | `protocol/src/models.rs`, `codex-api/` | **Низкий** (OpenAI-specific) |
| 7 | **Streaming** | Дельта-события привязаны к API | `core/src/client.rs`, `stream_events_utils.rs` | **Средний** |
| 8 | **Compaction** | `/v1/responses/compact` endpoint | `core/src/compact.rs`, `compact_remote.rs` | **Низкий** (OpenAI-specific) |
| 9 | **Client** | `ModelClient` привязан к Responses API | `core/src/client.rs` (~1300+ строк) | **Высокий** |
| 10 | **Config** | `WireApi` enum = только `Responses` | `core/src/model_provider_info.rs` | **Низкий** |

### 6.2 Минимальная архитектура для Model-Agnostic

```
         Provider Trait
              │
    ┌─────────┼─────────┐
    │         │         │
OpenAI     Anthropic   Gemini
Provider   Provider    Provider
    │         │         │
    └──── Provider-Agnostic Internal Types ────┘
                    │
         Core Orchestrator (codex.rs)
                    │
              Tool System
              Agent System
              Sandbox System
```

### 6.3 Необходимый Provider Trait

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    // Отправка запроса с потоковым ответом
    async fn stream_completion(
        &self,
        request: ProviderAgnosticRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = ProviderEvent> + Send>>>;
    
    // Список доступных моделей
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    
    // Возможности провайдера
    fn capabilities(&self) -> ProviderCapabilities;
}

pub struct ProviderCapabilities {
    supports_parallel_tool_calls: bool,
    supports_websockets: bool,
    supports_reasoning: bool,
    supports_streaming: bool,
    max_context_tokens: Option<i64>,
    supported_tool_types: Vec<ToolType>,
}

pub enum ProviderEvent {
    TextDelta(String),
    ToolCall { id: String, name: String, arguments: String },
    ToolCallEnd { id: String },
    ReasoningDelta(String),
    Usage(TokenUsage),
    Done,
    Error(ProviderError),
}
```

### 6.4 Необходимые Provider-Agnostic Internal Types

```rust
// Заменить ResponseItem на:
pub enum InternalMessage {
    User { content: Vec<ContentPart> },
    Assistant { content: Vec<ContentPart> },
    ToolCall { id: String, name: String, arguments: Value },
    ToolResult { id: String, result: Value },
}

pub enum ContentPart {
    Text(String),
    Image { url: String },
}

// Заменить ToolSpec (провайдерный формат) на:
pub struct InternalToolDef {
    pub name: String,
    pub description: String,
    pub parameters: JsonSchema,
    pub output_schema: Option<JsonSchema>,
}
```

---

## 7. Оценка объёма работ

### 7.1 Вариант A: Минимальная адаптация (поддержка OpenAI-совместимых)

**Что делаем:** Ничего не меняем в ядре. Используем Codex как есть с провайдерами, которые реализуют OpenAI Responses API.

**Провайдеры:** Ollama, LM Studio, vLLM (с OpenAI-совместимым endpoint), Together AI, Groq (если добавят Responses API).

**Объём:** **Минимальный** — 1-2 дня на настройку конфигурации.

**Ограничения:** Anthropic Claude, Google Gemini, Mistral — не работают, пока не добавят Responses API.

### 7.2 Вариант B: Прокси-слой (Translation Proxy)

**Что делаем:** Создаём промежуточный прокси-сервер, который транслирует Responses API ↔ целевой API провайдера.

```
Codex → Responses API → Translation Proxy → Anthropic/Gemini/Mistral API
```

**Провайдеры:** Любые, для которых написан транслятор.

**Объём:** **Средний** — 2-4 недели на прокси-сервер + трансляторы для каждого провайдера.

**Плюсы:** Не трогаем ядро Codex.  
**Минусы:** Задержка, дублирование логики, сложность синхронизации.

### 7.3 Вариант C: Полная переработка ядра (True Model-Agnostic)

**Что делаем:** Внедряем `LlmProvider` trait и provider-agnostic внутренние типы.

**Затрагиваемые крейты:**

| Крейт | Изменения | LOC (оценка) |
|---|---|---|
| `protocol` | Новые типы `InternalMessage`, `InternalToolDef` | ~2000 строк |
| `codex-api` | `LlmProvider` trait + реализация для OpenAI | ~3000 строк |
| `core/client.rs` | Переписать `ModelClient` для работы через trait | ~1500 строк |
| `core/client_common.rs` | Заменить `ToolSpec` → `InternalToolDef` | ~500 строк |
| `core/stream_events_utils.rs` | Адаптация под provider-agnostic events | ~800 строк |
| `core/codex.rs` | Адаптация оркестрации (7519 строк) | ~1500 строк |
| `core/tools/spec.rs` | Переработка генерации ToolSpec | ~600 строк |
| `core/compact.rs` | Адаптация (compaction — OpenAI-specific) | ~400 строк |
| Новые: `codex-anthropic/` | Реализация trait для Anthropic | ~2000 строк |
| Новые: `codex-gemini/` | Реализация trait для Gemini | ~2000 строк |
| Тесты | Обновление всех тестов | ~3000 строк |

**Итого:** ~17,000+ строк кода (создание/изменение)

**Сроки:** 2-3 месяца для опытного Rust-разработчика.

**Риски:**
- `codex.rs` (7519 строк) — крайне сложный для рефакторинга файл
- Глубокая вложенность serde-сериализации — изменение типов ломает сериализацию
- Compaction (суммаризация контекста) привязан к `/v1/responses/compact` — нужен альтернативный механизм
- Reasoning (`encrypted_content`) — OpenAI-specific, у других провайдеров этого нет
- 84 крейта — изменения в protocol/types каскадно затрагивают многие крейты
- Обширные тесты (сотни тестов с `model_providers.mock_provider`) нужно переписать

### 7.4 Вариант D: Fork + Выборочная адаптация

**Что делаем:** Fork'аем Codex, добавляем поддержку конкретных провайдеров через модификацию `ModelProviderInfo` + кастомные эндпоинты.

**Объём:** Зависит от провайдера. Для Anthropic: 2-4 недели.

**Риски:** Сложность мержа с upstream.

---

## 8. Сводная таблица вариантов

| Вариант | Объём | Сроки | Anthropic | Gemini | Ollama | Сложность поддержки |
|---|---|---|---|---|---|---|
| A. Как есть | Мин. | 1-2 дня | Нет | Нет | Да | Низкая |
| B. Прокси | Средний | 2-4 нед | Да | Да | Да | Средняя |
| C. Полный refactor | Высокий | 2-3 мес | Да | Да | Да | Низкая (идеальный вариант) |
| D. Fork + патч | Средний | 2-4 нед | Частично | Нет | Да | Высокая |

---

## 9. Рекомендации для Logos

### 9.1 Краткосрочная стратегия (MVP)

**Вариант A + B: Использовать Codex как есть + Translation Proxy**

1. Запустить Codex с Ollama/LM Studio для локальных моделей
2. Создать легковесный прокси-сервер (Node.js или Rust), который транслирует Responses API ↔ Anthropic API
3. Добавить конфигурацию в `config.toml` с `base_url`, указывающим на прокси

### 9.2 Среднесрочная стратегия

**Вариант C: Provider trait в Rust**

Если Codex станет основой Logos:
1. Создать `LlmProvider` trait в новом крейте `codex-provider`
2. Перенести текущую логику OpenAI в `OpenAiProvider` реализацию
3. Внедрить provider-agnostic типы в `protocol`
4. Добавить реализации для Anthropic и Gemini
5. Адаптировать оркестрацию в `core/codex.rs`

### 9.3 Альтернатива: Chaemera + свой движок

Учитывая, что Chaemera — это Tauri-приложение (Rust + JS frontend), возможно более эффективно:
1. Использовать Codex **как референсную архитектуру** (концепции агентов, инструментов, sandbox)
2. Написать свой LLM-движок с provider-agnostic дизайном с нуля
3. Интегрировать через Tauri IPC

---

## 10. Специфические сложности по провайдерам

### 10.1 Anthropic (Claude)

| Аспект | OpenAI Responses API | Anthropic Messages API | Сложность адаптации |
|---|---|---|---|
| Endpoint | `/v1/responses` | `/v1/messages` | Низкая (URL) |
| Формат запроса | `input: Vec<ResponseItem>` | `messages: Vec<Message>` | Средняя |
| Системные инструкции | `instructions` (строка) | `system` (строка/блоки) | Низкая |
| Инструменты | `tools` (встроенные типы) | `tools` (свой формат) | Средняя |
| Tool choice | `"auto"` | `{"type": "auto"}` | Низкая |
| Streaming | SSE с delta-событиями | SSE с `content_block_delta` | Средняя |
| Reasoning | `reasoning.encrypted_content` | `thinking` blocks | Высокая (разные модели) |
| Local Shell | Встроенный tool type | Нет аналога → function tool | Средняя |
| Web Search | Встроенный tool type | Нет аналога | Высокая (невозможно) |
| Parallel tool calls | Да | Да (beta) | Низкая |

### 10.2 Google Gemini

| Аспект | Сложность |
|---|---|
| API формат | Высокая (protobuf/REST, другая структура) |
| Function calling | Средняя (схожая концепция, другой JSON) |
| Streaming | Средняя (SSE, другая структура событий) |
| Thinking/reasoning | Низкая (нет encrypted_content) |

### 10.3 Особенности OpenAI, которые невозможно перенести

- **`LocalShellCall`** — встроенный tool type Responses API. Другие провайдеры не поддерживают shell execution как встроенный тип.
- **`encrypted_content`** для reasoning — проприетарная функция OpenAI.
- **`WebSearchCall` / `ImageGenerationCall`** — встроенные типы, недоступные у других провайдеров.
- **`/v1/responses/compact`** — endpoint для сжатия контекста.
- **`ToolSearchCall`** — динамический поиск инструментов.

---

## 11. Ключевые файлы для референса

| Файл | Назначение | Строк |
|---|---|---|
| `codex-rs/core/src/codex.rs` | Оркестрация сессии/turn'а | 7519 |
| `codex-rs/core/src/client.rs` | ModelClient, транспорт | ~1300 |
| `codex-rs/core/src/model_provider_info.rs` | Провайдеры (данные) | 356 |
| `codex-rs/core/src/client_common.rs` | ToolSpec, ResponsesApiRequest | ~500 |
| `codex-rs/protocol/src/models.rs` | ResponseItem (API-типы) | 2772 |
| `codex-rs/protocol/src/protocol.rs` | Event, Submission (протокол) | 3600 |
| `codex-rs/protocol/src/openai_models.rs` | ModelInfo (метаданные) | 785 |
| `codex-rs/codex-api/src/endpoint/` | API-клиенты (SSE + WS) | ~2000 |
| `codex-rs/codex-api/src/sse/` | SSE-парсинг | ~500 |
| `codex-rs/core/src/tools/spec.rs` | Генерация ToolSpec | ~2900 |
| `codex-rs/core/src/tools/registry.rs` | ToolHandler trait, ToolRegistry | ~500 |
| `codex-rs/core/src/tools/router.rs` | Диспетчеризация инструментов | ~250 |
| `codex-rs/core/models.json` | Встроенные модели | JSON |
| `codex-rs/core/config.schema.json` | JSON-схема конфигурации | 2577 |

---

## 12. Вывод

Codex CLI — это **высококачественный, но глубоко привязанный к OpenAI** продукт. Его архитектура делает добавление "истинной" model-agnostic поддержки **возможным, но трудоёмким**:

1. **Нет provider trait** — все провайдеры описываются данными (`ModelProviderInfo`), не поведением
2. **Внутренние типы = wire format** — `ResponseItem`, `ToolSpec`, `ResponsesApiRequest` напрямую сериализуются в OpenAI JSON
3. **OpenAI-специфичные features** — LocalShell, WebSearch, encrypted reasoning, compact endpoint
4. **84 крейта** — изменения в фундаментальных типах каскадно распространяются

**Для Logos рекомендуется:**
- MVP: Translation Proxy (Вариант B) — быстрый старт, поддержка Anthropic
- Production: Provider trait (Вариант C) или собственный движок по мотивам Codex

**Объём реальной работы для полного model-agnostic:** ~17,000+ строк Rust, 2-3 месяца.
