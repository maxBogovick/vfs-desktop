<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import BaseWidget from './BaseWidget.vue';

defineProps<{
  visible: boolean;
}>();

defineEmits<{
  (e: 'close'): void;
}>();

interface Rates {
  [key: string]: number;
}

const rates = ref<Rates>({});
const loading = ref(true);
const error = ref<string | null>(null);
const lastUpdated = ref<string>('');

// Fetch rates
const fetchRates = async () => {
  loading.value = true;
  error.value = null;
  try {
    const response = await fetch('https://api.exchangerate-api.com/v4/latest/USD');
    if (!response.ok) throw new Error('Network response was not ok');
    const data = await response.json();
    rates.value = data.rates;
    const now = new Date();
    lastUpdated.value = now.toLocaleTimeString();
  } catch (e) {
    console.error('Failed to fetch rates:', e);
    error.value = 'Failed to load rates';
    // Mock data fallback
    rates.value = {
      EUR: 0.92,
      GBP: 0.79,
      JPY: 148.5,
      RUB: 91.5,
      CNY: 7.19,
      BTC: 0.000023 // heavily fluctuating, just placeholder
    };
  } finally {
    loading.value = false;
  }
};

let refreshInterval: number | null = null;

onMounted(() => {
  fetchRates();
  // Refresh every 5 minutes
  refreshInterval = window.setInterval(fetchRates, 5 * 60 * 1000);
});

onUnmounted(() => {
  if (refreshInterval) clearInterval(refreshInterval);
});

const currenciesToShow = ['EUR', 'GBP', 'JPY', 'RUB', 'CNY'];
</script>

<template>
  <BaseWidget
    :visible="visible"
    title="Currency Rates (USD)"
    width="w-64"
    :initial-position="{ x: 500, y: 150 }"
    @close="$emit('close')"
  >
    <template #actions>
      <button 
        @click="fetchRates" 
        class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-accent-primary)] transition-colors"
        title="Refresh"
      >
        ↻
      </button>
    </template>

    <div class="p-4 space-y-2 text-[var(--vf-text-primary)]">
      <div v-if="loading && !Object.keys(rates).length" class="flex justify-center py-4">
        <span class="animate-pulse text-[var(--vf-text-secondary)]">Loading...</span>
      </div>
      
      <div v-else-if="error && !Object.keys(rates).length" class="text-red-400 text-center">
        {{ error }}
      </div>

      <div v-else class="space-y-2">
        <div v-for="curr in currenciesToShow" :key="curr" class="flex justify-between items-center hover:bg-[var(--vf-surface-hover)] p-1 rounded">
          <span class="font-medium w-8">{{ curr }}</span>
          <span class="font-mono">{{ rates[curr]?.toFixed(2) || '---' }}</span>
        </div>
        <div class="text-[10px] text-[var(--vf-text-tertiary)] text-center mt-2 border-t border-[var(--vf-border-subtle)] pt-1">
          Updated: {{ lastUpdated }}
        </div>
      </div>
    </div>
  </BaseWidget>
</template>
