import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
  QueuedOperation,
  QueueConfig,
  QueueStatistics,
  QueueOperationStatus,
  OperationPriority,
} from '../types';

// Module-level shared state
const operations = ref<Map<string, QueuedOperation>>(new Map());
const config = ref<QueueConfig>({
  maxParallelOperations: 3,
  autoStart: true,
  persistOnChange: true,
  checkScheduledIntervalSec: 60,
});

let updateInterval: number | null = null;
let unlistenQueueEvent: (() => void) | null = null;
let isInitialized = false;

export function useOperationsQueue() {
  // ===== Computed Properties =====

  const operationsList = computed(() => {
    return Array.from(operations.value.values()).sort((a, b) => {
      // Sort by priority first, then by creation time
      if (a.priority !== b.priority) {
        const priorityOrder = { urgent: 3, high: 2, normal: 1, low: 0 };
        return priorityOrder[b.priority] - priorityOrder[a.priority];
      }
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
    });
  });

  const queuedOperations = computed(() =>
    operationsList.value.filter((op) => op.status === 'queued')
  );

  const scheduledOperations = computed(() =>
    operationsList.value.filter((op) => op.status === 'scheduled')
  );

  const runningOperations = computed(() =>
    operationsList.value.filter((op) => op.status === 'running')
  );

  const completedOperations = computed(() =>
    operationsList.value.filter((op) => op.status === 'completed')
  );

  const failedOperations = computed(() =>
    operationsList.value.filter((op) => op.status === 'failed')
  );

  const statistics = computed<QueueStatistics>(() => ({
    total: operations.value.size,
    queued: queuedOperations.value.length,
    scheduled: scheduledOperations.value.length,
    running: runningOperations.value.length,
    completed: completedOperations.value.length,
    failed: failedOperations.value.length,
    cancelled: operationsList.value.filter((op) => op.status === 'cancelled').length,
  }));

  const hasActiveOperations = computed(
    () => runningOperations.value.length > 0 || queuedOperations.value.length > 0
  );

  // ===== Actions =====

  /**
   * Load all operations from backend
   */
  async function loadOperations() {
    try {
      const ops: QueuedOperation[] = await invoke('queue_get_all_operations');
      operations.value.clear();
      ops.forEach((op) => operations.value.set(op.id, op));
    } catch (err) {
      console.error('Failed to load queue:', err);
    }
  }

  /**
   * Add operation to queue
   */
  async function addOperation(
    operationType: string,
    params: any,
    options?: {
      priority?: OperationPriority;
      scheduledAt?: string;
      retryEnabled?: boolean;
      description?: string;
      tags?: string[];
    }
  ): Promise<string> {
    try {
      const operationId: string = await invoke('queue_add_operation', {
        operationType,
        params,
        priority: options?.priority,
        scheduledAt: options?.scheduledAt,
        retryEnabled: options?.retryEnabled,
        description: options?.description,
        tags: options?.tags,
      });

      // Reload to get the new operation
      await loadOperations();

      return operationId;
    } catch (err) {
      console.error('Failed to add operation:', err);
      throw err;
    }
  }

  /**
   * Cancel operation
   */
  async function cancelOperation(operationId: string) {
    try {
      await invoke('queue_cancel_operation', { operationId });
      await loadOperations();
    } catch (err) {
      console.error('Failed to cancel operation:', err);
    }
  }

  /**
   * Retry failed operation
   */
  async function retryOperation(operationId: string) {
    try {
      await invoke('queue_retry_operation', { operationId });
      await loadOperations();
    } catch (err) {
      console.error('Failed to retry operation:', err);
    }
  }

  /**
   * Remove operation (only completed/failed/cancelled)
   */
  async function removeOperation(operationId: string) {
    try {
      await invoke('queue_remove_operation', { operationId });
      operations.value.delete(operationId);
    } catch (err) {
      console.error('Failed to remove operation:', err);
    }
  }

  /**
   * Pause operation
   */
  async function pauseOperation(operationId: string) {
    try {
      await invoke('queue_pause_operation', { operationId });
      await loadOperations();
    } catch (err) {
      console.error('Failed to pause operation:', err);
    }
  }

  /**
   * Resume operation
   */
  async function resumeOperation(operationId: string) {
    try {
      await invoke('queue_resume_operation', { operationId });
      await loadOperations();
    } catch (err) {
      console.error('Failed to resume operation:', err);
    }
  }

  /**
   * Run operation immediately (move to front of queue)
   */
  async function runNow(operationId: string) {
    try {
      await invoke('queue_run_now', { operationId });
      await loadOperations();
    } catch (err) {
      console.error('Failed to run operation now:', err);
    }
  }

  /**
   * Update queue configuration
   */
  async function updateConfig(newConfig: QueueConfig) {
    try {
      await invoke('queue_update_config', { config: newConfig });
      config.value = newConfig;
    } catch (err) {
      console.error('Failed to update config:', err);
    }
  }

  /**
   * Load queue configuration
   */
  async function loadConfig() {
    try {
      const loadedConfig: QueueConfig = await invoke('queue_get_config');
      config.value = loadedConfig;
    } catch (err) {
      console.error('Failed to load config:', err);
    }
  }

  /**
   * Clear completed operations
   */
  async function clearCompleted() {
    const completedIds = completedOperations.value.map((op) => op.id);
    for (const id of completedIds) {
      try {
        await removeOperation(id);
      } catch (err) {
        console.error(`Failed to remove operation ${id}:`, err);
      }
    }
  }

  /**
   * Clear failed operations
   */
  async function clearFailed() {
    const failedIds = failedOperations.value.map((op) => op.id);
    for (const id of failedIds) {
      try {
        await removeOperation(id);
      } catch (err) {
        console.error(`Failed to remove operation ${id}:`, err);
      }
    }
  }

  /**
   * Get operation by ID
   */
  function getOperation(operationId: string): QueuedOperation | undefined {
    return operations.value.get(operationId);
  }

  /**
   * Filter operations by status
   */
  function filterByStatus(status: QueueOperationStatus): QueuedOperation[] {
    return operationsList.value.filter((op) => op.status === status);
  }

  // ===== Lifecycle =====

  /**
   * Initialize - load operations and start polling
   */
  async function initialize() {
    try {
      await loadConfig();
      await loadOperations();

      // Poll for updates every 2 seconds if there are active operations
      updateInterval = window.setInterval(async () => {
        if (hasActiveOperations.value) {
          await loadOperations();
        }
      }, 2000);

      // Listen for queue events (if backend emits them)
      try {
        const unlisten = await listen('queue-operation-update', (event: any) => {
          const operation: QueuedOperation = event.payload;
          operations.value.set(operation.id, operation);
        });
        unlistenQueueEvent = unlisten;
      } catch (err) {
        console.error('Failed to setup queue event listener:', err);
      }
    } catch (err) {
      console.error('Failed to initialize operations queue:', err);
    }
  }

  /**
   * Cleanup
   */
  function cleanup() {
    if (updateInterval !== null) {
      clearInterval(updateInterval);
      updateInterval = null;
    }
    if (unlistenQueueEvent) {
      unlistenQueueEvent();
      unlistenQueueEvent = null;
    }
  }

  // Auto-initialize on mount (only once)
  onMounted(() => {
    if (!isInitialized) {
      isInitialized = true;
      initialize();
    }
  });

  // Note: We don't cleanup on unmount because state is shared
  // cleanup() should only be called when the app is closing

  return {
    // State
    operations,
    operationsList,
    queuedOperations,
    scheduledOperations,
    runningOperations,
    completedOperations,
    failedOperations,
    config,
    statistics,
    hasActiveOperations,

    // Actions
    addOperation,
    cancelOperation,
    retryOperation,
    removeOperation,
    pauseOperation,
    resumeOperation,
    runNow,
    updateConfig,
    loadConfig,
    clearCompleted,
    clearFailed,
    getOperation,
    filterByStatus,
    loadOperations,

    // Lifecycle
    initialize,
    cleanup,
  };
}
