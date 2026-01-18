<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Components
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
import StegoVaultModal from './components/StegoVaultModal.vue';
import TorrentManager from './components/TorrentManager.vue';

// Composables - Core
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

// Composables - New Refactored
import { useAppUIState } from './composables/useAppUIState';
import { useEditorState } from './composables/useEditorState';
import { useContextMenuActions } from './composables/useContextMenuActions';

// Utils
import { createKeyboardShortcuts } from './utils/shortcuts';
import type { FileItem, BatchRenameConfig, BatchAttributeChange } from './types';

// ============================================================================
// CORE COMPOSABLES
// ============================================================================

// File System
const { files, isLoading, loadDirectory, normalizePath, getHomeDirectory, openTerminal } = useFileSystem();

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
const { processFiles, hasActiveFilters } = useSearch();

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
  success,
  error,
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

// Context Menu (global)
const { contextMenu, showContextMenu, closeContextMenu } = useContextMenu();

// Grouping
const { groupBy, groupByOptions, groupFiles } = useGrouping();

// Global Refresh
const { refreshAllPanels } = useGlobalRefresh();

// Widgets
const { toggleWidget, showWidgetSelector, openWidgetSelector, closeWidgetSelector } = useWidgets();

// Clipboard
const { hasClipboardItems } = useClipboard();

// Batch Operations
const { queueBatchRename, queueBatchAttributeChange, hasOperations } = useBatchOperations(async () => {
  await refreshAllPanels([]);
});

// Operations Queue
const { statistics: queueStatistics } = useOperationsQueue();

// Programmer Mode
const { isProgrammerMode, toggleProgrammerMode } = useProgrammerMode();

// Templates
const { loadTemplates } = useTemplates();

// ============================================================================
// NEW REFACTORED COMPOSABLES
// ============================================================================

// UI State Management
const appUI = useAppUIState();

// Editor State
const editor = useEditorState();

const stegoModal = ref({
  isOpen: false,
  sessionId: '',
  title: '',
});

// ============================================================================
// HELPERS
// ============================================================================

// Helper: Get current directory path as string
const getCurrentDirectoryPath = async (): Promise<string> => {
  let pathString = currentPath.value.join('/');
  if (pathString && !pathString.startsWith('/')) {
    pathString = '/' + pathString;
  }
  if (!pathString) {
    return await getHomeDirectory(appUI.currentFilesystemBackend.value);
  }
  return pathString;
};

// Refresh current directory
const refreshCurrentDirectory = async () => {
  const pathString = await getCurrentDirectoryPath();
  await loadDirectory(pathString, appUI.currentFilesystemBackend.value);
};

// File Operations (with auto-refresh callback)
const fileOps = useFileOperations(async () => {
  await refreshAllPanels(currentPath.value);
});

// ============================================================================
// CONTEXT MENU ACTIONS
// ============================================================================

const contextMenuActions = useContextMenuActions({
  isDualMode: () => isDualMode.value,
  getActivePanelMethods: () => getActivePanelMethods(),
  getActivePanelPath: () => activePanelPath.value,
  getCurrentPath: () => currentPath.value,
  getSelectedItems: () => getSelectedItems(files.value),
  clearSelection,
  refreshCurrentDirectory,
  openEditor: editor.openEditor,
  openBatchRename: appUI.openBatchRename,
  openBatchAttribute: appUI.openBatchAttribute,
  getActiveFilesystem: () => {
    if (isDualMode.value) {
      return activePanel.value === 'left' ? leftPanelFilesystem.value : rightPanelFilesystem.value;
    }
    return appUI.currentFilesystemBackend.value;
  },
});

// Handle share action separately (needs UI state)
const handleShareAction = async () => {
  const result = await contextMenuActions.share();
  if (result) {
    appUI.shareInfo.value = result;
    appUI.showShareDialog.value = true;
  }
};

// Context menu handlers object for template
const contextMenuHandlers = {
  ...contextMenuActions,
  share: handleShareAction,
  // Override newFile and selectAll with UI-aware versions
  newFile: () => {
    if (isDualMode.value) {
      getActivePanelMethods()?.handleNewFile();
    } else {
      appUI.openInlineCreator('file');
    }
  },
  selectAll: () => {
    if (isDualMode.value) {
      getActivePanelMethods()?.selectAll();
    } else {
      files.value.forEach(file => selectedIds.value.add(file.id));
    }
  },
};

// ============================================================================
// COMPUTED VALUES
// ============================================================================

const terminalWorkingDir = computed(() => {
  if (isDualMode.value) {
    return '/' + activePanelPath.value.join('/');
  }
  return '/' + currentPath.value.join('/');
});

const hasMultipleSelected = computed(() => {
  if (isDualMode.value) {
    const methods = getActivePanelMethods();
    const selected = methods?.getSelectedIds() || new Set();
    return selected.size > 1;
  }
  return selectedIds.value.size > 1;
});

const queueActiveCount = computed(() => {
  return queueStatistics.value.running + queueStatistics.value.queued + queueStatistics.value.scheduled;
});

const isCurrentPathBookmarked = computed(() => {
  const path = '/' + currentPath.value.join('/');
  return isBookmarked(path);
});

// Process, filter and sort files
const processedFiles = computed(() => {
  let result = processFiles(files.value);

  if (!isDualMode.value && !appUI.showHidden.value) {
    result = result.filter(file => !file.name.startsWith('.'));
  }

  if (!isDualMode.value) {
    result = [...result].sort((a, b) => {
      const aIsFolder = a.type === 'folder' || a.type === 'drive' || a.type === 'system';
      const bIsFolder = b.type === 'folder' || b.type === 'drive' || b.type === 'system';

      if (aIsFolder && !bIsFolder) return -1;
      if (!aIsFolder && bIsFolder) return 1;

      let comparison = 0;
      switch (appUI.sortBy.value) {
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

      return appUI.sortOrder.value === 'asc' ? comparison : -comparison;
    });
  }

  return result;
});

const fileGroups = computed(() => groupFiles(processedFiles.value));

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

// ============================================================================
// EVENT HANDLERS
// ============================================================================

// Navigation Handlers
const handleGoBack = () => {
  if (isDualMode.value) {
    getActivePanelMethods()?.goBack();
  } else {
    goBack();
  }
};

const handleGoForward = () => {
  if (isDualMode.value) {
    getActivePanelMethods()?.goForward();
  } else {
    goForward();
  }
};

const handleGoUp = () => {
  if (isDualMode.value) {
    getActivePanelMethods()?.goUp();
  } else {
    goUp();
  }
};

const handleGoHome = async () => {
  if (isDualMode.value) {
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

const handleAddTab = () => {
  if (isDualMode.value) {
    getActivePanelMethods()?.addTab();
  } else {
    addTab();
  }
};

const handleNavigateToPath = async (path: string) => {
  try {
    const normalizedPath = await normalizePath(path);
    const pathArray = normalizedPath.replace(/^\//, '').split('/').filter(p => p);
    navigateTo(pathArray);
  } catch (err) {
    error('Invalid path', err instanceof Error ? err.message : 'Path not found');
  }
};

// Item Handlers
const handleItemDoubleClick = (item: FileItem) => {
  if (item.name.endsWith('.safe')) {
    handleOpenStegoFile(item.path);
    return;
  }
  
  if (item.type === 'folder' || item.type === 'drive' || item.type === 'system' || item.type === 'archive') {
    const pathParts = item.path.split('/').filter(p => p);
    navigateTo(pathParts);
  } else {
    const isEditableFile = item.type === 'file' || item.type === 'code';
    if (editModeEnabled.value && isEditableFile) {
      editor.openEditor(item);
    } else {
      appUI.openPreview(item);
    }
  }
};

const handleContextMenu = (item: FileItem, event: MouseEvent) => {
  showContextMenu(item, event);
};

// Drag & Drop Handlers
const handleDragStart = (item: FileItem, event: DragEvent) => {
  const items = hasSelection.value && isSelected(item.id)
    ? getSelectedItems(files.value)
    : [item];

  console.log('[App] Starting drag with items:', items.length, items.map(i => i.name));
  startDrag(items, event);
};

const handleItemDrop = async (item: FileItem, event: DragEvent) => {
  event.preventDefault();
  console.log('[App] Drop on item:', item.name);

  const onMove = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'move', srcFs, destFs);
  const onCopy = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'copy', srcFs, destFs);

  await handleDrop(item, event, onMove, onCopy);
  await refreshCurrentDirectory();
};

const handleSidebarDrop = async (targetPath: string, event: DragEvent) => {
  event.preventDefault();
  console.log('[App] Drop on sidebar path:', targetPath);

  const targetItem: FileItem = {
    id: targetPath,
    name: targetPath.split('/').pop() || '',
    path: targetPath,
    type: 'folder',
    size: 0,
    modified: '',
  };

  const onMove = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'move', srcFs, destFs);
  const onCopy = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'copy', srcFs, destFs);

  await handleDrop(targetItem, event, onMove, onCopy);
  await refreshCurrentDirectory();
};

const handleBackgroundDrop = async (event: DragEvent) => {
  console.log('[App] Background Drop Detected!');

  const pathString = await getCurrentDirectoryPath();
  const targetItem: FileItem = {
    id: pathString,
    name: pathString.split('/').pop() || 'root',
    path: pathString,
    type: 'folder',
    size: 0,
    modified: '',
    tags: [],
    permissions: { readable: true, writable: true, executable: true }
  };

  const onMove = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'move', srcFs, destFs);
  const onCopy = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'copy', srcFs, destFs);

  await handleDrop(targetItem, event, onMove, onCopy);
};

// Toolbar Handlers
const handleSort = (field: 'name' | 'size' | 'modified' | 'type', order: 'asc' | 'desc') => {
  appUI.sortBy.value = field;
  appUI.sortOrder.value = order;
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
  appUI.showHidden.value = !appUI.showHidden.value;
};

const handleToggleEditMode = () => {
  editModeEnabled.value = !editModeEnabled.value;
};

// Steganography Handler
const handleOpenStegoFile = async (path: string) => {
  showInput(
    'Open Secure Folder',
    `Enter password for '${path.split('/').pop()}':`,
    async (password) => {
      if (password) {
        try {
          const sessionId = await vault.openStegoContainer(path, password);

          // Open modal with session ID
          stegoModal.value = {
            isOpen: true,
            sessionId: sessionId,
            title: `Secure Folder: ${path.split('/').pop()}`,
          };
          
          success('Unlocked', `Opened secure folder`);
        } catch (err) {
          error('Failed to open', err instanceof Error ? err.message : String(err));
        }
      }
    },
    '',
    'Password',
    'password'
  );
};

const handleOpenStego = async () => {
  try {
    const path = await invoke<string | null>('vault_select_file');
    if (path) {
      await handleOpenStegoFile(path);
    }
  } catch (err) {
    console.error('Failed to select file:', err);
  }
};

// Bookmark Handlers
const handleToggleBookmark = async () => {
  const path = await getCurrentDirectoryPath();

  if (isBookmarked(path)) {
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
    const folderName = currentPath.value[currentPath.value.length - 1] || 'Root';
    const bookmark = await addBookmark(path, folderName);
    if (bookmark) {
      success('Added to Favorites', `Added: ${bookmark.name}`);
    } else {
      error('Failed to add bookmark', 'This folder may already be bookmarked');
    }
  }
};

// Inline File Creator Handlers
const handleCreateFile = async (payload: { name: string; isFolder: boolean; templateId?: string }) => {
  try {
    if (payload.isFolder) {
      await fileOps.handleNewFile(currentPath.value, payload.name);
    } else {
      await fileOps.handleNewFile(currentPath.value, payload.name, payload.templateId);
    }
    appUI.closeInlineCreator();
  } catch (err) {
    console.error('Failed to create file:', err);
  }
};

const handleBatchCreateFiles = async (names: string[]) => {
  try {
    const files = names.map(name => ({ name }));
    await fileOps.handleBatchCreate(currentPath.value, files);
    appUI.closeInlineCreator();
  } catch (err) {
    console.error('Failed to batch create files:', err);
  }
};

// Batch Operations Handlers
const handleBatchRenameConfirm = async (config: BatchRenameConfig) => {
  try {
    await queueBatchRename(appUI.batchOperationFiles.value, config);
    appUI.closeBatchDialogs();
    appUI.showBatchQueue.value = true;
    clearSelection();
  } catch (err) {
    console.error('Batch rename failed:', err);
  }
};

const handleBatchAttributeConfirm = async (changes: BatchAttributeChange) => {
  try {
    await queueBatchAttributeChange(appUI.batchOperationFiles.value, changes);
    appUI.closeBatchDialogs();
    appUI.showBatchQueue.value = true;
    clearSelection();
  } catch (err) {
    console.error('Batch attribute change failed:', err);
  }
};

// Resize Handlers
const handleSidebarResize = (width: number) => {
  sidebarWidth.value = width;
};

const handlePreviewResize = (width: number) => {
  previewWidth.value = width;
};

const handleDashboardResize = (width: number) => {
  appUI.dashboardWidth.value = width;
};

// Global Refresh Handler
const handleGlobalRefresh = async () => {
  if (!isDualMode.value) {
    await refreshCurrentDirectory();
  }
};

// ============================================================================
// KEYBOARD SHORTCUTS & COMMANDS
// ============================================================================

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
  onSettings: () => appUI.openSettings('general'),
});

const executeCommand = (cmd: { id: string }) => {
  if (cmd.id === 'copy-path') {
    commands.copyPathCommand(getSelectedItems(files.value));
  } else if (cmd.id === 'select-all') {
    commands.selectAllCommand(files.value, selectAll);
  } else {
    commands.executeCommand(cmd);
  }
};

// Keyboard shortcuts
const shortcuts = createKeyboardShortcuts(
  {
    openCommandPalette: appUI.openCommandPalette,
    closeDialogs: () => {
      if (contextMenu.value) {
        closeContextMenu();
        return;
      }
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
      if (appUI.isCommandPaletteOpen.value) {
        appUI.closeCommandPalette();
        return;
      }
      if (propertiesDialog.value.isOpen) {
        closeProperties();
        return;
      }
      if (appUI.showBatchRenameDialog.value || appUI.showBatchAttributeDialog.value) {
        appUI.closeBatchDialogs();
        return;
      }
      if (appUI.showInlineCreator.value) {
        appUI.closeInlineCreator();
        return;
      }
      if (appUI.showSettings.value) {
        appUI.closeSettings();
        return;
      }
      if (appUI.showDashboard.value) {
        appUI.showDashboard.value = false;
        return;
      }
      if (appUI.showBatchQueue.value) {
        appUI.showBatchQueue.value = false;
        return;
      }
      if (appUI.previewFile.value) {
        appUI.closePreview();
        return;
      }
      if (editor.showTextEditor.value) {
        editor.closeEditor();
        return;
      }
      if (showWidgetSelector.value) {
        closeWidgetSelector();
        return;
      }
      if (activeNotifications.value.length > 0) {
        clearNotifications();
        return;
      }
      if (isDualMode.value) {
        getActivePanelMethods()?.clearSelection();
      } else {
        clearSelection();
      }
    },
    selectAll: (files: FileItem[]) => {
      if (isDualMode.value) {
        getActivePanelMethods()?.selectAll();
      } else {
        selectAll(files);
      }
    },
    addTab: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.addTab();
      } else {
        addTab();
      }
    },
    closeTab: (canClose: boolean) => {
      if (canClose) {
        if (isDualMode.value) {
          getActivePanelMethods()?.closeTab();
        } else if (tabs.value.length > 1) {
          closeTab(activeTabId.value);
        }
      }
    },
    goUp: (canGoUpValue: boolean) => {
      if (canGoUpValue) {
        handleGoUp();
      }
    },
    handleCopy: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.handleCopy();
      } else {
        fileOps.handleCopy(getSelectedItems(files.value));
      }
    },
    handleCut: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.handleCut();
      } else {
        fileOps.handleCut(getSelectedItems(files.value));
      }
    },
    handlePaste: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.handlePaste();
      } else {
        fileOps.handlePaste(currentPath.value);
      }
    },
    handleDelete: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.handleDelete();
      } else {
        fileOps.handleDelete(getSelectedItems(files.value), currentPath.value, clearSelection, showConfirm);
      }
    },
    handleRename: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.handleRename();
      } else {
        fileOps.handleRename(getSelectedItems(files.value), currentPath.value, showInput);
      }
    },
    handleRefresh: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.handleRefresh();
      } else {
        fileOps.handleRefresh(currentPath.value);
      }
    },
    handleNewFolder: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.handleNewFolder();
      } else {
        fileOps.handleNewFolder(currentPath.value, showInput);
      }
    },
    handleNewFile: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.handleNewFile();
      } else {
        appUI.openInlineCreator('file');
      }
    },
    toggleProgrammerMode: () => {
      toggleProgrammerMode();
    },
    toggleBookmark: handleToggleBookmark,
    openSettings: () => appUI.openSettings('general'),
    switchPanels: isDualMode.value ? () => {
      switchActivePanel(activePanel.value === 'left' ? 'right' : 'left');
    } : undefined,
    toggleTerminal: () => toggleTerminal(),
    moveFocusUp: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.moveFocusUp();
      } else {
        moveFocusUp(processedFiles.value);
      }
    },
    moveFocusDown: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.moveFocusDown();
      } else {
        moveFocusDown(processedFiles.value);
      }
    },
    moveFocusToFirst: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.moveFocusToFirst();
      } else {
        moveFocusToFirst(processedFiles.value);
      }
    },
    moveFocusToLast: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.moveFocusToLast();
      } else {
        moveFocusToLast(processedFiles.value);
      }
    },
    selectFocused: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.selectFocused();
      } else {
        selectFocused();
      }
    },
    toggleFocusedSelection: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.toggleFocusedSelection();
      } else {
        toggleFocusedSelection();
      }
    },
    openFocusedItem: () => {
      if (isDualMode.value) {
        getActivePanelMethods()?.openFocusedItem();
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

// Swipe navigation
useSwipeNavigation({
  onSwipeLeft: () => {
    if (isDualMode.value) {
      getActivePanelMethods()?.goBack();
    } else {
      goBack();
    }
  },
  onSwipeRight: () => {
    if (isDualMode.value) {
      getActivePanelMethods()?.goForward();
    } else {
      goForward();
    }
  },
  canSwipeLeft: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      return methods ? methods.canGoBack() : false;
    }
    return canGoBack.value;
  },
  canSwipeRight: () => {
    if (isDualMode.value) {
      const methods = getActivePanelMethods();
      return methods ? methods.canGoForward() : false;
    }
    return canGoForward.value;
  },
});

// ============================================================================
// WATCHERS
// ============================================================================

// Watch vault unlock to refresh files
watch(() => vault.status.value, async (newStatus) => {
  if (newStatus === 'UNLOCKED') {
    console.log('[App] 🔓 Vault unlocked - refreshing view');
    setTimeout(async () => {
      await handleGlobalRefresh();
    }, 50);
  }
});

// Watch current path and load directory
watch(currentPath, async () => {
  const pathString = await fileOps.getCurrentDirectory(currentPath.value, appUI.currentFilesystemBackend.value);
  await loadDirectory(pathString, appUI.currentFilesystemBackend.value);
  clearSelection();
  if (processedFiles.value.length > 0) {
    setFocused(processedFiles.value[0].id);
  } else {
    setFocused(null);
  }
});

// Save UI state
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
  console.log('[App] 🔥 State changed');

  try {
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

    console.log('[App] 💾 Saving state');
    await invoke('save_ui_state', { uiState: stateToSave });
    console.log('[App] ✅ Save successful!');
  } catch (error) {
    console.error('[App] ❌ Failed to save:', error);
  }
}, { deep: true });

// ============================================================================
// LIFECYCLE
// ============================================================================

onMounted(async () => {
  // Event listeners
  document.addEventListener('click', closeContextMenu);
  window.addEventListener('vf-refresh-all', handleGlobalRefresh);

  // Listen for filesystem backend changes
  window.addEventListener('fs-config-changed', async () => {
    try {
      console.log('[App] 🔄 FS Config changed, reloading backend...');

      // Reload filesystem backend from config
      await appUI.loadFilesystemBackend();
      await vault.checkStatus();

      const home = await getHomeDirectory(appUI.currentFilesystemBackend.value);
      const pathParts = home.split('/').filter(p => p);
      navigateTo(pathParts);

      await handleGlobalRefresh();
    } catch (e) {
      console.error('[App] Failed to handle FS config change:', e);
    }
  });

  // Check vault status
  await vault.checkStatus();

  // Load theme
  const { loadTheme } = useTheme();
  await loadTheme();

  // Load file coloring config
  const { loadConfig: loadColorConfig } = useFileColoring();
  loadColorConfig();

  // Load bookmarks
  await loadBookmarks();

  // Load templates (for programmer mode)
  if (isProgrammerMode.value) {
    await loadTemplates();
  }

  // Start system stats updates
  const updateSystemStats = async () => {
    try {
      const stats = await invoke<{ memory_mb: number; cpu_percent: number }>('get_system_stats');
      appUI.updateSystemStats(stats);
    } catch (error) {
      console.error('[App] Failed to get system stats:', error);
    }
  };

  updateSystemStats();
  setInterval(updateSystemStats, 2000);

  // Load UI state and restore tabs
  const uiState = await loadUIState();
  console.log('[App] ===== Loaded UI state:', uiState);

  // Restore panel mode
  if (uiState?.panel_mode) {
    console.log('[App] ✅ Restoring panel mode:', uiState.panel_mode);
    panelMode.value = uiState.panel_mode;

    if (uiState.panel_mode === 'dual' && uiState.dual_panel_config) {
      console.log('[App] ✅ Restoring dual panel config');
      loadDualPanelState(uiState.dual_panel_config);
    }
  }

  // Load filesystem backend from config FIRST
  await appUI.loadFilesystemBackend();

  // Restore tabs for single mode
  if (!isDualMode.value) {

    if (uiState && uiState.tabs && uiState.tabs.length > 0) {
      console.log('[App] ✅ Restoring', uiState.tabs.length, 'tabs');

      tabs.value = uiState.tabs.map(tabState => ({
        id: tabState.id,
        path: tabState.path,
        name: tabState.name,
        history: [tabState.path],
        historyIndex: 0,
      }));

      if (uiState.active_tab_id) {
        console.log('[App] ✅ Restoring active tab:', uiState.active_tab_id);
        activeTabId.value = uiState.active_tab_id;
      }
    } else if (uiState && uiState.last_path && uiState.last_path.length > 0) {
      console.log('[App] ✅ Restoring last path:', uiState.last_path);
      navigateTo(uiState.last_path);
    } else {
      console.log('[App] ℹ️ No tabs/path to restore - navigating to home');
      const currentFs = appUI.currentFilesystemBackend.value;
      const home = await getHomeDirectory(currentFs);
      console.log('[App] 🏠 Home directory:', home);

      const pathParts = home.split('/').filter(p => p);
      navigateTo(pathParts);
    }

    // Force initial load
    setTimeout(async () => {
      const pathString = await fileOps.getCurrentDirectory(currentPath.value, appUI.currentFilesystemBackend.value);
      console.log('[App] 🚀 Force initial load:', pathString, 'Backend:', appUI.currentFilesystemBackend.value);
      await loadDirectory(pathString, appUI.currentFilesystemBackend.value);
    }, 100);
  } else {
    const config = await invoke<any>('get_config');
    const isVirtualFS = config.filesystem_backend === 'virtual';
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
      :view-mode="appUI.viewMode.value"
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
      @update:view-mode="(mode) => appUI.viewMode.value = mode"
      @open-command-palette="appUI.openCommandPalette"
      @toggle-bookmark="handleToggleBookmark"
      @toggle-programmer-mode="toggleProgrammerMode"
      @toggle-panel-mode="togglePanelMode"
      @toggle-dashboard="appUI.toggleDashboard"
      @toggle-operations-queue="appUI.toggleOperationsQueue"
      @update:group-by="(value) => groupBy = value"
    />

    <!-- Programmer Toolbar -->
    <ProgrammerToolbar
      v-if="isProgrammerMode"
      :has-multiple-selected="hasMultipleSelected"
      :is-terminal-visible="isTerminalVisible"
      @toggle-terminal="toggleTerminal"
      @batch-rename="appUI.showBatchRenameDialog.value = true"
      @open-ftp="() => {}"
      @toggle-resource-monitor="toggleWidget('resource-monitor')"
      @open-file-colors="() => appUI.openSettings('colors')"
      @open-widgets="openWidgetSelector"
      @open-stego="handleOpenStego"
    />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="flex-1 flex overflow-hidden">
        <!-- Dual Panel Mode -->
        <DualPanelContainer
          v-if="isDualMode"
          :view-mode="appUI.viewMode.value"
          @edit-file="editor.openEditor"
          @preview-file="appUI.openPreview"
        />

        <!-- Single Panel Mode -->
        <template v-else>
          <Sidebar
            :current-path="'/' + currentPath.join('/')"
            :width="sidebarWidth"
            @navigate="(path) => navigateTo(path.split('/').filter((p: any) => p))"
            @drop="handleSidebarDrop"
            @resize="handleSidebarResize"
          />

          <div class="flex-1 flex flex-col overflow-hidden">
            <PanelToolbar
              :tabs="tabs"
              :active-tab-id="activeTabId"
              :current-path="currentPath"
              :sort-by="appUI.sortBy.value"
              :sort-order="appUI.sortOrder.value"
              :show-hidden="appUI.showHidden.value"
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
              <FileList
                :items="processedFiles"
                :groups="fileGroups"
                :view-mode="appUI.viewMode.value"
                :selected-ids="selectedIds"
                :focused-id="focusedId"
                :is-loading="isLoading"
                :is-dragging="isDragging"
                :drag-target-id="dragOverId"
                :show-inline-creator="appUI.showInlineCreator.value"
                :inline-creator-mode="appUI.inlineCreatorMode.value"
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
                @cancel-inline-creator="appUI.closeInlineCreator"
                @rename-item="(item) => fileOps.handleRename([item], currentPath, showInput)"
                @open-terminal="(item) => openTerminal(item.path)"
              />

              <Preview
                :file="appUI.previewFile.value"
                :width="previewWidth"
                @close="appUI.closePreview"
                @open="fileOps.handleOpenFile"
                @resize="handlePreviewResize"
              />

              <Dashboard
                v-if="appUI.showDashboard.value"
                :files="processedFiles"
                :width="appUI.dashboardWidth.value"
                @close="appUI.showDashboard.value = false"
                @resize="handleDashboardResize"
              />
            </div>
          </div>
        </template>
      </div>

      <!-- Terminal Panel -->
      <Terminal
        v-if="isTerminalVisible"
        :height="terminalHeight"
        :current-path="terminalWorkingDir"
        @resize="(h) => terminalHeight = h"
        @close="toggleTerminal"
      />
    </div>

    <!-- Dialogs & Overlays -->
    <CommandPalette
      :is-open="appUI.isCommandPaletteOpen.value"
      @close="appUI.closeCommandPalette"
      @execute="executeCommand"
    />

    <ContextMenu
      v-if="contextMenu"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :item="contextMenu.item"
      :selected-count="selectedCount"
      :has-clipboard-content="hasClipboardItems"
      :show-programmer-mode="isProgrammerMode"
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
      @hide-to="contextMenuHandlers.hideTo"
      @extract-hidden="contextMenuHandlers.extractHidden"
      @create-secure-folder="contextMenuHandlers.createSecureFolder"
      @protect-selection="contextMenuHandlers.protectSelection"
      @close="closeContextMenu"
    />

    <Notifications />

    <ConfirmDialog
      :is-open="confirmDialog.isOpen"
      :title="confirmDialog.title"
      :message="confirmDialog.message"
      :type="confirmDialog.type"
      @confirm="() => { confirmDialog.onConfirm(); closeConfirm(); }"
      @cancel="closeConfirm"
    />

    <PropertiesDialog
      :is-open="propertiesDialog.isOpen"
      :file="propertiesDialog.file"
      @close="closeProperties"
    />

    <InputDialog
      :is-open="inputDialog.isOpen"
      :title="inputDialog.title"
      :label="inputDialog.label"
      :default-value="inputDialog.defaultValue"
      :placeholder="inputDialog.placeholder"
      :input-type="inputDialog.inputType"
      @confirm="(value) => { inputDialog.onConfirm(value); closeInput(); }"
      @cancel="closeInput"
    />

    <ConflictDialog
      :is-open="isConflictDialogOpen"
      :conflict="currentConflict"
      @resolve="handleResolution"
      @cancel="handleConflictCancel"
    />

    <Settings
      v-if="appUI.showSettings.value"
      :initial-tab="appUI.settingsInitialTab.value"
      @close="appUI.closeSettings"
    />

    <BatchRenameDialog
      :is-open="appUI.showBatchRenameDialog.value"
      :files="appUI.batchOperationFiles.value"
      @confirm="handleBatchRenameConfirm"
      @cancel="appUI.closeBatchDialogs"
    />

    <BatchAttributeDialog
      :is-open="appUI.showBatchAttributeDialog.value"
      :files="appUI.batchOperationFiles.value"
      @confirm="handleBatchAttributeConfirm"
      @cancel="appUI.closeBatchDialogs"
    />

    <div
      v-if="appUI.showBatchQueue.value"
      class="fixed bottom-5 right-5 w-[500px] h-[400px] bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] rounded-lg shadow-2xl overflow-hidden z-40"
    >
      <div class="flex items-center justify-between p-2 border-b border-[var(--vf-border-default)] bg-[var(--vf-bg-secondary)]">
        <h3 class="font-semibold text-sm">Batch Operations</h3>
        <button
          @click="appUI.showBatchQueue.value = false"
          class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] text-xl leading-none"
        >
          ×
        </button>
      </div>
      <BatchOperationsQueue />
    </div>

    <div
      v-if="appUI.showOperationsQueue.value"
      class="fixed right-0 top-0 bottom-0 w-[500px] bg-[var(--vf-bg-primary)] border-l border-[var(--vf-border-default)] shadow-2xl z-[1000] flex flex-col"
    >
      <div class="flex items-center justify-between p-4 border-b border-[var(--vf-border-default)]">
        <h3 class="text-lg font-semibold text-[var(--vf-text-primary)]">Operations Queue</h3>
        <button
          @click="appUI.showOperationsQueue.value = false"
          class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] text-xl leading-none"
        >
          ×
        </button>
      </div>
      <div class="flex-1 overflow-hidden">
        <OperationsQueuePanel @open-settings="appUI.showQueueSettings.value = true" />
      </div>
    </div>

    <QueueSettingsDialog
      :is-open="appUI.showQueueSettings.value"
      @close="appUI.showQueueSettings.value = false"
    />

    <OperationsProgress />

    <TextEditor
      :file="editor.editorFile.value"
      :panel-filesystem="editor.editorFileFs.value"
      :is-open="editor.showTextEditor.value"
      @close="editor.closeEditor"
      @save="(content) => editor.saveFile(content, refreshCurrentDirectory)"
    />

    <!-- Status Bar -->
    <div class="h-[20px] bg-[var(--vf-bg-secondary)] border-t border-[var(--vf-border-default)] px-2 flex items-center text-[11px]">
      <span>{{ processedFiles.length }} items</span>
      <span v-if="selectedCount > 0" class="ml-4">{{ selectedCount }} selected</span>
      <span v-if="hasActiveFilters" class="ml-4 text-[var(--vf-accent-primary)]">🔍 Filters active</span>
      <span v-if="isDragging" class="ml-4 text-orange-600">📋 Dragging {{ draggedItems.length}} item(s)...</span>
      <button
        v-if="hasOperations"
        @click="appUI.showBatchQueue.value = !appUI.showBatchQueue.value"
        class="ml-4 text-[var(--vf-accent-primary)] hover:text-[var(--vf-accent-hover)] cursor-pointer"
      >
        📋 Batch Operations
      </button>
      <span class="ml-auto text-[var(--vf-text-secondary)]">
        RAM: {{ appUI.systemStats.value.memory_mb.toFixed(1) }} MB
        <span class="ml-3">CPU: {{ appUI.systemStats.value.cpu_percent.toFixed(1) }}%</span>
      </span>
    </div>

    <VaultOverlay />
    <WidgetLayer />
    <WidgetSelector
      :is-open="showWidgetSelector"
      @close="closeWidgetSelector"
    />
    <ShareDialog
      :is-open="appUI.showShareDialog.value"
      :share-info="appUI.shareInfo.value"
      @close="appUI.showShareDialog.value = false"
    />
    <StegoVaultModal
      :is-open="stegoModal.isOpen"
      :session-id="stegoModal.sessionId"
      :title="stegoModal.title"
      @close="stegoModal.isOpen = false"
    />
    <TorrentManager />
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
