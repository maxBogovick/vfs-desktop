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

const display = ref('0');
const mode = ref<'DEC' | 'HEX' | 'BIN'>('DEC');
const expression = ref('');
const lastResult = ref<number | null>(null);

const setMode = (m: 'DEC' | 'HEX' | 'BIN') => {
  mode.value = m;
  if (lastResult.value !== null) {
    display.value = formatNumber(lastResult.value, m);
  } else {
    if (display.value === '0') return;
    if (!expression.value) {
        try {
             const num = parseInt(display.value, mode.value === 'HEX' ? 16 : (mode.value === 'BIN' ? 2 : 10));
             if (!isNaN(num)) {
                 display.value = formatNumber(num, m);
             }
        } catch (e) {}
    }
  }
};

const formatNumber = (num: number, m: string): string => {
  if (isNaN(num)) return 'Error';
  switch (m) {
    case 'HEX': return num.toString(16).toUpperCase();
    case 'BIN': return num.toString(2);
    default: return num.toString(10);
  }
};

const append = (char: string) => {
  if (display.value === '0' || display.value === 'Error') {
    display.value = char;
  } else {
    display.value += char;
  }
  if (lastResult.value !== null) {
      lastResult.value = null;
      display.value = char;
  }
};

const clear = () => {
  display.value = '0';
  expression.value = '';
  lastResult.value = null;
};

const backspace = () => {
  if (display.value.length > 1) {
    display.value = display.value.slice(0, -1);
  } else {
    display.value = '0';
  }
};

const calculate = () => {
  try {
    const sanitized = display.value.replace(/×/g, '*').replace(/÷/g, '/');
    // eslint-disable-next-line no-new-func
    const result = new Function('return ' + sanitized)();
    
    lastResult.value = result;
    display.value = formatNumber(result, mode.value);
  } catch (e) {
    display.value = 'Error';
  }
};

const buttons = [
  ['C', 'DEL', '%', '÷'],
  ['7', '8', '9', '×'],
  ['4', '5', '6', '-'],
  ['1', '2', '3', '+'],
  ['0', '.', '(', ')'],
  ['=',]
];
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="Calculator"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <div class="p-3 bg-[var(--vf-bg-primary)] h-full flex flex-col">
        <!-- Display -->
        <div class="bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded p-2 mb-3 text-right font-mono text-lg truncate h-10 text-[var(--vf-text-primary)] shrink-0">
            {{ display }}
        </div>

        <!-- Mode Switcher -->
        <div class="flex gap-1 mb-3 text-[10px] shrink-0">
            <button 
                v-for="m in ['DEC', 'HEX', 'BIN']" 
                :key="m"
                @click="setMode(m as any)"
                class="flex-1 py-1 rounded border transition-colors"
                :class="mode === m 
                    ? 'bg-[var(--vf-accent-primary)] text-white border-[var(--vf-accent-active)]' 
                    : 'bg-[var(--vf-surface-default)] text-[var(--vf-text-secondary)] border-[var(--vf-border-default)] hover:bg-[var(--vf-surface-hover)]'"
            >
                {{ m }}
            </button>
        </div>

        <!-- Keypad -->
        <div class="grid grid-cols-4 gap-2 flex-1">
            <template v-for="(row, rIndex) in buttons" :key="rIndex">
                <template v-if="row.length === 1">
                     <button
                        @click="calculate"
                        class="col-span-4 bg-[var(--vf-accent-primary)] text-white hover:bg-[var(--vf-accent-hover)] rounded py-2 font-bold shadow-sm active:translate-y-0.5 transition-all h-full"
                    >
                        =
                    </button>
                </template>
                <template v-else>
                    <button
                        v-for="btn in row"
                        :key="btn"
                        @click="() => {
                            if (btn === 'C') clear();
                            else if (btn === 'DEL') backspace();
                            else if (btn === '÷') append('/');
                            else if (btn === '×') append('*');
                            else append(btn);
                        }"
                        class="bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] hover:bg-[var(--vf-surface-hover)] text-[var(--vf-text-primary)] rounded py-2 font-medium shadow-sm active:translate-y-0.5 transition-all text-sm h-full"
                        :class="{'text-[var(--vf-accent-primary)] font-bold': ['÷', '×', '-', '+', '%'].includes(btn)}"
                    >
                        {{ btn }}
                    </button>
                </template>
            </template>
        </div>
    </div>
  </BaseWidget>
</template>