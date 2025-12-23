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
  dragOverId,
  startDrag,
  handleDragOver,
  handleDragLeave,
  handleDrop,
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

const handleDragStart = (item: FileItem, event: DragEvent) => {
  const items = hasSelection.value && isSelected(item.id)
    ? getSelectedItems(files.value)
    : [item];
  startDrag(items, event);
};

const handleItemDrop = async (item: FileItem, event: DragEvent) => {
  const { copyItems, moveItems } = useFileSystem();
  await handleDrop(item, event, moveItems, copyItems);
};

const handleSidebarDrop = async (targetPath: string, event: DragEvent) => {
  const { copyItems, moveItems } = useFileSystem();

  // Create a temporary FileItem for the target path
  const targetItem: FileItem = {
    id: targetPath,
    name: targetPath.split('/').pop() || '',
    path: targetPath,
    type: 'folder',
    size: 0,
    modified: '',
  };

  await handleDrop(targetItem, event, moveItems, copyItems);
};

const openCommandPalette = () => {
  isCommandPaletteOpen.value = true;
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
  onSettings: () => {},
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

// Click outside handler
onMounted(() => {
  document.addEventListener('click', closeContextMenu);
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
    />

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Sidebar -->
      <Sidebar
        :current-path="'/' + currentPath.join('/')"
        @navigate="(path) => navigateTo(path.split('/').filter(p => p))"
        @drop="handleSidebarDrop"
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
          @close="previewFile = null"
          @open="fileOps.handleOpenFile"
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

    <!-- Status Bar -->
    <div class="h-[20px] bg-[#F1EFE2] border-t border-[#919B9C] px-2 flex items-center text-[11px]">
      <span>{{ processedFiles.length }} items</span>
      <span v-if="selectedCount > 0" class="ml-4">{{ selectedCount }} selected</span>
      <span v-if="hasActiveFilters" class="ml-4 text-blue-600">🔍 Filters active</span>
      <span v-if="isDragging" class="ml-4 text-orange-600">📋 Dragging items...</span>
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
