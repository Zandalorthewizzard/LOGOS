Все 16 анализов OpenCode завершены. Итоговая структура в docs-new/07-logos-phase/research/session layer/:
INDEX.md                              ← прогресс (16/16 OpenCode done)
task-plan.md                          ← план задач
RA-1.1-thread-manager.md              ✓ OpenCode analyzed
RA-1.2-session-persistence.md         ✓ OpenCode analyzed
RA-1.3-fork-resume.md                 ✓ OpenCode analyzed
RA-1.4-turn-lifecycle-notifications.md ✓ OpenCode analyzed
RA-1.5-shell-execution.md             ✓ OpenCode analyzed
RA-1.6-review-orchestration.md        ✓ OpenCode analyzed
RA-1.7-archive-metadata.md            ✓ OpenCode analyzed
RA-2.1-rollback.md                    ✓ OpenCode analyzed
RA-2.2-compaction.md                  ✓ OpenCode analyzed
RA-2.3-turn-interrupt.md              ✓ OpenCode analyzed
RA-2.4-background-terminals.md        ✓ OpenCode analyzed
RA-3.1-turn-steering.md               ✓ OpenCode analyzed
RA-3.2-advanced-compaction.md         ✓ OpenCode analyzed
RA-3.3-multi-provider-handoff.md      ✓ OpenCode analyzed
RA-3.4-streaming-optimization.md      ✓ OpenCode analyzed
SUMMARY.md                            ← not yet created (after Goose)
Ключевые находки из OpenCode:
- Event-sourced persistence (SQLite + SyncEvent) значительно надёжнее JSON snapshot
- Runner-based concurrency guard — простой и эффективный паттерн
- Двухфазный compaction (prune + summarize) — практичный подход
- Soft-delete архивация лучше file-move
- Provider resolution per-turn (не per-session) — более гибкая модель
- Doom loop detection, partial state на interrupt, tool output pruning — конкретные паттерны для заимствования
Goose placeholders оставлены во всех документах для заполнения когда исходник будет доступен.
▣  Build · glm-5.1 · 23m 22s
расскажи больше и простыми словами по каждой из ключевых находок
Thinking: The user wants me to explain each key finding from the OpenCode analysis in simple, plain language. Let me go through each one.
1. Event-sourced persistence (SQLite + SyncEvent)
Как делает OpenCode: Любое изменение сессии — не просто "перезаписать файл". Это событие. Пользователь переименовал поток? Это событие session.updated. Модель вернула ответ? Событие message.updated. Каждое событие:
- Записывается в SQLite в транзакции (атомарно — либо всё сохранилось, либо ничего)
- Обновляет "читаемую модель" (таблицы, по которым можно делать запросы)
- Публикуется в шину событий (чтобы UI обновился)
Почему это лучше JSON snapshot: Текущий LOGOS OR при каждом сохранении перезаписывает весь JSON файл целиком. Если процесс упал посреди записи — файл повреждён, сессия потеряна. Event sourcing = дописываем одну строку в лог. Упали посреди? Последнее событие не записалось, но всё до него цело. Аналогия: дневник где ты дописываешь страницу каждый день (event sourcing), vs блокнот где ты стираешь и переписываешь всё с начала каждый раз (JSON snapshot).
---
2. Runner-based concurrency guard
Проблема: Что если пользователь отправил запрос, модель думает, и пользователь отправляет ещё один? Или два клиента подключены к одному потоку и оба шлют turn/start?
Как делает OpenCode: У каждого потока есть Runner — объект который знает "занят" или "свободен". Если поток занят — новый запрос получает ошибку BusyError. Всё. Никаких race conditions, никаких "два хода параллельно".
Текущая ситуация в LOGOS OR: Нет защиты. Если два turn/start придут одновременно — spawned tasks выполнятся параллельно, история может разъехаться, уведомления перемешаются. Это бомба замедленного действия.
---
3. Двухфазный compaction (prune + summarize)
Проблема: История разговора растёт. У модели есть лимит контекстного окна (например, 128K токенов). Когда история не влезает — нужно сжать.
Фаза 1 — Pruning (обрезка): OpenCode проходит по истории и обрезает старые результаты инструментов. Модель вызвала read file 20 минут назад? Результат замещается на [Old tool result content cleared]. Но последние 40K токенов tool outputs защищены — они свежие и нужные. Это бесплатно (не требует вызова модели) и часто этого достаточно.
Фаза 2 — Summarization (суммаризация): Если после pruning всё ещё не влезает — отправляем историю модели с промптом "сделай краткое summary". Модель возвращает сжатую версию. Старая история заменяется на summary. Это стоит денег (вызов модели), но сжимает сильно.
Хитрость: Используется отдельный compaction агент с более дешёвой моделью. Не нужно использовать GPT-4 для summarization — достаточно маленькой быстрой модели.
---
4. Soft-delete архивация
Как делает LOGOS сейчас: Архивация = физически переместить JSON файл из active-threads/ в archived-threads/. Разархивация = переместить обратно. Проблемы: (1) неатомарно — может сломаться на полпути, (2) нельзя искать по архивам, (3) нужны две директории, (4) "а в каком каталоге этот поток?"
Как делает OpenCode: Архивация = поставить timestamp в поле time_archived. Всё. Поток остаётся в той же таблице SQLite. При листинге archived потоки просто фильтруются по наличию timestamp. Разархивация = очистить timestamp. Аналогия: вместо того чтобы убирать письмо в шкаф (file move), ты просто помечаешь его "архив" галочкой (timestamp). Проще, быстрее, обратимо, searchable.
---
5. Provider resolution per-turn (не per-session)
Как делает LOGOS сейчас: Поток создаётся с привязкой к провайдеру. thread/start → модель OpenAI → весь поток живёт через OpenAI. Хочешь другую модель? Создавай новый поток.
Как делает OpenCode: Каждый отдельный ход записывает какой provider/model его сгенерировал. Следующий ход может использовать другого provider/model. История целиком пересылается новому провайдеру — ему всё равно, кто генерировал предыдущие сообщения.
Почему это важно: Если OpenAI падает — пользователь может продолжить разговор через OpenRouter без потери контекста. Если хочется попробовать как Claude справится с той же задачей — переключился mid-conversation. LOGOS сейчас не может этого потому что поток привязан к провайдеру при создании.
---
6. Doom loop detection
Проблема: Иногда модель зацикливается — вызывает один и тот же инструмент с одними и теми же аргументами раз за разом. Например, read_file("foo.ts") → результат → read_file("foo.ts") → результат → read_file("foo.ts")... Бесконечный цикл, жжёт токены и деньги.
Как делает OpenCode: Если последние 3 вызова инструментов одинаковые (тот же инструмент, те же аргументы) — система останавливается и спрашивает пользователя "продолжить?" Простой счётчик, дешёвая проверка, спасает от дорогих зацикливаний.
Текущая ситуация в LOGOS OR: Нет защиты. Модель может зациклиться и крутиться пока не кончатся токены или терпение пользователя.
---
7. Partial state preservation on interrupt
Проблема: Пользователь нажимает "стоп" пока модель ещё генерирует ответ. Что делать с тем что уже сгенерировалось?
Как делает LOGOS Codex: Прерывает, выбрасывает всё. TurnAborted. Половина ответа потеряна.
Как делает OpenCode: Прерывает, но сохраняет то что уже получено:
- Незавершённый текст финализируется (добавляется "...")
- Незавершённые tool calls помечаются как "aborted"
- Ассистент-сообщение сохраняется в историю с пометкой что оно было прервано
- Пользователь видит что модель успела сгенерировать
Почему это лучше: Пользователь не теряет работу. Если модель уже 30 секунд генерировала полезный анализ — пользователь хочет его видеть, даже если нажал "стоп" досрочно.
---
8. Tool output pruning с protection window
Это расширение пункта 3, но заслуживает отдельного объяснения.
Ключевое наблюдение: Основной источник раздувания контекста — не текстовые сообщения, а результаты выполнения инструментов. Модель прочитала файл на 500 строк? Это 500 строк в контексте. Выполнила команду которая вернула 1000 строк логов? Это 1000 строк. За 10 ходов может накопиться 50K+ токенов только в tool outputs.
Как делает OpenCode: Идёт по истории с конца. Последние 40K токенов tool outputs — защищены (свежие, нужные). Всё что старше — помечается как compacted и заменяется на [Old tool result content cleared]. Модель видит что инструмент был вызван, но не видит старый результат. Часто этого достаточно — старые результаты уже не актуальны.
Почему это умно: Это бесплатно (не нужен вызов модели), быстрое (простой проход по массиву), и часто решает проблему без summarization.
▣  Build · glm-5.1 · 58.4s
а как тригерится самаризация? (сжатие) Да есть команда, но если контекст забивается во время работы модели, какие автоматические механизымы? 
Thinking: The user is asking about automatic triggering mechanisms for compaction/summarization in both OpenCode and LOGOS Codex. Let me look at what I know from the research.
From the analysis:
Codex (LOGOS):
- Auto-compaction triggered pre-turn when token usage exceeds auto_compact_token_limit (codex.rs:5735)
- Mid-turn compaction triggered during a turn when token limit reached (codex.rs:6042)
OpenCode:
- Overflow detection in session/overflow.ts: checks if total tokens exceed usable context
- Checked after finish-step in processor (processor.ts line 307)
- Checked in runLoop (prompt.ts line 1413)
- COMPACTION_BUFFER = 20,000 tokens reserved for compaction output
Let me explain the mechanisms in simple terms.
Codex (OpenAI path)
Два автоматических триггера:
1. Pre-turn auto-compact (codex.rs:5735): Перед каждым новым ходом проверяется: "сколько токенов уже использовано?" Если превышает auto_compact_token_limit — запускается compact до того как отправить запрос модели. Логика: лучше сжать сейчас, чем получить ошибку "context window exceeded" от OpenAI.
2. Mid-turn compaction (codex.rs:6042): Внутри хода, в цикле "prompt → response → tool call → execute → prompt → ...", после каждого шага проверяется: "влезет ли следующий запрос в контекст?" Если нет — compact прямо посреди хода. После compact previous_response_id сбрасывается в None (история изменилась, серверный указатель больше не валиден).
---
OpenCode
Три уровня проверки:
1. После каждого шага (processor.ts:307): Модель завершила step (вызвала инструменты, получила результаты) → проверяется isOverflow() → если да, compact запускается перед следующим шагом.
2. В начале каждой итерации runLoop (prompt.ts:1413): Перед тем как отправить запрос модели → проверяется isOverflow() → если да, compact запускается первым.
3. Формула overflow (overflow.ts):
usable = model.limit.input - reserved_tokens
overflow = current_token_count >= usable
Зарезервировано 20,000 токенов — это буфер на сам compact (summary тоже занимает токены, и модель должна уместить ответ).
---
Как это выглядит на практике
Ход 1: 5K токенов — норм
Ход 2: 15K токенов — норм
Ход 3: 30K токенов — норм
Ход 4: 60K токенов — норм
Ход 5: 90K токенов — норм
Ход 6: 110K токенов — норм
  └─ модель вызывает read_file(big_file.ts) → +20K
  └─ после tool execution: 130K токенов
  └─ проверка: 130K >= (128K - 20K) = 108K? ДА
  └─ → AUTO COMPACT
       └─ pruning: обрезаны старые tool outputs → 95K
       └─ всё ещё > 108K? НЕТ → compact не нужен, pruning хватило
       └─ продолжаем ход с 95K токенов
Или вариант где pruning не помог:
  └─ после pruning: 115K, всё ещё > 108K
  └─ → запускается summarization через compaction agent
  └─ модель генерирует summary: "Пользователь работал над X, сделал Y, текущее состояние Z"
  └─ старая история заменяется на summary → 30K токенов
  └─ continuation handle сброшен
  └─ ход продолжается с 30K токенов
---
Что важно для LOGOS OR
Сейчас ничего из этого нет. OR путь вообще не считает токены, не проверяет overflow, не делает auto-compact. Если контекст превысит лимит модели — OpenRouter просто вернёт ошибку, и ход упадёт.
Приоритет реализации:
1. Token counting — без него ничего не работает
2. Pre-turn overflow check — самый простой и надёжный триггер
3. Tool output pruning — дешёвая первая линия защиты
4. Auto-compact через summarization — тяжёлая артиллерия когда pruning не помог
