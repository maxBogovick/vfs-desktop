<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import BaseWidget from '../../components/BaseWidget.vue';
import type { WidgetLayout } from '../../composables/useWidgets';

defineProps<{
  visible: boolean;
  id: string;
  layout: WidgetLayout;
}>();

defineEmits<{
  (e: 'close'): void;
  (e: 'update:layout', layout: Partial<WidgetLayout>): void;
}>();

interface SystemStats {
  memory_mb: number;
  cpu_percent: number;
}

const stats = ref<SystemStats | null>(null);
const intervalId = ref<number | null>(null);

const fetchStats = async () => {
  try {
    stats.value = await invoke<SystemStats>('get_system_stats');
  } catch (e) {
    console.error('Failed to get system stats:', e);
  }
};

onMounted(() => {
  fetchStats();
  intervalId.value = window.setInterval(fetchStats, 1000);
});

onUnmounted(() => {
  if (intervalId.value) clearInterval(intervalId.value);
});
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="System Monitor"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <div class="p-4 space-y-4 text-[var(--vf-text-primary)] h-full">
      <div v-if="stats">
        <!-- CPU Section -->
        <div>
          <div class="flex justify-between mb-1">
            <span class="font-medium">CPU Usage</span>
            <span 
              :class="{
                'text-green-400': stats.cpu_percent < 50,
                'text-yellow-400': stats.cpu_percent >= 50 && stats.cpu_percent < 80,
                'text-red-400': stats.cpu_percent >= 80
              }"
            >
              {{ stats.cpu_percent.toFixed(1) }}%
            </span>
          </div>
          <div class="w-full bg-[var(--vf-bg-tertiary)] h-2 rounded-full overflow-hidden border border-[var(--vf-border-default)]">
            <div
              class="h-full bg-blue-500 transition-all duration-500 ease-out"
              :style="{ width: `${Math.min(stats.cpu_percent, 100)}%` }"
            ></div>
          </div>
        </div>

        <!-- Memory Section -->
        <div>
          <div class="flex justify-between mb-1">
            <span class="font-medium">Memory Usage</span>
            <span class="text-purple-300">{{ stats.memory_mb.toFixed(0) }} MB</span>
          </div>
          <!-- Bar scaled to 512MB for visualization -->
          <div class="w-full bg-[var(--vf-bg-tertiary)] h-2 rounded-full overflow-hidden border border-[var(--vf-border-default)]">
            <div
              class="h-full bg-purple-500 transition-all duration-500 ease-out"
              :style="{ width: `${Math.min((stats.memory_mb / 512) * 100, 100)}%` }"
            ></div>
          </div>
        </div>
      </div>
      
      <div v-else class="flex justify-center py-4">
        <span class="animate-pulse text-[var(--vf-text-secondary)]">Loading metrics...</span>
      </div>
    </div>
  </BaseWidget>
</template>