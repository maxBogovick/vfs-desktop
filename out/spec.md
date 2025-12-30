╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
План реализации Dual-Panel режима (как в Total Commander)

Обзор

Реализация двухпанельного режима для файлового менеджера VFDir с возможностью переключения между single/dual режимами, независимыми вкладками для каждой панели, resizable
разделителем и полным сохранением состояния.

Требования

- ✅ Двухпанельный режим (две независимые файловые панели)
- ✅ Скрытие Sidebar и Preview в dual-режиме
- ✅ Отдельные вкладки для каждой панели
- ✅ Кнопка переключения в Toolbar (справа, рядом с режимами просмотра)
- ✅ Resizable вертикальный разделитель (50/50 по умолчанию)
- ✅ Сохранение: режим, размеры, табы, пути для каждой панели
- ✅ Восстановление состояния при запуске

Архитектурное решение

Новые компоненты:

1. useDualPanel.ts - composable для управления состоянием dual-panel
2. DualPanelContainer.vue - контейнер с resizable разделителем
3. FilePanel.vue - компонент одной файловой панели
4. PanelToolbar.vue - компактная панель табов для одной панели

Модифицируемые файлы:

- Backend: config.rs - добавление структур данных
- Types: types/index.ts - TypeScript интерфейсы
- Composables: useNavigation.ts - версия для работы с внешними refs
- Components: Toolbar.vue, App.vue - интеграция

Пошаговый план реализации

Шаг 1: Backend (Rust) - Структуры данных

Файл: src-tauri/src/config.rs

Добавить после строки 25:

// Режим отображения панелей
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PanelMode {
Single,
Dual,
}

impl Default for PanelMode {
fn default() -> Self {
PanelMode::Single
}
}

// Состояние одной панели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
pub tabs: Vec<TabState>,
pub active_tab_id: Option<u64>,
}

impl Default for PanelState {
fn default() -> Self {
Self {
tabs: vec![],
active_tab_id: None,
}
}
}

// Конфигурация dual-panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualPanelConfig {
#[serde(default = "default_panel_split")]
pub left_panel_width_percent: u32,

     #[serde(default)]
     pub left_panel: PanelState,

     #[serde(default)]
     pub right_panel: PanelState,

     #[serde(default = "default_active_panel")]
     pub active_panel: String,
}

fn default_panel_split() -> u32 { 50 }
fn default_active_panel() -> String { "left".to_string() }

impl Default for DualPanelConfig {
fn default() -> Self {
Self {
left_panel_width_percent: 50,
left_panel: PanelState::default(),
right_panel: PanelState::default(),
active_panel: "left".to_string(),
}
}
}

Модифицировать UIState (строка 88):

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIState {
#[serde(default = "default_sidebar_width")]
pub sidebar_width: u32,

     #[serde(default = "default_preview_width")]
     pub preview_width: u32,

     // Single-mode state
     #[serde(default)]
     pub tabs: Vec<TabState>,

     #[serde(default)]
     pub active_tab_id: Option<u64>,

     #[serde(default)]
     pub last_path: Option<Vec<String>>,

     // Dual-panel state (НОВЫЕ поля)
     #[serde(default)]
     pub panel_mode: PanelMode,

     #[serde(default)]
     pub dual_panel_config: DualPanelConfig,

     #[serde(default)]
     pub window: WindowState,

     #[serde(default)]
     pub sidebar: SidebarState,
}

Важно: #[serde(default)] обеспечивает обратную совместимость со старыми конфигами.

Запустить cargo build для проверки компиляции.

 ---
Шаг 2: Frontend Types

Файл: src/types/index.ts

Добавить после строки 132:

export type PanelMode = 'single' | 'dual';
export type ActivePanel = 'left' | 'right';

export interface PanelState {
tabs: TabState[];
active_tab_id?: number;
}

export interface DualPanelConfig {
left_panel_width_percent: number;
left_panel: PanelState;
right_panel: PanelState;
active_panel: ActivePanel;
}

Модифицировать UIState interface (строка 134):

export interface UIState {
sidebar_width: number;
preview_width: number;
tabs: TabState[];
active_tab_id?: number;
last_path?: string[];
window: WindowState;
sidebar: SidebarState;

// НОВЫЕ поля
panel_mode: PanelMode;
dual_panel_config: DualPanelConfig;
}

 ---
Шаг 3: Core Composable - useDualPanel

Файл: src/composables/useDualPanel.ts (НОВЫЙ)

Создать composable с module-level shared state для управления dual-panel режимом:

Основной state:
- panelMode: Ref<PanelMode> - текущий режим
- leftPanelWidthPercent: Ref<number> - ширина левой панели (%)
- activePanel: Ref<ActivePanel> - активная панель
- leftPanelTabs, rightPanelTabs: Ref<Tab[]> - табы каждой панели
- leftPanelActiveTabId, rightPanelActiveTabId: Ref<number | undefined>

Ключевые методы:
- togglePanelMode() - переключить single/dual
- switchActivePanel(panel) - переключить активную панель
- setPanelSplit(percent) - изменить размер панелей (ограничение 20-80%)
- loadDualPanelState(config) - восстановить состояние из конфига
- serializeDualPanelState() - сериализовать для сохранения

Computed properties:
- isDualMode - boolean
- activePanelTabs - табы активной панели
- activePanelTabId - ID активного таба
- activePanelPath - текущий путь активной панели

 ---
Шаг 4: useNavigation расширение

Файл: src/composables/useNavigation.ts

Добавить в конец файла версию для работы с внешними refs:

export function useNavigationWithRefs(
tabs: Ref<Tab[]>,
activeTabId: Ref<number | undefined>
) {
// Та же логика что в useNavigation, но работает с переданными refs
// Это позволяет каждой панели иметь независимую навигацию
}

 ---
Шаг 5: UI Components

5.1. PanelToolbar.vue (НОВЫЙ)

Файл: src/components/PanelToolbar.vue

Компактная панель табов для одной панели:
- Отображение табов с возможностью переключения и закрытия
- Кнопка добавления нового таба
- Минимальная высота (чтобы сэкономить место)

5.2. FilePanel.vue (НОВЫЙ)

Файл: src/components/FilePanel.vue

Компонент одной файловой панели:
- Использует PanelToolbar для табов
- Использует FileList для отображения файлов
- Использует useNavigationWithRefs для независимой навигации
- Имеет свой экземпляр useSelection и useFileSystem
- Визуальное выделение активной панели (ring-2 ring-blue-500)
- Emit 'activate' при клике для переключения активности

Props:
- panelId: 'left' | 'right'
- isActive: boolean
- tabs: Tab[]
- activeTabId?: number

Emits:
- @activate - активировать эту панель
- @update:tabs - обновление табов
- @update:activeTabId - обновление активного таба

5.3. DualPanelContainer.vue (НОВЫЙ)

Файл: src/components/DualPanelContainer.vue

Контейнер с двумя FilePanel и resizable разделителем:
- Две панели: левая и правая
- Вертикальный разделитель (1px, cursor-col-resize)
- Resizer логика: mousedown -> mousemove -> mouseup
- Ограничения: 20%-80% для предотвращения схлопывания
- Использует useDualPanel для получения и обновления состояния

Layout:
┌──────────────────┬─┬──────────────────┐
│                  │ │                  │
│   Left Panel     │R│   Right Panel    │
│   (FilePanel)    │e│   (FilePanel)    │
│                  │s│                  │
│                  │i│                  │
│                  │z│                  │
│                  │e│                  │
│                  │r│                  │
└──────────────────┴─┴──────────────────┘

 ---
Шаг 6: Toolbar интеграция

Файл: src/components/Toolbar.vue

Добавить Props:
interface Props {
// ... существующие
panelMode?: PanelMode;
}

Добавить Emits:
interface Emits {
// ... существующие
(e: 'togglePanelMode'): void;
}

Добавить кнопку в template (после view mode кнопок, ~строка 335):

 <!-- Separator -->
 <div class="w-px h-[24px] bg-[#919B9C] ml-1"></div>

 <!-- Dual Panel Toggle Button -->
<button
@click="emit('togglePanelMode')"
:class="[
'w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4]',
'border border-[#8B8B8B] hover:border-[#0054E3]',
'flex items-center justify-center transition-all',
panelMode === 'dual' && 'bg-[#C1D2EE] from-[#C1D2EE] to-[#A8C0E8]'
]"
:title="panelMode === 'dual' ? 'Single Panel' : 'Dual Panel'"
>
{{ panelMode === 'dual' ? '⊟' : '⊞⊞' }}
</button>

 ---
Шаг 7: App.vue интеграция

Файл: src/App.vue

7.1. Добавить импорты (после строки 14):

import DualPanelContainer from './components/DualPanelContainer.vue';
import { useDualPanel } from './composables/useDualPanel';

7.2. Добавить composable (после строки 124):

// Dual Panel
const {
panelMode,
isDualMode,
activePanelPath,
activePanelTabs,
activePanelTabId,
togglePanelMode,
loadDualPanelState,
serializeDualPanelState,
} = useDualPanel();

7.3. Модифицировать template (строка 606):

 <!-- Main Content -->
 <div class="flex-1 flex overflow-hidden">
   <!-- Dual Panel Mode -->
   <DualPanelContainer v-if="isDualMode" />

   <!-- Single Panel Mode -->
   <template v-else>
     <Sidebar ... />
     <div class="flex-1 flex overflow-hidden">
       <FileList ... />
       <Preview ... />
     </div>
   </template>
 </div>

7.4. Модифицировать Toolbar (строка 582):

<Toolbar
:tabs="isDualMode ? activePanelTabs : tabs"
:active-tab-id="isDualMode ? activePanelTabId : activeTabId"
:current-path="isDualMode ? activePanelPath : currentPath"
:panel-mode="panelMode"
@toggle-panel-mode="togglePanelMode"
   <!-- ... остальные props и events -->
/>

7.5. Модифицировать watch для сохранения (строка 487):

Добавить в массив зависимостей:
watch([
tabs,
activeTabId,
currentPath,
panelMode,              // НОВОЕ
leftPanelTabs,          // НОВОЕ
leftPanelActiveTabId,   // НОВОЕ
rightPanelTabs,         // НОВОЕ
rightPanelActiveTabId,  // НОВОЕ
leftPanelWidthPercent,  // НОВОЕ
activePanel,            // НОВОЕ
expandedFolders,
sidebarSectionsExpanded
], async () => {
// ...
const stateToSave = {
sidebar_width: sidebarWidth.value,
preview_width: previewWidth.value,

     // Single-mode
     tabs: tabs.value.map(tab => ({
       id: tab.id,
       path: tab.path,
       name: tab.name,
     })),
     active_tab_id: activeTabId.value,
     last_path: currentPath.value,

     // Dual-panel (НОВОЕ)
     panel_mode: panelMode.value,
     dual_panel_config: serializeDualPanelState(),

     window: { maximized: false },
     sidebar: { /* ... */ }
};

await invoke('save_ui_state', { uiState: stateToSave });
// ...
}, { deep: true });

7.6. Модифицировать onMounted (строка 530):

onMounted(async () => {
document.addEventListener('click', closeContextMenu);

await loadBookmarks();
const uiState = await loadUIState();

// Восстановить режим панелей
if (uiState?.panel_mode) {
panelMode.value = uiState.panel_mode;

     if (uiState.panel_mode === 'dual' && uiState.dual_panel_config) {
       loadDualPanelState(uiState.dual_panel_config);
     }
}

// Восстановить single-mode табы (только если НЕ dual)
if (!isDualMode.value) {
if (uiState && uiState.tabs && uiState.tabs.length > 0) {
tabs.value = uiState.tabs.map(tabState => ({
id: tabState.id,
path: tabState.path,
name: tabState.name,
history: [tabState.path],
historyIndex: 0,
}));

       if (uiState.active_tab_id) {
         activeTabId.value = uiState.active_tab_id;
       }
     } else if (uiState && uiState.last_path) {
       navigateTo(uiState.last_path);
     }
}
});

 ---
Порядок выполнения (критический путь)

1. Backend → src-tauri/src/config.rs (30-60 мин)
- Добавить enums и structs
- Проверить компиляцию cargo build
2. Types → src/types/index.ts (10 мин)
- Добавить TypeScript интерфейсы
3. Composables → (1-2 часа)
- src/composables/useDualPanel.ts (новый)
- src/composables/useNavigation.ts (добавить useNavigationWithRefs)
4. Components → (2-3 часа)
- src/components/PanelToolbar.vue (новый)
- src/components/FilePanel.vue (новый)
- src/components/DualPanelContainer.vue (новый)
5. Integration → (1-2 часа)
- src/components/Toolbar.vue (кнопка)
- src/App.vue (полная интеграция)
6. Testing → (1 час)
- Переключение режимов
- Resizing панелей
- Сохранение/восстановление состояния
- Независимая навигация в панелях

Общее время: ~6-9 часов

 ---
Критические файлы

| Файл                                  | Тип изменения            | Важность |
 |---------------------------------------|--------------------------|----------|
| src-tauri/src/config.rs               | Модификация + Добавление | ⭐⭐⭐   |
| src/types/index.ts                    | Добавление               | ⭐⭐     |
| src/composables/useDualPanel.ts       | Новый файл               | ⭐⭐⭐   |
| src/composables/useNavigation.ts      | Добавление               | ⭐⭐     |
| src/components/DualPanelContainer.vue | Новый файл               | ⭐⭐⭐   |
| src/components/FilePanel.vue          | Новый файл               | ⭐⭐⭐   |
| src/components/PanelToolbar.vue       | Новый файл               | ⭐⭐     |
| src/components/Toolbar.vue            | Модификация              | ⭐⭐     |
| src/App.vue                           | Модификация              | ⭐⭐⭐   |

 ---
Особенности реализации

1. Изоляция состояния панелей

Каждая FilePanel имеет свой экземпляр:
- useSelection - независимое выделение файлов
- useFileSystem - независимая загрузка директорий
- useDragDrop - независимый drag & drop

2. Обратная совместимость

- #[serde(default)] в Rust обеспечивает дефолтные значения для новых полей
- Старые конфиги автоматически получат panel_mode: Single
- При первом запуске пользователь увидит привычный single-режим

3. Resizer ограничения

- Минимум 20%, максимум 80% для каждой панели
- Предотвращает "схлопывание" панелей до невидимости
- Сохраняется процентное соотношение (адаптивность к разным размерам окна)

4. Активная панель

- Визуальная индикация: ring-2 ring-blue-500 для активной панели
- Клик по панели автоматически делает её активной
- Keyboard shortcuts направляются в активную панель

5. Keyboard shortcut

- Добавить Ctrl+\ для переключения single/dual режима
- Регистрировать в src/utils/shortcuts.ts

 ---
После реализации

Возможные улучшения (не входят в текущий план):

1. Drag & Drop между панелями
- Копирование файлов перетаскиванием из левой в правую панель
- Потребует доработки useDragDrop
2. Синхронизация путей
- Опция "синхронизировать пути панелей"
- Обе панели показывают одну директорию
3. Quick switch
- Горячая клавиша для обмена путей между панелями
- Tab для переключения активной панели
4. Panel presets
- Сохранение "избранных" конфигураций панелей
- Быстрое восстановление часто используемых комбинаций путей

 ---
Проверка выполнения

После реализации убедиться:

- Кнопка переключения режима работает
- В dual-режиме скрыты Sidebar и Preview
- Каждая панель имеет свои независимые вкладки
- Resizable разделитель работает (ограничения 20-80%)
- Активная панель визуально выделена
- Клик по панели делает её активной
- Состояние сохраняется в config.json
- При перезапуске режим и конфигурация восстанавливаются
- Навигация в каждой панели независима
- Cargo build успешен (без ошибок компиляции)
- Обратная совместимость: старые конфиги работают

 ---
Итоговая архитектура

Single Mode:
┌─────────────────────────────────────────────┐
│ Toolbar                                     │
├──────────┬──────────────────────┬───────────┤
│ Sidebar  │ FileList             │ Preview   │
│          │                      │           │
└──────────┴──────────────────────┴───────────┘

Dual Mode:
┌─────────────────────────────────────────────┐
│ Toolbar                                     │
├──────────────────────┬──────────────────────┤
│ Left Panel           │ Right Panel          │
│ ┌──────────────────┐ │ ┌──────────────────┐ │
│ │ PanelToolbar     │ │ │ PanelToolbar     │ │
│ ├──────────────────┤ │ ├──────────────────┤ │
│ │ FileList         │ │ │ FileList         │ │
│ │                  │ │ │                  │ │
│ └──────────────────┘ │ └──────────────────┘ │
└──────────────────────┴──────────────────────┘
