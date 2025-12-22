<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';

// --- Types ---
type FileType = 'folder' | 'file' | 'image' | 'pdf' | 'code' | 'audio' | 'video' | 'archive';
type ViewMode = 'list' | 'grid';
type Theme = 'light' | 'dark';

interface FileItem {
  id: string;
  name: string;
  type: FileType;
  size: string;
  modified: string;
  tags: string[];
  path: string[];
  content?: string;
  color?: string;
}

interface Tab {
  id: string;
  name: string;
  path: string[];
}

// --- State ---
const files = ref<FileItem[]>([
  {
    id: '1',
    name: 'Projects',
    type: 'folder',
    size: '--',
    modified: 'Today, 10:23 AM',
    tags: ['work'],
    path: ['root', 'Projects'],
    color: 'bg-blue-500',
  },
  {
    id: '2',
    name: 'design_system.fig',
    type: 'file',
    size: '45 MB',
    modified: 'Yesterday',
    tags: ['design', 'urgent'],
    path: ['root', 'Projects'],
    color: 'bg-pink-500',
  },
  {
    id: '3',
    name: 'api_docs.pdf',
    type: 'pdf',
    size: '2.4 MB',
    modified: '2 days ago',
    tags: ['work', 'docs'],
    path: ['root', 'Projects'],
    color: 'bg-red-500',
  },
  {
    id: '4',
    name: 'app.js',
    type: 'code',
    size: '12 KB',
    modified: '1 week ago',
    tags: ['dev', 'javascript'],
    path: ['root', 'Projects'],
    color: 'bg-yellow-500',
    content: 'import Vue from "vue";\n// ...',
  },
  {
    id: '5',
    name: 'vacation_photos',
    type: 'folder',
    size: '--',
    modified: 'Jun 15, 2025',
    tags: ['personal', 'photos'],
    path: ['root'],
    color: 'bg-green-500',
  },
]);

const selectedIds = ref<Set<string>>(new Set());
const currentPath = ref<string[]>(['root']);
const tabs = ref<Tab[]>([{ id: 'tab-1', name: 'Home', path: ['root'] }]);
const activeTabId = ref('tab-1');
const viewMode = ref<ViewMode>('list');
const theme = ref<Theme>('light');
const searchQuery = ref('');
const isCommandPaletteOpen = ref(false);
const previewFile = ref<FileItem | null>(null);
const contextMenu = ref<{ x: number; y: number; file: FileItem } | null>(null);

// --- Computed ---
const currentFiles = computed(() => {
  const path = currentPath.value;
  return files.value.filter(file => {
    const isInPath = file.path.join('/') === path.join('/');
    const matchesSearch = file.name.toLowerCase().includes(searchQuery.value.toLowerCase());
    return isInPath && matchesSearch;
  });
});

const activeTab = computed(() => tabs.value.find(tab => tab.id === activeTabId.value));
const selectedCount = computed(() => selectedIds.value.size);
const isDark = computed(() => theme.value === 'dark');

// --- Methods ---
const toggleSelect = (id: string, multi: boolean = false) => {
  if (!multi) selectedIds.value.clear();
  if (selectedIds.value.has(id)) selectedIds.value.delete(id);
  else selectedIds.value.add(id);
};

const openFolder = (file: FileItem) => {
  currentPath.value = file.path;
  const newTab = { id: `tab-${Date.now()}`, name: file.name, path: file.path };
  tabs.value.push(newTab);
  activeTabId.value = newTab.id;
};

const goBack = () => {
  if (currentPath.value.length > 1) {
    currentPath.value.pop();
  }
};

const goHome = () => {
  currentPath.value = ['root'];
  activeTabId.value = 'tab-1';
};

const closeTab = (tabId: string) => {
  if (tabs.value.length > 1) {
    tabs.value = tabs.value.filter(tab => tab.id !== tabId);
    if (activeTabId.value === tabId) {
      activeTabId.value = tabs.value[tabs.value.length - 1].id;
      currentPath.value = tabs.value[tabs.value.length - 1].path;
    }
  }
};

const switchTab = (tabId: string) => {
  activeTabId.value = tabId;
  currentPath.value = tabs.value.find(tab => tab.id === tabId)!.path;
};

const openPreview = (file: FileItem) => {
  previewFile.value = file;
};

const closePreview = () => {
  previewFile.value = null;
};

const openContextMenu = (e: MouseEvent, file: FileItem) => {
  e.preventDefault();
  contextMenu.value = { x: e.clientX, y: e.clientY, file };
};

const closeContextMenu = () => {
  contextMenu.value = null;
};

const toggleTheme = () => {
  theme.value = theme.value === 'light' ? 'dark' : 'light';
};

const handleKeydown = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault();
    isCommandPaletteOpen.value = !isCommandPaletteOpen.value;
  }
  if (e.key === 'Escape') {
    isCommandPaletteOpen.value = false;
    closePreview();
    selectedIds.value.clear();
  }
};

// --- Lifecycle ---
onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
  document.addEventListener('click', closeContextMenu);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
  document.removeEventListener('click', closeContextMenu);
});

// --- Watch ---
watch(activeTabId, (newTabId) => {
  const tab = tabs.value.find(tab => tab.id === newTabId);
  if (tab) currentPath.value = tab.path;
});
</script>

<template>
  <div
      class="h-screen flex flex-col"
      :class="isDark ? 'bg-gray-900 text-gray-100' : 'bg-gray-50 text-gray-900'"
  >
    <!-- Top Bar -->
    <div
        class="h-10 flex items-center justify-between px-4 border-b"
        :class="isDark ? 'border-gray-700' : 'border-gray-200'"
    >
      <div class="flex items-center gap-4">
        <button @click="goBack" class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700">
          ←
        </button>
        <button class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 opacity-50 cursor-not-allowed">
          →
        </button>
        <button @click="goHome" class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700">
          🏠
        </button>
      </div>
      <div class="flex items-center gap-2">
        <button
            @click="toggleTheme"
            class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
            :title="`Switch to ${isDark ? 'Light' : 'Dark'} Mode`"
        >
          {{ isDark ? '☀️' : '🌙' }}
        </button>
        <button
            @click="isCommandPaletteOpen = true"
            class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
            title="Command Palette (Ctrl+K)"
        >
          ⌘K
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex items-center gap-1 px-2 py-1 border-b overflow-x-auto" :class="isDark ? 'border-gray-700' : 'border-gray-200'">
      <div
          v-for="tab in tabs"
          :key="tab.id"
          @click="switchTab(tab.id)"
          class="flex items-center gap-2 px-3 py-1 rounded-t cursor-pointer"
          :class="{
          'bg-white dark:bg-gray-800 border-t-2 border-blue-500': activeTabId === tab.id,
          'hover:bg-gray-100 dark:hover:bg-gray-700': activeTabId !== tab.id,
          'border-gray-200 dark:border-gray-700': true,
        }"
      >
        <span class="text-sm">{{ tab.name }}</span>
        <button
            @click.stop="closeTab(tab.id)"
            class="hover:bg-gray-200 dark:hover:bg-gray-700 rounded p-0.5"
        >
          ✕
        </button>
      </div>
      <button
          @click="tabs.push({ id: `tab-${Date.now()}`, name: 'New Tab', path: ['root'] })"
          class="px-2 py-1 text-sm rounded hover:bg-gray-200 dark:hover:bg-gray-700"
      >
        +
      </button>
    </div>

    <!-- Breadcrumb -->
    <div class="flex items-center gap-1 px-4 py-2 text-sm">
      <span
          v-for="(segment, i) in currentPath"
          :key="i"
          @click="currentPath = currentPath.slice(0, i + 1)"
          class="hover:underline cursor-pointer"
      >
        {{ segment }}
        <span v-if="i < currentPath.length - 1" class="mx-1">/</span>
      </span>
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Sidebar -->
      <div
          class="w-60 p-4 border-r overflow-y-auto"
          :class="isDark ? 'border-gray-700' : 'border-gray-200'"
      >
        <h3 class="text-xs font-bold uppercase mb-3 text-gray-500">Favorites</h3>
        <div class="space-y-1">
          <div
              v-for="file in files.filter(f => f.tags.includes('work'))"
              :key="file.id"
              @click="openFolder(file)"
              class="flex items-center gap-2 p-2 rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700"
          >
            <div class="w-4 h-4 rounded" :class="file.color"></div>
            <span class="text-sm">{{ file.name }}</span>
          </div>
        </div>
        <h3 class="text-xs font-bold uppercase mt-6 mb-3 text-gray-500">Tags</h3>
        <div class="flex flex-wrap gap-2">
          <span
              v-for="tag in ['work', 'design', 'dev', 'personal']"
              :key="tag"
              class="px-2 py-0.5 text-xs rounded-full"
              :class="{
              'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200': tag === 'work',
              'bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200': tag === 'design',
              'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200': tag === 'dev',
              'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200': tag === 'personal',
            }"
          >
            {{ tag }}
          </span>
        </div>
      </div>

      <!-- File List -->
      <div class="flex-1 overflow-auto p-4">
        <!-- Toolbar -->
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <button
                @click="viewMode = 'list'"
                class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
                :class="{ 'bg-gray-200 dark:bg-gray-700': viewMode === 'list' }"
            >
              ☰
            </button>
            <button
                @click="viewMode = 'grid'"
                class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
                :class="{ 'bg-gray-200 dark:bg-gray-700': viewMode === 'grid' }"
            >
              ⊞
            </button>
          </div>
          <div class="text-xs text-gray-500">
            {{ currentFiles.length }} items
            <span v-if="selectedCount > 0"> • {{ selectedCount }} selected</span>
          </div>
        </div>

        <!-- Search -->
        <input
            v-model="searchQuery"
            placeholder="Search files..."
            class="w-full p-2 mb-4 text-sm rounded border"
            :class="isDark ? 'border-gray-700 bg-gray-800' : 'border-gray-200'"
        />

        <!-- File List/Grid -->
        <div
            :class="{
            'grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4': viewMode === 'grid',
            'space-y-1': viewMode === 'list',
          }"
        >
          <div
              v-for="file in currentFiles"
              :key="file.id"
              @click="toggleSelect(file.id, e => e.ctrlKey || e.metaKey)"
              @dblclick="file.type === 'folder' ? openFolder(file) : openPreview(file)"
              @contextmenu="openContextMenu($event, file)"
              class="group p-2 rounded cursor-pointer"
              :class="{
              'bg-blue-50 dark:bg-blue-900/30': selectedIds.has(file.id),
              'hover:bg-gray-100 dark:hover:bg-gray-700': !selectedIds.has(file.id),
              'flex items-center gap-3': viewMode === 'list',
              'flex-col items-center': viewMode === 'grid',
            }"
          >
            <div class="w-12 h-12 flex items-center justify-center rounded" :class="file.color">
              <span class="text-white">
                {{ file.type === 'folder' ? '📁' : file.type === 'pdf' ? '📄' : file.type === 'code' ? '📜' : '🖼️' }}
              </span>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium truncate">{{ file.name }}</span>
                <div class="flex gap-1">
                  <span
                      v-for="tag in file.tags"
                      :key="tag"
                      class="w-2 h-2 rounded-full"
                      :class="{
                      'bg-blue-500': tag === 'work',
                      'bg-pink-500': tag === 'design',
                      'bg-yellow-500': tag === 'dev',
                      'bg-green-500': tag === 'personal',
                    }"
                  ></span>
                </div>
              </div>
              <div class="text-xs text-gray-500">
                <span>{{ file.modified }}</span>
                <span v-if="file.size"> • {{ file.size }}</span>
              </div>
            </div>
            <div
                v-if="selectedIds.has(file.id)"
                class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <button class="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded">🔗</button>
              <button class="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded">🗑️</button>
            </div>
          </div>
        </div>
      </div>

      <!-- Preview Pane -->
      <div
          v-if="previewFile"
          class="w-80 border-l overflow-y-auto p-4"
          :class="isDark ? 'border-gray-700' : 'border-gray-200'"
      >
        <div class="flex justify-between items-center mb-4">
          <h3 class="font-bold">Preview</h3>
          <button @click="closePreview" class="hover:bg-gray-200 dark:hover:bg-gray-700 rounded p-1">
            ✕
          </button>
        </div>
        <div class="flex flex-col items-center mb-4">
          <div class="w-24 h-24 flex items-center justify-center rounded-lg mb-2" :class="previewFile.color">
            <span class="text-white text-3xl">
              {{ previewFile.type === 'folder' ? '📁' : previewFile.type === 'pdf' ? '📄' : previewFile.type === 'code' ? '📜' : '🖼️' }}
            </span>
          </div>
          <h4 class="font-medium">{{ previewFile.name }}</h4>
          <span class="text-xs text-gray-500">{{ previewFile.type }}</span>
        </div>
        <div class="space-y-2 text-sm">
          <div>
            <span class="text-gray-500">Size:</span>
            <span>{{ previewFile.size }}</span>
          </div>
          <div>
            <span class="text-gray-500">Modified:</span>
            <span>{{ previewFile.modified }}</span>
          </div>
          <div>
            <span class="text-gray-500">Tags:</span>
            <div class="flex gap-1 mt-1">
              <span
                  v-for="tag in previewFile.tags"
                  :key="tag"
                  class="px-2 py-0.5 text-xs rounded-full"
                  :class="{
                  'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200': tag === 'work',
                  'bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200': tag === 'design',
                  'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200': tag === 'dev',
                  'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200': tag === 'personal',
                }"
              >
                {{ tag }}
              </span>
            </div>
          </div>
        </div>
        <div v-if="previewFile.content" class="mt-4 p-2 rounded bg-gray-100 dark:bg-gray-800">
          <pre class="text-xs">{{ previewFile.content }}</pre>
        </div>
      </div>
    </div>

    <!-- Command Palette -->
    <div
        v-if="isCommandPaletteOpen"
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
        @click="isCommandPaletteOpen = false"
    >
      <div
          @click.stop
          class="w-full max-w-md bg-white dark:bg-gray-800 rounded-lg shadow-xl p-4"
      >
        <input
            autofocus
            placeholder="Type a command or search..."
            class="w-full p-2 text-sm rounded border dark:bg-gray-700 dark:border-gray-600"
        />
        <div class="mt-2 text-xs text-gray-500">
          <kbd class="px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-700">Esc</kbd> to close
        </div>
      </div>
    </div>

    <!-- Context Menu -->
    <div
        v-if="contextMenu"
        :style="{ top: `${contextMenu.y}px`, left: `${contextMenu.x}px` }"
        class="fixed bg-white dark:bg-gray-800 border dark:border-gray-700 rounded shadow-lg z-50"
    >
      <div class="py-1">
        <div
            v-for="(item, i) in [
            { label: 'Open', action: () => openPreview(contextMenu!.file) },
            { label: 'Rename', action: () => {} },
            { label: 'Delete', action: () => {} },
            { label: 'Copy', action: () => {} },
            { label: 'Cut', action: () => {} },
          ]"
            :key="i"
            @click="item.action; closeContextMenu()"
            class="px-4 py-1.5 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer"
        >
          {{ item.label }}
        </div>
      </div>
    </div>
  </div>
</template>

<style>
/* Сбросим стили для pre */
pre {
  white-space: pre-wrap;
  word-wrap: break-word;
}
</style>
