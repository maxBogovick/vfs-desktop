<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useFileSystem } from '../composables/useFileSystem';
import FileList from './FileList.vue';
import TextEditor from './TextEditor.vue';
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

// Local File System Instance
const {
  files,
  isLoading,
  loadDirectory,
  openFile,
  normalizePath,
  writeFileContent
} = useFileSystem();

const currentPath = ref<string[]>([]);
const selectedIds = ref<Set<string>>(new Set());
const editorFile = ref<FileItem | null>(null);
const showEditor = ref(false);
const isSavingContainer = ref(false);

// Navigation
const loadPath = async (pathStr: string) => {
  try {
    await loadDirectory(pathStr, props.sessionId);
    // Parse path string back to array for navigation state
    // Note: virtual paths might just be /home/..., simplistic parsing:
    const parts = pathStr.split('/').filter(p => p);
    currentPath.value = parts;
    selectedIds.value.clear();
  } catch (err) {
    console.error('Failed to load stego directory:', err);
  }
};

const handleGoHome = async () => {
  // For virtual FS, home is usually /home or /
  // Let's assume root / for simplicity or ask backend for home
  // but useFileSystem.getHomeDirectory takes backend.
  await loadPath('/');
};

const handleGoUp = async () => {
  if (currentPath.value.length === 0) return;
  const parentPath = '/' + currentPath.value.slice(0, -1).join('/');
  await loadPath(parentPath);
};

const handleItemDoubleClick = (item: FileItem) => {
  if (item.type === 'folder' || item.type === 'drive') {
    // Navigate
    const newPath = item.path; // path is absolute from FS root
    loadPath(newPath);
  } else if (item.type === 'file' || item.type === 'code' || item.type === 'text') {
    // Open in Editor
    editorFile.value = item;
    showEditor.value = true;
  } else {
    // Try system open (likely won't work for virtual fs but kept for consistency)
    openFile(item.path, props.sessionId);
  }
};

const handleSaveFile = async (content: string) => {
  if (editorFile.value) {
    try {
      await writeFileContent(editorFile.value.path, content, props.sessionId);
      // Refresh to update stats if needed
      if (currentPath.value.length === 0) await loadPath('/');
      else await loadPath('/' + currentPath.value.join('/'));
    } catch (err) {
      console.error('Failed to save file:', err);
    }
  }
};

const handleSaveContainer = async () => {
  try {
    isSavingContainer.value = true;
    await vaultSaveStegoContainer(props.sessionId);
    alert('Changes saved to hidden container successfully!');
  } catch (err) {
    console.error('Failed to save container:', err);
    alert('Failed to save container: ' + err);
  } finally {
    isSavingContainer.value = false;
  }
};

// Initial Load
watch(() => props.isOpen, async (isOpen) => {
  if (isOpen && props.sessionId) {
    // Get actual home for this session
    const home = await invoke<string>('get_home_directory', { panelFs: props.sessionId });
    await loadPath(home);

    // Auto-open if only one file exists
    if (files.value.length === 1 && !['folder', 'drive', 'system'].includes(files.value[0].type)) {
      handleItemDoubleClick(files.value[0]);
    }
  } else {
    files.value = [];
    currentPath.value = [];
    showEditor.value = false;
    editorFile.value = null;
  }
});

</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div class="bg-[var(--vf-bg-primary)] w-[90vw] h-[85vh] shadow-2xl rounded-lg flex flex-col overflow-hidden border border-[var(--vf-border-default)]">
      
      <!-- Title Bar -->
      <div class="flex items-center justify-between px-4 py-2 bg-[var(--vf-bg-secondary)] border-b border-[var(--vf-border-default)]">
        <div class="flex items-center gap-2">
          <span class="text-xl">🕵️</span>
          <h3 class="font-bold text-[var(--vf-text-primary)]">{{ title }}</h3>
          <span class="text-xs text-[var(--vf-text-secondary)] px-2 py-0.5 bg-[var(--vf-surface-selected)] rounded">
            Secure Session
          </span>
        </div>
        <div class="flex gap-2">
          <button 
            @click="handleSaveContainer"
            :disabled="isSavingContainer"
            class="px-3 py-1 bg-green-600 hover:bg-green-700 text-white text-xs rounded disabled:opacity-50 transition-colors flex items-center gap-1"
          >
            <span v-if="isSavingContainer">Saving...</span>
            <span v-else>💾 Save Changes</span>
          </button>
          <button 
            @click="emit('close')"
            class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] text-2xl leading-none"
          >
            ×
          </button>
        </div>
      </div>
      <!-- Toolbar -->
      <div class="flex items-center gap-2 px-2 py-1 bg-[#F1EFE2] border-b border-[#919B9C]">
        <button 
          @click="handleGoUp"
          :disabled="currentPath.length === 0"
          class="p-1 rounded hover:bg-white disabled:opacity-50 disabled:cursor-not-allowed"
          title="Up"
        >
          ⬆️
        </button>
        <button 
          @click="handleGoHome"
          class="p-1 rounded hover:bg-white"
          title="Home"
        >
          🏠
        </button>
        
        <!-- Breadcrumbs (Simplified) -->
        <div class="flex-1 px-2 py-1 bg-white border border-[#7F9DB9] mx-2 text-sm flex items-center gap-1 overflow-hidden">
          <span class="text-gray-500">/</span>
          <span v-for="(part, index) in currentPath" :key="index" class="flex items-center">
            <span>{{ part }}</span>
            <span v-if="index < currentPath.length - 1" class="text-gray-400 mx-1">/</span>
          </span>
        </div>
      </div>

      <!-- File List -->
      <div class="flex-1 overflow-hidden relative">
        <FileList
          :items="files"
          :view-mode="'grid'"
          :selected-ids="selectedIds"
          :is-loading="isLoading"
          @item-double-click="handleItemDoubleClick"
          @item-click="(item) => { selectedIds.clear(); selectedIds.add(item.id); }"
          @toggle-selection="(item) => { if (selectedIds.has(item.id)) selectedIds.delete(item.id); else selectedIds.add(item.id); }"
        />
      </div>

      <!-- Status Bar -->
      <div class="px-4 py-1 bg-[var(--vf-bg-secondary)] border-t border-[var(--vf-border-default)] text-xs text-[var(--vf-text-secondary)] flex justify-between">
        <span>{{ files.length }} items</span>
        <span>Session: {{ sessionId }}</span>
      </div>

      <!-- Text Editor Overlay -->
      <TextEditor
        :file="editorFile"
        :panel-filesystem="sessionId"
        :is-open="showEditor"
        @close="showEditor = false"
        @save="handleSaveFile"
      />
    </div>
  </div>
</template>
