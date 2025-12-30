# VFDir - Refactoring Documentation

## Дата рефакторинга: 2025-12-22

---

## Цель рефакторинга

Исходный файл `App.vue` вырос до ~600+ строк кода и стал сложным для поддержки. Проведен рефакторинг для разделения ответственности и улучшения читаемости кода.

---

## Новая архитектура

### Структура composables:

```
src/composables/
├── useFileSystem.ts        # Низкоуровневые операции с файлами (Tauri API)
├── useNavigation.ts        # Навигация и tabs
├── useSelection.ts         # Выделение файлов
├── useSearch.ts            # Поиск и фильтрация
├── useDragDrop.ts          # Drag & drop
├── useKeyboard.ts          # Обработка горячих клавиш
├── useClipboard.ts         # Буфер обмена
├── useNotifications.ts     # Toast уведомления
├── useDialogs.ts           # ✨ НОВЫЙ: Управление диалогами
├── useFileOperations.ts    # ✨ НОВЫЙ: Высокоуровневые файловые операции
└── useCommands.ts          # ✨ НОВЫЙ: Команды Command Palette
```

### Структура utils:

```
src/utils/
└── shortcuts.ts            # ✨ НОВЫЙ: Конфигурация горячих клавиш
```

---

## Новые composables

### 1. **useDialogs.ts**

**Ответственность:** Централизованное управление всеми диалогами

**API:**
```typescript
const {
  // Confirm Dialog
  confirmDialog,
  showConfirm,
  closeConfirm,

  // Properties Dialog
  propertiesDialog,
  showProperties,
  closeProperties,

  // Input Dialog
  inputDialog,
  showInput,
  closeInput,
} = useDialogs();
```

**Преимущества:**
- ✅ Единое место для управления состоянием диалогов
- ✅ Простое API для показа/скрытия диалогов
- ✅ Типобезопасность

---

### 2. **useFileOperations.ts**

**Ответственность:** Высокоуровневые операции с файлами с обработкой ошибок и уведомлениями

**API:**
```typescript
const fileOps = useFileOperations();

// Методы:
fileOps.getCurrentDirectory(currentPath)
fileOps.handleCopy(selectedItems)
fileOps.handleCut(selectedItems)
fileOps.handlePaste(currentPath)
fileOps.handleDelete(selectedItems, currentPath, clearSelection, showConfirm)
fileOps.handleRename(selectedItems, currentPath, showInput)
fileOps.handleNewFolder(currentPath, showInput)
fileOps.handleOpenFile(file)
fileOps.handleProperties(selectedItems, showProperties)
fileOps.handleRevealInFinder(selectedItems)
fileOps.handleRefresh(currentPath)
```

**Преимущества:**
- ✅ Инкапсуляция бизнес-логики операций
- ✅ Встроенная обработка ошибок и уведомлений
- ✅ Автоматическое обновление директории после операций
- ✅ Переиспользуемый код

**Что делает:**
- Комбинирует низкоуровневые API (useFileSystem)
- Добавляет уведомления (useNotifications)
- Управляет диалогами (useDialogs через callbacks)
- Обрабатывает ошибки

---

### 3. **useCommands.ts**

**Ответственность:** Обработка команд из Command Palette

**API:**
```typescript
const commands = useCommands({
  onNewFolder: () => {},
  onNewFile: () => {},
  onSearch: () => {},
  onGoto: () => {},
  onRefresh: async () => {},
  onCopyPath: (selectedItems) => {},
  onSelectAll: (allFiles) => {},
  onNewTab: () => {},
  onCloseTab: () => {},
  onSettings: () => {},
});

// Выполнение команды
commands.executeCommand({ id: 'new-folder' });

// Специфичные команды
commands.copyPathCommand(selectedItems);
commands.selectAllCommand(allFiles, selectAll);
commands.closeTabCommand(tabsCount, closeTab, activeTabId);
```

**Преимущества:**
- ✅ Централизованная логика команд
- ✅ Легко добавлять новые команды
- ✅ Разделение UI и бизнес-логики

---

### 4. **shortcuts.ts** (Utils)

**Ответственность:** Конфигурация горячих клавиш

**API:**
```typescript
const shortcuts = createKeyboardShortcuts(
  {
    openCommandPalette: () => {},
    closeDialogs: () => {},
    selectAll: (files) => {},
    addTab: () => {},
    closeTab: (canClose) => {},
    goUp: (canGoUp) => {},
    handleCopy: () => {},
    handleCut: () => {},
    handlePaste: () => {},
    handleDelete: () => {},
    handleRename: () => {},
    handleRefresh: () => {},
    handleNewFolder: () => {},
  },
  () => files.value
);

useKeyboard(shortcuts);
```

**Преимущества:**
- ✅ Декларативная конфигурация
- ✅ Легко изменять/добавлять shortcuts
- ✅ Отделение от основной логики App.vue

---

## Результат рефакторинга

### До:
- **App.vue:** ~600 строк
- **Сложность:** Высокая
- **Поддерживаемость:** Низкая
- **Переиспользуемость:** Низкая

### После:
- **App.vue:** ~340 строк (↓43%)
- **useDialogs.ts:** ~100 строк
- **useFileOperations.ts:** ~230 строк
- **useCommands.ts:** ~80 строк
- **shortcuts.ts:** ~80 строк
- **Сложность:** Низкая
- **Поддерживаемость:** Высокая
- **Переиспользуемость:** Высокая

---

## Новая структура App.vue

```vue
<script setup lang="ts">
// Imports (composables, components, utils)

// 1. Composables initialization (~40 строк)
// 2. Local state (~5 строк)
// 3. Computed values (~2 строки)
// 4. Event handlers (~40 строк)
// 5. Command handlers (~30 строк)
// 6. Keyboard shortcuts setup (~20 строк)
// 7. Context menu handlers (~15 строк)
// 8. Watchers (~5 строк)
// 9. Lifecycle hooks (~3 строки)
</script>

<template>
  <!-- UI компоненты -->
</template>
```

---

## Преимущества новой архитектуры

### 1. **Разделение ответственности (SoC)**
- Каждый composable отвечает за одну область
- Легко найти код для конкретной функции
- Проще тестировать

### 2. **Переиспользуемость**
- `useDialogs` можно использовать в других компонентах
- `useFileOperations` инкапсулирует всю логику операций
- `createKeyboardShortcuts` легко настраивать

### 3. **Читаемость**
- App.vue теперь читается как "оркестратор"
- Бизнес-логика скрыта в composables
- Явная структура

### 4. **Масштабируемость**
- Легко добавлять новые операции в useFileOperations
- Легко добавлять новые команды в useCommands
- Легко добавлять новые диалоги в useDialogs

### 5. **Тестируемость**
- Каждый composable можно тестировать отдельно
- Меньше зависимостей
- Мокирование проще

---

## Миграция и обратная совместимость

- ✅ Старый файл сохранен как `App.vue.old`
- ✅ Все функции сохранены
- ✅ API не изменился
- ✅ Компоненты работают как раньше

---

## Дальнейшие улучшения

### Потенциальные рефакторинги:

1. **Разбить template на sub-компоненты:**
   - `<MenuBar>`
   - `<MainLayout>`
   - `<DialogsContainer>`

2. **Создать composable useAppState:**
   ```typescript
   const { viewMode, isCommandPaletteOpen, contextMenu, previewFile } = useAppState();
   ```

3. **Вынести типы в отдельные файлы:**
   - `types/dialogs.ts`
   - `types/commands.ts`
   - `types/shortcuts.ts`

4. **Unit тесты для composables:**
   - `useDialogs.test.ts`
   - `useFileOperations.test.ts`
   - `useCommands.test.ts`

---

## Чеклист для добавления новой функции

### Добавление новой операции с файлами:
1. ✅ Добавить low-level метод в `useFileSystem.ts` (если нужно)
2. ✅ Добавить high-level метод в `useFileOperations.ts`
3. ✅ Добавить обработчик в `App.vue`
4. ✅ Добавить в контекстное меню (если нужно)
5. ✅ Добавить горячую клавишу в `shortcuts.ts` (если нужно)

### Добавление новой команды:
1. ✅ Добавить команду в `CommandPalette.vue`
2. ✅ Добавить обработчик в `useCommands.ts`
3. ✅ Подключить в `App.vue`

### Добавление нового диалога:
1. ✅ Создать компонент диалога
2. ✅ Добавить состояние в `useDialogs.ts`
3. ✅ Добавить методы show/close
4. ✅ Импортировать и использовать в `App.vue`

---

## Заключение

Рефакторинг значительно улучшил структуру проекта:
- **Код стал чище и понятнее**
- **Поддержка стала проще**
- **Добавление новых функций стало быстрее**
- **Тестирование стало возможным**

Все изменения обратно совместимы, функциональность не изменилась.
