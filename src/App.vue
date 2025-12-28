<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import Toolbar from './components/Toolbar.vue';
import Sidebar from './components/Sidebar.vue';
import FileList from './components/FileList.vue';
import Preview from './components/Preview.vue';
import CommandPalette from './components/CommandPalette.vue';
import ContextMenu from './components/ContextMenu.vue';
import Notifications from './components/Notifications.vue';
import ConfirmDialog from './components/ConfirmDialog.vue';
import PropertiesDialog from './components/PropertiesDialog.vue';
import InputDialog from './components/InputDialog.vue';
import Settings from './components/Settings.vue';
import OperationsProgress from './components/OperationsProgress.vue';
import DualPanelContainer from './components/DualPanelContainer.vue';

import { useFileSystem } from './composables/useFileSystem';
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
import { createKeyboardShortcuts } from './utils/shortcuts';

import type { FileItem, ViewMode } from './types';

// File System
const { files, isLoading, loadDirectory, normalizePath, getHomeDirectory } = useFileSystem();

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
  isFocused,
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
  endDrag,
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
  rightPanelTabs,
  rightPanelActiveTabId,
  togglePanelMode,
  switchActivePanel,
  loadDualPanelState,
  serializeDualPanelState,
} = useDualPanel();

// Handle sidebar resize - напрямую обновляем ref, watch в App.vue сам сохранит
const handleSidebarResize = (width: number) => {
  sidebarWidth.value = width;
};

// Handle preview resize - напрямую обновляем ref, watch в App.vue сам сохранит
const handlePreviewResize = (width: number) => {
  previewWidth.value = width;
};

const handleBackgroundDrop = async (event: DragEvent) => {
  console.log('[App] Background Drop Detected!'); // DEBUG LOG

  const { copyItems, moveItems } = useFileSystem();

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

  await handleDrop(targetItem, event, moveItems, copyItems);
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

// File Operations
const fileOps = useFileOperations(refreshCurrentDirectory);

// Context Menu (global)
const { contextMenu, showContextMenu, closeContextMenu } = useContextMenu();

// Local state
const viewMode = ref<ViewMode>('list');
const isCommandPaletteOpen = ref(false);
const previewFile = ref<FileItem | null>(null);
const showSettings = ref(false);

// System stats
const systemStats = ref({ memory_mb: 0, cpu_percent: 0 });

// Check if current path is bookmarked
const isCurrentPathBookmarked = computed(() => {
  const path = '/' + currentPath.value.join('/');
  return isBookmarked(path);
});

// Computed
const processedFiles = computed(() => processFiles(files.value));

// Helper to get selected items
const getSelected = () => getSelectedItems(files.value);

// Handlers
const handleItemDoubleClick = (item: FileItem) => {
  if (item.type === 'folder' || item.type === 'drive' || item.type === 'system') {
    const pathParts = item.path.split('/').filter(p => p);
    navigateTo(pathParts);
  } else {
    previewFile.value = item;
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

  const { copyItems, moveItems } = useFileSystem();

  // Используем handleDrop из composable
  await handleDrop(item, event, moveItems, copyItems);

  // Обновляем директорию после drop
  await refreshCurrentDirectory();
};

// ИСПРАВЛЕНИЕ: Обновленная функция handleSidebarDrop
const handleSidebarDrop = async (targetPath: string, event: DragEvent) => {
  event.preventDefault();
  console.log('[App] Drop on sidebar path:', targetPath);

  const { copyItems, moveItems } = useFileSystem();

  // Создаем временный FileItem для целевого пути
  const targetItem: FileItem = {
    id: targetPath,
    name: targetPath.split('/').pop() || '',
    path: targetPath,
    type: 'folder',
    size: 0,
    modified: '',
  };

  // Используем handleDrop из composable
  await handleDrop(targetItem, event, moveItems, copyItems);

  // Обновляем директорию после drop
  await refreshCurrentDirectory();
};

const openCommandPalette = () => {
  isCommandPaletteOpen.value = true;
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
              const { error } = useFileOperations();
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
  onCloseTab: () => commands.closeTabCommand(tabs.value.length, closeTab, activeTabId.value),
  onSettings: () => { showSettings.value = true; },
});

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
        isCommandPaletteOpen.value = false;
        previewFile.value = null;
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
      toggleBookmark: handleToggleBookmark,
      openSettings: () => { showSettings.value = true; },
      // Dual panel switch (Tab)
      switchPanels: isDualMode.value ? () => {
        switchActivePanel(activePanel.value === 'left' ? 'right' : 'left');
      } : undefined,
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

// Context menu handlers (work in both single and dual modes)
const contextMenuHandlers = {
  open: () => {
    if (contextMenu.value?.item) {
      fileOps.handleOpenFile(contextMenu.value.item);
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
  sidebarSectionsExpanded
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
      }
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

  // Load bookmarks
  await loadBookmarks();

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

  // Восстановить табы если есть (только если НЕ dual mode)
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

      // Восстановить активный таб
      if (uiState.active_tab_id) {
        console.log('[App] ✅ Restoring active tab:', uiState.active_tab_id);
        activeTabId.value = uiState.active_tab_id;
      }
    } else if (uiState && uiState.last_path && uiState.last_path.length > 0) {
      console.log('[App] ✅ Restoring last path:', uiState.last_path);
      navigateTo(uiState.last_path);
    } else {
      console.log('[App] ℹ️ No tabs or path to restore');
    }
  }
});
</script>

<template>
  <div class="h-screen flex flex-col bg-[#ECE9D8] font-['Tahoma'] select-none overflow-hidden text-[#0b0b0b]">
    <!-- Menu Bar -->
    <div class="flex items-center h-[21px] bg-[#F1EFE2] border-b border-[#919B9C] text-[11px]">
      <div class="px-2 py-0.5 hover:bg-[#C1D2EE] hover:border hover:border-[#0A246A] cursor-pointer">File</div>
      <div class="px-2 py-0.5 hover:bg-[#C1D2EE] hover:border hover:border-[#0A246A] cursor-pointer">Edit</div>
      <div class="px-2 py-0.5 hover:bg-[#C1D2EE] hover:border hover:border-[#0A246A] cursor-pointer">View</div>
      <div class="px-2 py-0.5 hover:bg-[#C1D2EE] hover:border hover:border-[#0A246A] cursor-pointer">Favorites</div>
      <div class="px-2 py-0.5 hover:bg-[#C1D2EE] hover:border hover:border-[#0A246A] cursor-pointer">Tools</div>
      <div class="px-2 py-0.5 hover:bg-[#C1D2EE] hover:border hover:border-[#0A246A] cursor-pointer">Help</div>
      <div class="ml-auto px-2 text-[#666]">Arrows: navigate • Space: select • Enter: open • Ctrl+K: search</div>
    </div>

    <!-- Toolbar -->
    <Toolbar
        :tabs="isDualMode ? activePanelTabs : tabs"
        :active-tab-id="isDualMode ? activePanelTabId : activeTabId"
        :current-path="isDualMode ? activePanelPath : currentPath"
        :view-mode="viewMode"
        :panel-mode="panelMode"
        :can-go-back="canGoBackComputed"
        :can-go-forward="canGoForwardComputed"
        :can-go-up="canGoUp"
        :is-current-path-bookmarked="isCurrentPathBookmarked"
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
        @toggle-panel-mode="togglePanelMode"
    />

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Dual Panel Mode -->
      <DualPanelContainer v-if="isDualMode" :view-mode="viewMode" />

      <!-- Single Panel Mode -->
      <template v-else>
        <!-- Sidebar -->
        <Sidebar
            :current-path="'/' + currentPath.join('/')"
            :width="sidebarWidth"
            @navigate="(path) => navigateTo(path.split('/').filter(p => p))"
            @drop="handleSidebarDrop"
            @resize="handleSidebarResize"
        />

        <!-- Main Area -->
        <div class="flex-1 flex overflow-hidden">
          <!-- File List -->
          <FileList
              :items="processedFiles"
              :view-mode="viewMode"
              :selected-ids="selectedIds"
              :focused-id="focusedId"
              :is-loading="isLoading"
              :is-dragging="isDragging"
              :drag-target-id="dragOverId"
              @item-click="(item, event) => handleItemClick(item, files, event)"
              @item-double-click="handleItemDoubleClick"
              @item-context-menu="handleContextMenu"
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
        </div>
      </template>
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
        @open="contextMenuHandlers.open"
        @copy="contextMenuHandlers.copy"
        @cut="contextMenuHandlers.cut"
        @paste="contextMenuHandlers.paste"
        @rename="contextMenuHandlers.rename"
        @delete="contextMenuHandlers.delete"
        @add-to-favorites="contextMenuHandlers.addToFavorites"
        @open-terminal="contextMenuHandlers.openTerminal"
        @properties="contextMenuHandlers.properties"
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

    <!-- Settings -->
    <Settings
        v-if="showSettings"
        @close="showSettings = false"
    />

    <!-- File Operations Progress -->
    <OperationsProgress />

    <!-- Status Bar -->
    <div class="h-[20px] bg-[#F1EFE2] border-t border-[#919B9C] px-2 flex items-center text-[11px]">
      <span>{{ processedFiles.length }} items</span>
      <span v-if="selectedCount > 0" class="ml-4">{{ selectedCount }} selected</span>
      <span v-if="hasActiveFilters" class="ml-4 text-blue-600">🔍 Filters active</span>
      <span v-if="isDragging" class="ml-4 text-orange-600">📋 Dragging {{ draggedItems.length }} item(s)...</span>
      <span class="ml-auto text-[#555]">
        RAM: {{ systemStats.memory_mb.toFixed(1) }} MB
        <span class="ml-3">CPU: {{ systemStats.cpu_percent.toFixed(1) }}%</span>
      </span>
    </div>
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