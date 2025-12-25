<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
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
  loadUIState,
  saveTabsState,
  saveCompleteState,
  saveSidebarWidth,
  savePreviewWidth,
  saveWindowState,
} = useUIState();

// Handle sidebar resize
const handleSidebarResize = (width: number) => {
  saveSidebarWidth(width);
};

// Handle preview resize
const handlePreviewResize = (width: number) => {
  savePreviewWidth(width);
};

// Window state management
const appWindow = getCurrentWindow();
let windowStateTimer: ReturnType<typeof setTimeout> | null = null;

const saveCurrentWindowState = async () => {
  try {
    const position = await appWindow.outerPosition();
    const size = await appWindow.outerSize();
    const isMaximized = await appWindow.isMaximized();

    await saveWindowState({
      x: position.x,
      y: position.y,
      width: size.width,
      height: size.height,
      maximized: isMaximized,
    });
  } catch (error) {
    console.error('Failed to save window state:', error);
  }
};

const debouncedSaveWindowState = () => {
  if (windowStateTimer) {
    clearTimeout(windowStateTimer);
  }
  windowStateTimer = setTimeout(saveCurrentWindowState, 500);
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

// Local state
const viewMode = ref<ViewMode>('list');
const isCommandPaletteOpen = ref(false);
const contextMenu = ref<{ x: number; y: number; item: FileItem } | null>(null);
const previewFile = ref<FileItem | null>(null);
const showSettings = ref(false);

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
  event.preventDefault();
  contextMenu.value = { x: event.clientX, y: event.clientY, item };
};

const closeContextMenu = () => {
  contextMenu.value = null;
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

// Keyboard shortcuts
const shortcuts = createKeyboardShortcuts(
    {
      openCommandPalette: () => { isCommandPaletteOpen.value = true; },
      closeDialogs: () => {
        isCommandPaletteOpen.value = false;
        previewFile.value = null;
        clearSelection();
      },
      selectAll: (files: FileItem[]) => selectAll(files),
      addTab,
      closeTab: (canClose: boolean) => {
        if (canClose && tabs.value.length > 1) {
          closeTab(activeTabId.value);
        }
      },
      goUp: (canGoUpValue: boolean) => {
        if (canGoUpValue) goUp();
      },
      handleCopy: () => fileOps.handleCopy(getSelected()),
      handleCut: () => fileOps.handleCut(getSelected()),
      handlePaste: () => fileOps.handlePaste(currentPath.value),
      handleDelete: () => fileOps.handleDelete(getSelected(), currentPath.value, clearSelection, showConfirm),
      handleRename: () => fileOps.handleRename(getSelected(), currentPath.value, showInput),
      handleRefresh: () => fileOps.handleRefresh(currentPath.value),
      handleNewFolder: () => fileOps.handleNewFolder(currentPath.value, showInput),
      toggleBookmark: handleToggleBookmark,
      openSettings: () => { showSettings.value = true; },
      // Keyboard navigation
      moveFocusUp: () => moveFocusUp(processedFiles.value),
      moveFocusDown: () => moveFocusDown(processedFiles.value),
      moveFocusToFirst: () => moveFocusToFirst(processedFiles.value),
      moveFocusToLast: () => moveFocusToLast(processedFiles.value),
      selectFocused: () => selectFocused(),
      toggleFocusedSelection: () => toggleFocusedSelection(),
      openFocusedItem: () => {
        const item = getFocusedItem(processedFiles.value);
        if (item) {
          handleItemDoubleClick(item);
        }
      },
    },
    () => files.value
);

useKeyboard(shortcuts);

// Context menu handlers
const contextMenuHandlers = {
  open: () => {
    if (contextMenu.value?.item) {
      fileOps.handleOpenFile(contextMenu.value.item);
    }
  },
  copy: () => fileOps.handleCopy(getSelected()),
  cut: () => fileOps.handleCut(getSelected()),
  paste: () => fileOps.handlePaste(currentPath.value),
  rename: () => fileOps.handleRename(getSelected(), currentPath.value, showInput),
  delete: () => fileOps.handleDelete(getSelected(), currentPath.value, clearSelection, showConfirm),
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
    const selected = getSelected();
    if (selected.length === 1) {
      propertiesDialog.value = { isOpen: true, file: selected[0] };
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

// Save UI state when tabs change
watch([tabs, activeTabId], () => {
  saveTabsState(tabs.value, activeTabId.value);
}, { deep: true });

// Save last path when it changes
watch(currentPath, () => {
  saveCompleteState(tabs.value, activeTabId.value, currentPath.value);
});

// Click outside handler
onMounted(async () => {
  document.addEventListener('click', closeContextMenu);

  // Load bookmarks
  await loadBookmarks();

  // Load UI state
  const uiState = await loadUIState();

  // Restore tabs if available
  if (uiState && uiState.tabs && uiState.tabs.length > 0) {
    // Restore tabs from saved state
    tabs.value = uiState.tabs.map(tabState => ({
      id: tabState.id,
      path: tabState.path,
      name: tabState.name,
      history: [tabState.path],
      historyIndex: 0,
    }));

    // Restore active tab
    if (uiState.active_tab_id) {
      activeTabId.value = uiState.active_tab_id;
    }
  } else if (uiState?.last_path && uiState.last_path.length > 0) {
    // Restore last path if tabs not available
    navigateTo(uiState.last_path);
  }

  // Restore window state
  if (uiState?.window) {
    try {
      const { window: windowState } = uiState;

      // Restore window size and position
      if (windowState.width && windowState.height) {
        await appWindow.setSize({ width: windowState.width, height: windowState.height });
      }

      if (windowState.x !== undefined && windowState.y !== undefined) {
        await appWindow.setPosition({ x: windowState.x, y: windowState.y });
      }

      // Restore maximized state
      if (windowState.maximized) {
        await appWindow.maximize();
      }
    } catch (error) {
      console.error('Failed to restore window state:', error);
    }
  }

  // Listen for window resize events
  const resizeUnlisten = await appWindow.onResized(() => {
    debouncedSaveWindowState();
  });

  // Listen for window move events
  const moveUnlisten = await appWindow.onMoved(() => {
    debouncedSaveWindowState();
  });

  // Cleanup listeners on unmount
  onUnmounted(() => {
    resizeUnlisten();
    moveUnlisten();
  });
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
        :tabs="tabs"
        :active-tab-id="activeTabId"
        :current-path="currentPath"
        :view-mode="viewMode"
        :can-go-back="canGoBack"
        :can-go-forward="canGoForward"
        :can-go-up="canGoUp"
        :is-current-path-bookmarked="isCurrentPathBookmarked"
        @go-back="goBack"
        @go-forward="goForward"
        @go-up="goUp"
        @go-home="goHome"
        @navigate-to-breadcrumb="navigateToBreadcrumb"
        @navigate-to-path="handleNavigateToPath"
        @switch-tab="switchTab"
        @close-tab="closeTab"
        @add-tab="addTab"
        @update:view-mode="(mode) => viewMode = mode"
        @open-command-palette="() => isCommandPaletteOpen = true"
        @toggle-bookmark="handleToggleBookmark"
    />

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
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

    <!-- Status Bar -->
    <div class="h-[20px] bg-[#F1EFE2] border-t border-[#919B9C] px-2 flex items-center text-[11px]">
      <span>{{ processedFiles.length }} items</span>
      <span v-if="selectedCount > 0" class="ml-4">{{ selectedCount }} selected</span>
      <span v-if="hasActiveFilters" class="ml-4 text-blue-600">🔍 Filters active</span>
      <span v-if="isDragging" class="ml-4 text-orange-600">📋 Dragging {{ draggedItems.length }} item(s)...</span>
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