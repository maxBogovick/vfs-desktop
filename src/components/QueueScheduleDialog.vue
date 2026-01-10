<script setup lang="ts">
import { ref, computed } from 'vue';
import type { OperationPriority } from '../types';

interface Props {
  isOpen: boolean;
}

interface Emits {
  (e: 'close'): void;
  (e: 'submit', data: {
    scheduledAt: string;
    priority: OperationPriority;
    description: string;
    tags: string[];
    retryEnabled: boolean;
  }): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const scheduledDate = ref('');
const scheduledTime = ref('');
const priority = ref<OperationPriority>('normal');
const description = ref('');
const tagsInput = ref('');
const retryEnabled = ref(true);

const minDateTime = computed(() => {
  const now = new Date();
  return now.toISOString().slice(0, 16);
});

const tags = computed(() => {
  return tagsInput.value
    .split(',')
    .map((t) => t.trim())
    .filter((t) => t.length > 0);
});

function handleSubmit() {
  if (!scheduledDate.value || !scheduledTime.value) {
    return;
  }

  const scheduledAt = new Date(`${scheduledDate.value}T${scheduledTime.value}`).toISOString();

  emit('submit', {
    scheduledAt,
    priority: priority.value,
    description: description.value,
    tags: tags.value,
    retryEnabled: retryEnabled.value,
  });

  handleClose();
}

function handleClose() {
  // Reset form
  scheduledDate.value = '';
  scheduledTime.value = '';
  priority.value = 'normal';
  description.value = '';
  tagsInput.value = '';
  retryEnabled.value = true;

  emit('close');
}

function setQuickTime(minutes: number) {
  const now = new Date();
  now.setMinutes(now.getMinutes() + minutes);
  scheduledDate.value = now.toISOString().slice(0, 10);
  scheduledTime.value = now.toISOString().slice(11, 16);
}
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    @click.self="handleClose"
  >
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4">
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
          📅 Schedule Operation
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
      <div class="p-4 space-y-4">
        <!-- Date & Time -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            📆 Schedule Date & Time
          </label>
          <div class="grid grid-cols-2 gap-2">
            <input
              v-model="scheduledDate"
              type="date"
              :min="minDateTime.slice(0, 10)"
              class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              required
            />
            <input
              v-model="scheduledTime"
              type="time"
              class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              required
            />
          </div>

          <!-- Quick Time Buttons -->
          <div class="flex flex-wrap gap-2 mt-2">
            <button
              v-for="time in [30, 60, 120, 180, 360]"
              :key="time"
              @click="setQuickTime(time)"
              class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors"
            >
              +{{ time }}m
            </button>
          </div>
        </div>

        <!-- Priority -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            🎯 Priority
          </label>
          <select
            v-model="priority"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="low">Low</option>
            <option value="normal">Normal</option>
            <option value="high">High</option>
            <option value="urgent">Urgent</option>
          </select>
        </div>

        <!-- Description -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            📝 Description (optional)
          </label>
          <textarea
            v-model="description"
            rows="2"
            placeholder="Describe this operation..."
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
          ></textarea>
        </div>

        <!-- Tags -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            🏷️ Tags (optional)
          </label>
          <input
            v-model="tagsInput"
            type="text"
            placeholder="backup, important, cleanup (comma separated)"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <div v-if="tags.length > 0" class="flex flex-wrap gap-1 mt-2">
            <span
              v-for="tag in tags"
              :key="tag"
              class="px-2 py-0.5 text-xs bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded"
            >
              {{ tag }}
            </span>
          </div>
        </div>

        <!-- Retry -->
        <div class="flex items-center">
          <input
            v-model="retryEnabled"
            type="checkbox"
            id="retry-enabled"
            class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
          />
          <label for="retry-enabled" class="ml-2 text-sm text-gray-700 dark:text-gray-300">
            🔄 Enable automatic retry on failure
          </label>
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
          @click="handleSubmit"
          :disabled="!scheduledDate || !scheduledTime"
          class="px-4 py-2 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          Schedule Operation
        </button>
      </div>
    </div>
  </div>
</template>
