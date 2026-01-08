<script setup lang="ts">
import { ref } from 'vue';
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

const input = ref('');
const output = ref('');
const error = ref<string | null>(null);

const format = () => {
  if (!input.value.trim()) {
    output.value = '';
    error.value = null;
    return;
  }

  try {
    const parsed = JSON.parse(input.value);
    output.value = JSON.stringify(parsed, null, 2);
    error.value = null;
  } catch (e: any) {
    error.value = e.message;
    output.value = '';
  }
};

const minify = () => {
  if (!input.value.trim()) return;
  try {
    const parsed = JSON.parse(input.value);
    output.value = JSON.stringify(parsed);
    error.value = null;
  } catch (e: any) {
    error.value = e.message;
    output.value = '';
  }
};

const copyOutput = async () => {
  if (!output.value) return;
  try {
    await navigator.clipboard.writeText(output.value);
  } catch (e) {}
};

const clear = () => {
  input.value = '';
  output.value = '';
  error.value = null;
};
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="JSON Formatter"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <div class="flex flex-col h-full bg-[var(--vf-bg-primary)] p-2 gap-2">
      <!-- Toolbar -->
      <div class="flex gap-2 shrink-0">
        <button @click="format" class="px-3 py-1 bg-[var(--vf-accent-primary)] text-white rounded text-xs hover:bg-[var(--vf-accent-hover)]">Prettify</button>
        <button @click="minify" class="px-3 py-1 bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] rounded text-xs hover:bg-[var(--vf-surface-hover)]">Minify</button>
        <button @click="copyOutput" :disabled="!output" class="px-3 py-1 bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] rounded text-xs hover:bg-[var(--vf-surface-hover)] disabled:opacity-50">Copy</button>
        <button @click="clear" class="ml-auto px-3 py-1 text-red-500 hover:bg-red-50 rounded text-xs">Clear</button>
      </div>

      <!-- Editors -->
      <div class="flex-1 flex gap-2 overflow-hidden">
        <!-- Input -->
        <div class="flex-1 flex flex-col min-w-0">
          <span class="text-[10px] text-[var(--vf-text-secondary)] mb-1">Input</span>
          <textarea
            v-model="input"
            class="flex-1 w-full bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded p-2 text-xs font-mono resize-none focus:border-[var(--vf-accent-primary)] outline-none"
            placeholder="Paste JSON here..."
          ></textarea>
        </div>

        <!-- Output -->
        <div class="flex-1 flex flex-col min-w-0">
          <span class="text-[10px] text-[var(--vf-text-secondary)] mb-1">Output</span>
          <div class="flex-1 relative">
            <textarea
              v-if="!error"
              v-model="output"
              readonly
              class="w-full h-full bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded p-2 text-xs font-mono resize-none outline-none text-green-700"
            ></textarea>
            <div 
              v-else 
              class="w-full h-full bg-red-50 border border-red-200 rounded p-2 text-xs font-mono text-red-600 overflow-auto"
            >
              Error: {{ error }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </BaseWidget>
</template>