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
import { useProgrammerMode } from '../composables/useProgrammerMode';
import { useUIState } from '../composables/useUIState';
import { registerActivePanelMethods, type ActivePanelMethods } from '../composables/useDualPanel';
import type { Tab, ViewMode, ActivePanel, FileItem, FileSystemBackend } from '../types';

interface Props {
  panelId: ActivePanel;
  isActive: boolean;
  tabs: Tab[];
  activeTabId?: number;
  viewMode: ViewMode;
  panelFilesystem: FileSystemBackend;
}

interface Emits {
  (e: 'activate'): void;
  (e: 'update:tabs', tabs: Tab[]): void;
  (e: 'update:activeTabId', tabId: number | undefined): void;
  (e: 'switchFilesystem', backend: FileSystemBackend): void;
  (e: 'editFile', item: FileItem, panelFs?: string): void;
  (e: 'previewFile', item: FileItem, panelFs?: string): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// Programmer mode
const { isProgrammerMode } = useProgrammerMode();

// UI State (Global)
const { editModeEnabled } = useUIState();

// Sorting state
const sortBy = ref<'name' | 'size' | 'modified' | 'type'>('name');
const sortOrder = ref<'asc' | 'desc'>('asc');
const showHidden = ref(false);

// File System
const {
  files,
  isLoading,
  loadDirectory,
  copyItems,
  moveItems
} = useFileSystem();

// Computed: Sorted and filtered files
const sortedFiles = computed(() => {
  let result = [...files.value];

  // Filter hidden files if needed
  if (!showHidden.value) {
    result = result.filter(file => !file.name.startsWith('.'));
  }

  // Sort files
  result.sort((a, b) => {
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

  return result;
});

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
    return await getHomeDirectory(props.panelFilesystem);
  }
  return pathString;
};

// Function to refresh current directory
const refreshCurrentDirectory = async () => {
  const pathString = await getCurrentDirectoryPath();
  await loadDirectory(pathString, props.panelFilesystem);
};

// File Operations
const fileOps = useFileOperations(refreshCurrentDirectory);

// Inline File Creator state (per panel)
const showInlineCreator = ref(false);
const inlineCreatorMode = ref<'file' | 'folder'>('file');

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
  await loadDirectory(pathString, props.panelFilesystem);
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
  } else {
    // Check if edit mode is enabled and file is text/code
    const isEditableFile = item.type === 'file' || item.type === 'code';
    if (editModeEnabled.value && isEditableFile) {
      emit('editFile', item, props.panelFilesystem);
    } else {
      emit('previewFile', item, props.panelFilesystem);
    }
  }
};

const handleToggleEditMode = () => {
  editModeEnabled.value = !editModeEnabled.value;
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
  startDrag(itemsToDrag, event, props.panelFilesystem);
};

// Handle item drop
const handleItemDrop = async (item: FileItem, event: DragEvent) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }

  event.preventDefault();
  console.log('[FilePanel] Drop on item:', item.name);

  // Use wrappers to route through handleTransfer for conflict resolution
  const onMove = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'move', srcFs, destFs);
  const onCopy = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'copy', srcFs, destFs);

  await handleDrop(item, event, onMove, onCopy, props.panelFilesystem);
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

  // Use wrappers to route through handleTransfer for conflict resolution
  const onMove = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'move', srcFs, destFs);
  const onCopy = (src: string[], dest: string, srcFs?: string, destFs?: string) =>
    fileOps.handleTransfer(src, dest, 'copy', srcFs, destFs);

  await handleDrop(targetItem, event, onMove, onCopy, props.panelFilesystem);
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
  fileOps.handleCopy([item], props.panelFilesystem);
};

const handleCutItem = (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  fileOps.handleCut([item], props.panelFilesystem);
};

const handleDeleteItem = (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  fileOps.handleDelete([item], currentPath.value, clearSelection, showConfirm, props.panelFilesystem);
};

const handleRenameItem = (item: FileItem) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }
  fileOps.handleRename([item], currentPath.value, showInput, props.panelFilesystem);
};

// Handle file creation from inline creator
const handleCreateFile = async (payload: { name: string; isFolder: boolean; templateId?: string }) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }

  try {
    const { createFolder, createFile } = useFileSystem();
    const pathString = await getCurrentDirectoryPath();

    if (payload.isFolder) {
      await createFolder(pathString, payload.name, props.panelFilesystem);
    } else {
      const { getTemplateContent } = await import('../composables/useTemplates').then(m => m.useTemplates());
      const content = payload.templateId ? await getTemplateContent(payload.templateId) : undefined;
      await createFile(pathString, payload.name, content, props.panelFilesystem);
    }

    await refreshCurrentDirectory();
    showInlineCreator.value = false;
  } catch (err) {
    console.error('Failed to create file:', err);
  }
};

// Handle batch file creation
const handleBatchCreateFiles = async (names: string[]) => {
  // Activate panel on any interaction
  if (!props.isActive) {
    emit('activate');
  }

  try {
    const files = names.map(name => ({ name }));
    await fileOps.handleBatchCreate(currentPath.value, files, props.panelFilesystem);
    showInlineCreator.value = false;
  } catch (err) {
    console.error('Failed to batch create files:', err);
  }
};

// Handle inline creator cancel
const handleCancelInlineCreator = () => {
  showInlineCreator.value = false;
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

// Toolbar action handlers
const handleSort = (field: 'name' | 'size' | 'modified' | 'type', order: 'asc' | 'desc') => {
  if (!props.isActive) {
    emit('activate');
  }
  sortBy.value = field;
  sortOrder.value = order;
};

const handleSelectAll = () => {
  if (!props.isActive) {
    emit('activate');
  }
  files.value.forEach(file => selectedIds.value.add(file.id));
};

const handleInvertSelection = () => {
  if (!props.isActive) {
    emit('activate');
  }
  const newSelection = new Set<string>();
  files.value.forEach(file => {
    if (!selectedIds.value.has(file.id)) {
      newSelection.add(file.id);
    }
  });
  selectedIds.value = newSelection;
};

const handleRefresh = async () => {
  if (!props.isActive) {
    emit('activate');
  }
  await refreshCurrentDirectory();
};

const handleToggleHidden = () => {
  if (!props.isActive) {
    emit('activate');
  }
  showHidden.value = !showHidden.value;
};

const handleNavigateToBreadcrumb = (index: number) => {
  if (!props.isActive) {
    emit('activate');
  }
  const newPath = currentPath.value.slice(0, index + 1);
  updateTabPath(newPath);
};

// Register panel methods when it becomes active (for keyboard shortcuts in dual mode)
watch(() => props.isActive, (isActive) => {
  if (isActive) {
    const methods: ActivePanelMethods = {
      getFiles: () => files.value,
      getSelectedIds: () => selectedIds.value,
      getSelectedItems: () => getSelected(),
      selectAll: () => {
        files.value.forEach(file => selectedIds.value.add(file.id));
      },
      clearSelection,
      handleCopy: () => fileOps.handleCopy(getSelected(), props.panelFilesystem),
      handleEditFile: () => {
        const item = getSelectedItems(files.value)[0];
        console.log('handleEditFile', item)
        console.log('props.panelFilesystem', props.panelFilesystem)
        if (item) {
          emit('editFile', item, props.panelFilesystem);
        }
      },
      handleCut: () => fileOps.handleCut(getSelected(), props.panelFilesystem),
      handlePaste: () => fileOps.handlePaste(currentPath.value, props.panelFilesystem),
      handleDelete: () => fileOps.handleDelete(getSelected(), currentPath.value, clearSelection, showConfirm, props.panelFilesystem),
      handleRename: () => fileOps.handleRename(getSelected(), currentPath.value, showInput, props.panelFilesystem),
      handleRefresh: () => fileOps.handleRefresh(currentPath.value, props.panelFilesystem),
      handleNewFolder: () => fileOps.handleNewFolder(currentPath.value, showInput, props.panelFilesystem),
      handleNewFile: () => {
        inlineCreatorMode.value = 'file';
        showInlineCreator.value = true;
      },
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
            name: tab.history[tab.historyIndex + 1][tab.history[tab.historyIndex - 1].length - 1] || 'Home',
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
        :current-path="currentPath"
        :sort-by="sortBy"
        :sort-order="sortOrder"
        :show-hidden="showHidden"
        :is-programmer-mode="isProgrammerMode"
        :panel-filesystem="panelFilesystem"
        :edit-mode-enabled="editModeEnabled"
        @switch-tab="handleSwitchTab"
        @close-tab="handleCloseTab"
        @add-tab="handleAddTab"
        @sort="handleSort"
        @select-all="handleSelectAll"
        @invert-selection="handleInvertSelection"
        @refresh="handleRefresh"
        @toggle-hidden="handleToggleHidden"
        @toggle-edit-mode="handleToggleEditMode"
        @navigate-to-breadcrumb="handleNavigateToBreadcrumb"
        @switch-filesystem="(backend) => emit('switchFilesystem', backend)"
    />

    <!-- File List -->
    <FileList
        :items="sortedFiles"
        :view-mode="props.viewMode"
        :selected-ids="selectedIds"
        :focused-id="focusedId"
        :is-loading="isLoading"
        :is-dragging="isDragging"
        :drag-target-id="dragOverId"
        :show-inline-creator="showInlineCreator"
        :inline-creator-mode="inlineCreatorMode"
        :current-path="currentPath"
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
        @create-file="handleCreateFile"
              @batch-create-files="handleBatchCreateFiles"
              @cancel-inline-creator="handleCancelInlineCreator"
        @open-terminal="handleOpenTerminal"
    />
  </div>
</template>