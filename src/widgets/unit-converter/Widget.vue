<script setup lang="ts">
import { ref, computed, watch } from 'vue';
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

const categories = {
  Length: {
    base: 'm',
    units: {
      m: 1,
      km: 1000,
      cm: 0.01,
      mm: 0.001,
      mi: 1609.344,
      yd: 0.9144,
      ft: 0.3048,
      in: 0.0254,
    }
  },
  Mass: {
    base: 'kg',
    units: {
      kg: 1,
      g: 0.001,
      mg: 0.000001,
      lb: 0.45359237,
      oz: 0.02834952,
    }
  },
  Data: {
    base: 'B',
    units: {
      B: 1,
      KB: 1024,
      MB: 1048576,
      GB: 1073741824,
      TB: 1099511627776,
    }
  }
};

const activeCategory = ref<keyof typeof categories>('Length');
const fromUnit = ref('m');
const toUnit = ref('ft');
const inputValue = ref<number | ''>(1);

const result = computed(() => {
  if (inputValue.value === '' || isNaN(Number(inputValue.value))) return '---';
  
  const cat = categories[activeCategory.value];
  const val = Number(inputValue.value);
  
  // Convert to base
  const inBase = val * cat.units[fromUnit.value as keyof typeof cat.units];
  
  // Convert from base to target
  const final = inBase / cat.units[toUnit.value as keyof typeof cat.units];
  
  if (final < 0.0001 || final > 10000) return final.toExponential(4);
  return Number(final.toFixed(4));
});

watch(activeCategory, (newCat) => {
  const units = Object.keys(categories[newCat].units);
  fromUnit.value = units[0];
  toUnit.value = units[1] || units[0];
});
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="Unit Converter"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <div class="p-4 bg-[var(--vf-bg-primary)] text-[var(--vf-text-primary)] h-full overflow-y-auto">
      <!-- Category Tabs -->
      <div class="flex border-b border-[var(--vf-border-default)] mb-4">
        <button 
          v-for="(cat, key) in categories" 
          :key="key"
          @click="activeCategory = key"
          class="flex-1 pb-2 text-xs font-medium transition-colors border-b-2"
          :class="activeCategory === key ? 'border-[var(--vf-accent-primary)] text-[var(--vf-accent-primary)]' : 'border-transparent text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)]'"
        >
          {{ key }}
        </button>
      </div>

      <!-- Inputs -->
      <div class="space-y-3">
        <!-- From -->
        <div class="flex gap-2">
          <input 
            type="number" 
            v-model="inputValue" 
            class="w-2/3 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded px-2 py-1 text-sm focus:border-[var(--vf-accent-primary)] outline-none"
          />
          <select 
            v-model="fromUnit"
            class="w-1/3 bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] rounded px-1 text-xs outline-none"
          >
            <option v-for="u in Object.keys(categories[activeCategory].units)" :key="u" :value="u">{{ u }}</option>
          </select>
        </div>

        <div class="flex justify-center text-[var(--vf-text-tertiary)]">↓</div>

        <!-- To -->
        <div class="flex gap-2">
          <div class="w-2/3 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded px-2 py-1 text-sm font-mono flex items-center overflow-hidden">
            {{ result }}
          </div>
          <select 
            v-model="toUnit"
            class="w-1/3 bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] rounded px-1 text-xs outline-none"
          >
            <option v-for="u in Object.keys(categories[activeCategory].units)" :key="u" :value="u">{{ u }}</option>
          </select>
        </div>
      </div>
    </div>
  </BaseWidget>
</template>