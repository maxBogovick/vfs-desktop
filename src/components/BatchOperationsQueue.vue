<script setup lang="ts">
import { computed } from 'vue';
import { useBatchOperations } from '../composables/useBatchOperations';
import type { QueuedBatchOperation } from '../types';

// Don't pass callback here - it's set globally in App.vue
const {
  queuedOperations,
  pendingOperations,
  runningOperation,
  isQueueRunning,
  hasOperations,
  cancelOperation,
  removeOperation,
  clearCompleted,
  retryOperation,
} = useBatchOperations();

// Format time duration
function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  } else {
    return `${seconds}s`;
  }
}

// Format date
function formatDate(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - timestamp;

  if (diff < 60000) {
    return 'Just now';
  } else if (diff < 3600000) {
    return `${Math.floor(diff / 60000)}m ago`;
  } else if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString();
  } else {
    return date.toLocaleString();
  }
}

// Get operation duration
function getOperationDuration(operation: QueuedBatchOperation): string {
  if (operation.status === 'running' && operation.startedAt) {
    return formatDuration(Date.now() - operation.startedAt);
  } else if (operation.completedAt && operation.startedAt) {
    return formatDuration(operation.completedAt - operation.startedAt);
  }
  return '-';
}

// Get operation type label
function getOperationTypeLabel(type: string): string {
  switch (type) {
    case 'rename':
      return 'Batch Rename';
    case 'attribute_change':
      return 'Attribute Change';
    default:
      return type;
  }
}

// Get status color
function getStatusColor(status: string): string {
  switch (status) {
    case 'pending':
      return 'text-gray-600 bg-gray-100';
    case 'running':
      return 'text-blue-600 bg-blue-100';
    case 'completed':
      return 'text-green-600 bg-green-100';
    case 'failed':
      return 'text-red-600 bg-red-100';
    case 'cancelled':
      return 'text-orange-600 bg-orange-100';
    default:
      return 'text-gray-600 bg-gray-100';
  }
}

// Get progress percentage
function getProgressPercentage(operation: QueuedBatchOperation): number {
  if (operation.itemsCount === 0) return 0;
  return Math.floor((operation.processedCount / operation.itemsCount) * 100);
}
</script>

<template>
  <div class="flex flex-col h-full bg-[var(--window-bg)]">
    <!-- Header -->
    <div class="flex items-center justify-between p-3 border-b border-[var(--border)]">
      <div class="flex items-center space-x-2">
        <h3 class="font-semibold">Batch Operations Queue</h3>
        <span
          v-if="isQueueRunning"
          class="px-2 py-0.5 text-xs bg-blue-100 text-blue-700 rounded-full animate-pulse"
        >
          Running
        </span>
      </div>
      <div class="flex space-x-2">
        <button
          @click="clearCompleted"
          :disabled="!queuedOperations.some((op) => ['completed', 'failed', 'cancelled'].includes(op.status))"
          class="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Clear Completed
        </button>
      </div>
    </div>

    <!-- Operations List -->
    <div class="flex-1 overflow-auto">
      <!-- Empty State -->
      <div
        v-if="!hasOperations"
        class="flex flex-col items-center justify-center h-full text-gray-500"
      >
        <svg
          class="w-16 h-16 mb-4 text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
        <p class="text-sm">No batch operations in queue</p>
        <p class="text-xs mt-1">Operations will appear here when you run batch rename or attribute changes</p>
      </div>

      <!-- Operations List -->
      <div v-else class="divide-y divide-[var(--border)]">
        <div
          v-for="operation in queuedOperations"
          :key="operation.id"
          class="p-3 hover:bg-gray-50"
        >
          <!-- Operation Header -->
          <div class="flex items-start justify-between mb-2">
            <div class="flex-1">
              <div class="flex items-center space-x-2">
                <span class="font-medium">{{ getOperationTypeLabel(operation.type) }}</span>
                <span
                  :class="[
                    'px-2 py-0.5 text-xs rounded-full capitalize',
                    getStatusColor(operation.status),
                  ]"
                >
                  {{ operation.status }}
                </span>
              </div>
              <div class="text-xs text-gray-600 mt-1">
                {{ formatDate(operation.createdAt) }}
              </div>
            </div>

            <!-- Actions -->
            <div class="flex space-x-1">
              <button
                v-if="operation.status === 'running'"
                @click="cancelOperation(operation.id)"
                class="px-2 py-1 text-xs bg-orange-100 hover:bg-orange-200 text-orange-700 rounded"
                title="Cancel"
              >
                Cancel
              </button>
              <button
                v-if="['failed', 'cancelled'].includes(operation.status)"
                @click="retryOperation(operation.id)"
                class="px-2 py-1 text-xs bg-blue-100 hover:bg-blue-200 text-blue-700 rounded"
                title="Retry"
              >
                Retry
              </button>
              <button
                v-if="['completed', 'failed', 'cancelled'].includes(operation.status)"
                @click="removeOperation(operation.id)"
                class="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 text-gray-700 rounded"
                title="Remove"
              >
                Remove
              </button>
            </div>
          </div>

          <!-- Progress Bar -->
          <div v-if="operation.status === 'running' || operation.status === 'completed'" class="mb-2">
            <div class="w-full bg-gray-200 rounded-full h-2 overflow-hidden">
              <div
                class="h-full bg-blue-500 transition-all duration-300"
                :style="{ width: `${getProgressPercentage(operation)}%` }"
              ></div>
            </div>
            <div class="flex justify-between text-xs text-gray-600 mt-1">
              <span>{{ operation.processedCount }} / {{ operation.itemsCount }} items</span>
              <span>{{ getProgressPercentage(operation) }}%</span>
            </div>
          </div>

          <!-- Stats -->
          <div class="grid grid-cols-3 gap-2 text-xs">
            <div>
              <span class="text-gray-600">Items:</span>
              <span class="ml-1 font-medium">{{ operation.itemsCount }}</span>
            </div>
            <div v-if="operation.failedCount > 0">
              <span class="text-gray-600">Failed:</span>
              <span class="ml-1 font-medium text-red-600">{{ operation.failedCount }}</span>
            </div>
            <div v-if="operation.startedAt">
              <span class="text-gray-600">Duration:</span>
              <span class="ml-1 font-medium">{{ getOperationDuration(operation) }}</span>
            </div>
          </div>

          <!-- Error Message -->
          <div
            v-if="operation.status === 'failed' && operation.errorMessage"
            class="mt-2 p-2 bg-red-50 border border-red-200 rounded text-xs text-red-700"
          >
            {{ operation.errorMessage }}
          </div>

          <!-- Results Summary -->
          <div v-if="operation.results && operation.results.length > 0" class="mt-2">
            <details class="text-xs">
              <summary class="cursor-pointer text-gray-600 hover:text-gray-800">
                View results ({{ operation.results.length }} items)
              </summary>
              <div class="mt-2 max-h-40 overflow-auto bg-gray-50 rounded p-2">
                <div
                  v-for="(result, index) in operation.results"
                  :key="index"
                  :class="[
                    'py-1 border-b border-gray-200 last:border-0',
                    result.success ? 'text-gray-700' : 'text-red-600',
                  ]"
                >
                  <div class="flex items-start justify-between">
                    <span class="truncate flex-1">
                      {{ result.originalName || result.path.split('/').pop() || result.path }}
                    </span>
                    <span v-if="result.success" class="text-green-600 ml-2">✓</span>
                    <span v-else class="text-red-600 ml-2">✗</span>
                  </div>
                  <div v-if="result.newName" class="text-gray-500 ml-2">→ {{ result.newName }}</div>
                  <div v-if="!result.success && result.errorMessage" class="text-red-600 ml-2">
                    {{ result.errorMessage }}
                  </div>
                </div>
              </div>
            </details>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div
      v-if="hasOperations"
      class="p-3 border-t border-[var(--border)] bg-gray-50 text-xs text-gray-600"
    >
      <div class="flex justify-between">
        <span>{{ queuedOperations.length }} total operations</span>
        <span v-if="pendingOperations.length > 0">
          {{ pendingOperations.length }} pending
        </span>
      </div>
    </div>
  </div>
</template>
