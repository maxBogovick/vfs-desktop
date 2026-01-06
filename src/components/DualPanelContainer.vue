<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import FilePanel from './FilePanel.vue';
import { useDualPanel } from '../composables/useDualPanel';
import { useProgrammerMode } from '../composables/useProgrammerMode';
import { useVault } from '../composables/useVault';
import { useDialogs } from '../composables/useDialogs';
import type { ViewMode, FileSystemBackend, FileItem } from '../types';

interface Props {
  viewMode?: ViewMode;
}

interface Emits {
  (e: 'editFile', item: FileItem, panelFs?: string): void;
  (e: 'previewFile', item: FileItem, panelFs?: string): void;
}

const props = withDefaults(defineProps<Props>(), {
  viewMode: 'grid',
});

const emit = defineEmits<Emits>();

const {
  leftPanelWidthPercent,
  activePanel,
  leftPanelTabs,
  leftPanelActiveTabId,
  leftPanelFilesystem,
  rightPanelTabs,
  rightPanelActiveTabId,
  rightPanelFilesystem,
  switchActivePanel,
  switchPanelFilesystem,
  setPanelSplit,
} = useDualPanel();

const { isProgrammerMode } = useProgrammerMode();
const vault = useVault();
const { showConfirm } = useDialogs();

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

// Handle filesystem switching
const handleSwitchLeftFilesystem = async (backend: FileSystemBackend) => {
  // 1. Проверка programmer mode
  if (!isProgrammerMode.value) {
    console.warn('FS switching is only available in programmer mode');
    return;
  }

  // 2. Проверка vault unlock для Virtual FS
  if (backend === 'virtual' && !vault.isUnlocked.value) {
    vault.openVault();
    
    const unlocked = await new Promise<boolean>((resolve) => {
       const stop = watch([() => vault.isUnlocked.value, () => vault.isVaultOverlayVisible.value], 
       ([unlocked, visible]) => {
          if (unlocked) {
            stop();
            resolve(true);
          } else if (!visible) {
             stop();
             resolve(false);
          }
       });
    });
    
    if (!unlocked) return;
  }

  // 3. Подтверждение пользователя
  showConfirm(
    'Switch filesystem?',
    `All tabs will be closed. Switch to ${backend === 'virtual' ? 'Virtual' : 'Real'} filesystem?`,
    () => {
      // 4. Закрыть все табы и создать новый home таб
      leftPanelTabs.value = [{
        id: Date.now(),
        path: [''],
        name: 'Home',
        history: [['']],
        historyIndex: 0,
      }];
      leftPanelActiveTabId.value = leftPanelTabs.value[0].id;

      // 5. Переключить FS
      switchPanelFilesystem('left', backend);
    }
  );
};

const handleSwitchRightFilesystem = async (backend: FileSystemBackend) => {
  // 1. Проверка programmer mode
  if (!isProgrammerMode.value) {
    console.warn('FS switching is only available in programmer mode');
    return;
  }

  // 2. Проверка vault unlock для Virtual FS
  if (backend === 'virtual' && !vault.isUnlocked.value) {
    vault.openVault();
    
    const unlocked = await new Promise<boolean>((resolve) => {
       const stop = watch([() => vault.isUnlocked.value, () => vault.isVaultOverlayVisible.value], 
       ([unlocked, visible]) => {
          if (unlocked) {
            stop();
            resolve(true);
          } else if (!visible) {
             stop();
             resolve(false);
          }
       });
    });
    
    if (!unlocked) return;
  }

  // 3. Подтверждение пользователя
  showConfirm(
    'Switch filesystem?',
    `All tabs will be closed. Switch to ${backend === 'virtual' ? 'Virtual' : 'Real'} filesystem?`,
    () => {
      // 4. Закрыть все табы и создать новый home таб
      rightPanelTabs.value = [{
        id: Date.now(),
        path: [],
        name: 'Home',
        history: [[]],
        historyIndex: 0,
      }];
      rightPanelActiveTabId.value = rightPanelTabs.value[0].id;

      // 5. Переключить FS
      switchPanelFilesystem('right', backend);
    }
  );
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
        :view-mode="props.viewMode"
        :panel-filesystem="leftPanelFilesystem"
        @activate="handleActivateLeft"
        @update:tabs="(tabs) => leftPanelTabs = tabs"
        @update:active-tab-id="(id) => leftPanelActiveTabId = id"
        @switch-filesystem="handleSwitchLeftFilesystem"
        @edit-file="(item, panelFs) => emit('editFile', item, panelFs)"
        @preview-file="(item, panelFs) => emit('previewFile', item, panelFs)"
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
        :view-mode="props.viewMode"
        :panel-filesystem="rightPanelFilesystem"
        @activate="handleActivateRight"
        @update:tabs="(tabs) => rightPanelTabs = tabs"
        @update:active-tab-id="(id) => rightPanelActiveTabId = id"
        @switch-filesystem="handleSwitchRightFilesystem"
        @edit-file="(item, panelFs) => emit('editFile', item, panelFs)"
        @preview-file="(item, panelFs) => emit('previewFile', item, panelFs)"
      />
    </div>
  </div>
</template>
