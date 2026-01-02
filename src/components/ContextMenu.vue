<script setup lang="ts">
import type { FileItem } from '../types';

interface Props {
  x: number;
  y: number;
  item: FileItem | null;
  selectedCount?: number;
}

interface Emits {
  (e: 'open'): void;
  (e: 'edit'): void;
  (e: 'copy'): void;
  (e: 'cut'): void;
  (e: 'paste'): void;
  (e: 'rename'): void;
  (e: 'delete'): void;
  (e: 'addToFavorites'): void;
  (e: 'openTerminal'): void;
  (e: 'properties'): void;
  (e: 'batchRename'): void;
  (e: 'batchAttributes'): void;
  (e: 'close'): void;
}

const props = withDefaults(defineProps<Props>(), {
  selectedCount: 0,
});
const emit = defineEmits<Emits>();

const handleAction = (action: keyof Emits) => {
  emit(action);
  emit('close');
};
</script>

<template>
  <div
    :style="{ top: `${y}px`, left: `${x}px` }"
    class="fixed bg-white border border-[#919B9C] shadow-lg rounded text-[11px] py-1 z-50 min-w-[180px]"
  >
    <!-- Open -->
    <div
      @click="handleAction('open')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">📂</span>
      <span class="flex-1">Open</span>
      <span class="text-[9px] text-gray-400">Enter</span>
    </div>

    <!-- Edit (only for text/code files) -->
    <div
      v-if="item && (item.type === 'file' || item.type === 'code') && selectedCount <= 1"
      @click="handleAction('edit')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">✏️</span>
      <span class="flex-1">Edit</span>
      <span class="text-[9px] text-gray-400">F4</span>
    </div>

    <div class="border-t border-[#D0D0BF] my-1"></div>

    <!-- Copy -->
    <div
      @click="handleAction('copy')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">📋</span>
      <span class="flex-1">Copy</span>
      <span class="text-[9px] text-gray-400">Ctrl+C</span>
    </div>

    <!-- Cut -->
    <div
      @click="handleAction('cut')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">✂️</span>
      <span class="flex-1">Cut</span>
      <span class="text-[9px] text-gray-400">Ctrl+X</span>
    </div>

    <!-- Paste -->
    <div
      @click="handleAction('paste')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">📄</span>
      <span class="flex-1">Paste</span>
      <span class="text-[9px] text-gray-400">Ctrl+V</span>
    </div>

    <div class="border-t border-[#D0D0BF] my-1"></div>

    <!-- Rename -->
    <div
      v-if="selectedCount <= 1"
      @click="handleAction('rename')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">✏️</span>
      <span class="flex-1">Rename</span>
      <span class="text-[9px] text-gray-400">F2</span>
    </div>

    <!-- Batch Operations (when multiple items selected) -->
    <div v-if="selectedCount > 1" class="border-t border-[#D0D0BF] my-1"></div>

    <div
      v-if="selectedCount > 1"
      @click="handleAction('batchRename')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">📝</span>
      <span class="flex-1">Batch Rename</span>
      <span class="text-[9px] text-gray-400">{{ selectedCount }} items</span>
    </div>

    <div
      v-if="selectedCount > 1"
      @click="handleAction('batchAttributes')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">🔧</span>
      <span class="flex-1">Batch Attributes</span>
      <span class="text-[9px] text-gray-400">{{ selectedCount }} items</span>
    </div>

    <!-- Delete -->
    <div
      @click="handleAction('delete')"
      class="px-3 py-1.5 hover:bg-red-50 hover:text-red-600 cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">🗑️</span>
      <span class="flex-1">Delete</span>
      <span class="text-[9px] text-gray-400">Del</span>
    </div>

    <div class="border-t border-[#D0D0BF] my-1"></div>

    <!-- Add to Favorites (only for folders) -->
    <div
      v-if="item && item.type === 'folder'"
      @click="handleAction('addToFavorites')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">⭐</span>
      <span class="flex-1">Add to Favorites</span>
      <span class="text-[9px] text-gray-400">Ctrl+D</span>
    </div>

    <!-- Open in Terminal -->
    <div
      @click="handleAction('openTerminal')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">💻</span>
      <span class="flex-1">Open in Terminal</span>
    </div>

    <!-- Properties -->
    <div
      @click="handleAction('properties')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">ℹ️</span>
      <span class="flex-1">Properties</span>
      <span class="text-[9px] text-gray-400">Alt+Enter</span>
    </div>

    <!-- File Info Footer -->
    <div v-if="item" class="border-t border-[#D0D0BF] mt-1 px-3 py-2 bg-[#F5F5F5]">
      <div class="text-[9px] text-gray-500 mb-0.5">{{ item.name }}</div>
      <div class="text-[9px] text-gray-400">{{ item.type }} • {{ item.sizeFormatted }}</div>
    </div>
  </div>
</template>
