<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useOperationsQueue } from '../composables/useOperationsQueue';
import type { QueuedOperation, QueueOperationStatus } from '../types';

interface Emits {
  (e: 'openSettings'): void;
}

const emit = defineEmits<Emits>();

// Safe initialization
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
const selectedOperationId = ref<string | null>(null);
const selectedErrorOperation = ref<QueuedOperation | null>(null);

const filteredOperations = computed(() => {
  if (filterStatus.value === 'all') {
    return operationsList.value;
  }
  return operationsList.value.filter((op: QueuedOperation) => op.status === filterStatus.value);
});

const selectedOperation = computed(() => {
  if (!selectedOperationId.value) return null;
  return operationsList.value.find((op: QueuedOperation) => op.id === selectedOperationId.value) || null;
});

function selectOperation(id: string) {
  selectedOperationId.value = selectedOperationId.value === id ? null : id;
}

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
  <div class="h-full flex flex-col bg-white dark:bg-gray-900 operations-queue-panel">
    <!-- Header -->
    <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700 flex-shrink-0">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
            Operations Queue
          </h2>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
            {{ statistics.running }} running · {{ statistics.queued }} queued ·
            {{ statistics.scheduled }} scheduled
          </p>
        </div>
        <div class="flex space-x-2">
          <button
            @click="clearCompleted"
            :disabled="statistics.completed === 0"
            class="p-1.5 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            title="Clear Completed"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
          <button
            @click="emit('openSettings')"
            class="p-1.5 text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 rounded hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-colors"
            title="Settings"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Filter Tabs -->
    <div class="flex border-b border-gray-200 dark:border-gray-700 px-2 overflow-x-auto flex-shrink-0 no-scrollbar">
      <button
        v-for="status in ['all', 'queued', 'scheduled', 'running', 'completed', 'failed']"
        :key="status"
        @click="filterStatus = status as any"
        :class="[
          'px-3 py-2 text-xs font-medium border-b-2 transition-colors whitespace-nowrap',
          filterStatus === status
            ? 'border-blue-500 text-blue-600 dark:text-blue-400'
            : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300',
        ]"
      >
        {{ status.charAt(0).toUpperCase() + status.slice(1) }}
        <span v-if="status !== 'all'" class="ml-1 opacity-70">
          {{ statistics[status as keyof typeof statistics] }}
        </span>
      </button>
    </div>

    <!-- Operations List -->
    <div class="flex-1 overflow-y-auto">
      <!-- Empty State -->
      <div
        v-if="filteredOperations.length === 0"
        class="flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400 p-4"
      >
        <svg class="w-12 h-12 mb-3 opacity-20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
        <p class="text-sm font-medium">No operations</p>
        <p class="text-xs mt-1 text-center opacity-70">Queue is empty for this filter</p>
      </div>

      <!-- Operations -->
      <div v-else class="divide-y divide-gray-200 dark:divide-gray-700">
        <div
          v-for="operation in filteredOperations"
          :key="operation.id"
          @click="selectOperation(operation.id)"
          :class="[
            'p-3 transition-colors border-l-4 group relative',
            selectedOperationId === operation.id
              ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-500'
              : 'hover:bg-gray-50 dark:hover:bg-gray-800 border-transparent',
          ]"
        >
          <!-- Operation Row -->
          <div class="flex items-start gap-3">
            <!-- Icon -->
            <div class="text-xl flex-shrink-0 pt-0.5">{{ getOperationIcon(operation.operationType) }}</div>
            
            <!-- Content -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1 flex-wrap">
                <span class="font-medium text-sm text-gray-900 dark:text-white truncate">
                  {{ getOperationTypeLabel(operation.operationType) }}
                </span>
                <span :class="['px-1.5 py-0.5 text-[10px] uppercase font-bold tracking-wider rounded-sm', getStatusColor(operation.status)]">
                  {{ operation.status }}
                </span>
                <span v-if="operation.priority !== 'normal'" :class="['px-1.5 py-0.5 text-[10px] uppercase font-bold tracking-wider rounded-sm', getPriorityBadge(operation.priority)]">
                  {{ operation.priority }}
                </span>
              </div>
              
              <p v-if="operation.description" class="text-xs text-gray-600 dark:text-gray-400 mb-1 line-clamp-2">
                {{ operation.description }}
              </p>
              
              <!-- Meta info -->
              <div class="flex items-center gap-2 text-[10px] text-gray-400 dark:text-gray-500">
                <span>{{ formatDate(operation.createdAt) }}</span>
                <span v-if="operation.scheduledAt"> • Sched: {{ formatDate(operation.scheduledAt) }}</span>
                <span v-if="operation.retryAttempts.length > 0"> • Retries: {{ operation.currentAttempt }}/{{ operation.retryPolicy.maxAttempts }}</span>
              </div>

              <!-- Error Message (Inline) -->
              <div
                v-if="operation.errorMessage"
                class="mt-2 p-2 bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800/50 rounded flex items-start gap-2"
              >
                <span class="text-red-500 mt-0.5 text-xs">⚠️</span>
                <div class="flex-1 min-w-0">
                  <p class="text-xs text-red-700 dark:text-red-400 line-clamp-2 font-mono">
                    {{ operation.errorMessage }}
                  </p>
                </div>
                <button
                  @click.stop="showErrorDetails(operation)"
                  class="text-[10px] text-red-600 dark:text-red-400 hover:underline whitespace-nowrap"
                >
                  Details
                </button>
              </div>
            </div>
          </div>

          <!-- Inline Actions (Always visible or on hover/select) -->
          <div class="mt-3 flex items-center justify-end gap-1 pt-2 border-t border-gray-100 dark:border-gray-700/50">
             <!-- Run Now -->
             <button
                v-if="['queued', 'scheduled'].includes(operation.status)"
                @click.stop="runNow(operation.id)"
                class="px-2 py-1 text-xs text-green-600 hover:bg-green-50 dark:hover:bg-green-900/30 rounded transition-colors flex items-center gap-1"
                title="Run Now"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                Run
              </button>

              <!-- Pause -->
              <button
                v-if="operation.status === 'running'"
                @click.stop="pauseOperation(operation.id)"
                class="px-2 py-1 text-xs text-orange-600 hover:bg-orange-50 dark:hover:bg-orange-900/30 rounded transition-colors flex items-center gap-1"
                title="Pause"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6" /></svg>
                Pause
              </button>

              <!-- Resume -->
              <button
                v-if="operation.status === 'paused'"
                @click.stop="resumeOperation(operation.id)"
                class="px-2 py-1 text-xs text-green-600 hover:bg-green-50 dark:hover:bg-green-900/30 rounded transition-colors flex items-center gap-1"
                title="Resume"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /></svg>
                Resume
              </button>

              <!-- Stop -->
              <button
                v-if="['queued', 'scheduled', 'running', 'paused'].includes(operation.status)"
                @click.stop="cancelOperation(operation.id)"
                class="px-2 py-1 text-xs text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 rounded transition-colors flex items-center gap-1"
                title="Stop"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                Stop
              </button>

              <!-- Retry -->
              <button
                v-if="['failed', 'cancelled'].includes(operation.status)"
                @click.stop="retryOperation(operation.id)"
                class="px-2 py-1 text-xs text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/30 rounded transition-colors flex items-center gap-1"
                title="Retry"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
                Retry
              </button>

              <!-- Remove -->
              <button
                v-if="['completed', 'failed', 'cancelled'].includes(operation.status)"
                @click.stop="removeOperation(operation.id)"
                class="px-2 py-1 text-xs text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors flex items-center gap-1"
                title="Remove"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                Remove
              </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer Stats -->
    <div
      class="px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-[10px] text-gray-500 dark:text-gray-400 flex justify-between items-center flex-shrink-0"
    >
      <div>Max parallel: {{ config?.maxParallelOperations ?? 3 }}</div>
      <div v-if="statistics.total > 0">
        {{ statistics.completed }} done, {{ statistics.failed }} failed
      </div>
    </div>

    <!-- Error Details Modal -->
    <div
      v-if="selectedErrorOperation"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-[100] backdrop-blur-sm"
      @click.self="closeErrorDetails"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl max-w-lg w-full mx-4 flex flex-col max-h-[80vh] border border-gray-200 dark:border-gray-700">
        <!-- Modal Header -->
        <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between bg-gray-50 dark:bg-gray-850 rounded-t-lg">
          <h3 class="text-base font-semibold text-gray-900 dark:text-white flex items-center gap-2">
            <span class="text-red-500">⚠️</span> Error Details
          </h3>
          <button
            @click="closeErrorDetails"
            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
          </button>
        </div>

        <!-- Modal Body -->
        <div class="p-4 overflow-y-auto">
          <div class="mb-4">
            <div class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide mb-1">Operation</div>
            <div class="text-sm text-gray-900 dark:text-white font-medium">
              {{ getOperationTypeLabel(selectedErrorOperation.operationType) }}
            </div>
            <div class="text-xs text-gray-500 mt-0.5">{{ selectedErrorOperation.description || 'No description' }}</div>
          </div>

          <div class="mb-4">
            <div class="flex items-center justify-between mb-1">
              <div class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">Error Message</div>
              <button @click="copyErrorToClipboard" class="text-xs text-blue-500 hover:text-blue-700">Copy</button>
            </div>
            <div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800/50 rounded-md">
              <pre class="text-xs text-red-700 dark:text-red-400 whitespace-pre-wrap font-mono break-all">{{ selectedErrorOperation.errorMessage }}</pre>
            </div>
          </div>
          
          <div class="grid grid-cols-2 gap-4 text-xs">
             <div>
                <span class="text-gray-500">ID:</span> <span class="font-mono">{{ selectedErrorOperation.id.substring(0, 8) }}...</span>
             </div>
             <div>
                <span class="text-gray-500">Attempts:</span> {{ selectedErrorOperation.currentAttempt }}/{{ selectedErrorOperation.retryPolicy.maxAttempts }}
             </div>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="px-4 py-3 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-850 rounded-b-lg flex justify-end gap-2">
          <button
            @click="closeErrorDetails"
            class="px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded hover:bg-white dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 transition-colors"
          >
            Close
          </button>
          <button
            @click="retryOperation(selectedErrorOperation.id); closeErrorDetails()"
            class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors shadow-sm"
          >
            Retry Operation
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
.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>