<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import FilePanel from './FilePanel.vue';
import { useDualPanel } from '../composables/useDualPanel';
import type { ViewMode } from '../types';

interface Props {
  viewMode?: ViewMode;
}

const props = withDefaults(defineProps<Props>(), {
  viewMode: 'grid',
});

const {
  leftPanelWidthPercent,
  activePanel,
  leftPanelTabs,
  leftPanelActiveTabId,
  rightPanelTabs,
  rightPanelActiveTabId,
  switchActivePanel,
  setPanelSplit,
} = useDualPanel();

// Resizer state
const isResizing = ref(false);
const containerRef = ref<HTMLElement | null>(null);

const startResize = (event: MouseEvent) => {
  isResizing.value = true;
  event.preventDefault();
};

const stopResize = () => {
  isResizing.value = false;
};

const handleResize = (event: MouseEvent) => {
  if (!isResizing.value || !containerRef.value) return;

  const containerRect = containerRef.value.getBoundingClientRect();
  const offsetX = event.clientX - containerRect.left;
  const percent = (offsetX / containerRect.width) * 100;

  // Ограничение 20-80%
  setPanelSplit(percent);
};

onMounted(() => {
  document.addEventListener('mousemove', handleResize);
  document.addEventListener('mouseup', stopResize);
});

onUnmounted(() => {
  document.removeEventListener('mousemove', handleResize);
  document.removeEventListener('mouseup', stopResize);
});

// Handle panel activation
const handleActivateLeft = () => {
  switchActivePanel('left');
};

const handleActivateRight = () => {
  switchActivePanel('right');
};
</script>

<template>
  <div
    ref="containerRef"
    class="flex-1 flex overflow-hidden"
    :class="{ 'cursor-col-resize': isResizing }"
  >
    <!-- Left Panel -->
    <div
      :style="{ width: `${leftPanelWidthPercent}%` }"
      class="flex flex-col overflow-hidden"
    >
      <FilePanel
        panel-id="left"
        :is-active="activePanel === 'left'"
        :tabs="leftPanelTabs"
        :active-tab-id="leftPanelActiveTabId"
        :view-mode="viewMode"
        @activate="handleActivateLeft"
        @update:tabs="(tabs) => leftPanelTabs = tabs"
        @update:active-tab-id="(id) => leftPanelActiveTabId = id"
      />
    </div>

    <!-- Resizer -->
    <div
      @mousedown="startResize"
      class="w-[4px] bg-[#919B9C] hover:bg-blue-500 cursor-col-resize flex-shrink-0 transition-colors"
      :class="{ 'bg-blue-500': isResizing }"
    />

    <!-- Right Panel -->
    <div
      :style="{ width: `${100 - leftPanelWidthPercent}%` }"
      class="flex flex-col overflow-hidden"
    >
      <FilePanel
        panel-id="right"
        :is-active="activePanel === 'right'"
        :tabs="rightPanelTabs"
        :active-tab-id="rightPanelActiveTabId"
        :view-mode="viewMode"
        @activate="handleActivateRight"
        @update:tabs="(tabs) => rightPanelTabs = tabs"
        @update:active-tab-id="(id) => rightPanelActiveTabId = id"
      />
    </div>
  </div>
</template>
