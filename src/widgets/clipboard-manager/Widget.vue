<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
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

interface ClipItem {
  id: number;
  text: string;
  timestamp: number;
}

const history = ref<ClipItem[]>([]);
const lastClipText = ref('');
const errorMsg = ref<string | null>(null);
let pollInterval: number | null = null;

const MAX_ITEMS = 20;

const formatTime = (ts: number) => {
  const diff = Math.floor((Date.now() - ts) / 1000);
  if (diff < 60) return 'Just now';
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  return `${Math.floor(diff / 3600)}h ago`;
};

const checkClipboard = async (silent = true) => {
  try {
    const text = await navigator.clipboard.readText();
    
    if (text && text !== lastClipText.value) {
      if (history.value.length > 0 && history.value[0].text === text) {
        lastClipText.value = text;
        return;
      }
      addToHistory(text);
      lastClipText.value = text;
      errorMsg.value = null;
    }
  } catch (e: any) {
    if (!silent) {
      console.error('Clipboard error:', e);
      errorMsg.value = 'Failed to read clipboard';
    }
  }
};

const addToHistory = (text: string) => {
  const existingIndex = history.value.findIndex(item => item.text === text);
  if (existingIndex !== -1) {
    history.value.splice(existingIndex, 1);
  }
  
  history.value.unshift({
    id: Date.now(),
    text,
    timestamp: Date.now()
  });
  
  if (history.value.length > MAX_ITEMS) {
    history.value.pop();
  }
  
  saveHistory();
};

const copyItem = async (item: ClipItem) => {
  try {
    await navigator.clipboard.writeText(item.text);
    addToHistory(item.text);
    lastClipText.value = item.text;
  } catch (e) {
    console.error('Failed to copy', e);
  }
};

const deleteItem = (id: number) => {
  history.value = history.value.filter(item => item.id !== id);
  saveHistory();
};

const clearHistory = () => {
  history.value = [];
  saveHistory();
};

const saveHistory = () => {
  localStorage.setItem('vfdir-clipboard-history', JSON.stringify(history.value));
};

const loadHistory = () => {
  const saved = localStorage.getItem('vfdir-clipboard-history');
  if (saved) {
    try {
      history.value = JSON.parse(saved);
      if (history.value.length > 0) {
        lastClipText.value = history.value[0].text;
      }
    } catch (e) {}
  }
};

onMounted(() => {
  loadHistory();
  pollInterval = window.setInterval(() => checkClipboard(true), 1000);
  window.addEventListener('focus', () => checkClipboard(false));
});

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval);
  window.removeEventListener('focus', () => checkClipboard(false));
});
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="Clipboard History"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <template #actions>
      <div class="flex gap-2">
        <button 
          @click="checkClipboard(false)" 
          class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-accent-primary)] transition-colors text-xs"
          title="Refresh Clipboard"
        >
          ↻
        </button>
        <button 
          @click="clearHistory" 
          class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-accent-danger)] transition-colors text-xs"
          title="Clear All"
        >
          Clear
        </button>
      </div>
    </template>

    <div class="flex-1 overflow-y-auto p-1 bg-[var(--vf-bg-primary)] h-full">
      <div v-if="errorMsg" class="bg-red-100 text-red-600 text-[10px] p-1 mb-1 rounded text-center">
        {{ errorMsg }}
      </div>

      <div v-if="history.length === 0" class="flex flex-col items-center justify-center h-full text-[var(--vf-text-tertiary)] p-4 text-center">
        <span class="text-2xl mb-2">📋</span>
        <span class="text-xs">History is empty.<br>Copy some text to see it here.</span>
      </div>

      <div v-else class="space-y-1">
        <div 
          v-for="item in history" 
          :key="item.id"
          class="group relative bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] rounded p-2 hover:bg-[var(--vf-surface-hover)] hover:border-[var(--vf-accent-primary)] transition-all cursor-pointer"
          @click="copyItem(item)"
        >
          <!-- Content -->
          <div class="text-xs text-[var(--vf-text-primary)] font-mono break-all mb-3 overflow-hidden" style="display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical;">
            {{ item.text }}
          </div>
          
          <!-- Metadata -->
          <div class="flex justify-between items-end">
            <span class="text-[9px] text-[var(--vf-text-tertiary)]">{{ formatTime(item.timestamp) }}</span>
            <span class="text-[9px] text-[var(--vf-accent-primary)] opacity-0 group-hover:opacity-100 transition-opacity font-bold">Click to Copy</span>
          </div>

          <!-- Delete Action -->
          <button 
            @click.stop="deleteItem(item.id)"
            class="absolute top-1 right-1 p-1 text-[var(--vf-text-tertiary)] hover:text-[var(--vf-accent-danger)] opacity-0 group-hover:opacity-100 transition-opacity"
            title="Remove"
          >
            ✕
          </button>
        </div>
      </div>
    </div>
  </BaseWidget>
</template>
