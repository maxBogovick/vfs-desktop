<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useVault } from '../composables/useVault';
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
  queueActiveCount?: number;
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
  (e: 'update:view-mode', mode: ViewMode): void;
  (e: 'openCommandPalette'): void;
  (e: 'toggleBookmark'): void;
  (e: 'toggleProgrammerMode'): void;
  (e: 'togglePanelMode'): void;
  (e: 'toggleDashboard'): void;
  (e: 'toggleOperationsQueue'): void;
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

// File system type and vault status
const fsType = ref<'real' | 'virtual'>('real');
const vaultStatus = ref<'UNLOCKED' | 'LOCKED' | 'UNINITIALIZED' | 'DISABLED'>('DISABLED');
const vaultActionInProgress = ref(false);
const vault = useVault();

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

// Vault management functions
const lockVault = async () => {
  try {
    vaultActionInProgress.value = true;
    await vault.lock();
    // Reload page to ensure all state is cleared and vault overlay shows
    setTimeout(() => {
      window.location.reload();
    }, 300);
  } catch (error) {
    console.error('[Toolbar] Failed to lock vault:', error);
    vaultActionInProgress.value = false;
  }
};

const unlockVault = () => {
  // This will trigger the vault overlay to show
  vault.forceLock();
};

// Load file system type and vault status
const loadFsInfo = async () => {
  try {
    const config = await invoke<any>('get_config');

    // Backend can be 'real', 'Real', 'virtual', or 'Virtual'
    const backend = config.filesystem_backend?.toLowerCase() || 'real';
    fsType.value = backend === 'virtual' ? 'virtual' : 'real';

    if (fsType.value === 'virtual') {
      const status = await invoke<string>('vault_get_status');
      vaultStatus.value = status as any;
    } else {
      vaultStatus.value = 'DISABLED';
    }
  } catch (error) {
    console.error('[Toolbar] Failed to load FS info:', error);
  }
};

onMounted(() => {
  loadFsInfo();
  // Refresh every 2 seconds to keep status updated
  setInterval(loadFsInfo, 2000);

  // Listen for config changes from Settings
  window.addEventListener('fs-config-changed', loadFsInfo);
});

// Cleanup on unmount
onUnmounted(() => {
  window.removeEventListener('fs-config-changed', loadFsInfo);
});
</script>

<template>
  <div class="bg-gradient-to-b from-[var(--vf-bg-secondary)] to-[var(--vf-bg-tertiary)] border-b border-[var(--vf-border-default)]">
    <!-- Main Toolbar -->
    <div class="flex items-center h-[38px] px-1 gap-1">
      <!-- Navigation Buttons -->
      <button
        @click="emit('goBack')"
        :disabled="!canGoBack"
        :class="['w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] flex items-center justify-center', !canGoBack && 'opacity-50 cursor-not-allowed']"
        title="Back"
      >
        ←
      </button>
      <button
        @click="emit('goForward')"
        :disabled="!canGoForward"
        :class="['w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] flex items-center justify-center', !canGoForward && 'opacity-50 cursor-not-allowed']"
        title="Forward"
      >
        →
      </button>
      <button
        @click="emit('goUp')"
        :disabled="!canGoUp"
        :class="['w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] flex items-center justify-center ml-1', !canGoUp && 'opacity-50 cursor-not-allowed']"
        title="Up"
      >
        ↑
      </button>
      <button
        @click="emit('goHome')"
        class="w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] flex items-center justify-center"
        title="Home"
      >
        🏠
      </button>

      <div class="w-px h-[24px] bg-[var(--vf-border-default)]"></div>

      <!-- Bookmark Button -->
      <button
        @click="emit('toggleBookmark')"
        :class="[
          'w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] flex items-center justify-center transition-all',
          isCurrentPathBookmarked && 'bg-[#FFE066] from-[#FFE066] to-[#FFD700]'
        ]"
        :title="isCurrentPathBookmarked ? 'Remove from Favorites (Ctrl+D)' : 'Add to Favorites (Ctrl+D)'"
      >
        {{ isCurrentPathBookmarked ? '⭐' : '☆' }}
      </button>

      <!-- Dashboard Button -->
      <button
        @click="emit('toggleDashboard')"
        class="w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] flex items-center justify-center transition-all"
        title="Folder Statistics Dashboard"
      >
        📊
      </button>

      <!-- Operations Queue Button -->
      <button
        @click="emit('toggleOperationsQueue')"
        class="w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] flex items-center justify-center transition-all relative"
        :title="`Operations Queue${queueActiveCount ? ` (${queueActiveCount} active)` : ''}`"
      >
        📋
        <!-- Badge for active operations -->
        <span
          v-if="queueActiveCount && queueActiveCount > 0"
          class="absolute -top-1 -right-1 bg-blue-500 text-white text-[9px] font-bold rounded-full min-w-[14px] h-[14px] flex items-center justify-center px-0.5"
        >
          {{ queueActiveCount > 99 ? '99+' : queueActiveCount }}
        </span>
      </button>

      <!-- Programmer Mode Button -->
      <button
        @click="emit('toggleProgrammerMode')"
        :class="[
          'w-[30px] h-[28px] bg-gradient-to-b border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] flex items-center justify-center transition-all',
          isProgrammerMode
            ? 'bg-[var(--vf-surface-hover)] from-[var(--vf-surface-hover)] to-[var(--vf-surface-selected)] border-[var(--vf-accent-hover)]'
            : 'from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] active:bg-[var(--vf-surface-hover)]'
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

      <div class="w-px h-[24px] bg-[var(--vf-border-default)]"></div>

      <!-- Address Bar (hidden in dual panel mode) -->
      <div v-if="panelMode !== 'dual'" class="flex-1 max-w-3xl flex items-center gap-2">
        <!-- Address Label -->
        <span class="text-[11px] text-gray-600 font-bold">Address</span>

        <!-- Address Input/Breadcrumb -->
        <div class="flex-1 relative">
          <div
            class="bg-white border-2 border-[var(--vf-border-accent)] rounded h-[26px] flex items-center overflow-hidden shadow-inner"
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
            class="flex-1 flex items-center px-2 cursor-text hover:bg-[var(--vf-surface-hover)] transition-colors"
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
                    class="hover:bg-[var(--vf-surface-hover)] hover:border hover:border-[var(--vf-accent-hover)] px-1.5 py-0.5 rounded whitespace-nowrap transition-colors font-medium"
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
              class="ml-1 p-1 hover:bg-[var(--vf-surface-hover)] rounded text-[10px] opacity-0 group-hover:opacity-100 transition-opacity"
              title="Refresh"
            >
              🔄
            </button>
          </div>
        </div>

          <!-- Autocomplete Suggestions Dropdown -->
          <div
            v-if="isEditMode && showSuggestions && suggestions.length > 0"
            class="absolute top-full left-0 right-0 mt-1 bg-white border-2 border-[var(--vf-border-accent)] rounded shadow-lg max-h-[300px] overflow-y-auto z-50"
          >
            <div
              v-for="(suggestion, index) in suggestions"
              :key="suggestion"
              @mousedown.prevent="selectSuggestion(suggestion)"
              :class="[
                'px-3 py-2 text-[11px] cursor-pointer flex items-center gap-2',
                index === selectedSuggestionIndex
                  ? 'bg-[var(--vf-accent-primary)] text-white'
                  : 'hover:bg-[var(--vf-surface-hover)]'
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
          class="px-3 h-[26px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] rounded text-[11px] font-bold"
          title="Search (Ctrl+K)"
        >
          🔍
        </button>
      </div>

      <!-- View Mode Buttons -->
      <div class="flex gap-0.5">
        <button
          @click="emit('update:view-mode', 'list')"
          :class="['w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] flex items-center justify-center', viewMode === 'list' ? 'bg-[var(--vf-surface-hover)]' : '']"
          title="List View"
        >
          ☰
        </button>
        <button
          @click="emit('update:view-mode', 'grid')"
          :class="['w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] flex items-center justify-center', viewMode === 'grid' ? 'bg-[var(--vf-surface-hover)]' : '']"
          title="Grid View"
        >
          ⊞
        </button>
      </div>

      <!-- Separator -->
      <div class="w-px h-[24px] bg-[var(--vf-border-default)] ml-1"></div>

      <!-- Dual Panel Toggle Button -->
      <button
        @click="emit('togglePanelMode')"
        :class="[
          'w-[30px] h-[28px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)]',
          'border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)]',
          'flex items-center justify-center transition-all',
          panelMode === 'dual' && 'bg-[var(--vf-surface-hover)] from-[var(--vf-surface-hover)] to-[var(--vf-surface-selected)]'
        ]"
        :title="panelMode === 'dual' ? 'Single Panel' : 'Dual Panel'"
      >
        {{ panelMode === 'dual' ? '⊟' : '⊞⊞' }}
      </button>

      <!-- Separator -->
      <div class="w-px h-[24px] bg-[var(--vf-border-default)] ml-1"></div>

      <!-- File System Type & Vault Status Badge -->
      <div class="flex items-center gap-1.5 px-2">
        <!-- FS Type Badge -->
        <div :class="[
          'px-2 py-1 rounded text-[10px] font-semibold uppercase tracking-wide',
          fsType === 'virtual'
            ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400'
            : 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'
        ]" :title="fsType === 'virtual' ? 'Virtual File System' : 'Real File System'">
          {{ fsType === 'virtual' ? '💾 Virtual' : '📁 Real' }}
        </div>

        <!-- Vault Status Badge (only for Virtual FS) -->
        <div v-if="fsType === 'virtual' && vaultStatus !== 'DISABLED'" :class="[
          'px-2 py-1 rounded text-[10px] font-semibold uppercase tracking-wide flex items-center gap-1',
          vaultStatus === 'UNLOCKED' && 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
          vaultStatus === 'LOCKED' && 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400',
          vaultStatus === 'UNINITIALIZED' && 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
        ]" :title="`Vault ${vaultStatus.toLowerCase()}`">
          <span>{{ vaultStatus === 'UNLOCKED' ? '🔓' : vaultStatus === 'LOCKED' ? '🔒' : '🔑' }}</span>
          <span>{{ vaultStatus === 'UNLOCKED' ? 'Unlocked' : vaultStatus === 'LOCKED' ? 'Locked' : 'Setup' }}</span>
        </div>

        <!-- Vault Action Buttons (only for Virtual FS and when initialized) -->
        <div v-if="fsType === 'virtual' && vaultStatus !== 'DISABLED' && vaultStatus !== 'UNINITIALIZED'" class="flex gap-0.5">
          <!-- Lock Button (when unlocked) -->
          <button
            v-if="vaultStatus === 'UNLOCKED'"
            @click="lockVault"
            :disabled="vaultActionInProgress"
            class="w-[24px] h-[24px] bg-orange-500 hover:bg-orange-600 text-white rounded flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-xs"
            title="Lock Vault"
          >
            🔒
          </button>

          <!-- Unlock Button (when locked) -->
          <button
            v-if="vaultStatus === 'LOCKED'"
            @click="unlockVault"
            :disabled="vaultActionInProgress"
            class="w-[24px] h-[24px] bg-green-500 hover:bg-green-600 text-white rounded flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-xs"
            title="Unlock Vault"
          >
            🔓
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
