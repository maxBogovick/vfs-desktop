<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

defineProps<{
  visible: boolean;
}>();

defineEmits<{
  (e: 'close'): void;
}>();

interface SystemStats {
  memory_mb: number;
  cpu_percent: number;
}

const stats = ref<SystemStats | null>(null);
const intervalId = ref<number | null>(null);
const dragging = ref(false);
const position = ref({ x: 200, y: 150 });
const offset = ref({ x: 0, y: 0 });

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

const startDrag = (e: MouseEvent) => {
  dragging.value = true;
  offset.value = {
    x: e.clientX - position.value.x,
    y: e.clientY - position.value.y
  };
  window.addEventListener('mousemove', onDrag);
  window.addEventListener('mouseup', stopDrag);
};

const onDrag = (e: MouseEvent) => {
  if (dragging.value) {
    position.value = {
      x: e.clientX - offset.value.x,
      y: e.clientY - offset.value.y
    };
  }
};

const stopDrag = () => {
  dragging.value = false;
  window.removeEventListener('mousemove', onDrag);
  window.removeEventListener('mouseup', stopDrag);
};
</script>

<template>
  <div
    v-if="visible"
    class="fixed z-50 bg-gray-800 border border-gray-600 rounded shadow-xl flex flex-col w-64 text-sm font-sans"
    :style="{ top: `${position.y}px`, left: `${position.x}px` }"
  >
    <!-- Header -->
    <div
      class="flex justify-between items-center px-3 py-2 bg-gray-700 cursor-move border-b border-gray-600 rounded-t select-none"
      @mousedown="startDrag"
    >
      <span class="font-bold text-gray-200">System Monitor</span>
      <button 
        @click="$emit('close')" 
        class="text-gray-400 hover:text-white hover:bg-gray-600 rounded px-1 transition-colors"
      >
        ✕
      </button>
    </div>

    <!-- Content -->
    <div class="p-4 space-y-4 text-gray-300">
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
          <div class="w-full bg-gray-900 h-2 rounded-full overflow-hidden border border-gray-700">
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
          <div class="w-full bg-gray-900 h-2 rounded-full overflow-hidden border border-gray-700">
            <div
              class="h-full bg-purple-500 transition-all duration-500 ease-out"
              :style="{ width: `${Math.min((stats.memory_mb / 512) * 100, 100)}%` }"
            ></div>
          </div>
        </div>
      </div>
      
      <div v-else class="flex justify-center py-4">
        <span class="animate-pulse text-gray-500">Loading metrics...</span>
      </div>
    </div>
  </div>
</template>
