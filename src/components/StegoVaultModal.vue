<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue';
import { useFileSystem } from '../composables/useFileSystem';
import { useFileOperations } from '../composables/useFileOperations';
import { useSelection } from '../composables/useSelection';
import { useContextMenu } from '../composables/useContextMenu';
import { useContextMenuActions } from '../composables/useContextMenuActions';
import { useDialogs } from '../composables/useDialogs';
import { useNotifications } from '../composables/useNotifications';
import { useDragDrop } from '../composables/useDragDrop';
import { useClipboard } from '../composables/useClipboard';

import FileList from './FileList.vue';
import TextEditor from './TextEditor.vue';
import ContextMenu from './ContextMenu.vue';
import PanelToolbar from './PanelToolbar.vue';
import ConfirmDialog from './ConfirmDialog.vue';
import InputDialog from './InputDialog.vue';
import PropertiesDialog from './PropertiesDialog.vue';
import Preview from './Preview.vue';

import type { FileItem } from '../types';
import { invoke } from '@tauri-apps/api/core';
import { vaultSaveStegoContainer } from '../utils/api';

interface Props {
  isOpen: boolean;
  sessionId: string;
  title: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'close'): void;
}>();

// ============================================================================
// STATE & COMPOSABLES
// ============================================================================

// Local File System Instance
const {
  files,
  isLoading,
  loadDirectory,
  openFile,
  normalizePath,
  writeFileContent,
  openTerminal
} = useFileSystem();

const currentPath = ref<string[]>([]);
const currentPathString = computed(() => {
    const p = '/' + currentPath.value.join('/');
    return p.startsWith('//') ? p.substring(1) : p;
});

// Selection
const {
  selectedIds,
  selectedCount,
  handleItemClick,
  getSelectedItems,
  clearSelection,
  selectAll
} = useSelection();

// Dialogs & Notifications
const { 
  confirmDialog, showConfirm, closeConfirm,
  inputDialog, showInput, closeInput,
  propertiesDialog, showProperties, closeProperties
} = useDialogs();
const { success, error } = useNotifications();

// Clipboard
const { hasClipboardItems } = useClipboard();

// Editor & Preview
const editorFile = ref<FileItem | null>(null);
const showEditor = ref(false);
const previewFile = ref<FileItem | null>(null);

// Saving State
const isSavingContainer = ref(false);

// View State
const viewMode = ref<'list' | 'grid'>('grid');
const sortBy = ref<'name' | 'size' | 'modified' | 'type'>('name');
const sortOrder = ref<'asc' | 'desc'>('asc');
const showHidden = ref(false);

// ============================================================================
// NAVIGATION & LOADING
// ============================================================================

const loadPath = async (pathStr: string) => {
  try {
    await loadDirectory(pathStr, props.sessionId);
    // Parse path string back to array for navigation state
    const parts = pathStr.replace(/^\//, '').split('/').filter(p => p);
    currentPath.value = parts;
    clearSelection();
  } catch (err) {
    console.error('Failed to load stego directory:', err);
    error('Navigation Error', String(err));
  }
};

const refreshCurrent = async () => {
    await loadPath(currentPathString.value);
};

// File Operations (bound to this session)
const fileOps = useFileOperations(refreshCurrent);

const handleGoHome = async () => {
  const home = await invoke<string>('get_home_directory', { panelFs: props.sessionId });
  await loadPath(home);
};

const handleGoUp = async () => {
  if (currentPath.value.length === 0) return;
  const parentPath = '/' + currentPath.value.slice(0, -1).join('/');
  await loadPath(parentPath);
};

const navigateToBreadcrumb = (index: number) => {
    const newPath = '/' + currentPath.value.slice(0, index + 1).join('/');
    loadPath(newPath);
};

// ============================================================================
// ITEM ACTIONS
// ============================================================================

const handleItemDoubleClick = (item: FileItem) => {
  if (item.type === 'folder' || item.type === 'drive') {
    loadPath(item.path);
  } else if (item.type === 'file' || item.type === 'code' || item.type === 'text') {
    editorFile.value = item;
    showEditor.value = true;
  } else {
    // Preview for images/others
    previewFile.value = item;
  }
};

const handleSaveFile = async (content: string) => {
  if (editorFile.value) {
    try {
      await writeFileContent(editorFile.value.path, content, props.sessionId);
      await refreshCurrent();
    } catch (err) {
      console.error('Failed to save file:', err);
      error('Save Error', String(err));
    }
  }
};

const handleSaveContainer = async () => {
  try {
    isSavingContainer.value = true;
    await vaultSaveStegoContainer(props.sessionId);
    success('Saved', 'Changes saved to hidden container successfully!');
  } catch (err) {
    console.error('Failed to save container:', err);
    error('Save Failed', String(err));
  } finally {
    isSavingContainer.value = false;
  }
};

// ============================================================================
// CONTEXT MENU
// ============================================================================

const { contextMenu, showContextMenu, closeContextMenu } = useContextMenu();

const contextMenuActions = useContextMenuActions({
  isDualMode: () => false,
  getActivePanelMethods: () => null,
  getActivePanelPath: () => [],
  getCurrentPath: () => currentPath.value,
  getSelectedItems: () => getSelectedItems(files.value),
  clearSelection,
  refreshCurrentDirectory: refreshCurrent,
  openEditor: (item) => { editorFile.value = item; showEditor.value = true; },
  openBatchRename: () => {}, // Not implemented in modal yet
  openBatchAttribute: () => {}, // Not implemented in modal yet
  getActiveFilesystem: () => props.sessionId,
});

const contextMenuHandlers = {
  ...contextMenuActions,
  share: async () => {}, // Not relevant in secure modal
  newFile: () => {
      showInput(
          'New File',
          'Enter file name:',
          (name) => {
              if (name) {
                  fileOps.handleNewFile(currentPath.value, name, undefined, props.sessionId);
              }
          },
          'New File.txt',
          'File Name',
          'text'
      );
  },
  selectAll: () => selectAll(files.value),
  // Override openTerminal to use correct FS context if needed, though mostly local
  openTerminal: async () => {
      if (contextMenu.value?.item) {
          await openTerminal(contextMenu.value.item.path);
      }
  }
};

// ============================================================================
// DRAG & DROP
// ============================================================================

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

const handleDragStart = (item: FileItem, event: DragEvent) => {
  const items = selectedIds.value.has(item.id)
    ? getSelectedItems(files.value)
    : [item];
  // Pass the current session ID as the source filesystem
  startDrag(items, event, props.sessionId);
};

const handleItemDrop = async (item: FileItem, event: DragEvent) => {
  // Use sourceFs from global drag state (passed by handleDrop), destination is always current session
  const onMove = (src: string[], dest: string, srcFs?: string) => 
      fileOps.handleTransfer(src, dest, 'move', srcFs, props.sessionId);
  
  const onCopy = (src: string[], dest: string, srcFs?: string) => 
      fileOps.handleTransfer(src, dest, 'copy', srcFs, props.sessionId);
  
  await handleDrop(item, event, onMove, onCopy, props.sessionId);
  await refreshCurrent();
};

const handleBackgroundDrop = async (event: DragEvent) => {
  const targetItem: FileItem = {
    id: currentPathString.value,
    name: 'root',
    path: currentPathString.value,
    type: 'folder',
    size: 0,
    modified: ''
  };
  
  const onMove = (src: string[], dest: string, srcFs?: string) => 
      fileOps.handleTransfer(src, dest, 'move', srcFs, props.sessionId);
      
  const onCopy = (src: string[], dest: string, srcFs?: string) => 
      fileOps.handleTransfer(src, dest, 'copy', srcFs, props.sessionId);

  await handleDrop(targetItem, event, onMove, onCopy, props.sessionId);
  await refreshCurrent();
};

// ============================================================================
// SORTING & PROCESSING
// ============================================================================

const processedFiles = computed(() => {
  let result = [...files.value];

  if (!showHidden.value) {
    result = result.filter(file => !file.name.startsWith('.'));
  }

  result.sort((a, b) => {
      const aIsFolder = ['folder', 'drive', 'system'].includes(a.type);
      const bIsFolder = ['folder', 'drive', 'system'].includes(b.type);

      if (aIsFolder && !bIsFolder) return -1;
      if (!aIsFolder && bIsFolder) return 1;

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

// ============================================================================
// LIFECYCLE
// ============================================================================

watch(() => props.isOpen, async (isOpen) => {
  if (isOpen && props.sessionId) {
    const home = await invoke<string>('get_home_directory', { panelFs: props.sessionId });
    await loadPath(home);
  } else {
    files.value = [];
    currentPath.value = [];
    showEditor.value = false;
    editorFile.value = null;
    previewFile.value = null;
  }
});
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm" @click.self="emit('close')">
    <div class="bg-[var(--vf-bg-primary)] w-[90vw] h-[85vh] shadow-2xl rounded-lg flex flex-col overflow-hidden border border-[var(--vf-border-default)]">
      
      <!-- Title Bar -->
      <div class="flex items-center justify-between px-4 py-2 bg-[var(--vf-bg-secondary)] border-b border-[var(--vf-border-default)]">
        <div class="flex items-center gap-2">
          <span class="text-xl">🔐</span>
          <h3 class="font-bold text-[var(--vf-text-primary)]">{{ title }}</h3>
          <span class="text-xs text-[var(--vf-text-secondary)] px-2 py-0.5 bg-[var(--vf-surface-selected)] rounded">
            Secure Session
          </span>
        </div>
        <div class="flex gap-2">
          <button 
            @click="handleSaveContainer"
            :disabled="isSavingContainer"
            class="px-3 py-1 bg-green-600 hover:bg-green-700 text-white text-xs rounded disabled:opacity-50 transition-colors flex items-center gap-1 shadow-sm"
          >
            <span v-if="isSavingContainer">Saving...</span>
            <span v-else>💾 Save Changes</span>
          </button>
          <button 
            @click="emit('close')"
            class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] text-2xl leading-none ml-2"
          >
            ×
          </button>
        </div>
      </div>

      <!-- Toolbar -->
      <PanelToolbar
        :tabs="[{ id: 1, name: 'Secure Folder', path: currentPath, history: [], historyIndex: 0 }]"
        :active-tab-id="1"
        :current-path="currentPath"
        :sort-by="sortBy"
        :sort-order="sortOrder"
        :show-hidden="showHidden"
        :edit-mode-enabled="false"
        @sort="(field, order) => { sortBy = field; sortOrder = order; }"
        @toggle-hidden="showHidden = !showHidden"
        @refresh="refreshCurrent"
        @navigate-to-breadcrumb="navigateToBreadcrumb"
        @select-all="selectAll(files)"
        @invert-selection="() => {}"
      />

      <!-- Custom Toolbar Actions (Specific to this modal if needed) -->
      <div class="px-2 py-1 bg-[#F1EFE2] border-b border-[#919B9C] flex gap-2 text-xs">
          <button @click="handleGoHome" class="hover:bg-white px-1 rounded">🏠 Home</button>
          <button @click="handleGoUp" class="hover:bg-white px-1 rounded">⬆️ Up</button>
          <div class="w-[1px] bg-gray-300 h-4 my-auto"></div>
          <button @click="fileOps.handleNewFolder(currentPath, showInput, props.sessionId)" class="hover:bg-white px-1 rounded">📁 New Folder</button>
          <button @click="contextMenuHandlers.newFile" class="hover:bg-white px-1 rounded">📄 New File</button>
      </div>

      <!-- File List -->
      <div class="flex-1 overflow-hidden relative flex">
        <FileList
          :items="processedFiles"
          :view-mode="viewMode"
          :selected-ids="selectedIds"
          :is-loading="isLoading"
          :current-path="currentPath"
          :is-dragging="isDragging"
          :drag-target-id="dragOverId"
          @item-double-click="handleItemDoubleClick"
          @item-click="(item, e) => handleItemClick(item, files, e)"
          @item-context-menu="(item, e) => showContextMenu(item, e)"
          @background-context-menu="(e) => showContextMenu(null, e)"
          @toggle-selection="(item) => handleItemClick(item, files, { ctrlKey: true } as any)"
          
          @drag-start="handleDragStart"
          @drag-over="handleDragOver"
          @drag-leave="handleDragLeave"
          @drop="handleItemDrop"
          @drop-on-background="handleBackgroundDrop"
          @drag-over-background="handleDragOverBackground"

          @copy-item="(item) => fileOps.handleCopy([item], props.sessionId)"
          @cut-item="(item) => fileOps.handleCut([item], props.sessionId)"
          @delete-item="(item) => fileOps.handleDelete([item], currentPath, clearSelection, showConfirm, props.sessionId)"
          @rename-item="(item) => fileOps.handleRename([item], currentPath, showInput, props.sessionId)"
        />
      </div>

      <!-- Status Bar -->
      <div class="px-4 py-1 bg-[var(--vf-bg-secondary)] border-t border-[var(--vf-border-default)] text-xs text-[var(--vf-text-secondary)] flex justify-between items-center">
        <div class="flex gap-4">
            <span>{{ files.length }} items</span>
            <span v-if="selectedCount > 0">{{ selectedCount }} selected</span>
        </div>
        <div class="flex items-center gap-2">
            <button 
                @click="viewMode = viewMode === 'grid' ? 'list' : 'grid'" 
                class="hover:text-[var(--vf-text-primary)]"
            >
                {{ viewMode === 'grid' ? '📄 List View' : 'grid_view Grid View' }}
            </button>
        </div>
      </div>

      <!-- Overlays -->
      <TextEditor
        :file="editorFile"
        :panel-filesystem="sessionId"
        :is-open="showEditor"
        @close="showEditor = false"
        @save="handleSaveFile"
      />

      <Preview 
        v-if="previewFile"
        :file="previewFile"
        :width="800"
        @close="previewFile = null"
      />

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
        @new-folder="fileOps.handleNewFolder(currentPath, showInput, props.sessionId)"
        @new-file="contextMenuHandlers.newFile"
        @select-all="contextMenuHandlers.selectAll"
        @refresh="refreshCurrent"
        @properties="() => { if(contextMenu?.item) showProperties(contextMenu.item) }"
        @close="closeContextMenu"
      />

      <ConfirmDialog
        :is-open="confirmDialog.isOpen"
        :title="confirmDialog.title"
        :message="confirmDialog.message"
        :type="confirmDialog.type"
        @confirm="() => { confirmDialog.onConfirm(); closeConfirm(); }"
        @cancel="closeConfirm"
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

      <PropertiesDialog
        :is-open="propertiesDialog.isOpen"
        :file="propertiesDialog.file"
        @close="closeProperties"
      />

    </div>
  </div>
</template>
