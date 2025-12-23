<script setup lang="ts">
import type { FileItem } from '../types';

interface Props {
  x: number;
  y: number;
  item: FileItem | null;
}

interface Emits {
  (e: 'open'): void;
  (e: 'copy'): void;
  (e: 'cut'): void;
  (e: 'paste'): void;
  (e: 'rename'): void;
  (e: 'delete'): void;
  (e: 'properties'): void;
  (e: 'close'): void;
}

const props = defineProps<Props>();
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
      @click="handleAction('rename')"
      class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
    >
      <span class="w-4">✏️</span>
      <span class="flex-1">Rename</span>
      <span class="text-[9px] text-gray-400">F2</span>
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
