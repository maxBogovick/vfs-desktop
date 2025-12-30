/**
 * Batch Operations Composable
 *
 * Manages batch operations queue, execution, and state.
 * Uses module-level shared state pattern consistent with the codebase.
 */

import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  QueuedBatchOperation,
  QueuedOperationStatus,
  BatchRenameConfig,
  BatchAttributeChange,
  BatchOperationResult,
  BatchOperationProgress,
  FileItem,
  RenamePreviewItem,
  AttributePreviewItem,
} from '../types';
import { generateRenamePreview, validateRenameOperation } from '../utils/batchRenamePatterns';

// Module-level shared state (singleton pattern)
const operations = ref<Map<string, QueuedBatchOperation>>(new Map());
const activeOperationId = ref<string | null>(null);
const isQueueRunning = ref(false);

// Callback for refreshing directory after operations
let onOperationCompleteCallback: (() => Promise<void>) | null = null;

/**
 * Batch Operations Composable
 */
export function useBatchOperations(onOperationComplete?: () => Promise<void>) {
  // Set callback if provided
  if (onOperationComplete) {
    onOperationCompleteCallback = onOperationComplete;
  }
  // ===== Computed Properties =====

  const queuedOperations = computed(() => {
    return Array.from(operations.value.values()).sort((a, b) => b.createdAt - a.createdAt);
  });

  const pendingOperations = computed(() => {
    return queuedOperations.value.filter((op) => op.status === 'pending');
  });

  const runningOperation = computed(() => {
    return queuedOperations.value.find((op) => op.status === 'running');
  });

  const hasOperations = computed(() => operations.value.size > 0);

  const hasPendingOperations = computed(() => pendingOperations.value.length > 0);

  // ===== Queue Management =====

  /**
   * Add batch rename operation to queue
   */
  async function queueBatchRename(
    files: FileItem[],
    config: BatchRenameConfig
  ): Promise<{ operationId: string; preview: RenamePreviewItem[] }> {
    // Generate preview
    const preview = generateRenamePreview(files, config);

    // Validate
    const validation = validateRenameOperation(preview);
    if (!validation.isValid) {
      throw new Error(`Validation failed: ${validation.errors.map((e) => e.error).join(', ')}`);
    }

    // Filter out items with no changes
    const itemsToRename = preview.filter((item) => !item.hasError && item.originalName !== item.newName);

    if (itemsToRename.length === 0) {
      throw new Error('No files would be renamed with the current settings');
    }

    // Create operation
    const operationId = generateOperationId();
    const operation: QueuedBatchOperation = {
      id: operationId,
      type: 'rename',
      status: 'pending',
      itemsCount: itemsToRename.length,
      processedCount: 0,
      failedCount: 0,
      createdAt: Date.now(),
      config,
      items: itemsToRename.map((item) => item.originalPath),
      results: [],
    };

    operations.value.set(operationId, operation);

    // Auto-start if not running
    if (!isQueueRunning.value) {
      startQueue();
    }

    return { operationId, preview };
  }

  /**
   * Add batch attribute change operation to queue
   */
  async function queueBatchAttributeChange(
    files: FileItem[],
    changes: BatchAttributeChange
  ): Promise<string> {
    // Validate changes
    if (!changes.permissions && !changes.dates && !changes.tags) {
      throw new Error('No attribute changes specified');
    }

    // Create operation
    const operationId = generateOperationId();
    const operation: QueuedBatchOperation = {
      id: operationId,
      type: 'attribute_change',
      status: 'pending',
      itemsCount: files.length,
      processedCount: 0,
      failedCount: 0,
      createdAt: Date.now(),
      config: changes,
      items: files.map((file) => file.path),
      results: [],
    };

    operations.value.set(operationId, operation);

    // Auto-start if not running
    if (!isQueueRunning.value) {
      startQueue();
    }

    return operationId;
  }

  /**
   * Start processing queue
   */
  async function startQueue() {
    if (isQueueRunning.value) return;

    isQueueRunning.value = true;

    while (pendingOperations.value.length > 0) {
      const operation = pendingOperations.value[0];
      await executeOperation(operation);
    }

    isQueueRunning.value = false;
  }

  /**
   * Execute a single operation
   */
  async function executeOperation(operation: QueuedBatchOperation) {
    try {
      // Update status
      updateOperationStatus(operation.id, 'running');
      activeOperationId.value = operation.id;

      if (operation.type === 'rename') {
        await executeBatchRename(operation);
      } else if (operation.type === 'attribute_change') {
        await executeBatchAttributeChange(operation);
      }

      // Mark as completed
      updateOperationStatus(operation.id, 'completed');

      // Refresh directory after successful operation
      if (onOperationCompleteCallback) {
        await onOperationCompleteCallback();
      }
    } catch (err) {
      // Mark as failed
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      updateOperationStatus(operation.id, 'failed', errorMessage);
    } finally {
      activeOperationId.value = null;
    }
  }

  /**
   * Execute batch rename operation
   */
  async function executeBatchRename(operation: QueuedBatchOperation) {
    const config = operation.config as BatchRenameConfig;
    const results: BatchOperationResult[] = [];

    for (let i = 0; i < operation.items.length; i++) {
      const filePath = operation.items[i];

      try {
        // Extract directory and filename
        const lastSeparator = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'));
        const directory = filePath.substring(0, lastSeparator);
        const originalName = filePath.substring(lastSeparator + 1);

        // Generate new name using patterns
        const { applyRenamePatterns } = await import('../utils/batchRenamePatterns');
        const { newName, error } = applyRenamePatterns(
          originalName,
          config.patterns,
          i,
          config.preserveExtension
        );

        if (error) {
          throw new Error(error);
        }

        // Skip if no change
        if (originalName === newName) {
          results.push({
            path: filePath,
            success: true,
            originalName,
            newName,
          });
          continue;
        }

        // Call Tauri backend to rename
        await invoke('rename_item', {
          oldPath: filePath,
          newName: newName,
        });

        results.push({
          path: filePath,
          success: true,
          originalName,
          newName,
        });

        // Update progress
        updateOperationProgress(operation.id, i + 1, filePath);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';
        results.push({
          path: filePath,
          success: false,
          errorMessage,
        });

        // Increment failed count
        const op = operations.value.get(operation.id);
        if (op) {
          op.failedCount++;
        }
      }
    }

    // Store results
    const op = operations.value.get(operation.id);
    if (op) {
      op.results = results;
    }
  }

  /**
   * Execute batch attribute change operation
   */
  async function executeBatchAttributeChange(operation: QueuedBatchOperation) {
    const changes = operation.config as BatchAttributeChange;
    const results: BatchOperationResult[] = [];

    for (let i = 0; i < operation.items.length; i++) {
      const filePath = operation.items[i];

      try {
        // Call Tauri backend to change attributes
        await invoke('batch_change_attributes', {
          path: filePath,
          permissions: changes.permissions,
          dates: changes.dates,
          tags: changes.tags,
        });

        results.push({
          path: filePath,
          success: true,
        });

        // Update progress
        updateOperationProgress(operation.id, i + 1, filePath);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';
        results.push({
          path: filePath,
          success: false,
          errorMessage,
        });

        // Increment failed count
        const op = operations.value.get(operation.id);
        if (op) {
          op.failedCount++;
        }
      }
    }

    // Store results
    const op = operations.value.get(operation.id);
    if (op) {
      op.results = results;
    }
  }

  /**
   * Update operation status
   */
  function updateOperationStatus(
    operationId: string,
    status: QueuedOperationStatus,
    errorMessage?: string
  ) {
    const operation = operations.value.get(operationId);
    if (!operation) return;

    operation.status = status;

    if (status === 'running') {
      operation.startedAt = Date.now();
    } else if (status === 'completed' || status === 'failed' || status === 'cancelled') {
      operation.completedAt = Date.now();
      if (errorMessage) {
        operation.errorMessage = errorMessage;
      }
    }
  }

  /**
   * Update operation progress
   */
  function updateOperationProgress(operationId: string, processedCount: number, currentFile: string) {
    const operation = operations.value.get(operationId);
    if (!operation) return;

    operation.processedCount = processedCount;
  }

  /**
   * Cancel operation
   */
  function cancelOperation(operationId: string) {
    const operation = operations.value.get(operationId);
    if (!operation) return;

    if (operation.status === 'pending') {
      updateOperationStatus(operationId, 'cancelled');
    } else if (operation.status === 'running') {
      // For running operations, we need to implement cancellation in the execution loop
      updateOperationStatus(operationId, 'cancelled');
      activeOperationId.value = null;
    }
  }

  /**
   * Remove operation from queue
   */
  function removeOperation(operationId: string) {
    operations.value.delete(operationId);
  }

  /**
   * Clear completed operations
   */
  function clearCompleted() {
    for (const [id, operation] of operations.value.entries()) {
      if (operation.status === 'completed' || operation.status === 'failed' || operation.status === 'cancelled') {
        operations.value.delete(id);
      }
    }
  }

  /**
   * Clear all operations
   */
  function clearAll() {
    operations.value.clear();
    activeOperationId.value = null;
    isQueueRunning.value = false;
  }

  /**
   * Get operation by ID
   */
  function getOperation(operationId: string): QueuedBatchOperation | undefined {
    return operations.value.get(operationId);
  }

  /**
   * Retry failed operation
   */
  async function retryOperation(operationId: string) {
    const operation = operations.value.get(operationId);
    if (!operation) return;

    if (operation.status === 'failed') {
      // Reset operation
      operation.status = 'pending';
      operation.processedCount = 0;
      operation.failedCount = 0;
      operation.errorMessage = undefined;
      operation.results = [];

      // Start queue
      if (!isQueueRunning.value) {
        startQueue();
      }
    }
  }

  // ===== Helper Functions =====

  /**
   * Generate unique operation ID
   */
  function generateOperationId(): string {
    return `batch-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
  }

  return {
    // State
    operations,
    queuedOperations,
    pendingOperations,
    runningOperation,
    activeOperationId,
    isQueueRunning,
    hasOperations,
    hasPendingOperations,

    // Actions
    queueBatchRename,
    queueBatchAttributeChange,
    startQueue,
    cancelOperation,
    removeOperation,
    clearCompleted,
    clearAll,
    getOperation,
    retryOperation,
  };
}
