<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useOperationsQueue } from '../composables/useOperationsQueue';
import type { QueuedOperation, QueueOperationStatus } from '../types';

interface Emits {
  (e: 'openSettings'): void;
}

const emit = defineEmits<Emits>();

// Try to use the queue composable with error handling
let queueData: any = null;
try {
  queueData = useOperationsQueue();
} catch (err) {
  console.error('Failed to initialize operations queue:', err);
}

const {
  operationsList = computed(() => []),
  statistics = computed(() => ({
    total: 0,
    queued: 0,
    scheduled: 0,
    running: 0,
    completed: 0,
    failed: 0,
    cancelled: 0,
  })),
  config = computed(() => ({
    maxParallelOperations: 3,
    autoStart: true,
    persistOnChange: true,
    checkScheduledIntervalSec: 60,
  })),
  cancelOperation = () => {},
  retryOperation = () => {},
  removeOperation = () => {},
  pauseOperation = () => {},
  resumeOperation = () => {},
  runNow = () => {},
  clearCompleted = () => {},
} = queueData || {};

const filterStatus = ref<'all' | QueueOperationStatus>('all');
const selectedErrorOperation = ref<QueuedOperation | null>(null);

const filteredOperations = computed(() => {
  if (filterStatus.value === 'all') {
    return operationsList.value;
  }
  return operationsList.value.filter((op: QueuedOperation) => op.status === filterStatus.value);
});

function getStatusColor(status: QueueOperationStatus): string {
  const colors = {
    queued: 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300',
    scheduled: 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300',
    running: 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300 animate-pulse',
    paused: 'bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300',
    completed: 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300',
    failed: 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300',
    cancelled: 'bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400',
  };
  return colors[status] || 'bg-gray-100 text-gray-700';
}

function getPriorityBadge(priority: string): string {
  const colors = {
    urgent: 'bg-red-500 text-white',
    high: 'bg-orange-500 text-white',
    normal: 'bg-blue-500 text-white',
    low: 'bg-gray-400 text-white',
  };
  return colors[priority as keyof typeof colors] || colors.normal;
}

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (minutes < 1) return 'Just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  return date.toLocaleDateString();
}

function getOperationTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    copy: 'Copy',
    move: 'Move',
    delete: 'Delete',
    archive: 'Archive',
    extract: 'Extract',
    batch_rename: 'Batch Rename',
    batch_attribute: 'Batch Attributes',
    custom: 'Custom',
  };
  return labels[type] || type;
}

function getOperationIcon(type: string): string {
  const icons: Record<string, string> = {
    copy: '📋',
    move: '➡️',
    delete: '🗑️',
    archive: '📦',
    extract: '📂',
    batch_rename: '✏️',
    batch_attribute: '⚙️',
    custom: '🔧',
  };
  return icons[type] || '📄';
}

function showErrorDetails(operation: QueuedOperation) {
  selectedErrorOperation.value = operation;
}

function closeErrorDetails() {
  selectedErrorOperation.value = null;
}

async function copyErrorToClipboard() {
  if (selectedErrorOperation.value?.errorMessage) {
    try {
      await navigator.clipboard.writeText(selectedErrorOperation.value.errorMessage);
    } catch (err) {
      console.error('Failed to copy error to clipboard:', err);
    }
  }
}
</script>

<template>
  <div class="operations-queue-panel h-full flex flex-col bg-white dark:bg-gray-900">
    <!-- Header -->
    <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
            Operations Queue
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            {{ statistics.running }} running · {{ statistics.queued }} queued ·
            {{ statistics.scheduled }} scheduled
          </p>
        </div>
        <div class="flex space-x-2">
          <button
            @click="clearCompleted"
            :disabled="statistics.completed === 0"
            class="px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Clear Completed
          </button>
          <button
            @click="emit('openSettings')"
            class="px-3 py-1.5 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
          >
            ⚙️ Settings
          </button>
        </div>
      </div>
    </div>

    <!-- Filter Tabs -->
    <div class="flex border-b border-gray-200 dark:border-gray-700 px-4 overflow-x-auto">
      <button
        v-for="status in ['all', 'queued', 'scheduled', 'running', 'completed', 'failed']"
        :key="status"
        @click="filterStatus = status as any"
        :class="[
          'px-4 py-2 text-sm font-medium border-b-2 transition-colors whitespace-nowrap',
          filterStatus === status
            ? 'border-blue-500 text-blue-600 dark:text-blue-400'
            : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300',
        ]"
      >
        {{ status.charAt(0).toUpperCase() + status.slice(1) }}
        <span v-if="status !== 'all'" class="ml-1 text-xs">
          ({{ statistics[status as keyof typeof statistics] }})
        </span>
      </button>
    </div>

    <!-- Operations List -->
    <div class="flex-1 overflow-y-auto">
      <!-- Empty State -->
      <div
        v-if="filteredOperations.length === 0"
        class="flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400"
      >
        <svg class="w-16 h-16 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
          />
        </svg>
        <p class="text-lg font-medium">No operations in queue</p>
        <p class="text-sm mt-1">Operations will appear here when you add them</p>
      </div>

      <!-- Operations -->
      <div v-else class="divide-y divide-gray-200 dark:divide-gray-700">
        <div
          v-for="operation in filteredOperations"
          :key="operation.id"
          class="p-4 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
        >
          <!-- Operation Header -->
          <div class="flex items-start justify-between mb-2">
            <div class="flex-1">
              <div class="flex items-center space-x-2 mb-1">
                <span class="text-xl">{{ getOperationIcon(operation.operationType) }}</span>
                <span class="font-medium text-gray-900 dark:text-white">
                  {{ getOperationTypeLabel(operation.operationType) }}
                </span>
                <span :class="['px-2 py-0.5 text-xs rounded-full', getStatusColor(operation.status)]">
                  {{ operation.status }}
                </span>
                <span :class="['px-2 py-0.5 text-xs rounded-full', getPriorityBadge(operation.priority)]">
                  {{ operation.priority }}
                </span>
              </div>
              <p v-if="operation.description" class="text-sm text-gray-600 dark:text-gray-400">
                {{ operation.description }}
              </p>
              <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                Created {{ formatDate(operation.createdAt) }}
                <span v-if="operation.scheduledAt">
                  · Scheduled for {{ formatDate(operation.scheduledAt) }}
                </span>
              </p>
            </div>

            <!-- Actions -->
            <div class="flex space-x-1 ml-4">
              <!-- Run Now button for queued/scheduled operations -->
              <button
                v-if="['queued', 'scheduled'].includes(operation.status)"
                @click="runNow(operation.id)"
                class="p-1 text-green-600 hover:bg-green-100 dark:hover:bg-green-900 rounded transition-colors"
                title="Run Now"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M13 10V3L4 14h7v7l9-11h-7z"
                  />
                </svg>
              </button>

              <button
                v-if="operation.status === 'running'"
                @click="pauseOperation(operation.id)"
                class="p-1 text-orange-600 hover:bg-orange-100 dark:hover:bg-orange-900 rounded transition-colors"
                title="Pause"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6" />
                </svg>
              </button>
              <button
                v-if="operation.status === 'paused'"
                @click="resumeOperation(operation.id)"
                class="p-1 text-green-600 hover:bg-green-100 dark:hover:bg-green-900 rounded transition-colors"
                title="Resume"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                  />
                </svg>
              </button>
              <button
                v-if="['queued', 'running', 'paused'].includes(operation.status)"
                @click="cancelOperation(operation.id)"
                class="p-1 text-red-600 hover:bg-red-100 dark:hover:bg-red-900 rounded transition-colors"
                title="Cancel"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
              <button
                v-if="operation.status === 'failed'"
                @click="showErrorDetails(operation)"
                class="p-1 text-orange-600 hover:bg-orange-100 dark:hover:bg-orange-900 rounded transition-colors"
                title="View Error Details"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
              </button>
              <button
                v-if="['failed', 'cancelled'].includes(operation.status)"
                @click="retryOperation(operation.id)"
                class="p-1 text-blue-600 hover:bg-blue-100 dark:hover:bg-blue-900 rounded transition-colors"
                title="Retry"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                  />
                </svg>
              </button>
              <button
                v-if="['completed', 'failed', 'cancelled'].includes(operation.status)"
                @click="removeOperation(operation.id)"
                class="p-1 text-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                title="Remove"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                  />
                </svg>
              </button>
            </div>
          </div>

          <!-- Error Message -->
          <div
            v-if="operation.errorMessage"
            class="mt-2 p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded"
          >
            <div class="flex items-start justify-between gap-2">
              <div class="flex-1 text-sm text-red-700 dark:text-red-400">
                <span class="font-medium">❌ Error:</span>
                <span class="ml-1">
                  {{ operation.errorMessage.length > 100
                    ? operation.errorMessage.substring(0, 100) + '...'
                    : operation.errorMessage
                  }}
                </span>
              </div>
              <button
                @click="showErrorDetails(operation)"
                class="px-2 py-1 text-xs bg-red-100 dark:bg-red-800 text-red-700 dark:text-red-300 rounded hover:bg-red-200 dark:hover:bg-red-700 transition-colors whitespace-nowrap"
                title="View full error details"
              >
                Details
              </button>
            </div>
          </div>

          <!-- Tags -->
          <div v-if="operation.tags && operation.tags.length > 0" class="mt-2 flex flex-wrap gap-1">
            <span
              v-for="tag in operation.tags"
              :key="tag"
              class="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded"
            >
              🏷️ {{ tag }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer Stats -->
    <div
      class="px-4 py-3 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800"
    >
      <div class="grid grid-cols-4 gap-4 text-center text-sm">
        <div>
          <div class="text-gray-500 dark:text-gray-400">Total</div>
          <div class="font-semibold text-gray-900 dark:text-white">{{ statistics.total }}</div>
        </div>
        <div>
          <div class="text-gray-500 dark:text-gray-400">Running</div>
          <div class="font-semibold text-blue-600 dark:text-blue-400">
            {{ statistics.running }}
          </div>
        </div>
        <div>
          <div class="text-gray-500 dark:text-gray-400">Completed</div>
          <div class="font-semibold text-green-600 dark:text-green-400">
            {{ statistics.completed }}
          </div>
        </div>
        <div>
          <div class="text-gray-500 dark:text-gray-400">Failed</div>
          <div class="font-semibold text-red-600 dark:text-red-400">{{ statistics.failed }}</div>
        </div>
      </div>
      <div class="mt-2 text-xs text-gray-500 dark:text-gray-400 text-center">
        Max parallel: {{ config.maxParallelOperations ?? 3 }} operations
      </div>
    </div>

    <!-- Error Details Modal -->
    <div
      v-if="selectedErrorOperation"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="closeErrorDetails"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] flex flex-col">
        <!-- Modal Header -->
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
              Operation Error Details
            </h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
              {{ getOperationTypeLabel(selectedErrorOperation.operationType) }} - {{ selectedErrorOperation.description || 'No description' }}
            </p>
          </div>
          <button
            @click="closeErrorDetails"
            class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Modal Body -->
        <div class="flex-1 overflow-y-auto px-6 py-4">
          <!-- Operation Info -->
          <div class="mb-4 grid grid-cols-2 gap-4 text-sm">
            <div>
              <span class="text-gray-500 dark:text-gray-400">Operation ID:</span>
              <span class="ml-2 font-mono text-gray-900 dark:text-white">{{ selectedErrorOperation.id }}</span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">Status:</span>
              <span :class="['ml-2 px-2 py-0.5 text-xs rounded-full', getStatusColor(selectedErrorOperation.status)]">
                {{ selectedErrorOperation.status }}
              </span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">Priority:</span>
              <span :class="['ml-2 px-2 py-0.5 text-xs rounded-full', getPriorityBadge(selectedErrorOperation.priority)]">
                {{ selectedErrorOperation.priority }}
              </span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">Created:</span>
              <span class="ml-2 text-gray-900 dark:text-white">{{ formatDate(selectedErrorOperation.createdAt) }}</span>
            </div>
            <div v-if="selectedErrorOperation.startedAt">
              <span class="text-gray-500 dark:text-gray-400">Started:</span>
              <span class="ml-2 text-gray-900 dark:text-white">{{ formatDate(selectedErrorOperation.startedAt) }}</span>
            </div>
            <div v-if="selectedErrorOperation.currentAttempt > 0">
              <span class="text-gray-500 dark:text-gray-400">Attempts:</span>
              <span class="ml-2 text-gray-900 dark:text-white">{{ selectedErrorOperation.currentAttempt }}</span>
            </div>
          </div>

          <!-- Error Message -->
          <div class="mb-4">
            <div class="flex items-center justify-between mb-2">
              <h4 class="font-medium text-gray-900 dark:text-white">Error Message:</h4>
              <button
                @click="copyErrorToClipboard"
                class="px-3 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
              >
                📋 Copy
              </button>
            </div>
            <div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded">
              <pre class="text-sm text-red-700 dark:text-red-400 whitespace-pre-wrap font-mono">{{ selectedErrorOperation.errorMessage }}</pre>
            </div>
          </div>

          <!-- Tags -->
          <div v-if="selectedErrorOperation.tags && selectedErrorOperation.tags.length > 0" class="mb-4">
            <h4 class="font-medium text-gray-900 dark:text-white mb-2">Tags:</h4>
            <div class="flex flex-wrap gap-2">
              <span
                v-for="tag in selectedErrorOperation.tags"
                :key="tag"
                class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded"
              >
                🏷️ {{ tag }}
              </span>
            </div>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-2">
          <button
            @click="closeErrorDetails"
            class="px-4 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
          >
            Close
          </button>
          <button
            @click="retryOperation(selectedErrorOperation.id); closeErrorDetails()"
            class="px-4 py-2 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
          >
            🔄 Retry Operation
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.operations-queue-panel {
  font-family: system-ui, -apple-system, sans-serif;
}
</style>
