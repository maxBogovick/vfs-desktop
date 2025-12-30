<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import GroupByDropdown from './GroupByDropdown.vue';
import type { Tab, ViewMode, PanelMode } from '../types';
import type { GroupBy } from '../composables/useGrouping';

interface Props {
  tabs: Tab[];
  activeTabId: number;
  currentPath: string[];
  viewMode: ViewMode;
  canGoBack: boolean;
  canGoForward: boolean;
  canGoUp: boolean;
  isCurrentPathBookmarked?: boolean;
  isProgrammerMode?: boolean;
  panelMode?: PanelMode;
  groupBy?: GroupBy;
  groupByOptions?: ReadonlyArray<{ value: string; label: string; icon: string }>;
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
  (e: 'toggleBookmark'): void;
  (e: 'toggleProgrammerMode'): void;
  (e: 'togglePanelMode'): void;
  (e: 'toggleDashboard'): void;
  (e: 'update:groupBy', value: GroupBy): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const isEditMode = ref(false);
const editPathValue = ref('');
const pathInputRef = ref<HTMLInputElement | null>(null);
const suggestions = ref<string[]>([]);
const selectedSuggestionIndex = ref(-1);
const showSuggestions = ref(false);
let suggestionTimeout: number | null = null;

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
  showSuggestions.value = false;
  suggestions.value = [];
  selectedSuggestionIndex.value = -1;

  // Очищаем таймер если есть
  if (suggestionTimeout !== null) {
    clearTimeout(suggestionTimeout);
    suggestionTimeout = null;
  }
};

const handlePathSubmit = () => {
  if (editPathValue.value.trim()) {
    emit('navigateToPath', editPathValue.value.trim());
  }
  exitEditMode();
};

// Получение подсказок для автодополнения
const fetchSuggestions = async (path: string) => {
  if (!path || path.length < 1) {
    suggestions.value = [];
    showSuggestions.value = false;
    return;
  }

  try {
    const result = await invoke<string[]>('get_path_suggestions', { partialPath: path });
    suggestions.value = result;
    showSuggestions.value = result.length > 0;
    selectedSuggestionIndex.value = -1;
  } catch (e) {
    suggestions.value = [];
    showSuggestions.value = false;
  }
};

// Обработка ввода с debounce
const handlePathInput = () => {
  // Очищаем предыдущий таймер
  if (suggestionTimeout !== null) {
    clearTimeout(suggestionTimeout);
  }

  // Запускаем новый таймер с задержкой 300мс
  suggestionTimeout = window.setTimeout(() => {
    fetchSuggestions(editPathValue.value);
  }, 300);
};

// Выбор подсказки
const selectSuggestion = (suggestion: string) => {
  editPathValue.value = suggestion;
  showSuggestions.value = false;
  suggestions.value = [];
  selectedSuggestionIndex.value = -1;
  pathInputRef.value?.focus();
};

const handlePathKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    if (showSuggestions.value) {
      showSuggestions.value = false;
      suggestions.value = [];
      selectedSuggestionIndex.value = -1;
    } else {
      exitEditMode();
    }
  } else if (event.key === 'Enter') {
    if (showSuggestions.value && selectedSuggestionIndex.value >= 0) {
      // Выбираем выделенную подсказку
      selectSuggestion(suggestions.value[selectedSuggestionIndex.value]);
      event.preventDefault();
    } else {
      handlePathSubmit();
    }
  } else if (event.key === 'ArrowDown') {
    if (showSuggestions.value && suggestions.value.length > 0) {
      event.preventDefault();
      selectedSuggestionIndex.value = Math.min(
        selectedSuggestionIndex.value + 1,
        suggestions.value.length - 1
      );
    }
  } else if (event.key === 'ArrowUp') {
    if (showSuggestions.value && suggestions.value.length > 0) {
      event.preventDefault();
      selectedSuggestionIndex.value = Math.max(selectedSuggestionIndex.value - 1, 0);
    }
  } else if (event.key === 'Tab') {
    if (showSuggestions.value && suggestions.value.length > 0) {
      event.preventDefault();
      // Автодополнение первой подсказкой
      const suggestionToUse = selectedSuggestionIndex.value >= 0
        ? suggestions.value[selectedSuggestionIndex.value]
        : suggestions.value[0];
      selectSuggestion(suggestionToUse);
    }
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

      <!-- Bookmark Button -->
      <button
        @click="emit('toggleBookmark')"
        :class="[
          'w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] flex items-center justify-center transition-all',
          isCurrentPathBookmarked && 'bg-[#FFE066] from-[#FFE066] to-[#FFD700]'
        ]"
        :title="isCurrentPathBookmarked ? 'Remove from Favorites (Ctrl+D)' : 'Add to Favorites (Ctrl+D)'"
      >
        {{ isCurrentPathBookmarked ? '⭐' : '☆' }}
      </button>

      <!-- Dashboard Button -->
      <button
        @click="emit('toggleDashboard')"
        class="w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] flex items-center justify-center transition-all"
        title="Folder Statistics Dashboard"
      >
        📊
      </button>

      <!-- Programmer Mode Button -->
      <button
        @click="emit('toggleProgrammerMode')"
        :class="[
          'w-[30px] h-[28px] bg-gradient-to-b border border-[#8B8B8B] hover:border-[#0054E3] flex items-center justify-center transition-all',
          isProgrammerMode
            ? 'bg-[#C1D2EE] from-[#C1D2EE] to-[#A8C0E8] border-[#0A246A]'
            : 'from-white to-[#E3DED4] active:bg-[#C1D2EE]'
        ]"
        :title="isProgrammerMode ? 'Programmer Mode: ON (Ctrl+Shift+P)' : 'Programmer Mode: OFF (Ctrl+Shift+P)'"
      >
        {{ isProgrammerMode ? '🔧' : '💻' }}
      </button>

      <!-- Group By Dropdown -->
      <GroupByDropdown
        v-if="groupBy && groupByOptions"
        :model-value="groupBy"
        :options="groupByOptions"
        @update:model-value="(value) => emit('update:groupBy', value)"
      />

      <div class="w-px h-[24px] bg-[#919B9C]"></div>

      <!-- Address Bar -->
      <div class="flex-1 max-w-3xl flex items-center gap-2">
        <!-- Address Label -->
        <span class="text-[11px] text-gray-600 font-bold">Address</span>

        <!-- Address Input/Breadcrumb -->
        <div class="flex-1 relative">
          <div
            class="bg-white border-2 border-[#91A7D0] rounded h-[26px] flex items-center overflow-hidden shadow-inner"
          >
            <!-- Edit Mode: Text Input -->
            <input
              v-if="isEditMode"
              ref="pathInputRef"
              v-model="editPathValue"
              @input="handlePathInput"
              @blur="() => setTimeout(handlePathSubmit, 200)"
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

          <!-- Autocomplete Suggestions Dropdown -->
          <div
            v-if="isEditMode && showSuggestions && suggestions.length > 0"
            class="absolute top-full left-0 right-0 mt-1 bg-white border-2 border-[#91A7D0] rounded shadow-lg max-h-[300px] overflow-y-auto z-50"
          >
            <div
              v-for="(suggestion, index) in suggestions"
              :key="suggestion"
              @mousedown.prevent="selectSuggestion(suggestion)"
              :class="[
                'px-3 py-2 text-[11px] cursor-pointer flex items-center gap-2',
                index === selectedSuggestionIndex
                  ? 'bg-[#0054E3] text-white'
                  : 'hover:bg-[#F0F8FF]'
              ]"
            >
              <span class="text-sm">📁</span>
              <span class="flex-1 truncate">{{ suggestion }}</span>
            </div>
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

      <!-- Separator -->
      <div class="w-px h-[24px] bg-[#919B9C] ml-1"></div>

      <!-- Dual Panel Toggle Button -->
      <button
        @click="emit('togglePanelMode')"
        :class="[
          'w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4]',
          'border border-[#8B8B8B] hover:border-[#0054E3]',
          'flex items-center justify-center transition-all',
          panelMode === 'dual' && 'bg-[#C1D2EE] from-[#C1D2EE] to-[#A8C0E8]'
        ]"
        :title="panelMode === 'dual' ? 'Single Panel' : 'Dual Panel'"
      >
        {{ panelMode === 'dual' ? '⊟' : '⊞⊞' }}
      </button>
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
