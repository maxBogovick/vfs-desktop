<script setup lang="ts">
import { ref, watch } from 'vue';
import { useOperationsQueue } from '../composables/useOperationsQueue';

interface Props {
  isOpen: boolean;
}

interface Emits {
  (e: 'close'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const { config, updateConfig } = useOperationsQueue();

// Local state for editing with safe defaults
const maxParallelOperations = ref(config.value?.maxParallelOperations ?? 3);
const autoStart = ref(config.value?.autoStart ?? true);
const persistOnChange = ref(config.value?.persistOnChange ?? true);
const checkScheduledIntervalSec = ref(config.value?.checkScheduledIntervalSec ?? 60);

// Watch for external config changes
watch(
  () => config.value,
  (newConfig) => {
    if (newConfig) {
      maxParallelOperations.value = newConfig.maxParallelOperations;
      autoStart.value = newConfig.autoStart;
      persistOnChange.value = newConfig.persistOnChange;
      checkScheduledIntervalSec.value = newConfig.checkScheduledIntervalSec;
    }
  },
  { immediate: true }
);

async function handleSave() {
  try {
    await updateConfig({
      maxParallelOperations: maxParallelOperations.value,
      autoStart: autoStart.value,
      persistOnChange: persistOnChange.value,
      checkScheduledIntervalSec: checkScheduledIntervalSec.value,
    });
    handleClose();
  } catch (err) {
    console.error('Failed to update queue config:', err);
    // You can add a notification here if needed
  }
}

function handleClose() {
  // Reset to current config on close
  maxParallelOperations.value = config.value.maxParallelOperations;
  autoStart.value = config.value.autoStart;
  persistOnChange.value = config.value.persistOnChange;
  checkScheduledIntervalSec.value = config.value.checkScheduledIntervalSec;

  emit('close');
}
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    @click.self="handleClose"
  >
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-lg mx-4">
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
          ⚙️ Queue Settings
        </h3>
        <button
          @click="handleClose"
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-4 space-y-6">
        <!-- Max Parallel Operations -->
        <div>
          <div class="flex items-center justify-between mb-2">
            <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
              🔄 Max Parallel Operations
            </label>
            <span class="text-sm font-semibold text-blue-600 dark:text-blue-400">
              {{ maxParallelOperations }}
            </span>
          </div>
          <input
            v-model.number="maxParallelOperations"
            type="range"
            min="1"
            max="10"
            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
          />
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Number of operations that can run simultaneously
          </p>
        </div>

        <!-- Auto Start -->
        <div class="flex items-start space-x-3">
          <input
            v-model="autoStart"
            type="checkbox"
            id="auto-start"
            class="mt-1 w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
          />
          <div class="flex-1">
            <label for="auto-start" class="text-sm font-medium text-gray-700 dark:text-gray-300 cursor-pointer">
              ▶️ Auto-start queue
            </label>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
              Automatically start processing queue when operations are added
            </p>
          </div>
        </div>

        <!-- Persist on Change -->
        <div class="flex items-start space-x-3">
          <input
            v-model="persistOnChange"
            type="checkbox"
            id="persist-on-change"
            class="mt-1 w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
          />
          <div class="flex-1">
            <label for="persist-on-change" class="text-sm font-medium text-gray-700 dark:text-gray-300 cursor-pointer">
              💾 Persist on change
            </label>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
              Automatically save queue to disk on every change (recommended)
            </p>
          </div>
        </div>

        <!-- Check Scheduled Interval -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            ⏱️ Check Scheduled Interval (seconds)
          </label>
          <input
            v-model.number="checkScheduledIntervalSec"
            type="number"
            min="10"
            max="600"
            step="10"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            How often to check for scheduled tasks (default: 60 seconds)
          </p>
        </div>

        <!-- Info Box -->
        <div class="p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded">
          <p class="text-sm text-blue-700 dark:text-blue-300">
            💡 <strong>Tip:</strong> Higher parallelism speeds up multiple operations but uses more system resources.
            Keep it at 3-5 for balanced performance.
          </p>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end space-x-2 p-4 border-t border-gray-200 dark:border-gray-700">
        <button
          @click="handleClose"
          class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
        >
          Cancel
        </button>
        <button
          @click="handleSave"
          class="px-4 py-2 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
        >
          Save Settings
        </button>
      </div>
    </div>
  </div>
</template>
