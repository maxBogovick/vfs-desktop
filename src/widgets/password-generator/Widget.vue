<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
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

const length = ref(16);
const useUpper = ref(true);
const useLower = ref(true);
const useNumbers = ref(true);
const useSymbols = ref(true);
const password = ref('');
const copied = ref(false);

const history = ref<string[]>([]);

const generate = () => {
  const upper = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
  const lower = 'abcdefghijklmnopqrstuvwxyz';
  const numbers = '0123456789';
  const symbols = '!@#$%^&*()_+~`|}{[]:;?><,./-=';

  let chars = '';
  if (useUpper.value) chars += upper;
  if (useLower.value) chars += lower;
  if (useNumbers.value) chars += numbers;
  if (useSymbols.value) chars += symbols;

  if (chars === '') {
    password.value = '';
    return;
  }

  let result = '';
  const charsLength = chars.length;
  // Ensure at least one of each selected type is included if length permits
  const mandatoryChars = [];
  if (useUpper.value) mandatoryChars.push(upper.charAt(Math.floor(Math.random() * upper.length)));
  if (useLower.value) mandatoryChars.push(lower.charAt(Math.floor(Math.random() * lower.length)));
  if (useNumbers.value) mandatoryChars.push(numbers.charAt(Math.floor(Math.random() * numbers.length)));
  if (useSymbols.value) mandatoryChars.push(symbols.charAt(Math.floor(Math.random() * symbols.length)));

  for (let i = 0; i < length.value - mandatoryChars.length; i++) {
    result += chars.charAt(Math.floor(Math.random() * charsLength));
  }

  // Shuffle mandatory chars back in
  for (const char of mandatoryChars) {
    const pos = Math.floor(Math.random() * (result.length + 1));
    result = result.slice(0, pos) + char + result.slice(pos);
  }

  password.value = result;
  copied.value = false;
  
  // Add to history
  if (result) {
    history.value.unshift(result);
    if (history.value.length > 5) history.value.pop();
  }
};

const copyToClipboard = async () => {
  if (!password.value) return;
  try {
    await navigator.clipboard.writeText(password.value);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch (e) {
    console.error('Failed to copy', e);
  }
};

const strength = computed(() => {
  if (!password.value) return 0;
  let score = 0;
  if (password.value.length > 8) score += 20;
  if (password.value.length > 12) score += 20;
  if (password.value.length >= 16) score += 20;
  if (useUpper.value && useLower.value) score += 10;
  if (useNumbers.value) score += 15;
  if (useSymbols.value) score += 15;
  return Math.min(score, 100);
});

const strengthColor = computed(() => {
  const s = strength.value;
  if (s < 40) return 'bg-red-500';
  if (s < 70) return 'bg-yellow-500';
  return 'bg-green-500';
});

const strengthText = computed(() => {
  const s = strength.value;
  if (s < 40) return 'Weak';
  if (s < 70) return 'Fair';
  return 'Strong';
});

onMounted(() => {
  generate();
});
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="Password Generator"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <div class="p-4 text-[var(--vf-text-primary)] bg-[var(--vf-bg-primary)] h-full overflow-y-auto">
      <!-- Display Area -->
      <div 
        class="relative flex items-center justify-center bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded p-3 mb-4 cursor-pointer hover:border-[var(--vf-accent-primary)] group transition-colors"
        @click="copyToClipboard"
        title="Click to Copy"
      >
        <div class="font-mono text-lg break-all text-center select-all">
          {{ password || '...' }}
        </div>
        
        <!-- Copied Tooltip -->
        <div 
          class="absolute inset-0 flex items-center justify-center bg-[var(--vf-accent-primary)] text-white text-sm font-bold rounded opacity-0 transition-opacity duration-200 pointer-events-none"
          :class="{ 'opacity-90': copied }"
        >
          COPIED!
        </div>
      </div>

      <!-- Strength Bar -->
      <div class="h-1.5 w-full bg-[var(--vf-bg-tertiary)] rounded-full overflow-hidden mb-1">
        <div 
          class="h-full transition-all duration-500 ease-out" 
          :class="strengthColor"
          :style="{ width: `${strength}%` }"
        ></div>
      </div>
      <div class="text-[10px] text-right text-[var(--vf-text-tertiary)] mb-4 uppercase font-bold tracking-wide">
        {{ strengthText }}
      </div>

      <!-- Controls -->
      <div class="space-y-3 mb-4">
        <!-- Length Slider -->
        <div class="flex items-center gap-3">
          <span class="text-xs w-12">Length: {{ length }}</span>
          <input 
            type="range" 
            min="6" 
            max="50" 
            v-model.number="length" 
            @input="generate"
            class="flex-1 accent-[var(--vf-accent-primary)] h-1 bg-[var(--vf-bg-tertiary)] rounded-lg appearance-none cursor-pointer"
          />
        </div>

        <!-- Checkboxes -->
        <div class="grid grid-cols-2 gap-2 text-xs">
          <label class="flex items-center gap-2 cursor-pointer select-none">
            <input type="checkbox" v-model="useUpper" @change="generate" class="accent-[var(--vf-accent-primary)]">
            <span>ABC (Upper)</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer select-none">
            <input type="checkbox" v-model="useLower" @change="generate" class="accent-[var(--vf-accent-primary)]">
            <span>abc (Lower)</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer select-none">
            <input type="checkbox" v-model="useNumbers" @change="generate" class="accent-[var(--vf-accent-primary)]">
            <span>123 (Numbers)</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer select-none">
            <input type="checkbox" v-model="useSymbols" @change="generate" class="accent-[var(--vf-accent-primary)]">
            <span>!@# (Symbols)</span>
          </label>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-2">
        <button 
          @click="generate"
          class="flex-1 bg-[var(--vf-accent-primary)] hover:bg-[var(--vf-accent-hover)] text-white py-2 rounded text-sm font-medium transition-colors shadow-sm active:translate-y-0.5"
        >
          Generate New
        </button>
      </div>
    </div>
  </BaseWidget>
</template>