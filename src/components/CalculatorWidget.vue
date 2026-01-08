<script setup lang="ts">
import { ref, computed } from 'vue';
import BaseWidget from './BaseWidget.vue';

defineProps<{
  visible: boolean;
}>();

defineEmits<{
  (e: 'close'): void;
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
    // If we have an expression but no result yet, we can't easily convert the partial input
    // So we just clear or keep as is? Let's keep as is if it's 0, otherwise reset.
    if (display.value === '0') return;
    // For simplicity in this widget, switching mode clears if not looking at a result
    if (!expression.value) {
        // trying to convert current display value
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
  // Reset last result state so we can start a new calculation
  if (lastResult.value !== null) {
      lastResult.value = null;
      display.value = char; // Start fresh
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
    // Basic evaluation for the widget prototype
    // Security note: In a real app, use a safer math parser. 
    // For a local electron/tauri app widget with user input, this is acceptable for a prototype.
    // We need to handle HEX/BIN inputs if we allowed them in the string, 
    // but for now let's assume standard math on current view.
    
    // Simplification: We only evaluate DEC math. 
    // If in HEX/BIN mode, we convert to DEC first? 
    // Or we just strictly allow basic math operations.
    
    // Let's stick to standard math for now and assume input is valid JS.
    // Replace visual operators
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
  ['0', '.', '(', ')'], // Added parenthesis for programmer utility
  ['=',]
];

// Special layout for the last row
</script>

<template>
  <BaseWidget
    :visible="visible"
    title="Calculator"
    width="w-64"
    :initial-position="{ x: 800, y: 150 }"
    @close="$emit('close')"
  >
    <div class="p-3 bg-[var(--vf-bg-primary)]">
        <!-- Display -->
        <div class="bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded p-2 mb-3 text-right font-mono text-lg truncate h-10 text-[var(--vf-text-primary)]">
            {{ display }}
        </div>

        <!-- Mode Switcher -->
        <div class="flex gap-1 mb-3 text-[10px]">
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
        <div class="grid grid-cols-4 gap-2">
            <template v-for="(row, rIndex) in buttons" :key="rIndex">
                <template v-if="row.length === 1">
                     <!-- Equals button spans full width -->
                     <button
                        @click="calculate"
                        class="col-span-4 bg-[var(--vf-accent-primary)] text-white hover:bg-[var(--vf-accent-hover)] rounded py-2 font-bold shadow-sm active:translate-y-0.5 transition-all"
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
                        class="bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] hover:bg-[var(--vf-surface-hover)] text-[var(--vf-text-primary)] rounded py-2 font-medium shadow-sm active:translate-y-0.5 transition-all text-sm"
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
