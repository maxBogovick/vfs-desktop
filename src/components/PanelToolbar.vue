<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Tab } from '../types';

interface Props {
  tabs: Tab[];
  activeTabId?: number;
  currentPath?: string[];
  sortBy?: 'name' | 'size' | 'modified' | 'type';
  sortOrder?: 'asc' | 'desc';
  showHidden?: boolean;
}

interface Emits {
  (e: 'switchTab', tabId: number): void;
  (e: 'closeTab', tabId: number): void;
  (e: 'addTab'): void;
  (e: 'sort', field: 'name' | 'size' | 'modified' | 'type', order: 'asc' | 'desc'): void;
  (e: 'selectAll'): void;
  (e: 'invertSelection'): void;
  (e: 'refresh'): void;
  (e: 'toggleHidden'): void;
  (e: 'navigateToBreadcrumb', index: number): void;
}

const props = withDefaults(defineProps<Props>(), {
  sortBy: 'name',
  sortOrder: 'asc',
  showHidden: false,
  currentPath: () => [],
});

const emit = defineEmits<Emits>();

const showSortMenu = ref(false);

const handleSortChange = (field: 'name' | 'size' | 'modified' | 'type') => {
  // Toggle order if clicking same field, otherwise default to asc
  const newOrder = props.sortBy === field && props.sortOrder === 'asc' ? 'desc' : 'asc';
  emit('sort', field, newOrder);
  showSortMenu.value = false;
};

const getSortLabel = () => {
  const fieldLabels = {
    name: 'Name',
    size: 'Size',
    modified: 'Modified',
    type: 'Type',
  };
  const orderIcon = props.sortOrder === 'asc' ? '↑' : '↓';
  return `${fieldLabels[props.sortBy]} ${orderIcon}`;
};

const fullPath = computed(() => {
  const path = props.currentPath.join('/');
  return path && !path.startsWith('/') ? '/' + path : path;
});
</script>

<template>
  <div class="flex flex-col">
    <!-- Tabs Row -->
    <div class="flex items-center gap-1 px-2 py-1 bg-gradient-to-b from-[var(--vf-bg-secondary)] to-[var(--vf-bg-tertiary)] border-b border-[var(--vf-border-default)] min-h-[32px] overflow-x-auto">
      <!-- Tabs -->
      <div
        v-for="tab in tabs"
        :key="tab.id"
        @click="emit('switchTab', tab.id)"
        :class="[
          'flex items-center gap-2 px-3 py-1 rounded-t border cursor-pointer',
          activeTabId === tab.id
            ? 'bg-[var(--vf-surface-default)] border-[var(--vf-border-default)] border-b-[var(--vf-surface-default)] -mb-px'
            : 'bg-[var(--vf-bg-tertiary)] border-[var(--vf-border-subtle)] hover:bg-[var(--vf-bg-primary)]'
        ]"
      >
        <span class="text-[11px]">{{ tab.name }}</span>
        <svg
          v-if="tabs.length > 1"
          @click.stop="emit('closeTab', tab.id)"
          class="w-3 h-3 hover:bg-[var(--vf-surface-hover)] rounded"
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
        class="w-[24px] h-[24px] bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] flex items-center justify-center text-[14px] rounded"
        title="New Tab"
      >
        +
      </button>
    </div>

    <!-- Action Buttons Row -->
    <div class="flex items-center gap-1 px-2 py-1 bg-[var(--vf-bg-primary)] border-b border-[var(--vf-border-subtle)]">
      <!-- Sort Button with Dropdown -->
      <div class="relative">
        <button
          @click="showSortMenu = !showSortMenu"
          class="px-2 py-1 bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] rounded text-[10px] font-['Tahoma'] flex items-center gap-1"
          title="Sort by"
        >
          <span>📊</span>
          <span>{{ getSortLabel() }}</span>
          <span class="text-[8px]">▼</span>
        </button>

        <!-- Sort Dropdown Menu -->
        <div
          v-if="showSortMenu"
          @click.stop
          class="absolute top-full left-0 mt-1 bg-[var(--vf-surface-default)] border-2 border-[var(--vf-border-default)] shadow-lg z-50 min-w-[120px]"
        >
          <button
            @click="handleSortChange('name')"
            :class="[
              'w-full text-left px-3 py-1.5 text-[10px] hover:bg-[var(--vf-surface-hover)] flex items-center justify-between',
              sortBy === 'name' && 'bg-[var(--vf-surface-selected)] font-bold'
            ]"
          >
            <span>Name</span>
            <span v-if="sortBy === 'name'" class="text-[8px]">{{ sortOrder === 'asc' ? '↑' : '↓' }}</span>
          </button>
          <button
            @click="handleSortChange('size')"
            :class="[
              'w-full text-left px-3 py-1.5 text-[10px] hover:bg-[var(--vf-surface-hover)] flex items-center justify-between',
              sortBy === 'size' && 'bg-[var(--vf-surface-selected)] font-bold'
            ]"
          >
            <span>Size</span>
            <span v-if="sortBy === 'size'" class="text-[8px]">{{ sortOrder === 'asc' ? '↑' : '↓' }}</span>
          </button>
          <button
            @click="handleSortChange('modified')"
            :class="[
              'w-full text-left px-3 py-1.5 text-[10px] hover:bg-[var(--vf-surface-hover)] flex items-center justify-between',
              sortBy === 'modified' && 'bg-[var(--vf-surface-selected)] font-bold'
            ]"
          >
            <span>Modified</span>
            <span v-if="sortBy === 'modified'" class="text-[8px]">{{ sortOrder === 'asc' ? '↑' : '↓' }}</span>
          </button>
          <button
            @click="handleSortChange('type')"
            :class="[
              'w-full text-left px-3 py-1.5 text-[10px] hover:bg-[var(--vf-surface-hover)] flex items-center justify-between',
              sortBy === 'type' && 'bg-[var(--vf-surface-selected)] font-bold'
            ]"
          >
            <span>Type</span>
            <span v-if="sortBy === 'type'" class="text-[8px]">{{ sortOrder === 'asc' ? '↑' : '↓' }}</span>
          </button>
        </div>
      </div>

      <div class="w-px h-4 bg-[var(--vf-border-subtle)]"></div>

      <!-- Select All Button -->
      <button
        @click="emit('selectAll')"
        class="px-2 py-1 bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] rounded text-[10px] font-['Tahoma']"
        title="Select All (Ctrl+A)"
      >
        ☑️ Select All
      </button>

      <!-- Invert Selection Button -->
      <button
        @click="emit('invertSelection')"
        class="px-2 py-1 bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] rounded text-[10px] font-['Tahoma']"
        title="Invert Selection"
      >
        🔄 Invert
      </button>

      <div class="w-px h-4 bg-[var(--vf-border-subtle)]"></div>

      <!-- Refresh Button -->
      <button
        @click="emit('refresh')"
        class="px-2 py-1 bg-gradient-to-b from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] active:bg-[var(--vf-surface-hover)] rounded text-[10px] font-['Tahoma']"
        title="Refresh (F5)"
      >
        🔄 Refresh
      </button>

      <!-- Show Hidden Toggle -->
      <button
        @click="emit('toggleHidden')"
        :class="[
          'px-2 py-1 bg-gradient-to-b border rounded text-[10px] font-[\'Tahoma\']',
          showHidden
            ? 'from-[var(--vf-accent-primary)] to-[var(--vf-accent-hover)] border-[var(--vf-accent-active)] text-white'
            : 'from-[var(--vf-surface-default)] to-[var(--vf-bg-tertiary)] border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)]'
        ]"
        title="Show Hidden Files"
      >
        👁️ {{ showHidden ? 'Hide' : 'Show' }} Hidden
      </button>
    </div>

    <!-- Address Bar Row -->
    <div class="flex items-center gap-2 px-2 py-1 bg-[var(--vf-bg-primary)] border-b border-[var(--vf-border-subtle)]">
      <!-- Address Label -->
      <span class="text-[10px] text-gray-600 font-bold">Address</span>

      <!-- Breadcrumb Path -->
      <div class="flex-1 bg-white border border-[var(--vf-border-accent)] rounded h-[22px] flex items-center px-2 overflow-hidden shadow-inner">
        <!-- Folder Icon -->
        <span class="text-xs mr-1">📁</span>

        <!-- Breadcrumb -->
        <div class="flex items-center text-[10px] gap-0.5 flex-1 overflow-hidden">
          <template v-if="currentPath.length === 0">
            <span class="text-gray-600 font-medium">Home</span>
          </template>
          <template v-else>
            <template v-for="(segment, i) in currentPath" :key="i">
              <button
                @click="emit('navigateToBreadcrumb', i)"
                class="hover:bg-[var(--vf-surface-hover)] hover:border hover:border-[var(--vf-accent-hover)] px-1 py-0.5 rounded whitespace-nowrap transition-colors font-medium"
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
          @click="emit('refresh')"
          class="ml-1 p-0.5 hover:bg-[var(--vf-surface-hover)] rounded text-[8px] transition-opacity"
          title="Refresh"
        >
          🔄
        </button>
      </div>
    </div>
  </div>
</template>
