import {computed, ref} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import type {FileOperation, ProgressEvent} from '../types';
import {useNotifications} from './useNotifications';

const operations = ref<Map<string, FileOperation>>(new Map());
let progressUnlisten: (() => void) | null = null;

export function useFileOperationsProgress() {
  const { success, error: showError } = useNotifications();

  // Initialize event listener
  const initializeListener = async () => {
    if (progressUnlisten) return;

    progressUnlisten = await listen<ProgressEvent>('file-operation-progress', (event) => {
      const progress = event.payload;
      const operation = operations.value.get(progress.operationId);

      if (operation) {
        operation.progress = progress;
        operation.status = progress.status;

        // Auto-remove completed/cancelled/failed operations after 10 seconds
        if (
          progress.status === 'completed' ||
          progress.status === 'cancelled' ||
          progress.status === 'failed'
        ) {
          setTimeout(() => {
            operations.value.delete(progress.operationId);
          }, 10000);
        }
      }
    });
  };

  // Generate unique operation ID
  const generateOperationId = (): string => {
    return `op_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  };

  // Format bytes to human-readable format
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  // Format seconds to human-readable time
  const formatTime = (seconds: number): string => {
    if (seconds < 60) return `${Math.round(seconds)}s`;
    if (seconds < 3600) {
      const mins = Math.floor(seconds / 60);
      const secs = Math.round(seconds % 60);
      return `${mins}m ${secs}s`;
    }
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${mins}m`;
  };

  // Copy items with progress
  const copyItemsWithProgress = async (sources: string[], destination: string): Promise<void> => {
    await initializeListener();

    const operationId = generateOperationId();
    const operation: FileOperation = {
      id: operationId,
      type: 'copy',
      status: 'running',
      startTime: Date.now(),
      progress: {
        operationId,
        operationType: 'copy',
        status: 'running',
        currentBytes: 0,
        totalBytes: 0,
        currentItems: 0,
        totalItems: 0,
        currentFile: null,
        speedBytesPerSec: 0,
        etaSeconds: null,
        errorMessage: null,
      },
    };

    operations.value.set(operationId, operation);

    try {
      await invoke('copy_items_with_progress_command', {
        operationId,
        sources,
        destination,
      });

      success('Copy completed', `Successfully copied ${sources.length} item(s)`);
    } catch (err) {
      showError('Copy failed', err instanceof Error ? err.message : String(err));
      throw err;
    }
  };

  // Move items with progress
  const moveItemsWithProgress = async (sources: string[], destination: string): Promise<void> => {
    await initializeListener();

    const operationId = generateOperationId();
    const operation: FileOperation = {
      id: operationId,
      type: 'move',
      status: 'running',
      startTime: Date.now(),
      progress: {
        operationId,
        operationType: 'move',
        status: 'running',
        currentBytes: 0,
        totalBytes: 0,
        currentItems: 0,
        totalItems: 0,
        currentFile: null,
        speedBytesPerSec: 0,
        etaSeconds: null,
        errorMessage: null,
      },
    };

    operations.value.set(operationId, operation);

    try {
      await invoke('move_items_with_progress_command', {
        operationId,
        sources,
        destination,
      });

      success('Move completed', `Successfully moved ${sources.length} item(s)`);
    } catch (err) {
      showError('Move failed', err instanceof Error ? err.message : String(err));
      throw err;
    }
  };

  // Delete items with progress
  const deleteItemsWithProgress = async (paths: string[]): Promise<void> => {
    await initializeListener();

    const operationId = generateOperationId();
    const operation: FileOperation = {
      id: operationId,
      type: 'delete',
      status: 'running',
      startTime: Date.now(),
      progress: {
        operationId,
        operationType: 'delete',
        status: 'running',
        currentBytes: 0,
        totalBytes: 0,
        currentItems: 0,
        totalItems: 0,
        currentFile: null,
        speedBytesPerSec: 0,
        etaSeconds: null,
        errorMessage: null,
      },
    };

    operations.value.set(operationId, operation);

    try {
      await invoke('delete_items_with_progress_command', {
        operationId,
        paths,
      });

      success('Delete completed', `Successfully deleted ${paths.length} item(s)`);
    } catch (err) {
      showError('Delete failed', err instanceof Error ? err.message : String(err));
      throw err;
    }
  };

  // Cancel operation
  const cancelOperation = async (operationId: string): Promise<void> => {
    try {
      await invoke('cancel_operation', { operationId });
    } catch (err) {
      showError('Cancel failed', err instanceof Error ? err.message : String(err));
    }
  };

  // Pause operation
  const pauseOperation = async (operationId: string): Promise<void> => {
    try {
      await invoke('pause_operation', { operationId });
    } catch (err) {
      showError('Pause failed', err instanceof Error ? err.message : String(err));
    }
  };

  // Resume operation
  const resumeOperation = async (operationId: string): Promise<void> => {
    try {
      await invoke('resume_operation', { operationId });
    } catch (err) {
      showError('Resume failed', err instanceof Error ? err.message : String(err));
    }
  };

  // Remove operation (for completed/failed/cancelled operations)
  const removeOperation = (operationId: string): void => {
    operations.value.delete(operationId);
  };

  // Get active operations
  const activeOperations = computed(() => {
    return Array.from(operations.value.values()).filter(
      (op) => op.status === 'running' || op.status === 'paused'
    );
  });

  // Has active operations
  const hasActiveOperations = computed(() => activeOperations.value.length > 0);

  // Cleanup listener on unmount
  const cleanup = () => {
    if (progressUnlisten) {
      progressUnlisten();
      progressUnlisten = null;
    }
  };

  return {
    operations,
    activeOperations,
    hasActiveOperations,
    copyItemsWithProgress,
    moveItemsWithProgress,
    deleteItemsWithProgress,
    cancelOperation,
    pauseOperation,
    resumeOperation,
    removeOperation,
    formatBytes,
    formatTime,
    cleanup,
  };
}
