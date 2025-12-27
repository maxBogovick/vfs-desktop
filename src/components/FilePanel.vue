<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import PanelToolbar from './PanelToolbar.vue';
import FileList from './FileList.vue';
import { useFileSystem } from '../composables/useFileSystem';
import { useSelection } from '../composables/useSelection';
import { useDragDrop } from '../composables/useDragDrop';
import { useFileOperations } from '../composables/useFileOperations';
import { useDialogs } from '../composables/useDialogs';
import { useContextMenu } from '../composables/useContextMenu';
import { registerActivePanelMethods, type ActivePanelMethods } from '../composables/useDualPanel';
import type { Tab, ViewMode, ActivePanel, FileItem } from '../types';

interface Props {
  panelId: ActivePanel;
  isActive: boolean;
  tabs: Tab[];
  activeTabId?: number;
  viewMode: ViewMode;
}

interface Emits {
  (e: 'activate'): void;
  (e: 'update:tabs', tabs: Tab[]): void;
  (e: 'update:activeTabId', tabId: number | undefined): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// File System
const {
  files,
  isLoading,
  loadDirectory,
  copyItems,
  moveItems
} = useFileSystem();

// Selection
const {
  selectedIds,
  focusedId,
  handleItemClick,
  clearSelection,
  getSelectedItems,
} = useSelection();

// Drag & Drop
const {
  isDragging,
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
  showConfirm,
  showInput,
} = useDialogs();

// Context Menu
const { showContextMenu } = useContextMenu();

// Computed: Active Tab
const activeTab = ref<Tab | undefined>(
    props.tabs.find(t => t.id === props.activeTabId)
);

// Computed: Current Path
const currentPath = ref<string[]>(activeTab.value?.path || []);

// Helper to get selected items
const getSelected = () => getSelectedItems(files.value);

// Helper to get current directory path
const getCurrentDirectoryPath = async (): Promise<string> => {
  const { getHomeDirectory } = useFileSystem();
  let pathString = currentPath.value.join('/');
  if (pathString && !pathString.startsWith('/')) {
    pathString = '/' + pathString;
  }
  if (!pathString) {
    return await getHomeDirectory();
  }
  return pathString;
};

// Function to refresh current directory
const refreshCurrentDirectory = async () => {
  const pathString = await getCurrentDirectoryPath();
  await loadDirectory(pathString);
};

// File Operations
const fileOps = useFileOperations(refreshCurrentDirectory);

// Watch active tab and update current path
watch(() => props.activeTabId, () => {
  activeTab.value = props.tabs.find(t => t.id === props.activeTabId);
  if (activeTab.value) {
    currentPath.value = activeTab.value.path;
  }
}, { immediate: true });

// Watch tabs array for changes to active tab
watch(() => props.tabs, (newTabs) => {
  activeTab.value = newTabs.find(t => t.id === props.activeTabId);
  if (activeTab.value) {
    currentPath.value = activeTab.value.path;
  }
}, { deep: true });

// Watch current path and load directory
watch(currentPath, async () => {
  const pathString = await getCurrentDirectoryPath();
  await loadDirectory(pathString);
  clearSelection();
  // Set focus on first element after loading
  if (files.value.length > 0) {
    focusedId.value = files.value[0].id;
  } else {
    focusedId.value = null;
  }
}, { immediate: true });

// Navigation: Update tab path
const updateTabPath = (newPath: string[]) => {
  if (!activeTab.value) return;

  const tabIndex = props.tabs.findIndex(t => t.id === props.activeTabId);
  if (tabIndex === -1) return;

  const updatedTabs = [...props.tabs];
  const tab = { ...updatedTabs[tabIndex] };

  // Update history
  const newHistory = tab.history.slice(0, tab.historyIndex + 1);
  newHistory.push(newPath);

  const MAX_HISTORY_SIZE = 50;
  if (newHistory.length > MAX_HISTORY_SIZE) {
    newHistory.splice(0, newHistory.length - MAX_HISTORY_SIZE);
  }

  updatedTabs[tabIndex] = {
    ...tab,
    path: newPath,
    name: newPath[newPath.length - 1] || 'Home',
    history: newHistory,
    historyIndex: newHistory.length - 1,
  };

  emit('update:tabs', updatedTabs);
};

// Handle item double click (navigate into folder)
const handleItemDoubleClick = (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  if (item.type === 'folder' || item.type === 'drive' || item.type === 'system') {
    const pathParts = item.path.split('/').filter(p => p);
    updateTabPath(pathParts);
  }
};

// Handle item context menu
const handleItemContextMenu = (item: FileItem, event: MouseEvent) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  // If item is not selected, select it first
  if (!selectedIds.value.has(item.id)) {
    handleItemClick(item, files.value, { ctrlKey: false } as MouseEvent);
  }
  // Show context menu
  showContextMenu(item, event);
};

// Handle panel activation
const handlePanelClick = () => {
  if (!props.isActive) {
    emit('activate');
  }
};

// Handle tab switching
const handleSwitchTab = (tabId: number) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  emit('update:activeTabId', tabId);
};

// Handle tab closing
const handleCloseTab = (tabId: number) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }

  if (props.tabs.length <= 1) return;

  const tabIndex = props.tabs.findIndex(t => t.id === tabId);
  const updatedTabs = props.tabs.filter(t => t.id !== tabId);

  emit('update:tabs', updatedTabs);

  if (props.activeTabId === tabId) {
    const newActiveTab = updatedTabs[Math.max(0, tabIndex - 1)];
    if (newActiveTab) {
      emit('update:activeTabId', newActiveTab.id);
    }
  }
};

// Handle adding tab
const handleAddTab = () => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }

  const newTab: Tab = {
    id: Date.now(),
    path: [],
    name: 'Home',
    history: [[]],
    historyIndex: 0,
  };

  const updatedTabs = [...props.tabs, newTab];
  emit('update:tabs', updatedTabs);
  emit('update:activeTabId', newTab.id);
};

// Handle drag start
const handleDragStart = (item: FileItem, event: DragEvent) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }

  const itemsToDrag = selectedIds.value.has(item.id)
      ? files.value.filter(i => selectedIds.value.has(i.id))
      : [item];

  console.log('[FilePanel] Starting drag with items:', itemsToDrag.length, itemsToDrag.map(i => i.name));
  startDrag(itemsToDrag, event);
};

// Handle item drop
const handleItemDrop = async (item: FileItem, event: DragEvent) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }

  event.preventDefault();
  console.log('[FilePanel] Drop on item:', item.name);

  await handleDrop(item, event, moveItems, copyItems);
  await refreshCurrentDirectory();
};

// Handle background drop
const handleBackgroundDrop = async (event: DragEvent) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }

  console.log('[FilePanel] Background Drop Detected!');

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

  await handleDrop(targetItem, event, moveItems, copyItems);
  await refreshCurrentDirectory();
};

// Wrapper for item click to activate panel
const handleItemClickWithActivation = (item: FileItem, event: MouseEvent) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  handleItemClick(item, files.value, event);
};

// Quick action handlers
const handleToggleSelection = (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  handleItemClick(item, files.value, { ctrlKey: true } as MouseEvent);
};

const handleCopyItem = (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  fileOps.handleCopy([item]);
};

const handleCutItem = (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  fileOps.handleCut([item]);
};

const handleDeleteItem = (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  fileOps.handleDelete([item], currentPath.value, clearSelection, showConfirm);
};

const handleRenameItem = (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  fileOps.handleRename([item], currentPath.value, showInput);
};

const handleOpenTerminal = async (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }

  const { openTerminal } = useFileSystem();
  const { success, error } = await import('../composables/useNotifications').then(m => m.useNotifications());

  try {
    await openTerminal(item.path);
    success('Terminal opened', `Opened terminal in ${item.name}`);
  } catch (err) {
    error('Failed to open terminal', err instanceof Error ? err.message : 'Unknown error');
  }
};

// Register panel methods when it becomes active (for keyboard shortcuts in dual mode)
watch(() => props.isActive, (isActive) => {
  if (isActive) {
    const methods: ActivePanelMethods = {
      getFiles: () => files.value,
      getSelectedItems: () => getSelected(),
      selectAll: () => {
        files.value.forEach(file => selectedIds.value.add(file.id));
      },
      clearSelection,
      handleCopy: () => fileOps.handleCopy(getSelected()),
      handleCut: () => fileOps.handleCut(getSelected()),
      handlePaste: () => fileOps.handlePaste(currentPath.value),
      handleDelete: () => fileOps.handleDelete(getSelected(), currentPath.value, clearSelection, showConfirm),
      handleRename: () => fileOps.handleRename(getSelected(), currentPath.value, showInput),
      handleRefresh: () => fileOps.handleRefresh(currentPath.value),
      handleNewFolder: () => fileOps.handleNewFolder(currentPath.value, showInput),
      addTab: handleAddTab,
      closeTab: () => {
        if (props.tabs.length > 1 && props.activeTabId) {
          handleCloseTab(props.activeTabId);
        }
      },
      goUp: () => {
        if (currentPath.value.length > 0) {
          const newPath = currentPath.value.slice(0, -1);
          updateTabPath(newPath);
        }
      },
      goBack: () => {
        if (!activeTab.value) return;
        const tab = activeTab.value;
        if (tab.historyIndex > 0) {
          const tabIndex = props.tabs.findIndex(t => t.id === props.activeTabId);
          if (tabIndex === -1) return;

          const updatedTabs = [...props.tabs];
          updatedTabs[tabIndex] = {
            ...tab,
            historyIndex: tab.historyIndex - 1,
            path: tab.history[tab.historyIndex - 1],
            name: tab.history[tab.historyIndex - 1][tab.history[tab.historyIndex - 1].length - 1] || 'Home',
          };
          emit('update:tabs', updatedTabs);
        }
      },
      goForward: () => {
        if (!activeTab.value) return;
        const tab = activeTab.value;
        if (tab.historyIndex < tab.history.length - 1) {
          const tabIndex = props.tabs.findIndex(t => t.id === props.activeTabId);
          if (tabIndex === -1) return;

          const updatedTabs = [...props.tabs];
          updatedTabs[tabIndex] = {
            ...tab,
            historyIndex: tab.historyIndex + 1,
            path: tab.history[tab.historyIndex + 1],
            name: tab.history[tab.historyIndex + 1][tab.history[tab.historyIndex + 1].length - 1] || 'Home',
          };
          emit('update:tabs', updatedTabs);
        }
      },
      canGoBack: () => {
        return activeTab.value ? activeTab.value.historyIndex > 0 : false;
      },
      canGoForward: () => {
        return activeTab.value ? activeTab.value.historyIndex < activeTab.value.history.length - 1 : false;
      },
      moveFocusUp: () => {
        const currentIndex = files.value.findIndex(f => f.id === focusedId.value);
        if (currentIndex > 0) {
          focusedId.value = files.value[currentIndex - 1].id;
        }
      },
      moveFocusDown: () => {
        const currentIndex = files.value.findIndex(f => f.id === focusedId.value);
        if (currentIndex >= 0 && currentIndex < files.value.length - 1) {
          focusedId.value = files.value[currentIndex + 1].id;
        }
      },
      moveFocusToFirst: () => {
        if (files.value.length > 0) {
          focusedId.value = files.value[0].id;
        }
      },
      moveFocusToLast: () => {
        if (files.value.length > 0) {
          focusedId.value = files.value[files.value.length - 1].id;
        }
      },
      selectFocused: () => {
        if (focusedId.value) {
          const item = files.value.find(f => f.id === focusedId.value);
          if (item) {
            selectedIds.value.clear();
            selectedIds.value.add(item.id);
          }
        }
      },
      toggleFocusedSelection: () => {
        if (focusedId.value) {
          if (selectedIds.value.has(focusedId.value)) {
            selectedIds.value.delete(focusedId.value);
          } else {
            selectedIds.value.add(focusedId.value);
          }
        }
      },
      openFocusedItem: () => {
        if (focusedId.value) {
          const item = files.value.find(f => f.id === focusedId.value);
          if (item) {
            handleItemDoubleClick(item);
          }
        }
      },
    };
    registerActivePanelMethods(methods);
  }
}, { immediate: true });
</script>

<template>
  <div
      @click="handlePanelClick"
      :class="[
      'flex flex-col overflow-hidden bg-white',
      isActive ? 'ring-2 ring-blue-500' : 'ring-1 ring-gray-300'
    ]"
  >
    <!-- Panel Toolbar -->
    <PanelToolbar
        :tabs="tabs"
        :active-tab-id="activeTabId"
        @switch-tab="handleSwitchTab"
        @close-tab="handleCloseTab"
        @add-tab="handleAddTab"
    />

    <!-- File List -->
    <FileList
        :items="files"
        :view-mode="viewMode"
        :selected-ids="selectedIds"
        :focused-id="focusedId"
        :is-loading="isLoading"
        :is-dragging="isDragging"
        :drag-target-id="dragOverId"
        @item-click="handleItemClickWithActivation"
        @item-double-click="handleItemDoubleClick"
        @item-context-menu="handleItemContextMenu"
        @drag-start="handleDragStart"
        @drag-over="handleDragOver"
        @drag-leave="handleDragLeave"
        @drag-end="endDrag"
        @drop="handleItemDrop"
        @drop-on-background="handleBackgroundDrop"
        @drag-over-background="handleDragOverBackground"
        @toggle-selection="handleToggleSelection"
        @copy-item="handleCopyItem"
        @cut-item="handleCutItem"
        @delete-item="handleDeleteItem"
        @rename-item="handleRenameItem"
        @open-terminal="handleOpenTerminal"
    />
  </div>
</template>