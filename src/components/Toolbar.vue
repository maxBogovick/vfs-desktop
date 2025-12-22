<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import type { Tab, ViewMode } from '../types';

interface Props {
  tabs: Tab[];
  activeTabId: number;
  currentPath: string[];
  viewMode: ViewMode;
  canGoBack: boolean;
  canGoForward: boolean;
  canGoUp: boolean;
}

interface Emits {
  (e: 'goBack'): void;
  (e: 'goForward'): void;
  (e: 'goUp'): void;
  (e: 'goHome'): void;
  (e: 'navigateToBreadcrumb', index: number): void;
  (e: 'navigateToPath', path: string): void;
  (e: 'switchTab', tabId: number): void;
  (e: 'closeTab', tabId: number): void;
  (e: 'addTab'): void;
  (e: 'update:viewMode', mode: ViewMode): void;
  (e: 'openCommandPalette'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const isEditMode = ref(false);
const editPathValue = ref('');
const pathInputRef = ref<HTMLInputElement | null>(null);

const fullPath = computed(() => {
  const path = props.currentPath.join('/');
  return path && !path.startsWith('/') ? '/' + path : path;
});

const enterEditMode = () => {
  isEditMode.value = true;
  editPathValue.value = fullPath.value || '/';
  nextTick(() => {
    pathInputRef.value?.focus();
    pathInputRef.value?.select();
  });
};

const exitEditMode = () => {
  isEditMode.value = false;
};

const handlePathSubmit = () => {
  if (editPathValue.value.trim()) {
    emit('navigateToPath', editPathValue.value.trim());
  }
  exitEditMode();
};

const handlePathKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    exitEditMode();
  } else if (event.key === 'Enter') {
    handlePathSubmit();
  }
};

// Exit edit mode when path changes externally
watch(fullPath, () => {
  if (isEditMode.value) {
    exitEditMode();
  }
});
</script>

<template>
  <div class="bg-gradient-to-b from-[#F1EFE2] to-[#E3DED4] border-b border-[#919B9C]">
    <!-- Main Toolbar -->
    <div class="flex items-center h-[38px] px-1 gap-1">
      <!-- Navigation Buttons -->
      <button
        @click="emit('goBack')"
        :disabled="!canGoBack"
        :class="['w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] flex items-center justify-center', !canGoBack && 'opacity-50 cursor-not-allowed']"
        title="Back"
      >
        ←
      </button>
      <button
        @click="emit('goForward')"
        :disabled="!canGoForward"
        :class="['w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] flex items-center justify-center', !canGoForward && 'opacity-50 cursor-not-allowed']"
        title="Forward"
      >
        →
      </button>
      <button
        @click="emit('goUp')"
        :disabled="!canGoUp"
        :class="['w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] flex items-center justify-center ml-1', !canGoUp && 'opacity-50 cursor-not-allowed']"
        title="Up"
      >
        ↑
      </button>
      <button
        @click="emit('goHome')"
        class="w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] flex items-center justify-center"
        title="Home"
      >
        🏠
      </button>

      <div class="w-px h-[24px] bg-[#919B9C]"></div>

      <!-- Address Bar -->
      <div class="flex-1 max-w-3xl flex items-center gap-2">
        <!-- Address Label -->
        <span class="text-[11px] text-gray-600 font-bold">Address</span>

        <!-- Address Input/Breadcrumb -->
        <div
          class="flex-1 bg-white border-2 border-[#91A7D0] rounded h-[26px] flex items-center overflow-hidden shadow-inner"
        >
          <!-- Edit Mode: Text Input -->
          <input
            v-if="isEditMode"
            ref="pathInputRef"
            v-model="editPathValue"
            @blur="handlePathSubmit"
            @keydown="handlePathKeydown"
            type="text"
            class="flex-1 px-2 text-[11px] outline-none bg-white"
            placeholder="Enter path..."
          />

          <!-- View Mode: Breadcrumb -->
          <div
            v-else
            @click="enterEditMode"
            class="flex-1 flex items-center px-2 cursor-text hover:bg-[#F0F8FF] transition-colors"
          >
            <!-- Folder Icon -->
            <span class="text-sm mr-1.5">📁</span>

            <!-- Breadcrumb Path -->
            <div class="flex items-center text-[11px] gap-0.5 flex-1 overflow-hidden">
              <template v-if="currentPath.length === 0">
                <span class="text-gray-600 font-medium">Home</span>
              </template>
              <template v-else>
                <template v-for="(segment, i) in currentPath" :key="i">
                  <button
                    @click.stop="emit('navigateToBreadcrumb', i)"
                    class="hover:bg-[#C1D2EE] hover:border hover:border-[#0A246A] px-1.5 py-0.5 rounded whitespace-nowrap transition-colors font-medium"
                  >
                    {{ segment }}
                  </button>
                  <span
                    v-if="i < currentPath.length - 1"
                    class="text-gray-400 mx-0.5"
                  >
                    ▸
                  </span>
                </template>
              </template>
            </div>

            <!-- Refresh Button -->
            <button
              @click.stop="emit('navigateToPath', fullPath)"
              class="ml-1 p-1 hover:bg-[#C1D2EE] rounded text-[10px] opacity-0 group-hover:opacity-100 transition-opacity"
              title="Refresh"
            >
              🔄
            </button>
          </div>
        </div>

        <!-- Go Button -->
        <button
          @click="emit('openCommandPalette')"
          class="px-3 h-[26px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] rounded text-[11px] font-bold"
          title="Search (Ctrl+K)"
        >
          🔍
        </button>
      </div>

      <!-- View Mode Buttons -->
      <div class="flex gap-0.5">
        <button
          @click="emit('update:viewMode', 'list')"
          :class="['w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] flex items-center justify-center', viewMode === 'list' ? 'bg-[#C1D2EE]' : '']"
          title="List View"
        >
          ☰
        </button>
        <button
          @click="emit('update:viewMode', 'grid')"
          :class="['w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] flex items-center justify-center', viewMode === 'grid' ? 'bg-[#C1D2EE]' : '']"
          title="Grid View"
        >
          ⊞
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex items-center gap-1 px-2 pb-1 overflow-x-auto">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        @click="emit('switchTab', tab.id)"
        :class="['flex items-center gap-2 px-3 py-1 rounded-t border cursor-pointer', activeTabId === tab.id ? 'bg-white border-[#919B9C] border-b-white -mb-px' : 'bg-[#E3DED4] border-[#8B8B8B] hover:bg-[#ECE9D8]']"
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
      <button
        @click="emit('addTab')"
        class="px-2 py-1 text-[11px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] rounded"
        title="New Tab"
      >
        +
      </button>
    </div>
  </div>
</template>
