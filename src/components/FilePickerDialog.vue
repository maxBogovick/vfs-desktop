<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { FileItem, FileSystemEntry } from '../types';

interface Props {
  isOpen: boolean;
  title?: string;
  allowedExtensions?: string[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select', path: string): void;
}>();

const currentPath = ref<string[]>([]);
const files = ref<FileItem[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const selectedFile = ref<string | null>(null);

// Reuse the helper functions from useFileSystem locally or duplicate them to avoid state pollution
// Since I can't easily import `entriesToFileItems` from `useFileSystem` without refactoring, I'll duplicate minimal logic here.
// Actually, `useFileSystem` exports the function, and `files` is local to the component IF `useFileSystem` was a factory.
// BUT `useFileSystem` defines `const files = ref([])` outside the function, making it singleton.
// So I MUST implement local loading logic.

// Determine file type from extension
const getFileType = (name: string, isDir: boolean): string => {
  if (isDir) return 'folder';
  const ext = name.split('.').pop()?.toLowerCase() || '';
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'].includes(ext)) return 'image';
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) return 'archive';
  if (['mp3', 'wav', 'flac'].includes(ext)) return 'audio';
  if (['mp4', 'mkv', 'avi'].includes(ext)) return 'video';
  if (['js', 'ts', 'rs', 'py', 'html', 'css', 'json'].includes(ext)) return 'code';
  return 'file';
};

const formatFileSize = (bytes?: number): string => {
  if (bytes === undefined) return '--';
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

const loadDirectory = async (pathStr: string) => {
  isLoading.value = true;
  error.value = null;
  selectedFile.value = null;
  try {
    const entries: FileSystemEntry[] = await invoke('read_directory', { path: pathStr, panelFs: 'real' });
    
    // Sort: folders first, then files
    entries.sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });

    files.value = entries.map(entry => ({
      id: entry.path,
      name: entry.name,
      path: entry.path,
      type: getFileType(entry.name, entry.isDir) as any,
      size: entry.size,
      sizeFormatted: formatFileSize(entry.size),
      modified: entry.modified ? new Date(entry.modified * 1000).toLocaleDateString() : undefined,
      permissions: { readable: true, writable: true, executable: entry.isDir },
      tags: []
    }));
  } catch (e) {
    console.error(e);
    error.value = "Failed to load directory";
  } finally {
    isLoading.value = false;
  }
};

const init = async () => {
  try {
    const home = await invoke<string>('get_home_directory', { panelFs: 'real' });
    const parts = home.split('/').filter(p => p);
    currentPath.value = parts;
    await loadDirectory(home);
  } catch (e) {
    console.error(e);
  }
};

watch(() => props.isOpen, (val) => {
  if (val) {
    init();
  }
});

const navigateTo = async (index: number) => {
  const newPath = currentPath.value.slice(0, index + 1);
  currentPath.value = newPath;
  const pathStr = '/' + newPath.join('/');
  await loadDirectory(pathStr);
};

const handleItemClick = async (item: FileItem) => {
  if (item.type === 'folder' || item.type === 'drive') {
    currentPath.value.push(item.name);
    const pathStr = '/' + currentPath.value.join('/');
    await loadDirectory(pathStr);
  } else {
    // Check extension
    if (props.allowedExtensions && props.allowedExtensions.length > 0) {
      const ext = item.name.split('.').pop()?.toLowerCase();
      if (!ext || !props.allowedExtensions.includes(ext)) {
        return; 
      }
    }
    selectedFile.value = item.path;
  }
};

const handleItemDoubleClick = (item: FileItem) => {
    if (item.type !== 'folder' && item.type !== 'drive') {
        if (selectedFile.value) {
            confirmSelection();
        }
    }
};

const goUp = async () => {
  if (currentPath.value.length > 0) {
    currentPath.value.pop();
    const pathStr = currentPath.value.length > 0 ? '/' + currentPath.value.join('/') : '/';
    await loadDirectory(pathStr);
  }
};

const confirmSelection = () => {
  if (selectedFile.value) {
    emit('select', selectedFile.value);
    emit('close');
  }
};

const cancel = () => {
  emit('close');
};
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[2000] flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div class="bg-white dark:bg-[#1e1e1e] w-[600px] h-[500px] rounded-lg shadow-xl flex flex-col border border-gray-200 dark:border-[#333]">
      
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-[#333] bg-gray-50 dark:bg-[#252526]">
        <h2 class="text-sm font-semibold text-gray-800 dark:text-gray-200">{{ title || 'Select File' }}</h2>
        <button @click="cancel" class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 text-lg leading-none">&times;</button>
      </div>

      <!-- Toolbar / Breadcrumbs -->
      <div class="flex items-center gap-2 p-2 bg-white dark:bg-[#1e1e1e] border-b border-gray-200 dark:border-[#333] text-xs">
        <button @click="goUp" :disabled="currentPath.length === 0" class="px-2 py-1 hover:bg-gray-100 dark:hover:bg-[#333] rounded disabled:opacity-50">
          ⬆
        </button>
        <div class="flex-1 flex overflow-hidden">
             <span class="text-gray-500 mr-1">/</span>
             <template v-for="(part, index) in currentPath" :key="index">
                 <button 
                   @click="navigateTo(index)" 
                   class="hover:bg-gray-100 dark:hover:bg-[#333] px-1 rounded truncate max-w-[100px]"
                 >
                   {{ part }}
                 </button>
                 <span v-if="index < currentPath.length - 1" class="text-gray-500 mx-0.5">/</span>
             </template>
        </div>
      </div>

      <!-- File List -->
      <div class="flex-1 overflow-y-auto p-2 bg-white dark:bg-[#1e1e1e]">
        <div v-if="isLoading" class="flex justify-center p-4 text-gray-400">Loading...</div>
        <div v-else-if="error" class="text-red-500 p-4">{{ error }}</div>
        <div v-else class="space-y-0.5">
          <div 
            v-for="file in files" 
            :key="file.path"
            @click="handleItemClick(file)"
            @dblclick="handleItemDoubleClick(file)"
            :class="[
                'flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-sm select-none',
                selectedFile === file.path ? 'bg-blue-100 dark:bg-[#094771] text-blue-700 dark:text-white' : 'hover:bg-gray-100 dark:hover:bg-[#2a2d2e] text-gray-700 dark:text-gray-300',
                (file.type !== 'folder' && allowedExtensions && allowedExtensions.length && !allowedExtensions.includes(file.name.split('.').pop()?.toLowerCase() || '')) ? 'opacity-50' : ''
            ]"
          >
             <span class="text-base">{{ file.type === 'folder' ? '📁' : '📄' }}</span>
             <span class="flex-1 truncate">{{ file.name }}</span>
             <span v-if="file.type !== 'folder'" class="text-xs text-gray-400">{{ file.sizeFormatted }}</span>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="p-3 border-t border-gray-200 dark:border-[#333] bg-gray-50 dark:bg-[#252526] flex justify-end gap-2">
         <button @click="cancel" class="px-3 py-1.5 text-xs border border-gray-300 dark:border-[#3c3c3c] rounded hover:bg-gray-100 dark:hover:bg-[#333] text-gray-700 dark:text-gray-200">Cancel</button>
         <button 
           @click="confirmSelection" 
           :disabled="!selectedFile"
           class="px-3 py-1.5 text-xs bg-blue-600 hover:bg-blue-700 text-white rounded disabled:opacity-50 disabled:cursor-not-allowed"
         >
           Select
         </button>
      </div>

    </div>
  </div>
</template>
