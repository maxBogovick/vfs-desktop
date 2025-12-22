# VFDir - Windows XP File Explorer для macOS

Современный файловый менеджер с аутентичным дизайном Windows XP, созданный с помощью Tauri, Vue 3, TypeScript и Tailwind CSS.

![Windows XP Style](https://img.shields.io/badge/style-Windows%20XP-blue)
![Vue 3](https://img.shields.io/badge/Vue-3.5-green)
![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)

## ✨ Особенности

- 🎨 **Аутентичный Windows XP дизайн** - классическая Luna тема
- 📁 **Мультитабы** - работа с несколькими папками одновременно
- 🔍 **Расширенный поиск** - фильтры по типу, размеру, дате, тегам
- ⌨️ **Горячие клавиши** - быстрая навигация и операции
- 🎯 **Command Palette** (Ctrl+K) - быстрый доступ к командам
- 👆 **Drag & Drop** - перетаскивание файлов
- 📋 **Smart Selection** - одиночное, множественное, диапазонное выделение
- 🔙 **История навигации** - Back/Forward как в браузере
- 👁️ **Preview панель** - быстрый просмотр свойств файлов
- 🏷️ **Теги** - организация файлов с цветными метками

## 🚀 Быстрый старт

### Требования

- Node.js 18+
- Rust 1.70+
- macOS 11+

### Установка

```bash
# Установить зависимости
npm install

# Запустить в dev режиме
npm run dev

# Собрать приложение
npm run tauri build
```

## ⌨️ Горячие клавиши

| Комбинация | Действие |
|-----------|----------|
| `Ctrl+K` | Открыть Command Palette |
| `Ctrl+A` | Выделить все |
| `Ctrl+T` | Новая вкладка |
| `Ctrl+W` | Закрыть вкладку |
| `Backspace` | Перейти на уровень выше |
| `Escape` | Закрыть диалоги / Снять выделение |
| `Enter` | Открыть файл/папку |
| `F2` | Переименовать |
| `Delete` | Удалить |
| `Ctrl+Click` | Множественное выделение |
| `Shift+Click` | Диапазонное выделение |

## 🏗️ Архитектура

Проект использует современную модульную архитектуру:

```
src/
├── types/              # TypeScript типы
│   └── index.ts
├── composables/        # Переиспользуемая логика
│   ├── useFileSystem.ts   # Файловые операции
│   ├── useNavigation.ts   # Навигация и табы
│   ├── useSelection.ts    # Выделение файлов
│   ├── useSearch.ts       # Поиск и фильтрация
│   ├── useDragDrop.ts     # Drag & Drop
│   └── useKeyboard.ts     # Горячие клавиши
├── components/         # Vue компоненты
│   ├── Toolbar.vue
│   ├── Sidebar.vue
│   ├── FileList.vue
│   ├── Preview.vue
│   ├── CommandPalette.vue
│   └── ContextMenu.vue
└── App.vue            # Главный компонент
```

### Composables

Вся бизнес-логика вынесена в переиспользуемые composables:

**useFileSystem** - работа с файловой системой через Tauri
```typescript
const { files, loadDirectory, deleteItem, renameItem, createFolder } = useFileSystem();
```

**useNavigation** - навигация, табы, история
```typescript
const { currentPath, goBack, goForward, addTab, navigateInto } = useNavigation();
```

**useSelection** - выделение файлов
```typescript
const { selectedIds, handleItemClick, selectAll, clearSelection } = useSelection();
```

**useSearch** - поиск и фильтрация
```typescript
const { searchQuery, processFiles, addFileTypeFilter, setSorting } = useSearch();
```

**useDragDrop** - перетаскивание
```typescript
const { startDrag, handleDrop, isDragTarget } = useDragDrop();
```

См. [ARCHITECTURE.md](./ARCHITECTURE.md) для подробной документации.

## 🔌 Tauri Integration

Для подключения реальной файловой системы:

1. Раскомментируйте реальную реализацию в `src/composables/useFileSystem.ts`
2. Добавьте Tauri команды в Rust backend (см. комментарии в файле)

## 🎨 Режимы отображения

- **Grid View** (⊞) - иконки в сетке
- **List View** (☰) - компактный список
- **Details View** - таблица с подробной информацией

## 📝 TODO

- [ ] Реализовать Tauri backend команды
- [ ] Добавить операции с файлами (copy, cut, paste, delete)
- [ ] Реализовать переименование файлов
- [ ] Добавить систему тегов
- [ ] Сохранение настроек
- [ ] Альтернативные темы (Classic, Royale)
- [ ] Preview для изображений, PDF, текста
- [ ] Favorites/Bookmarks
- [ ] Продвинутый фильтр UI

## 🤝 Разработка

```bash
# Запустить dev сервер
npm run dev

# Type checking
npm run build

# Собрать Tauri приложение
npm run tauri build
```

## 💡 Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 📖 Документация

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Архитектура проекта
- [Vue 3 Docs](https://vuejs.org/)
- [Tauri Docs](https://tauri.app/)
- [Tailwind CSS](https://tailwindcss.com/)

## 📄 License

MIT

## 🙏 Благодарности

Вдохновлено классическим дизайном Windows XP File Explorer.
