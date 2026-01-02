<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { useTerminal } from '../composables/useTerminal';

interface Props {
  height?: number;
  currentPath: string;
}

interface Emits {
  (e: 'resize', height: number): void;
  (e: 'close'): void;
}

const props = withDefaults(defineProps<Props>(), {
  height: 200,
});

const emit = defineEmits<Emits>();

const {
  history,
  currentCommand,
  isExecuting,
  executeCommand,
  clearHistory,
} = useTerminal();

// Resize logic (same as Preview.vue)
const isResizing = ref(false);
const startY = ref(0);
const startHeight = ref(0);

const handleResizeStart = (event: MouseEvent) => {
  isResizing.value = true;
  startY.value = event.clientY;
  startHeight.value = props.height;

  document.addEventListener('mousemove', handleResizeMove);
  document.addEventListener('mouseup', handleResizeEnd);
  event.preventDefault();
};

const handleResizeMove = (event: MouseEvent) => {
  if (!isResizing.value) return;
  const delta = startY.value - event.clientY;
  const newHeight = Math.max(100, Math.min(600, startHeight.value + delta));
  emit('resize', newHeight);
};

const handleResizeEnd = () => {
  isResizing.value = false;
  document.removeEventListener('mousemove', handleResizeMove);
  document.removeEventListener('mouseup', handleResizeEnd);
};

// Auto-scroll to top
const outputRef = ref<HTMLElement | null>(null);
watch(history, () => {
  nextTick(() => {
    if (outputRef.value) {
      outputRef.value.scrollTop = 0;
    }
  });
});

// Execute command
const handleExecute = () => {
  if (currentCommand.value.trim()) {
    executeCommand(currentCommand.value, props.currentPath);
  }
};

// Format timestamp
const formatTime = (timestamp: number) => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('en-US', { hour12: false });
};
</script>

<template>
  <div
    class="flex flex-col bg-[#1E1E1E] border-t border-[#919B9C]"
    :style="{ height: `${height}px` }"
  >
    <!-- Resize handle -->
    <div
      class="h-1 bg-[#919B9C] cursor-row-resize hover:bg-blue-500"
      @mousedown="handleResizeStart"
    />

    <!-- Header -->
    <div class="flex items-center justify-between px-2 py-1 bg-[#2D2D2D] border-b border-[#919B9C]">
      <div class="flex items-center gap-2">
        <span class="text-[11px] text-[#00FF00] font-['Courier_New',monospace]">Terminal</span>
        <span class="text-[9px] text-gray-400">{{ currentPath }}</span>
        <span class="text-[8px] text-gray-500">(30s timeout)</span>
        <span v-if="isExecuting" class="text-[9px] text-yellow-400">⏳ Executing...</span>
      </div>
      <div class="flex items-center gap-2">
        <button
          @click="clearHistory"
          class="text-[10px] text-gray-400 hover:text-white px-2 py-1"
          title="Clear history"
        >
          Clear
        </button>
        <button
          @click="emit('close')"
          class="text-gray-400 hover:text-white px-2"
          title="Close terminal"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Output area -->
    <div
      ref="outputRef"
      class="flex-1 overflow-y-auto px-2 py-2 text-[12px] font-['Courier_New',monospace]"
    >
      <div v-if="history.length === 0" class="text-gray-500 text-center py-4">
        No commands executed yet. Type a command below and press Enter.
      </div>

      <!-- History in reverse order (newest first) -->
      <div
        v-for="entry in history"
        :key="entry.id"
        class="mb-3 pb-3 border-b border-gray-700"
      >
        <!-- Prompt line -->
        <div class="text-gray-400">
          <span class="text-gray-500">[{{ formatTime(entry.timestamp) }}]</span>
          <span class="text-gray-300 ml-2">{{ entry.workingDir }} $</span>
          <span class="text-white ml-2">{{ entry.command }}</span>
        </div>

        <!-- stdout -->
        <div v-if="entry.stdout" class="text-white whitespace-pre-wrap mt-1">{{ entry.stdout }}</div>

        <!-- stderr -->
        <div v-if="entry.stderr" class="text-red-400 whitespace-pre-wrap mt-1">{{ entry.stderr }}</div>

        <!-- Exit code -->
        <div class="text-[10px] mt-1" :class="entry.success ? 'text-green-500' : 'text-red-500'">
          [Exit: {{ entry.exitCode }}]
        </div>
      </div>
    </div>

    <!-- Input area -->
    <div class="border-t border-gray-700 px-2 py-1 flex items-center gap-2 bg-[#2D2D2D]">
      <span class="text-gray-400 text-[11px] font-['Courier_New',monospace]">$</span>
      <input
        v-model="currentCommand"
        type="text"
        class="flex-1 bg-transparent text-white text-[12px] outline-none font-['Courier_New',monospace]"
        placeholder="Enter command (30s timeout, use: ping -c 4 google.com)..."
        :disabled="isExecuting"
        @keydown.enter="handleExecute"
      />
    </div>
  </div>
</template>
