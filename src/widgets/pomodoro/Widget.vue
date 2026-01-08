<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue';
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

type Mode = 'work' | 'short' | 'long';

const modes = {
  work: { label: 'Focus', minutes: 25, color: 'text-red-500', bg: 'bg-red-500' },
  short: { label: 'Short Break', minutes: 5, color: 'text-green-500', bg: 'bg-green-500' },
  long: { label: 'Long Break', minutes: 15, color: 'text-blue-500', bg: 'bg-blue-500' },
};

const currentMode = ref<Mode>('work');
const timeLeft = ref(modes.work.minutes * 60);
const isRunning = ref(false);
let timerId: number | null = null;

const progress = computed(() => {
  const total = modes[currentMode.value].minutes * 60;
  return ((total - timeLeft.value) / total) * 100;
});

const formattedTime = computed(() => {
  const m = Math.floor(timeLeft.value / 60);
  const s = timeLeft.value % 60;
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
});

const toggleTimer = () => {
  if (isRunning.value) {
    if (timerId) clearInterval(timerId);
    isRunning.value = false;
  } else {
    timerId = window.setInterval(() => {
      if (timeLeft.value > 0) {
        timeLeft.value--;
      } else {
        if (timerId) clearInterval(timerId);
        isRunning.value = false;
      }
    }, 1000);
    isRunning.value = true;
  }
};

const setMode = (mode: Mode) => {
  if (timerId) clearInterval(timerId);
  isRunning.value = false;
  currentMode.value = mode;
  timeLeft.value = modes[mode].minutes * 60;
};

const reset = () => {
  if (timerId) clearInterval(timerId);
  isRunning.value = false;
  timeLeft.value = modes[currentMode.value].minutes * 60;
};

onUnmounted(() => {
  if (timerId) clearInterval(timerId);
});
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="Pomodoro"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <div class="p-4 bg-[var(--vf-bg-primary)] flex flex-col items-center h-full">
      <!-- Mode Toggles -->
      <div class="flex gap-1 bg-[var(--vf-bg-secondary)] p-1 rounded-lg mb-6 w-full shrink-0">
        <button 
          v-for="(conf, key) in modes" 
          :key="key"
          @click="setMode(key as Mode)"
          class="flex-1 text-[10px] py-1 rounded transition-colors"
          :class="currentMode === key ? 'bg-[var(--vf-surface-default)] shadow text-[var(--vf-text-primary)] font-bold' : 'text-[var(--vf-text-secondary)] hover:bg-[var(--vf-surface-hover)]'"
        >
          {{ conf.label }}
        </button>
      </div>

      <!-- Timer Circle (CSS only) -->
      <div class="relative w-32 h-32 mb-6 flex items-center justify-center shrink-0">
        <svg class="absolute inset-0 w-full h-full -rotate-90" viewBox="0 0 100 100">
          <circle cx="50" cy="50" r="45" fill="none" stroke="var(--vf-bg-tertiary)" stroke-width="6" />
          <circle 
            cx="50" cy="50" r="45" 
            fill="none" 
            :stroke="modes[currentMode].color === 'text-red-500' ? '#ef4444' : (modes[currentMode].color === 'text-green-500' ? '#22c55e' : '#3b82f6')" 
            stroke-width="6"
            stroke-dasharray="283"
            :stroke-dashoffset="283 - (283 * progress / 100)"
            stroke-linecap="round"
            class="transition-all duration-1000 ease-linear"
          />
        </svg>
        
        <div class="text-3xl font-mono font-bold text-[var(--vf-text-primary)] relative z-10">
          {{ formattedTime }}
        </div>
      </div>

      <!-- Controls -->
      <div class="flex gap-3 w-full shrink-0 mt-auto">
        <button 
          @click="toggleTimer"
          class="flex-1 py-2 rounded text-white font-bold transition-transform active:scale-95 shadow-sm"
          :class="modes[currentMode].bg"
        >
          {{ isRunning ? 'PAUSE' : 'START' }}
        </button>
        <button 
          @click="reset"
          class="px-4 py-2 rounded border border-[var(--vf-border-default)] bg-[var(--vf-surface-default)] text-[var(--vf-text-secondary)] hover:bg-[var(--vf-surface-hover)]"
        >
          ↺
        </button>
      </div>
    </div>
  </BaseWidget>
</template>