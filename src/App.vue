<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import Toolbar from './components/Toolbar.vue';
import Sidebar from './components/Sidebar.vue';
import FileList from './components/FileList.vue';
import Preview from './components/Preview.vue';
import Dashboard from './components/Dashboard.vue';
import CommandPalette from './components/CommandPalette.vue';
import ContextMenu from './components/ContextMenu.vue';
import Notifications from './components/Notifications.vue';
import ConfirmDialog from './components/ConfirmDialog.vue';
import ConflictDialog from './components/ConflictDialog.vue';
import PropertiesDialog from './components/PropertiesDialog.vue';
import InputDialog from './components/InputDialog.vue';
import Settings from './components/Settings.vue';
import OperationsProgress from './components/OperationsProgress.vue';
import DualPanelContainer from './components/DualPanelContainer.vue';
import BatchRenameDialog from './components/BatchRenameDialog.vue';
import BatchAttributeDialog from './components/BatchAttributeDialog.vue';
import BatchOperationsQueue from './components/BatchOperationsQueue.vue';
import OperationsQueuePanel from './components/OperationsQueuePanel.vue';
import QueueSettingsDialog from './components/QueueSettingsDialog.vue';
import Terminal from './components/Terminal.vue';
import ProgrammerToolbar from './components/ProgrammerToolbar.vue';
import PanelToolbar from './components/PanelToolbar.vue';
import TextEditor from './components/TextEditor.vue';
import VaultOverlay from './components/VaultOverlay.vue';
import WidgetLayer from './components/WidgetLayer.vue';
import WidgetSelector from './components/WidgetSelector.vue';
import ShareDialog from './components/ShareDialog.vue';

import { useFileSystem } from './composables/useFileSystem';
import { useVault } from './composables/useVault';
import { useNavigation } from './composables/useNavigation';
import { useSelection } from './composables/useSelection';
import { useSearch } from './composables/useSearch';
import { useDragDrop } from './composables/useDragDrop';
import { useKeyboard } from './composables/useKeyboard';
import { useDialogs } from './composables/useDialogs';
import { useFileOperations } from './composables/useFileOperations';
import { useCommands } from './composables/useCommands';
import { useNotifications } from './composables/useNotifications';
import { useFileContentCache } from './composables/useFileContentCache';
import { useBookmarks } from './composables/useBookmarks';
import { useUIState } from './composables/useUIState';
import { useSwipeNavigation } from './composables/useSwipeNavigation';
import { useDualPanel, getActivePanelMethods } from './composables/useDualPanel';
import { useContextMenu } from './composables/useContextMenu';
import { useGrouping } from './composables/useGrouping';
import { useConflictResolution } from './composables/useConflictResolution';
import { useBatchOperations } from './composables/useBatchOperations';
import { useOperationsQueue } from './composables/useOperationsQueue';
import { useProgrammerMode } from './composables/useProgrammerMode';
import { useTemplates } from './composables/useTemplates';
import { useTerminal } from './composables/useTerminal';
import { useTheme } from './composables/useTheme';
import { useFileColoring } from './composables/useFileColoring';
import { useGlobalRefresh } from './composables/useGlobalRefresh';
import { useWidgets } from './composables/useWidgets';
import { useClipboard } from './composables/useClipboard';
import { createKeyboardShortcuts } from './utils/shortcuts';

import type {FileItem, ViewMode, BatchRenameConfig, BatchAttributeChange} from './types';

// File System
const { files, isLoading, loadDirectory, normalizePath, getHomeDirectory, writeFileContent, extractArchive, createArchive } = useFileSystem();

// Vault Security
const vault = useVault();

// Navigation
const {
  tabs,
  activeTabId,
  currentPath,
  canGoBack,
  canGoForward,
  canGoUp,
  goBack,
  goForward,
  goUp,
  goHome,
  navigateTo,
  navigateToBreadcrumb,
  addTab,
  closeTab,
  switchTab,
} = useNavigation();

// Selection
const {
  selectedIds,
  selectedCount,
  hasSelection,
  isSelected,
  handleItemClick,
  getSelectedItems,
  clearSelection,
  selectAll,
  focusedId,
  setFocused,
  moveFocusUp,
  moveFocusDown,
  moveFocusToFirst,
  moveFocusToLast,
  selectFocused,
  toggleFocusedSelection,
  getFocusedItem,
} = useSelection();

// Search & Filters
const {
  processFiles,
  hasActiveFilters,
} = useSearch();

// Drag & Drop
const {
  isDragging,
  draggedItems,
  dragOverId,
  startDrag,
  handleDragOver,
  handleDragLeave,
  handleDrop,
  handleDragOverBackground,
} = useDragDrop();

// Dialogs
const {
  confirmDialog,
  showConfirm,
  closeConfirm,
  propertiesDialog,
  closeProperties,
  inputDialog,
  showInput,
  closeInput,
} = useDialogs();

// Conflict Resolution
const {
  currentConflict,
  isConflictDialogOpen,
  handleResolution,
  handleCancel: handleConflictCancel,
} = useConflictResolution();

// Notifications
const {
  notifications: activeNotifications,
  clear: clearNotifications,
} = useNotifications();

// Bookmarks
const {
  bookmarks,
  loadBookmarks,
  addBookmark,
  removeBookmark,
  isBookmarked,
} = useBookmarks();

// UI State
const {
  sidebarWidth,
  previewWidth,
  expandedFolders,
  sidebarSectionsExpanded,
  editModeEnabled,
  loadUIState,
} = useUIState();

// Dual Panel
const {
  panelMode,
  isDualMode,
  activePanelPath,
  activePanelTabs,
  activePanelTabId,
  leftPanelWidthPercent,
  activePanel,
  leftPanelTabs,
  leftPanelActiveTabId,
  leftPanelFilesystem,
  rightPanelTabs,
  rightPanelActiveTabId,
  rightPanelFilesystem,
  togglePanelMode,
  switchActivePanel,
  loadDualPanelState,
  serializeDualPanelState,
} = useDualPanel();

// Terminal
const {
  isVisible: isTerminalVisible,
  terminalHeight,
  toggleTerminal,
} = useTerminal();

// Computed для текущего пути терминала
const terminalWorkingDir = computed(() => {
  if (isDualMode.value) {
    return '/' + activePanelPath.value.join('/');
  }
  return '/' + currentPath.value.join('/');
});

// Computed для определения множественного выбора
const hasMultipleSelected = computed(() => {
  if (isDualMode.value) {
    const methods = getActivePanelMethods();
    const selected = methods?.getSelectedIds() || new Set();
    return selected.size > 1;
  }
  return selectedIds.value.size > 1;
});

// Computed for queue active operations count
const queueActiveCount = computed(() => {
  return queueStatistics.value.running + queueStatistics.value.queued + queueStatistics.value.scheduled;
});

// Handle sidebar resize - напрямую обновляем ref, watch в App.vue сам сохранит
const handleSidebarResize = (width: number) => {
  sidebarWidth.value = width;
};

// Handle preview resize - напрямую обновляем ref, watch в App.vue сам сохранит
const handlePreviewResize = (width: number) => {
  previewWidth.value = width;
};

// Handle dashboard resize
const handleDashboardResize = (width: number) => {
  dashboardWidth.value = width;
};

// Toggle dashboard
const handleToggleDashboard = () => {
  showDashboard.value = !showDashboard.value;
  // Close preview when opening dashboard
  if (showDashboard.value) {
    previewFile.value = null;
  }
};

const handleBackgroundDrop = async (event: DragEvent) => {
  console.log('[App] Background Drop Detected!'); // DEBUG LOG

  // Create a target representing the current directory
  const pathString = await getCurrentDirectoryPath();
  const targetItem: FileItem = {
    id: pathString,
    name: pathString.split('/').pop() || 'root',
    path: pathString,
    type: 'folder',
    size: 0,
    modified: '',
    // Add defaults for other required fields
    tags: [],
    permissions: { readable: true, writable: true, executable: true }
  };

  // Use wrappers to route through handleTransfer for conflict resolution
  const onMove = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'move', srcFs, destFs);
  const onCopy = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'copy', srcFs, destFs);

  await handleDrop(targetItem, event, onMove, onCopy);
};

// Helper для получения текущей директории
const getCurrentDirectoryPath = async (): Promise<string> => {
  let pathString = currentPath.value.join('/');
  if (pathString && !pathString.startsWith('/')) {
    pathString = '/' + pathString;
  }
  if (!pathString) {
    return await getHomeDirectory();
  }
  return pathString;
};

// Функция для обновления текущей директории
const refreshCurrentDirectory = async () => {
  const pathString = await getCurrentDirectoryPath();
  await loadDirectory(pathString);
};

// Handle global refresh event
const handleGlobalRefresh = async () => {
  if (!isDualMode.value) {
    await refreshCurrentDirectory();
  }
};

// File Operations
const fileOps = useFileOperations(async () => {
  await refreshAllPanels(currentPath.value);
});

// Context Menu (global)
const { contextMenu, showContextMenu, closeContextMenu } = useContextMenu();

// Grouping
const { groupBy, groupByOptions, groupFiles } = useGrouping();

// Global Refresh
const { refreshAllPanels } = useGlobalRefresh();

// Widgets
const { toggleWidget, showWidgetSelector, closeWidgetSelector } = useWidgets();

// Clipboard
const { hasClipboardItems } = useClipboard();

// Batch Operations (with auto-refresh callback)
const { queueBatchRename, queueBatchAttributeChange, hasOperations } = useBatchOperations(async () => {
  await refreshAllPanels([]);
});

// Operations Queue
const { addOperation, statistics: queueStatistics, hasActiveOperations } = useOperationsQueue();

// Programmer Mode
const { isProgrammerMode, toggleProgrammerMode } = useProgrammerMode();

// Templates
const { loadTemplates } = useTemplates();

// Local state
const viewMode = ref<ViewMode>('list');
const sortBy = ref<'name' | 'size' | 'modified' | 'type'>('name');
const sortOrder = ref<'asc' | 'desc'>('asc');
const showHidden = ref(false);
const isCommandPaletteOpen = ref(false);
const previewFile = ref<FileItem | null>(null);
const showSettings = ref(false);
const settingsInitialTab = ref<'general' | 'colors'>('general');
const showDashboard = ref(false);
const dashboardWidth = ref(400);
const showBatchRenameDialog = ref(false);
const showBatchAttributeDialog = ref(false);
const showBatchQueue = ref(false);
const showOperationsQueue = ref(false);
const showQueueSettings = ref(false);
const showShareDialog = ref(false);
const shareInfo = ref<{ url: string; qr_svg: string; filename: string } | null>(null);
const batchOperationFiles = ref<FileItem[]>([]);
const showTextEditor = ref(false);
const editorFile = ref<FileItem | null>(null);
const editorFileFs = ref<string | undefined>(undefined);

// Inline File Creator state
const showInlineCreator = ref(false);
const inlineCreatorMode = ref<'file' | 'folder'>('file');

// System stats
const systemStats = ref({ memory_mb: 0, cpu_percent: 0 });

// Check if current path is bookmarked
const isCurrentPathBookmarked = computed(() => {
  const path = '/' + currentPath.value.join('/');
  return isBookmarked(path);
});

// Computed: Process, filter and sort files
const processedFiles = computed(() => {
  let result = processFiles(files.value);

  // Filter hidden files if needed (only in single mode)
  if (!isDualMode.value && !showHidden.value) {
    result = result.filter(file => !file.name.startsWith('.'));
  }

  // Sort files (only in single mode, dual mode handles its own sorting)
  if (!isDualMode.value) {
    result = [...result].sort((a, b) => {
      // Folders first
      const aIsFolder = a.type === 'folder' || a.type === 'drive' || a.type === 'system';
      const bIsFolder = b.type === 'folder' || b.type === 'drive' || b.type === 'system';

      if (aIsFolder && !bIsFolder) return -1;
      if (!aIsFolder && bIsFolder) return 1;

      // Then sort by selected field
      let comparison = 0;
      switch (sortBy.value) {
        case 'name':
          comparison = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
          break;
        case 'size':
          comparison = (a.size || 0) - (b.size || 0);
          break;
        case 'modified':
          comparison = (a.modified || '').localeCompare(b.modified || '');
          break;
        case 'type':
          comparison = a.type.localeCompare(b.type);
          break;
      }

      return sortOrder.value === 'asc' ? comparison : -comparison;
    });
  }

  return result;
});

// File groups
const fileGroups = computed(() => groupFiles(processedFiles.value));

// Helper to get selected items
const getSelected = () => getSelectedItems(files.value);

// Handlers
const handleItemDoubleClick = (item: FileItem) => {
  if (item.type === 'folder' || item.type === 'drive' || item.type === 'system' || item.type === 'archive') {
    const pathParts = item.path.split('/').filter(p => p);
    navigateTo(pathParts);
  } else {
    // Check if edit mode is enabled and file is text/code
    const isEditableFile = item.type === 'file' || item.type === 'code';
    if (editModeEnabled.value && isEditableFile) {
      handleEditFile(item);
    } else {
      handlePreviewFile(item);
    }
  }
};

const handleContextMenu = (item: FileItem, event: MouseEvent) => {
  showContextMenu(item, event);
};

// ИСПРАВЛЕНИЕ: Обновленная функция handleDragStart
const handleDragStart = (item: FileItem, event: DragEvent) => {
  // Определяем, какие элементы нужно перетаскивать
  const items = hasSelection.value && isSelected(item.id)
      ? getSelectedItems(files.value)
      : [item];

  console.log('[App] Starting drag with items:', items.length, items.map(i => i.name));

  // Запускаем drag через composable
  startDrag(items, event);
};

// ИСПРАВЛЕНИЕ: Обновленная функция handleItemDrop
const handleItemDrop = async (item: FileItem, event: DragEvent) => {
  event.preventDefault();
  console.log('[App] Drop on item:', item.name);

  // Use wrappers to route through handleTransfer for conflict resolution
  const onMove = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'move', srcFs, destFs);
  const onCopy = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'copy', srcFs, destFs);

  // Используем handleDrop из composable
  await handleDrop(item, event, onMove, onCopy);

  // Обновляем директорию после drop
  await refreshCurrentDirectory();
};

// ИСПРАВЛЕНИЕ: Обновленная функция handleSidebarDrop
const handleSidebarDrop = async (targetPath: string, event: DragEvent) => {
  event.preventDefault();
  console.log('[App] Drop on sidebar path:', targetPath);

  // Создаем временный FileItem для целевого пути
  const targetItem: FileItem = {
    id: targetPath,
    name: targetPath.split('/').pop() || '',
    path: targetPath,
    type: 'folder',
    size: 0,
    modified: '',
  };

  // Use wrappers to route through handleTransfer for conflict resolution
  const onMove = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'move', srcFs, destFs);
  const onCopy = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'copy', srcFs, destFs);

  // Используем handleDrop из composable
  await handleDrop(targetItem, event, onMove, onCopy);

  // Обновляем директорию после drop
  await refreshCurrentDirectory();
};



// Toggle bookmark for current directory
const handleToggleBookmark = async () => {
  const { success, error } = useNotifications();
  const path = await getCurrentDirectoryPath();

  if (isBookmarked(path)) {
    // Remove bookmark
    const bookmark = bookmarks.value.find(b => b.path === path);
    if (bookmark) {
      const removed = await removeBookmark(bookmark.id);
      if (removed) {
        success('Removed from Favorites', `Removed: ${bookmark.name}`);
      } else {
        error('Failed to remove bookmark');
      }
    }
  } else {
    // Add bookmark
    const folderName = currentPath.value[currentPath.value.length - 1] || 'Root';
    const bookmark = await addBookmark(path, folderName);
    if (bookmark) {
      success('Added to Favorites', `Added: ${bookmark.name}`);
    } else {
      error('Failed to add bookmark', 'This folder may already be bookmarked');
    }
  }
};

// Add folder to bookmarks (from context menu)
const handleAddFolderToBookmarks = async (item: FileItem) => {
  const { success, error } = useNotifications();

  if (item.type !== 'folder') {
    error('Cannot bookmark', 'Only folders can be added to favorites');
    return;
  }

  const bookmark = await addBookmark(item.path, item.name);
  if (bookmark) {
    success('Added to Favorites', `Added: ${bookmark.name}`);
  } else {
    error('Failed to add bookmark', 'This folder may already be bookmarked');
  }
};

const handleOpenTerminal = async (item: FileItem) => {
  const { openTerminal } = useFileSystem();
  const { success, error } = useNotifications();

  try {
    await openTerminal(item.path);
    success('Terminal opened', `Opened terminal in ${item.name}`);
  } catch (err) {
    error('Failed to open terminal', err instanceof Error ? err.message : 'Unknown error');
  }
};

// Inline File Creator handlers
const handleCreateFile = async (payload: { name: string; isFolder: boolean; templateId?: string }) => {
  try {
    if (payload.isFolder) {
      await fileOps.handleNewFile(currentPath.value, payload.name);
    } else {
      await fileOps.handleNewFile(currentPath.value, payload.name, payload.templateId);
    }
    showInlineCreator.value = false;
  } catch (err) {
    console.error('Failed to create file:', err);
  }
};

const handleBatchCreateFiles = async (names: string[]) => {
  try {
    const files = names.map(name => ({ name }));
    await fileOps.handleBatchCreate(currentPath.value, files);
    showInlineCreator.value = false;
  } catch (err) {
    console.error('Failed to batch create files:', err);
  }
};

const handleCancelInlineCreator = () => {
  showInlineCreator.value = false;
};

// Navigation handlers for toolbar (work in both single and dual modes)
const handleGoBack = () => {
  if (isDualMode.value) {
    const methods = getActivePanelMethods();
    if (methods) methods.goBack();
  } else {
    goBack();
  }
};

const handleGoForward = () => {
  if (isDualMode.value) {
    const methods = getActivePanelMethods();
    if (methods) methods.goForward();
  } else {
    goForward();
  }
};

const handleGoUp = () => {
  if (isDualMode.value) {
    const methods = getActivePanelMethods();
    if (methods) methods.goUp();
  } else {
    goUp();
  }
};

const handleAddTab = () => {
  if (isDualMode.value) {
    const methods = getActivePanelMethods();
    if (methods) methods.addTab();
  } else {
    addTab();
  }
};

const handleGoHome = async () => {
  if (isDualMode.value) {
    // In dual mode, navigate active panel to home
    const homeDir = await getHomeDirectory();
    const pathParts = homeDir.replace(/^\//, '').split('/').filter(p => p);

    if (activePanel.value === 'left') {
      const activeTab = leftPanelTabs.value.find(t => t.id === leftPanelActiveTabId.value);
      if (activeTab) {
        const tabIndex = leftPanelTabs.value.findIndex(t => t.id === leftPanelActiveTabId.value);
        const updatedTabs = [...leftPanelTabs.value];
        const newHistory = activeTab.history.slice(0, activeTab.historyIndex + 1);
        newHistory.push(pathParts);
        updatedTabs[tabIndex] = {
          ...activeTab,
          path: pathParts,
          name: 'Home',
          history: newHistory,
          historyIndex: newHistory.length - 1,
        };
        leftPanelTabs.value = updatedTabs;
      }
    } else {
      const activeTab = rightPanelTabs.value.find(t => t.id === rightPanelActiveTabId.value);
      if (activeTab) {
        const tabIndex = rightPanelTabs.value.findIndex(t => t.id === rightPanelActiveTabId.value);
        const updatedTabs = [...rightPanelTabs.value];
        const newHistory = activeTab.history.slice(0, activeTab.historyIndex + 1);
        newHistory.push(pathParts);
        updatedTabs[tabIndex] = {
          ...activeTab,
          path: pathParts,
          name: 'Home',
          history: newHistory,
          historyIndex: newHistory.length - 1,
        };
        rightPanelTabs.value = updatedTabs;
      }
    }
  } else {
    goHome();
  }
};

const canGoBackComputed = computed(() => {
  if (isDualMode.value) {
    const methods = getActivePanelMethods();
    return methods ? methods.canGoBack() : false;
  }
  return canGoBack.value;
});

const canGoForwardComputed = computed(() => {
  if (isDualMode.value) {
    const methods = getActivePanelMethods();
    return methods ? methods.canGoForward() : false;
  }
  return canGoForward.value;
});

// Handle navigation to path from address bar
const handleNavigateToPath = async (path: string) => {
  try {
    // Normalize the path (expand ~, resolve to absolute path)
    const normalizedPath = await normalizePath(path);

    // Convert absolute path to array format for navigation
    // Remove leading slash and split by '/'
    const pathArray = normalizedPath.replace(/^\//, '').split('/').filter(p => p);

    navigateTo(pathArray);
  } catch (err) {
    const { error } = useNotifications();
    error('Invalid path', err instanceof Error ? err.message : 'Path not found');
  }
};

// Command palette commands
const commands = useCommands({
  onNewFolder: () => fileOps.handleNewFolder(currentPath.value, showInput),
  onNewFile: () => {},
  onSearch: () => {},
  onGoto: () => {
    showInput(
        'Go To',
        'Enter path:',
        (path: string) => {
          if (path) {
            try {
              navigateTo(path.split('/').filter(p => p));
            } catch (err) {
              console.error(err);
            }
          }
          closeInput();
        },
        '',
        '/Users/username/Documents'
    );
  },
  onRefresh: () => fileOps.handleRefresh(currentPath.value),
  onCopyPath: (selectedItems: FileItem[]) => commands.copyPathCommand(selectedItems),
  onSelectAll: (allFiles: FileItem[]) => commands.selectAllCommand(allFiles, selectAll),
  onNewTab: addTab,
  onCloseTab: () => commands.closeTabCommand(tabs.value.length, closeTab, activeTabId.value ?? 0),
  onSettings: () => { settingsInitialTab.value = 'general'; showSettings.value = true; },
});

// Toolbar handlers (for single panel mode)
const handleSort = (field: 'name' | 'size' | 'modified' | 'type', order: 'asc' | 'desc') => {
  sortBy.value = field;
  sortOrder.value = order;
};

const handleSelectAll = () => {
  files.value.forEach(file => selectedIds.value.add(file.id));
};

const handleInvertSelection = () => {
  const newSelection = new Set<string>();
  files.value.forEach(file => {
    if (!selectedIds.value.has(file.id)) {
      newSelection.add(file.id);
    }
  });
  selectedIds.value = newSelection;
};

const handleRefresh = async () => {
  await refreshCurrentDirectory();
};

const handleToggleHidden = () => {
  showHidden.value = !showHidden.value;
};

const handleToggleEditMode = () => {
  editModeEnabled.value = !editModeEnabled.value;
};

const executeCommand = (cmd: { id: string }) => {
  if (cmd.id === 'copy-path') {
    commands.copyPathCommand(getSelected());
  } else if (cmd.id === 'select-all') {
    commands.selectAllCommand(files.value, selectAll);
  } else {
    commands.executeCommand(cmd);
  }
};

// Keyboard shortcuts (work in both single and dual modes)
const shortcuts = createKeyboardShortcuts(
    {
      openCommandPalette: () => { isCommandPaletteOpen.value = true; },
      closeDialogs: () => {
        // 1. Context Menu
        if (contextMenu.value) {
          closeContextMenu();
          return;
        }

        // 2. Critical Modals
        if (confirmDialog.value.isOpen) {
          closeConfirm();
          return;
        }
        if (inputDialog.value.isOpen) {
          closeInput();
          return;
        }
        if (isConflictDialogOpen.value) {
          handleConflictCancel();
          return;
        }

        // 3. Feature Dialogs
        if (isCommandPaletteOpen.value) {
          isCommandPaletteOpen.value = false;
          return;
        }
        if (propertiesDialog.value.isOpen) {
          closeProperties();
          return;
        }
        if (showBatchRenameDialog.value) {
          showBatchRenameDialog.value = false;
          return;
        }
        if (showBatchAttributeDialog.value) {
          showBatchAttributeDialog.value = false;
          return;
        }
        if (showInlineCreator.value) {
          showInlineCreator.value = false;
          return;
        }

        // 4. Overlays
        if (showSettings.value) {
          showSettings.value = false;
          return;
        }
        if (showDashboard.value) {
          showDashboard.value = false;
          return;
        }
        if (showBatchQueue.value) {
          showBatchQueue.value = false;
          return;
        }
        if (previewFile.value) {
          previewFile.value = null;
          return;
        }
        if (showTextEditor.value) {
          handleCloseEditor();
          return;
        }

        // Widget Selector
        if (showWidgetSelector.value) {
          showWidgetSelector.value = false;
          return;
        }/*

        // Active Widgets
        // Close all active widgets
        const activeWidgets = widgets.value.filter(w => w.active);
        if (activeWidgets.length > 0) {
          activeWidgets.forEach(w => toggleWidget(w.id));
          return;
        }*/

        // 5. Notifications
        if (activeNotifications.value.length > 0) {
          clearNotifications();
          return;
        }

        // 6. Selection
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.clearSelection();
        } else {
          clearSelection();
        }
      },
      selectAll: (files: FileItem[]) => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.selectAll();
        } else {
          selectAll(files);
        }
      },
      addTab: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.addTab();
        } else {
          addTab();
        }
      },
      closeTab: (canClose: boolean) => {
        if (canClose) {
          if (isDualMode.value) {
            const methods = getActivePanelMethods();
            if (methods) methods.closeTab();
          } else if (tabs.value.length > 1) {
            closeTab(activeTabId.value);
          }
        }
      },
      goUp: (canGoUpValue: boolean) => {
        if (canGoUpValue) {
          if (isDualMode.value) {
            const methods = getActivePanelMethods();
            if (methods) methods.goUp();
          } else {
            goUp();
          }
        }
      },
      handleCopy: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.handleCopy();
        } else {
          fileOps.handleCopy(getSelected());
        }
      },
      handleCut: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.handleCut();
        } else {
          fileOps.handleCut(getSelected());
        }
      },
      handlePaste: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.handlePaste();
        } else {
          fileOps.handlePaste(currentPath.value);
        }
      },
      handleDelete: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.handleDelete();
        } else {
          fileOps.handleDelete(getSelected(), currentPath.value, clearSelection, showConfirm);
        }
      },
      handleRename: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.handleRename();
        } else {
          fileOps.handleRename(getSelected(), currentPath.value, showInput);
        }
      },
      handleRefresh: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.handleRefresh();
        } else {
          fileOps.handleRefresh(currentPath.value);
        }
      },
      handleNewFolder: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.handleNewFolder();
        } else {
          fileOps.handleNewFolder(currentPath.value, showInput);
        }
      },
      handleNewFile: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.handleNewFile();
        } else {
          inlineCreatorMode.value = 'file';
          showInlineCreator.value = true;
        }
      },
      toggleProgrammerMode: () => {
        toggleProgrammerMode();
      },
      toggleBookmark: handleToggleBookmark,
      openSettings: () => { settingsInitialTab.value = 'general'; showSettings.value = true; },
      // Dual panel switch (Tab)
      switchPanels: isDualMode.value ? () => {
        switchActivePanel(activePanel.value === 'left' ? 'right' : 'left');
      } : undefined,
      // Terminal toggle (Ctrl+`)
      toggleTerminal: () => toggleTerminal(),
      // Keyboard navigation
      moveFocusUp: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.moveFocusUp();
        } else {
          moveFocusUp(processedFiles.value);
        }
      },
      moveFocusDown: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.moveFocusDown();
        } else {
          moveFocusDown(processedFiles.value);
        }
      },
      moveFocusToFirst: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.moveFocusToFirst();
        } else {
          moveFocusToFirst(processedFiles.value);
        }
      },
      moveFocusToLast: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.moveFocusToLast();
        } else {
          moveFocusToLast(processedFiles.value);
        }
      },
      selectFocused: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.selectFocused();
        } else {
          selectFocused();
        }
      },
      toggleFocusedSelection: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.toggleFocusedSelection();
        } else {
          toggleFocusedSelection();
        }
      },
      openFocusedItem: () => {
        if (isDualMode.value) {
          const methods = getActivePanelMethods();
          if (methods) methods.openFocusedItem();
        } else {
          const item = getFocusedItem(processedFiles.value);
          if (item) {
            handleItemDoubleClick(item);
          }
        }
      },
    },
    () => isDualMode.value ? (getActivePanelMethods()?.getFiles() || []) : files.value
);

useKeyboard(shortcuts);

// Swipe navigation (two-finger swipe on trackpad) - works in both modes
useSwipeNavigation({
  onSwipeLeft: () => {
    if (isDualMode.value) {
      // In dual mode, navigate back in active panel's history
      const methods = getActivePanelMethods();
      if (methods) methods.goBack();
    } else {
      goBack();
    }
  },
  onSwipeRight: () => {
    if (isDualMode.value) {
      // In dual mode, navigate forward in active panel's history
      const methods = getActivePanelMethods();
      if (methods) methods.goForward();
    } else {
      goForward();
    }
  },
  canSwipeLeft: () => {
    if (isDualMode.value) {
      // In dual mode, check if active panel can go back
      const methods = getActivePanelMethods();
      return methods ? methods.canGoBack() : false;
    }
    return canGoBack.value;
  },
  canSwipeRight: () => {
    if (isDualMode.value) {
      // In dual mode, check if active panel can go forward
      const methods = getActivePanelMethods();
      return methods ? methods.canGoForward() : false;
    }
    return canGoForward.value;
  },
});

// Text editor handlers
const handleEditFile = (file: FileItem, panelFs?: string) => {
  editorFile.value = file;
  editorFileFs.value = panelFs;
  showTextEditor.value = true;
};

const handlePreviewFile = (file: FileItem) => {
  previewFile.value = file;
  showDashboard.value = false;
};

const handleCloseEditor = () => {
  showTextEditor.value = false;
  editorFile.value = null;
  editorFileFs.value = undefined;
};

const handleSaveFile = async (content: string) => {
  const { success, error } = useNotifications();
  const { invalidate } = useFileContentCache();
  if (!editorFile.value) return;

  try {
    await writeFileContent(editorFile.value.path, content, editorFileFs.value);
    // Invalidate cache for this file so next time it loads fresh content
    invalidate(editorFile.value.path, editorFileFs.value);
    success('File saved', `Saved: ${editorFile.value.name}`);
    showTextEditor.value = false;
    editorFile.value = null;
    editorFileFs.value = undefined;
    // Refresh directory to show updated file
    await refreshCurrentDirectory();
  } catch (err) {
    error('Failed to save file', err instanceof Error ? err.message : 'Unknown error');
  }
};

// State for queue operation dialog
const queueOperationPending = ref<{
  type: 'copy' | 'move' | 'delete' | 'archive' | 'extract';
  items: FileItem[];
} | null>(null);

// Add to queue handler
const handleQueueOperation = async (operationType: 'copy' | 'move' | 'delete' | 'archive' | 'extract') => {
  const { success, error } = useNotifications();

  try {
    // Determine FS
    let sourceFs: string | undefined = undefined;
    let destinationFs: string | undefined = undefined;
    if (isDualMode.value) {
        sourceFs = activePanel.value === 'left' ? leftPanelFilesystem.value : rightPanelFilesystem.value;
        destinationFs = activePanel.value === 'left' ? leftPanelFilesystem.value : rightPanelFilesystem.value;
    } else {
        const config = await invoke<any>('get_config');
        sourceFs = config.filesystem_backend === 'virtual' ? 'virtual' : 'real';
    }

    // Get selected items
    let selectedItems: FileItem[] = [];
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) {
        selectedItems = methods.getSelectedItems();
      }
    } else {
      selectedItems = getSelected();
    }

    if (selectedItems.length === 0) {
      return;
    }

    // For Copy and Move, we need to ask for destination
    if (operationType === 'copy' || operationType === 'move') {
      queueOperationPending.value = { type: operationType, items: selectedItems };

      // Show input dialog to select destination
      showInput(
        operationType === 'copy' ? 'Copy to...' : 'Move to...',
        'Enter destination path:',
        async (destinationPath: string) => {
          if (destinationPath) {
            // Assume destination FS is same as source if entering path manually
            await executeQueueOperation(operationType, selectedItems, destinationPath, sourceFs, destinationFs);
          }
          queueOperationPending.value = null;
        },
        '/', // default value
        '/path/to/destination' // placeholder
      );
      return;
    }

    // For Delete, Extract, Archive - execute directly
    // Use sourceFs as destFs for Archive/Extract (same dir by default)
    await executeQueueOperation(operationType, selectedItems, undefined, sourceFs, destinationFs);
  } catch (err) {
    error('Failed to add to queue', err instanceof Error ? err.message : String(err));
    queueOperationPending.value = null;
  }
};

// Execute queue operation
const executeQueueOperation = async (
  operationType: 'copy' | 'move' | 'delete' | 'archive' | 'extract',
  items: FileItem[],
  destination?: string,
  sourceFs?: string,
  destFs?: string
) => {
  const { success, error } = useNotifications();

  try {
    for (const item of items) {
      let params: any;
      let description = '';

      switch (operationType) {
        case 'copy':
          params = {
            type: 'Copy',
            sources: [item.path],
            destination: destination || '',
            sourceFs: sourceFs,
            destFs: destFs,
          };
          description = `Copy ${item.name} to ${destination}`;
          break;

        case 'move':
          params = {
            type: 'Move',
            sources: [item.path],
            destination: destination || '',
            sourceFs: sourceFs,
            destFs: destFs,
          };
          description = `Move ${item.name} to ${destination}`;
          break;

        case 'delete':
          params = {
            type: 'Delete',
            paths: [item.path],
            panelFs: sourceFs,
            sourceFs: sourceFs,
          };
          description = `Delete ${item.name}`;
          break;

        case 'archive':
          // Get current directory path
          let currentDir = '';
          if (isDualMode.value) {
            currentDir = '/' + activePanelPath.value.join('/');
          } else {
            currentDir = '/' + currentPath.value.join('/');
          }

          const archiveName = `${item.name}.zip`;
          const archivePathVal = currentDir.endsWith('/') ? `${currentDir}${archiveName}` : `${currentDir}/${archiveName}`;

          params = {
            type: 'Archive',
            sources: [item.path],
            archivePath: archivePathVal,
            format: 'zip',
            sourceFs: sourceFs,
            destFs: destFs,
          };
          description = `Create archive ${archiveName}`;
          break;

        case 'extract':
          // Get current directory path
          let extractDir = '';
          if (isDualMode.value) {
            extractDir = '/' + activePanelPath.value.join('/');
          } else {
            extractDir = '/' + currentPath.value.join('/');
          }

          params = {
            type: 'Extract',
            archivePath: item.path,
            destination: extractDir,
            sourceFs: sourceFs,
            destFs: destFs,
          };
          description = `Extract ${item.name}`;
          break;

        default:
          throw new Error(`Unknown operation type: ${operationType}`);
      }

      await addOperation(operationType, params, {
        priority: 'normal',
        description,
        tags: ['context-menu'],
      });
    }

    success('Added to Queue', `${items.length} item(s) added to operations queue`);

    // Open queue panel to show the added operations
    showOperationsQueue.value = true;
  } catch (err) {
    error('Failed to add to queue', err instanceof Error ? err.message : String(err));
  }
};

// Context menu handlers (work in both single and dual modes)
const contextMenuHandlers = {
  open: () => {
    if (contextMenu.value?.item) {
      fileOps.handleOpenFile(contextMenu.value.item);
    }
  },
  edit: () => {
    if (contextMenu.value?.item) {
      if (isDualMode.value) {
        const methods = getActivePanelMethods();
        if (methods) methods.handleEditFile();
      } else {
        handleEditFile(contextMenu.value.item, );
      }
    }
  },
  copy: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) methods.handleCopy();
    } else {
      fileOps.handleCopy(getSelected());
    }
  },
  cut: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) methods.handleCut();
    } else {
      fileOps.handleCut(getSelected());
    }
  },
  paste: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) methods.handlePaste();
    } else {
      fileOps.handlePaste(currentPath.value);
    }
  },
  rename: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) methods.handleRename();
    } else {
      fileOps.handleRename(getSelected(), currentPath.value, showInput);
    }
  },
  delete: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) methods.handleDelete();
    } else {
      fileOps.handleDelete(getSelected(), currentPath.value, clearSelection, showConfirm);
    }
  },
  addToFavorites: () => {
    if (contextMenu.value?.item) {
      handleAddFolderToBookmarks(contextMenu.value.item);
    }
  },
  openTerminal: () => {
    if (contextMenu.value?.item) {
      handleOpenTerminal(contextMenu.value.item);
    }
  },
  extractHere: async () => {
    if (contextMenu.value?.item) {
        const item = contextMenu.value.item;
        const { success, error } = useNotifications();
        
        let currentDirPath = '';
        if (isDualMode.value) {
            currentDirPath = '/' + activePanelPath.value.join('/');
        } else {
            currentDirPath = '/' + currentPath.value.join('/');
        }
        
        const destination = currentDirPath.endsWith('/') ? currentDirPath : currentDirPath + '/';
            
        try {
            await extractArchive(item.path, destination);
            success('Extracted', `Extracted ${item.name}`);
            
            // Refresh
             if (isDualMode.value) {
                getActivePanelMethods()?.handleRefresh();
             } else {
                handleRefresh();
             }
        } catch (err) {
            error('Extraction failed', err instanceof Error ? err.message : String(err));
        }
    }
  },
  extractToFolder: async () => {
    if (contextMenu.value?.item) {
        const item = contextMenu.value.item;
        const { success, error } = useNotifications();
        
        // Extract to a folder with the same name as the archive
        const folderName = item.name.replace(/\.(zip|tar|gz|tgz)$/i, '');
        let currentDirPath = '';
        if (isDualMode.value) {
            currentDirPath = '/' + activePanelPath.value.join('/');
        } else {
            currentDirPath = '/' + currentPath.value.join('/');
        }
        
        const destination = currentDirPath.endsWith('/') 
            ? `${currentDirPath}${folderName}` 
            : `${currentDirPath}/${folderName}`;
            
        try {
            await extractArchive(item.path, destination);
            success('Extracted', `Extracted ${item.name} to ${folderName}`);
            
            // Refresh
             if (isDualMode.value) {
                getActivePanelMethods()?.handleRefresh();
             } else {
                handleRefresh();
             }
        } catch (err) {
            error('Extraction failed', err instanceof Error ? err.message : String(err));
        }
    }
  },
  compressToZip: async () => {
      await handleCompress('zip');
  },
  compressToTar: async () => {
      await handleCompress('tar');
  },
  compressToTarGz: async () => {
      await handleCompress('tar.gz');
  },
  properties: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) {
        const selected = methods.getSelectedItems();
        if (selected.length === 1) {
          propertiesDialog.value = { isOpen: true, file: selected[0] };
        }
      }
    } else {
      const selected = getSelected();
      if (selected.length === 1) {
        propertiesDialog.value = { isOpen: true, file: selected[0] };
      }
    }
  },
  batchRename: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) {
        const selected = methods.getSelectedItems();
        if (selected.length > 0) {
          batchOperationFiles.value = selected;
          showBatchRenameDialog.value = true;
        }
      }
    } else {
      const selected = getSelected();
      if (selected.length > 0) {
        batchOperationFiles.value = selected;
        showBatchRenameDialog.value = true;
      }
    }
  },
  batchAttributes: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) {
        const selected = methods.getSelectedItems();
        if (selected.length > 0) {
          batchOperationFiles.value = selected;
          showBatchAttributeDialog.value = true;
        }
      }
    } else {
      const selected = getSelected();
      if (selected.length > 0) {
        batchOperationFiles.value = selected;
        showBatchAttributeDialog.value = true;
      }
    }
  },
  refresh: () => {
     if (isDualMode.value) {
        getActivePanelMethods()?.handleRefresh();
     } else {
        handleRefresh();
     }
  },
  newFolder: () => {
     if (isDualMode.value) {
        getActivePanelMethods()?.handleNewFolder();
     } else {
        fileOps.handleNewFolder(currentPath.value, showInput);
     }
  },
  newFile: () => {
     if (isDualMode.value) {
        getActivePanelMethods()?.handleNewFile();
     } else {
        inlineCreatorMode.value = 'file';
        showInlineCreator.value = true;
     }
  },
  selectAll: () => {
     if (isDualMode.value) {
        getActivePanelMethods()?.selectAll();
     } else {
        handleSelectAll();
     }
  },
  queueCopy: async () => {
    await handleQueueOperation('copy');
  },
  queueMove: async () => {
    await handleQueueOperation('move');
  },
  queueDelete: async () => {
    await handleQueueOperation('delete');
  },
  queueArchive: async () => {
    await handleQueueOperation('archive');
  },
  queueExtract: async () => {
    await handleQueueOperation('extract');
  },
  share: async () => {
    if (contextMenu.value?.item) {
      const { success, error } = useNotifications();
      
      let currentFs: string | undefined = undefined;
      if (isDualMode.value) {
          currentFs = activePanel.value === 'left' ? leftPanelFilesystem.value : rightPanelFilesystem.value;
      } else {
          try {
            const config = await invoke<any>('get_config');
            currentFs = config.filesystem_backend === 'virtual' ? 'virtual' : 'real';
          } catch (e) {
            console.error('Failed to get config for share:', e);
          }
      }

      try {
        const result = await invoke<{ url: string; qr_svg: string; filename: string }>('share_file', {
          path: contextMenu.value.item.path,
          filesystem: currentFs
        });
        shareInfo.value = result;
        showShareDialog.value = true;
        success('Ready to Share', `Scan QR code to download ${result.filename}`);
      } catch (err) {
        error('Share failed', err instanceof Error ? err.message : String(err));
      }
    }
  },
};

// Batch operations handlers
const handleBatchRenameConfirm = async (config: BatchRenameConfig) => {
  try {
    await queueBatchRename(batchOperationFiles.value, config);
    showBatchRenameDialog.value = false;
    batchOperationFiles.value = [];
    showBatchQueue.value = true;
    clearSelection();
    // Refresh will happen automatically after operation completes
  } catch (err) {
    console.error('Batch rename failed:', err);
    // Show error notification
  }
};

const handleBatchAttributeConfirm = async (changes: BatchAttributeChange) => {
  try {
    await queueBatchAttributeChange(batchOperationFiles.value, changes);
    showBatchAttributeDialog.value = false;
    batchOperationFiles.value = [];
    showBatchQueue.value = true;
    clearSelection();
    // Refresh will happen automatically after operation completes
  } catch (err) {
    console.error('Batch attribute change failed:', err);
    // Show error notification
  }
};

const handleBatchDialogCancel = () => {
  showBatchRenameDialog.value = false;
  showBatchAttributeDialog.value = false;
  batchOperationFiles.value = [];
};

const handleCompress = async (format: 'zip' | 'tar' | 'tar.gz') => {
    const { success, error } = useNotifications();
    
    // Get selected items
    let selectedItems: FileItem[] = [];
    if (isDualMode.value) {
        const methods = getActivePanelMethods();
        if (methods) {
            selectedItems = methods.getSelectedItems();
        }
    } else {
        selectedItems = getSelected();
    }

    if (selectedItems.length === 0) return;

    // Determine archive name
    let archiveName = 'archive';
    if (selectedItems.length === 1) {
        archiveName = selectedItems[0].name;
    } else {
        // Use parent folder name if multiple items
        let currentDirPath = '';
        if (isDualMode.value) {
            currentDirPath = '/' + activePanelPath.value.join('/');
        } else {
            currentDirPath = '/' + currentPath.value.join('/');
        }
        const parentName = currentDirPath.split('/').pop() || 'archive';
        archiveName = parentName;
    }

    // Prompt for name
    showInput(
        'Create Archive',
        'Enter archive name:',
        async (name: string) => {
            if (!name) return closeInput();
            
            let filename = name;
            if (!filename.endsWith(`.${format}`)) {
                filename += `.${format}`;
            }

            let currentDirPath = '';
            if (isDualMode.value) {
                currentDirPath = '/' + activePanelPath.value.join('/');
            } else {
                currentDirPath = '/' + currentPath.value.join('/');
            }
            
            const destinationPath = currentDirPath.endsWith('/') 
                ? `${currentDirPath}${filename}` 
                : `${currentDirPath}/${filename}`;

            const sourcePaths = selectedItems.map(i => i.path);

            try {
                await createArchive(sourcePaths, destinationPath);
                success('Archive Created', `Created ${filename}`);
                closeInput();
                
                // Refresh
                 if (isDualMode.value) {
                    getActivePanelMethods()?.handleRefresh();
                 } else {
                    handleRefresh();
                 }
            } catch (err) {
                error('Compression failed', err instanceof Error ? err.message : String(err));
            }
        },
        archiveName
    );
};

// Watch current path and load directory
watch(currentPath, async () => {
  const pathString = await fileOps.getCurrentDirectory(currentPath.value);
  await loadDirectory(pathString);
  clearSelection();
  // Устанавливаем фокус на первый элемент после загрузки
  if (processedFiles.value.length > 0) {
    setFocused(processedFiles.value[0].id);
  } else {
    setFocused(null);
  }
}, { immediate: true });

// Save tabs and paths - DIRECT SIMPLE VERSION
// ВАЖНО: deep: true чтобы отслеживать изменения внутри объектов!
watch([
  tabs,
  activeTabId,
  currentPath,
  panelMode,
  leftPanelTabs,
  leftPanelActiveTabId,
  rightPanelTabs,
  rightPanelActiveTabId,
  leftPanelWidthPercent,
  activePanel,
  expandedFolders,
  sidebarSectionsExpanded,
  isTerminalVisible,
  terminalHeight
], async () => {
  console.log('[App] 🔥 State changed:');
  console.log('  - Tabs:', tabs.value.length);
  console.log('  - Active tab:', activeTabId.value);
  console.log('  - Current path:', currentPath.value);
  console.log('  - Panel mode:', panelMode.value);
  console.log('  - Expanded folders:', expandedFolders.value.length);
  console.log('  - Sidebar quickAccess:', sidebarSectionsExpanded.value.quickAccess);
  console.log('  - Sidebar folderTree:', sidebarSectionsExpanded.value.folderTree);
  console.log('  - Sidebar favorites:', sidebarSectionsExpanded.value.favorites);

  try {
    const { invoke } = await import('@tauri-apps/api/core');

    // Сохраняем все табы со своими путями
    const tabsToSave = tabs.value.map(tab => ({
      id: tab.id,
      path: tab.path,
      name: tab.name,
    }));

    const stateToSave = {
      sidebar_width: sidebarWidth.value,
      preview_width: previewWidth.value,
      tabs: tabsToSave,
      active_tab_id: activeTabId.value,
      last_path: currentPath.value,
      panel_mode: panelMode.value,
      dual_panel_config: serializeDualPanelState(),
      window: { maximized: false },
      sidebar: {
        expanded_folders: expandedFolders.value,
        quick_access_expanded: sidebarSectionsExpanded.value.quickAccess,
        folder_tree_expanded: sidebarSectionsExpanded.value.folderTree,
        favorites_expanded: sidebarSectionsExpanded.value.favorites,
      },
      terminal_height: terminalHeight.value,
      terminal_visible: isTerminalVisible.value,
      edit_mode_enabled: editModeEnabled.value,
    };

    console.log('[App] 💾 Saving state with sidebar:', stateToSave.sidebar);
    await invoke('save_ui_state', { uiState: stateToSave });
    console.log('[App] ✅ Save successful!');
  } catch (error) {
    console.error('[App] ❌ Failed to save:', error);
  }
}, { deep: true });

// System stats update function
const updateSystemStats = async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const stats = await invoke<{ memory_mb: number; cpu_percent: number }>('get_system_stats');
    systemStats.value = stats;
  } catch (error) {
    console.error('[App] Failed to get system stats:', error);
  }
};

// Click outside handler
onMounted(async () => {
  document.addEventListener('click', closeContextMenu);
  window.addEventListener('vf-refresh-all', handleGlobalRefresh);

  // Check vault status FIRST
  await vault.checkStatus();

  // Load and apply theme FIRST (before any other initialization)
  const { loadTheme } = useTheme();
  await loadTheme();

  // Load File Coloring Config
  const { loadConfig: loadColorConfig } = useFileColoring();
  loadColorConfig();

  // Load bookmarks
  await loadBookmarks();

  // Load templates (for programmer mode)
  if (isProgrammerMode.value) {
    await loadTemplates();
  }

  // Start system stats updates
  updateSystemStats();
  setInterval(updateSystemStats, 2000); // Update every 2 seconds

  // Load UI state and restore tabs
  // ВАЖНО: сначала загружаем через useUIState чтобы sidebar состояние восстановилось
  const uiState = await loadUIState();
  console.log('[App] ===== Loaded UI state:', uiState);

  // Восстановить режим панелей
  if (uiState?.panel_mode) {
    console.log('[App] ✅ Restoring panel mode:', uiState.panel_mode);
    panelMode.value = uiState.panel_mode;

    if (uiState.panel_mode === 'dual' && uiState.dual_panel_config) {
      console.log('[App] ✅ Restoring dual panel config');
      loadDualPanelState(uiState.dual_panel_config);
    }
  }

  // Восстановить табы если есть (для обоих режимов)
  if (!isDualMode.value) {
    // Check current filesystem backend configuration
    const config = await invoke<any>('get_config');
    const isVirtualFS = config.filesystem_backend === 'virtual';
    console.log("is vfs = ", isVirtualFS)
    // Allow restoration for both Real and Virtual FS
    if (uiState && uiState.tabs && uiState.tabs.length > 0) {
      console.log('[App] ✅ Restoring', uiState.tabs.length, 'tabs');

      tabs.value = uiState.tabs.map(tabState => ({
        id: tabState.id,
        path: tabState.path,
        name: tabState.name,
        history: [tabState.path],
        historyIndex: 0,
      }));

      // Восстановить активный таб
      if (uiState.active_tab_id) {
        console.log('[App] ✅ Restoring active tab:', uiState.active_tab_id);
        activeTabId.value = uiState.active_tab_id;
      }
    } else if (uiState && uiState.last_path && uiState.last_path.length > 0 && !isVirtualFS) {
      // Only restore last path for Real FS (Virtual FS paths won't exist)
      console.log('[App] ✅ Restoring last path:', uiState.last_path);
      navigateTo(uiState.last_path);
    } else {
      // No saved state or empty - default behavior
      console.log('[App] ℹ️ No tabs/path to restore - navigating to home');
      const currentFs = isVirtualFS ? 'virtual' : 'real';
      const home = await getHomeDirectory(currentFs);
      console.log('[App] 🏠 Home directory:', home);
      await loadDirectory(home, currentFs);
    }
  } else {
    // Dual mode: check if we need to initialize home for virtual FS
    const config = await invoke<any>('get_config');
    const isVirtualFS = config.filesystem_backend === 'Virtual';
    if (isVirtualFS && (!uiState || !uiState.dual_panel_config)) {
      console.log('[App] ℹ️ Virtual FS in dual mode - initializing with home');
      const currentFs = 'virtual';
      const home = await getHomeDirectory(currentFs);
      await loadDirectory(home, currentFs);
    }
  }

  // Restore terminal state
  if (uiState?.terminal_height) {
    terminalHeight.value = uiState.terminal_height;
  }
  if (uiState?.terminal_visible !== undefined) {
    isTerminalVisible.value = uiState.terminal_visible;
  }
});
</script>

<template>
  <div class="h-screen flex flex-col bg-[var(--vf-bg-primary)] font-['Tahoma'] select-none overflow-hidden text-[var(--vf-text-primary)]">
    <!-- Menu Bar -->
    <div class="flex items-center h-[21px] bg-[var(--vf-bg-secondary)] border-b border-[var(--vf-border-default)] text-[11px]">
      <div class="px-2 py-0.5 hover:bg-[var(--vf-surface-hover)] hover:border hover:border-[var(--vf-accent-hover)] cursor-pointer">File</div>
      <div class="px-2 py-0.5 hover:bg-[var(--vf-surface-hover)] hover:border hover:border-[var(--vf-accent-hover)] cursor-pointer">Edit</div>
      <div class="px-2 py-0.5 hover:bg-[var(--vf-surface-hover)] hover:border hover:border-[var(--vf-accent-hover)] cursor-pointer">View</div>
      <div class="px-2 py-0.5 hover:bg-[var(--vf-surface-hover)] hover:border hover:border-[var(--vf-accent-hover)] cursor-pointer">Favorites</div>
      <div class="px-2 py-0.5 hover:bg-[var(--vf-surface-hover)] hover:border hover:border-[var(--vf-accent-hover)] cursor-pointer">Tools</div>
      <div class="px-2 py-0.5 hover:bg-[var(--vf-surface-hover)] hover:border hover:border-[var(--vf-accent-hover)] cursor-pointer">Help</div>
      <div class="ml-auto px-2 text-[var(--vf-text-tertiary)]">Arrows: navigate • Space: select • Enter: open • Ctrl+K: search</div>
    </div>

    <!-- Toolbar -->
    <Toolbar
        :tabs="isDualMode ? activePanelTabs : tabs"
        :active-tab-id="(isDualMode ? activePanelTabId : activeTabId) ?? 0"
        :current-path="isDualMode ? activePanelPath : currentPath"
        :view-mode="viewMode"
        :panel-mode="panelMode"
        :can-go-back="canGoBackComputed"
        :can-go-forward="canGoForwardComputed"
        :can-go-up="canGoUp"
        :is-current-path-bookmarked="isCurrentPathBookmarked"
        :is-programmer-mode="isProgrammerMode"
        :group-by="groupBy"
        :group-by-options="groupByOptions"
        :queue-active-count="queueActiveCount"
        @go-back="handleGoBack"
        @go-forward="handleGoForward"
        @go-up="handleGoUp"
        @go-home="handleGoHome"
        @navigate-to-breadcrumb="navigateToBreadcrumb"
        @navigate-to-path="handleNavigateToPath"
        @switch-tab="switchTab"
        @close-tab="closeTab"
        @add-tab="handleAddTab"
        @update:view-mode="(mode) => viewMode = mode"
        @open-command-palette="() => isCommandPaletteOpen = true"
        @toggle-bookmark="handleToggleBookmark"
        @toggle-programmer-mode="toggleProgrammerMode"
        @toggle-panel-mode="togglePanelMode"
        @toggle-dashboard="handleToggleDashboard"
        @toggle-operations-queue="() => showOperationsQueue = !showOperationsQueue"
        @update:group-by="(value) => groupBy = value"
    />

    <!-- Programmer Toolbar (only visible in Programmer Mode) -->
    <ProgrammerToolbar
        v-if="isProgrammerMode"
        :has-multiple-selected="hasMultipleSelected"
        :is-terminal-visible="isTerminalVisible"
        @toggle-terminal="toggleTerminal"
        @batch-rename="showBatchRenameDialog = true"
        @open-ftp="() => { /* TODO: implement FTP */ }"
        @toggle-resource-monitor="toggleWidget('resource-monitor')"
        @open-file-colors="() => { settingsInitialTab = 'colors'; showSettings = true; }"
        @open-widgets="showWidgetSelector = true"
    />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- File panels (flex-1) -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Dual Panel Mode -->
        <DualPanelContainer
          v-if="isDualMode"
          :view-mode="viewMode"
          @edit-file="handleEditFile"
          @preview-file="handlePreviewFile"
        />

        <!-- Single Panel Mode -->
        <template v-else>
        <!-- Sidebar -->
        <Sidebar
            :current-path="'/' + currentPath.join('/')"
            :width="sidebarWidth"
            @navigate="(path) => navigateTo(path.split('/').filter((p: any) => p))"
            @drop="handleSidebarDrop"
            @resize="handleSidebarResize"
        />

        <!-- Main Area -->
        <div class="flex-1 flex flex-col overflow-hidden">
          <!-- Panel Toolbar (for single mode) -->
          <PanelToolbar
              :tabs="tabs"
              :active-tab-id="activeTabId"
              :current-path="currentPath"
              :sort-by="sortBy"
              :sort-order="sortOrder"
              :show-hidden="showHidden"
              :edit-mode-enabled="editModeEnabled"
              @switch-tab="(id) => activeTabId = id"
              @close-tab="closeTab"
              @add-tab="addTab"
              @sort="handleSort"
              @select-all="handleSelectAll"
              @invert-selection="handleInvertSelection"
              @refresh="handleRefresh"
              @toggle-hidden="handleToggleHidden"
              @toggle-edit-mode="handleToggleEditMode"
              @navigate-to-breadcrumb="navigateToBreadcrumb"
          />

          <div class="flex-1 flex overflow-hidden">
          <!-- File List -->
          <FileList
              :items="processedFiles"
              :groups="fileGroups"
              :view-mode="viewMode"
              :selected-ids="selectedIds"
              :focused-id="focusedId"
              :is-loading="isLoading"
              :is-dragging="isDragging"
              :drag-target-id="dragOverId"
              :show-inline-creator="showInlineCreator"
              :inline-creator-mode="inlineCreatorMode"
              :current-path="currentPath"
              @item-click="(item, event) => handleItemClick(item, files, event)"
              @item-double-click="handleItemDoubleClick"
              @item-context-menu="handleContextMenu"
              @background-context-menu="(event) => showContextMenu(null, event)"
              @drag-start="handleDragStart"
              @drag-over="handleDragOver"
              @drag-leave="handleDragLeave"
              @drop="handleItemDrop"
              @drop-on-background="handleBackgroundDrop"
              @drag-over-background="handleDragOverBackground"
              @toggle-selection="(item) => handleItemClick(item, files, { ctrlKey: true } as MouseEvent)"
              @copy-item="(item) => fileOps.handleCopy([item])"
              @cut-item="(item) => fileOps.handleCut([item])"
              @delete-item="(item) => fileOps.handleDelete([item], currentPath, clearSelection, showConfirm)"
              @create-file="handleCreateFile"
              @batch-create-files="handleBatchCreateFiles"
              @cancel-inline-creator="handleCancelInlineCreator"
              @rename-item="(item) => fileOps.handleRename([item], currentPath, showInput)"
              @open-terminal="handleOpenTerminal"
          />

          <!-- Preview Panel -->
          <Preview
              :file="previewFile"
              :width="previewWidth"
              @close="previewFile = null"
              @open="fileOps.handleOpenFile"
              @resize="handlePreviewResize"
          />

          <!-- Dashboard Panel -->
          <Dashboard
              v-if="showDashboard"
              :files="processedFiles"
              :width="dashboardWidth"
              @close="showDashboard = false"
              @resize="handleDashboardResize"
          />
          </div>
        </div>
      </template>
      </div>

      <!-- Terminal Panel (bottom, if visible) -->
      <Terminal
        v-if="isTerminalVisible"
        :height="terminalHeight"
        :current-path="terminalWorkingDir"
        @resize="(h) => terminalHeight = h"
        @close="toggleTerminal"
      />
    </div>

    <!-- Command Palette -->
    <CommandPalette
        :is-open="isCommandPaletteOpen"
        @close="isCommandPaletteOpen = false"
        @execute="executeCommand"
    />

    <!-- Context Menu -->
    <ContextMenu
        v-if="contextMenu"
        :x="contextMenu.x"
        :y="contextMenu.y"
        :item="contextMenu.item"
        :selected-count="selectedCount"
        :has-clipboard-content="hasClipboardItems"
        @open="contextMenuHandlers.open"
        @edit="contextMenuHandlers.edit"
        @copy="contextMenuHandlers.copy"
        @cut="contextMenuHandlers.cut"
        @paste="contextMenuHandlers.paste"
        @rename="contextMenuHandlers.rename"
        @delete="contextMenuHandlers.delete"
        @add-to-favorites="contextMenuHandlers.addToFavorites"
        @open-terminal="contextMenuHandlers.openTerminal"
        @extract-here="contextMenuHandlers.extractHere"
        @extract-to-folder="contextMenuHandlers.extractToFolder"
        @compress-to-zip="contextMenuHandlers.compressToZip"
        @compress-to-tar="contextMenuHandlers.compressToTar"
        @compress-to-tar-gz="contextMenuHandlers.compressToTarGz"
        @properties="contextMenuHandlers.properties"
        @batch-rename="contextMenuHandlers.batchRename"
        @batch-attributes="contextMenuHandlers.batchAttributes"
        @refresh="contextMenuHandlers.refresh"
        @new-folder="contextMenuHandlers.newFolder"
        @new-file="contextMenuHandlers.newFile"
        @select-all="contextMenuHandlers.selectAll"
        @queue-copy="contextMenuHandlers.queueCopy"
        @queue-move="contextMenuHandlers.queueMove"
        @queue-delete="contextMenuHandlers.queueDelete"
        @queue-archive="contextMenuHandlers.queueArchive"
        @queue-extract="contextMenuHandlers.queueExtract"
        @share="contextMenuHandlers.share"
        @close="closeContextMenu"
    />

    <!-- Notifications -->
    <Notifications />

    <!-- Confirm Dialog -->
    <ConfirmDialog
        :is-open="confirmDialog.isOpen"
        :title="confirmDialog.title"
        :message="confirmDialog.message"
        :type="confirmDialog.type"
        @confirm="() => { confirmDialog.onConfirm(); closeConfirm(); }"
        @cancel="closeConfirm"
    />

    <!-- Properties Dialog -->
    <PropertiesDialog
        :is-open="propertiesDialog.isOpen"
        :file="propertiesDialog.file"
        @close="closeProperties"
    />

    <!-- Input Dialog -->
    <InputDialog
        :is-open="inputDialog.isOpen"
        :title="inputDialog.title"
        :label="inputDialog.label"
        :default-value="inputDialog.defaultValue"
        :placeholder="inputDialog.placeholder"
        @confirm="(value) => { inputDialog.onConfirm(value); closeInput(); }"
        @cancel="closeInput"
    />

    <!-- Conflict Resolution Dialog -->
    <ConflictDialog
        :is-open="isConflictDialogOpen"
        :conflict="currentConflict"
        @resolve="handleResolution"
        @cancel="handleConflictCancel"
    />

    <!-- Settings -->
    <Settings
        v-if="showSettings"
        :initial-tab="settingsInitialTab"
        @close="() => { showSettings = false; settingsInitialTab = 'general'; }"
    />

    <!-- Batch Rename Dialog -->
    <BatchRenameDialog
        :is-open="showBatchRenameDialog"
        :files="batchOperationFiles"
        @confirm="handleBatchRenameConfirm"
        @cancel="handleBatchDialogCancel"
    />

    <!-- Batch Attribute Dialog -->
    <BatchAttributeDialog
        :is-open="showBatchAttributeDialog"
        :files="batchOperationFiles"
        @confirm="handleBatchAttributeConfirm"
        @cancel="handleBatchDialogCancel"
    />

    <!-- Batch Operations Queue -->
    <div
        v-if="showBatchQueue"
        class="fixed bottom-5 right-5 w-[500px] h-[400px] bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] rounded-lg shadow-2xl overflow-hidden z-40"
    >
      <div class="flex items-center justify-between p-2 border-b border-[var(--vf-border-default)] bg-[var(--vf-bg-secondary)]">
        <h3 class="font-semibold text-sm">Batch Operations</h3>
        <button
          @click="showBatchQueue = false"
          class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] text-xl leading-none"
        >
          ×
        </button>
      </div>
      <BatchOperationsQueue />
    </div>

    <!-- Operations Queue Panel -->
    <div
      v-if="showOperationsQueue"
      class="fixed right-0 top-0 bottom-0 w-[500px] bg-[var(--vf-bg-primary)] border-l border-[var(--vf-border-default)] shadow-2xl z-[1000] flex flex-col"
    >
      <div class="flex items-center justify-between p-4 border-b border-[var(--vf-border-default)]">
        <h3 class="text-lg font-semibold text-[var(--vf-text-primary)]">Operations Queue</h3>
        <button
          @click="showOperationsQueue = false"
          class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] text-xl leading-none"
        >
          ×
        </button>
      </div>
      <div class="flex-1 overflow-hidden">
        <OperationsQueuePanel @open-settings="showQueueSettings = true" />
      </div>
    </div>

    <!-- Queue Settings Dialog -->
    <QueueSettingsDialog
      :is-open="showQueueSettings"
      @close="showQueueSettings = false"
    />

    <!-- File Operations Progress -->
    <OperationsProgress />

    <!-- Text Editor -->
    <TextEditor
      :file="editorFile"
      :panel-filesystem="editorFileFs"
      :is-open="showTextEditor"
      @close="handleCloseEditor"
      @save="handleSaveFile"
    />

    <!-- Status Bar -->
    <div class="h-[20px] bg-[var(--vf-bg-secondary)] border-t border-[var(--vf-border-default)] px-2 flex items-center text-[11px]">
      <span>{{ processedFiles.length }} items</span>
      <span v-if="selectedCount > 0" class="ml-4">{{ selectedCount }} selected</span>
      <span v-if="hasActiveFilters" class="ml-4 text-[var(--vf-accent-primary)]">🔍 Filters active</span>
      <span v-if="isDragging" class="ml-4 text-orange-600">📋 Dragging {{ draggedItems.length }} item(s)...</span>
      <button
        v-if="hasOperations"
        @click="showBatchQueue = !showBatchQueue"
        class="ml-4 text-[var(--vf-accent-primary)] hover:text-[var(--vf-accent-hover)] cursor-pointer"
      >
        📋 Batch Operations
      </button>
      <span class="ml-auto text-[var(--vf-text-secondary)]">
        RAM: {{ systemStats.memory_mb.toFixed(1) }} MB
        <span class="ml-3">CPU: {{ systemStats.cpu_percent.toFixed(1) }}%</span>
      </span>
    </div>

    <!-- Vault Security Overlay -->
    <VaultOverlay />

    <!-- Dynamic Widget Layer -->
    <WidgetLayer />

    <!-- Widget Selector -->
    <WidgetSelector
      :is-open="showWidgetSelector"
      @close="closeWidgetSelector"
    />

    <!-- Share Dialog -->
    <ShareDialog
      :is-open="showShareDialog"
      :share-info="shareInfo"
      @close="showShareDialog = false"
    />
  </div>
</template>

<style scoped>
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #94a3b8;
}
</style>