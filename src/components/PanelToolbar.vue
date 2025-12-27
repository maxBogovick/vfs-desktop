<script setup lang="ts">
import type { Tab } from '../types';

interface Props {
  tabs: Tab[];
  activeTabId?: number;
}

interface Emits {
  (e: 'switchTab', tabId: number): void;
  (e: 'closeTab', tabId: number): void;
  (e: 'addTab'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();
</script>

<template>
  <div class="flex items-center gap-1 px-2 py-1 bg-gradient-to-b from-[#F1EFE2] to-[#E3DED4] border-b border-[#919B9C] min-h-[32px] overflow-x-auto">
    <!-- Tabs -->
    <div
      v-for="tab in tabs"
      :key="tab.id"
      @click="emit('switchTab', tab.id)"
      :class="[
        'flex items-center gap-2 px-3 py-1 rounded-t border cursor-pointer',
        activeTabId === tab.id
          ? 'bg-white border-[#919B9C] border-b-white -mb-px'
          : 'bg-[#E3DED4] border-[#8B8B8B] hover:bg-[#ECE9D8]'
      ]"
    >
      <span class="text-[11px]">{{ tab.name }}</span>
      <svg
        v-if="tabs.length > 1"
        @click.stop="emit('closeTab', tab.id)"
        class="w-3 h-3 hover:bg-[#C1D2EE] rounded"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <line x1="18" y1="6" x2="6" y2="18"/>
        <line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
    </div>

    <!-- Add Tab Button -->
    <button
      @click="emit('addTab')"
      class="w-[24px] h-[24px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] flex items-center justify-center text-[14px] rounded"
      title="New Tab"
    >
      +
    </button>
  </div>
</template>
