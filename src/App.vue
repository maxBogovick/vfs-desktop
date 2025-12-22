<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import Toolbar from './components/Toolbar.vue';
import Sidebar from './components/Sidebar.vue';
import FileList from './components/FileList.vue';
import Preview from './components/Preview.vue';
import CommandPalette from './components/CommandPalette.vue';
import ContextMenu from './components/ContextMenu.vue';
import { useFileSystem } from './composables/useFileSystem';
import { useNavigation } from './composables/useNavigation';
import { useSelection } from './composables/useSelection';
import { useSearch } from './composables/useSearch';
import { useDragDrop } from './composables/useDragDrop';
import { useKeyboard } from './composables/useKeyboard';
import { useClipboard } from './composables/useClipboard';
import { useNotifications } from './composables/useNotifications';
import Notifications from './components/Notifications.vue';
import ConfirmDialog from './components/ConfirmDialog.vue';
import PropertiesDialog from './components/PropertiesDialog.vue';
import type { FileItem, ViewMode } from './types';

// Composables
const {
  files,
  isLoading,
  loadDirectory,
  moveItems,
  copyItems,
  deleteItem,
  renameItem,
  createFolder,
  getHomeDirectory,
  openFile: openFileInApp,
  revealInFinder,
} = useFileSystem();
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
  navigateInto,
  navigateTo,
  navigateToBreadcrumb,
  addTab,
  closeTab,
  switchTab,
} = useNavigation();
const {
  selectedIds,
  selectedCount,
  hasSelection,
  isSelected,
  handleItemClick,
  getSelectedItems,
  clearSelection,
  selectAll,
} = useSelection();
const {
  searchQuery,
  processFiles,
  setSearchQuery,
  hasActiveFilters,
} = useSearch();
const {
  isDragging,
  dragOverId,
  startDrag,
  handleDragOver,
  handleDragLeave,
  handleDrop,
} = useDragDrop();
const {
  hasClipboardItems,
  copy: copyToClipboard,
  cut: cutToClipboard,
  paste: pasteFromClipboard,
  clear: clearClipboard,
} = useClipboard();
const { success, error: showError, warning, info } = useNotifications();

// Local state
const viewMode = ref<ViewMode>('list');
const isCommandPaletteOpen = ref(false);
const contextMenu = ref<{ x: number; y: number; item: FileItem } | null>(null);
const previewFile = ref<FileItem | null>(null);
const confirmDialog = ref<{
  isOpen: boolean;
  title: string;
  message: string;
  type: 'warning' | 'danger' | 'info';
  onConfirm: () => void;
}>({
  isOpen: false,
  title: '',
  message: '',
  type: 'warning',
  onConfirm: () => {},
});
const propertiesDialog = ref<{ isOpen: boolean; file: FileItem | null }>({
  isOpen: false,
  file: null,
});

// Computed
const processedFiles = computed(() => processFiles(files.value));

// Handlers
const handleItemDoubleClick = (item: FileItem) => {
  if (item.type === 'folder' || item.type === 'drive' || item.type === 'system') {
    // Navigate to the full path of the folder
    // Split path and filter out empty strings, preserving path structure
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
  await handleDrop(item, event, moveItems, copyItems);
};

const openCommandPalette = () => {
  isCommandPaletteOpen.value = true;
};

const executeCommand = (cmd: { id: string }) => {
  const commandHandlers: Record<string, () => void> = {
    'new-folder': handleNewFolder,
    'new-file': () => {
      warning('Not implemented', 'File creation is not yet implemented');
    },
    'search': () => {
      info('Search', 'Use the search bar in the toolbar');
    },
    'goto': async () => {
      const path = prompt('Enter path:');
      if (path) {
        try {
          navigateTo(path.split('/'));
        } catch (err) {
          showError('Navigation failed', err instanceof Error ? err.message : 'Invalid path');
        }
      }
    },
    'refresh': async () => {
      const currentDir = currentPath.value.join('/') || await getHomeDirectory();
      await loadDirectory(currentDir);
      success('Refreshed', 'Directory refreshed');
    },
    'copy-path': () => {
      const selected = getSelectedItems(files.value);
      if (selected.length > 0) {
        const paths = selected.map(item => item.path).join('\n');
        navigator.clipboard.writeText(paths);
        success('Copied path', `${selected.length} path(s) copied to clipboard`);
      } else {
        warning('No selection', 'Please select files to copy their paths');
      }
    },
    'select-all': () => {
      selectAll(files.value);
      success('Selected all', `${files.value.length} items selected`);
    },
    'new-tab': () => {
      addTab();
      success('New tab', 'Created new tab');
    },
    'close-tab': () => {
      if (tabs.value.length > 1) {
        closeTab(activeTabId.value);
        success('Tab closed', 'Tab closed successfully');
      } else {
        warning('Cannot close', 'Cannot close the last tab');
      }
    },
    'settings': () => {
      info('Settings', 'Settings panel coming soon');
    },
  };

  const handler = commandHandlers[cmd.id];
  if (handler) {
    handler();
  }
};

const openFile = async (file: FileItem) => {
  try {
    await openFileInApp(file.path);
    success('File opened', `${file.name} opened successfully`);
  } catch (err) {
    showError('Failed to open file', err instanceof Error ? err.message : 'Unknown error');
  }
};

// File operations
const handleCopy = () => {
  const selected = getSelectedItems(files.value);
  if (selected.length > 0) {
    copyToClipboard(selected);
    success('Copied', `${selected.length} item(s) copied to clipboard`);
  }
};

const handleCut = () => {
  const selected = getSelectedItems(files.value);
  if (selected.length > 0) {
    cutToClipboard(selected);
    warning('Cut', `${selected.length} item(s) cut to clipboard`);
  }
};

const handlePaste = async () => {
  if (!hasClipboardItems.value) {
    warning('Nothing to paste', 'Clipboard is empty');
    return;
  }

  try {
    const currentDir = currentPath.value.join('/') || await getHomeDirectory();
    await pasteFromClipboard(currentDir, copyItems, moveItems);
    // Refresh directory after paste
    await loadDirectory(currentDir);
    success('Pasted successfully', 'Items pasted');
  } catch (err) {
    showError('Paste failed', err instanceof Error ? err.message : 'Unknown error');
  }
};

const handleDelete = () => {
  const selected = getSelectedItems(files.value);
  if (selected.length === 0) return;

  confirmDialog.value = {
    isOpen: true,
    title: 'Confirm Delete',
    message: `Are you sure you want to permanently delete ${selected.length} item(s)?`,
    type: 'danger',
    onConfirm: async () => {
      try {
        for (const item of selected) {
          await deleteItem(item.path);
        }
        // Refresh directory
        const currentDir = currentPath.value.join('/') || await getHomeDirectory();
        await loadDirectory(currentDir);
        clearSelection();
        success('Deleted', `${selected.length} item(s) deleted`);
      } catch (err) {
        showError('Delete failed', err instanceof Error ? err.message : 'Unknown error');
      }
    },
  };
};

const handleRename = async () => {
  const selected = getSelectedItems(files.value);
  if (selected.length !== 1) {
    warning('Invalid selection', 'Please select exactly one item to rename');
    return;
  }

  const item = selected[0];
  const newName = prompt('Enter new name:', item.name);
  if (!newName || newName === item.name) return;

  try {
    await renameItem(item.path, newName);
    // Refresh directory
    const currentDir = currentPath.value.join('/') || await getHomeDirectory();
    await loadDirectory(currentDir);
    success('Renamed', `Renamed to ${newName}`);
  } catch (err) {
    showError('Rename failed', err instanceof Error ? err.message : 'Unknown error');
  }
};

const handleNewFolder = async () => {
  const name = prompt('Enter folder name:');
  if (!name) return;

  try {
    const currentDir = currentPath.value.join('/') || await getHomeDirectory();
    await createFolder(currentDir, name);
    // Refresh directory
    await loadDirectory(currentDir);
    success('Folder created', `Created ${name}`);
  } catch (err) {
    showError('Create folder failed', err instanceof Error ? err.message : 'Unknown error');
  }
};

const handleProperties = () => {
  const selected = getSelectedItems(files.value);
  if (selected.length !== 1) {
    warning('Invalid selection', 'Please select exactly one item to view properties');
    return;
  }

  propertiesDialog.value = {
    isOpen: true,
    file: selected[0],
  };
};

const handleRevealInFinder = async () => {
  const selected = getSelectedItems(files.value);
  if (selected.length !== 1) return;

  try {
    await revealInFinder(selected[0].path);
  } catch (err) {
    showError('Failed to reveal', err instanceof Error ? err.message : 'Unknown error');
  }
};

// Keyboard shortcuts
useKeyboard([
  {
    key: 'k',
    ctrl: true,
    description: 'Open command palette',
    callback: openCommandPalette,
  },
  {
    key: 'Escape',
    description: 'Close dialogs',
    callback: () => {
      isCommandPaletteOpen.value = false;
      previewFile.value = null;
      clearSelection();
    },
  },
  {
    key: 'a',
    ctrl: true,
    description: 'Select all',
    callback: () => selectAll(files.value),
  },
  {
    key: 't',
    ctrl: true,
    description: 'New tab',
    callback: addTab,
  },
  {
    key: 'w',
    ctrl: true,
    description: 'Close tab',
    callback: () => {
      if (tabs.value.length > 1) {
        closeTab(activeTabId.value);
      }
    },
  },
  {
    key: 'Backspace',
    description: 'Go up',
    callback: () => { if (canGoUp.value) goUp(); },
  },
  {
    key: 'c',
    ctrl: true,
    description: 'Copy',
    callback: handleCopy,
  },
  {
    key: 'x',
    ctrl: true,
    description: 'Cut',
    callback: handleCut,
  },
  {
    key: 'v',
    ctrl: true,
    description: 'Paste',
    callback: handlePaste,
  },
  {
    key: 'Delete',
    description: 'Delete',
    callback: handleDelete,
  },
  {
    key: 'F2',
    description: 'Rename',
    callback: handleRename,
  },
  {
    key: 'F5',
    description: 'Refresh',
    callback: async () => {
      const currentDir = currentPath.value.join('/') || await getHomeDirectory();
      await loadDirectory(currentDir);
    },
  },
  {
    key: 'n',
    ctrl: true,
    shift: true,
    description: 'New folder',
    callback: handleNewFolder,
  },
]);

// Watch current path and load directory
watch(currentPath, async (newPath) => {
  // Reconstruct full path with leading slash
  let pathString = newPath.join('/');

  // Add leading slash if path is not empty and doesn't start with slash
  if (pathString && !pathString.startsWith('/')) {
    pathString = '/' + pathString;
  }

  await loadDirectory(pathString || await getHomeDirectory());
  clearSelection();
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
      <div class="ml-auto px-2 text-[#666]">Ctrl+K for quick search • Ctrl+A select all • Backspace go up</div>
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
      @navigate-to-path="(path) => navigateTo(path.split('/').filter(p => p))"
      @switch-tab="switchTab"
      @close-tab="closeTab"
      @add-tab="addTab"
      @update:view-mode="(mode) => viewMode = mode"
      @open-command-palette="openCommandPalette"
    />

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Sidebar -->
      <Sidebar @navigate="(path) => navigateTo(path.split('/'))" />

      <!-- Main Area -->
      <div class="flex-1 flex overflow-hidden">
        <!-- File List -->
        <FileList
          :items="processedFiles"
          :view-mode="viewMode"
          :selected-ids="selectedIds"
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
        />

        <!-- Preview Panel -->
        <Preview
          :file="previewFile"
          @close="previewFile = null"
          @open="openFile"
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
      @open="() => contextMenu?.item && openFile(contextMenu.item)"
      @copy="handleCopy"
      @cut="handleCut"
      @paste="handlePaste"
      @rename="handleRename"
      @delete="handleDelete"
      @properties="handleProperties"
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
      @confirm="() => { confirmDialog.onConfirm(); confirmDialog.isOpen = false; }"
      @cancel="confirmDialog.isOpen = false"
    />

    <!-- Properties Dialog -->
    <PropertiesDialog
      :is-open="propertiesDialog.isOpen"
      :file="propertiesDialog.file"
      @close="propertiesDialog.isOpen = false"
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
