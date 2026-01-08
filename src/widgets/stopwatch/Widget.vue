<script setup lang="ts">
import { ref, onUnmounted } from 'vue';
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

const time = ref(0);
const isRunning = ref(false);
const laps = ref<number[]>([]);
let intervalId: number | null = null;

const formatTime = (ms: number) => {
  const mins = Math.floor(ms / 60000);
  const secs = Math.floor((ms % 60000) / 1000);
  const hundredths = Math.floor((ms % 1000) / 10);
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${hundredths.toString().padStart(2, '0')}`;
};

const startStop = () => {
  if (isRunning.value) {
    if (intervalId) clearInterval(intervalId);
    isRunning.value = false;
  } else {
    const startTime = Date.now() - time.value;
    intervalId = window.setInterval(() => {
      time.value = Date.now() - startTime;
    }, 10);
    isRunning.value = true;
  }
};

const reset = () => {
  if (intervalId) clearInterval(intervalId);
  time.value = 0;
  isRunning.value = false;
  laps.value = [];
};

const addLap = () => {
  if (time.value > 0) {
    laps.value.unshift(time.value);
  }
};

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId);
});
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="Stopwatch"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <div class="p-4 flex flex-col items-center bg-[var(--vf-bg-primary)] h-full">
      <!-- Time Display -->
      <div class="text-3xl font-mono font-bold mb-4 text-[var(--vf-text-primary)] tracking-wider shrink-0">
        {{ formatTime(time) }}
      </div>

      <!-- Controls -->
      <div class="flex gap-2 mb-4 w-full shrink-0">
        <button
          @click="startStop"
          class="flex-1 py-1.5 rounded border transition-colors text-xs font-bold"
          :class="isRunning 
            ? 'bg-red-100 text-red-600 border-red-200 hover:bg-red-200' 
            : 'bg-green-100 text-green-600 border-green-200 hover:bg-green-200'"
        >
          {{ isRunning ? 'STOP' : 'START' }}
        </button>
        
        <button
          @click="addLap"
          :disabled="!isRunning"
          class="flex-1 py-1.5 rounded border border-[var(--vf-border-default)] bg-[var(--vf-surface-default)] text-[var(--vf-text-primary)] hover:bg-[var(--vf-surface-hover)] disabled:opacity-50 disabled:cursor-not-allowed text-xs"
        >
          LAP
        </button>

        <button
          @click="reset"
          class="flex-1 py-1.5 rounded border border-[var(--vf-border-default)] bg-[var(--vf-surface-default)] text-[var(--vf-text-primary)] hover:bg-[var(--vf-surface-hover)] text-xs"
        >
          RESET
        </button>
      </div>

      <!-- Laps List -->
      <div v-if="laps.length > 0" class="w-full flex-1 overflow-y-auto border-t border-[var(--vf-border-subtle)] pt-2 space-y-1 min-h-0">
        <div 
          v-for="(lap, index) in laps" 
          :key="index"
          class="flex justify-between text-[10px] text-[var(--vf-text-secondary)] px-1"
        >
          <span>Lap {{ laps.length - index }}</span>
          <span class="font-mono">{{ formatTime(lap) }}</span>
        </div>
      </div>
    </div>
  </BaseWidget>
</template>