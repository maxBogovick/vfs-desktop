<template>
  <Transition name="slide-up">
    <div v-if="hasActiveOperations" class="operations-progress">
      <div class="operations-container">
        <TransitionGroup name="operation-list" tag="div" class="operations-list">
          <div
            v-for="operation in activeOperations"
            :key="operation.id"
            class="operation-item"
            :class="[`operation-${operation.type}`, `status-${operation.status}`]"
          >
            <div class="operation-header">
              <div class="operation-info">
                <div class="operation-icon">
                  <svg v-if="operation.type === 'copy'" viewBox="0 0 24 24" fill="none">
                    <path d="M16 1H4C2.9 1 2 1.9 2 3V17H4V3H16V1ZM19 5H8C6.9 5 6 5.9 6 7V21C6 22.1 6.9 23 8 23H19C20.1 23 21 22.1 21 21V7C21 5.9 20.1 5 19 5ZM19 21H8V7H19V21Z" fill="currentColor"/>
                  </svg>
                  <svg v-else-if="operation.type === 'move'" viewBox="0 0 24 24" fill="none">
                    <path d="M12 2L8 6H11V12H13V6H16L12 2ZM6 18V8H4V18C4 19.1 4.9 20 6 20H18C19.1 20 20 19.1 20 18V8H18V18H6Z" fill="currentColor"/>
                  </svg>
                  <svg v-else viewBox="0 0 24 24" fill="none">
                    <path d="M6 19C6 20.1 6.9 21 8 21H16C17.1 21 18 20.1 18 19V7H6V19ZM19 4H15.5L14.5 3H9.5L8.5 4H5V6H19V4Z" fill="currentColor"/>
                  </svg>
                </div>
                <div class="operation-text">
                  <div class="operation-title">
                    {{ getOperationTitle(operation) }}
                  </div>
                  <div class="operation-file" v-if="operation.progress.currentFile">
                    {{ operation.progress.currentFile }}
                  </div>
                </div>
              </div>
              <div class="operation-buttons">
                <button
                  v-if="operation.status === 'running' || operation.status === 'paused'"
                  class="action-btn pause-btn"
                  @click="handlePauseResume(operation)"
                  :title="operation.status === 'running' ? 'Pause' : 'Resume'"
                >
                  <svg v-if="operation.status === 'running'" viewBox="0 0 24 24" fill="none">
                    <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" fill="currentColor"/>
                  </svg>
                  <svg v-else viewBox="0 0 24 24" fill="none">
                    <path d="M8 5v14l11-7z" fill="currentColor"/>
                  </svg>
                </button>
                <button
                  class="action-btn cancel-btn"
                  @click="handleCloseOrCancel(operation)"
                  :title="operation.status === 'running' || operation.status === 'paused' ? 'Cancel operation' : 'Close'"
                >
                  <svg viewBox="0 0 24 24" fill="none">
                    <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" fill="currentColor"/>
                  </svg>
                </button>
              </div>
            </div>

            <div class="progress-bar-container">
              <div class="progress-bar">
                <div
                  class="progress-fill"
                  :style="{ width: getProgressPercent(operation) + '%' }"
                ></div>
              </div>
            </div>

            <div class="operation-stats">
              <div class="stat-item">
                <span class="stat-label">Progress:</span>
                <span class="stat-value">{{ getProgressPercent(operation).toFixed(1) }}%</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">Items:</span>
                <span class="stat-value">
                  {{ operation.progress.currentItems }} / {{ operation.progress.totalItems }}
                </span>
              </div>
              <div class="stat-item">
                <span class="stat-label">Size:</span>
                <span class="stat-value">
                  {{ formatBytes(operation.progress.currentBytes) }} / {{ formatBytes(operation.progress.totalBytes) }}
                </span>
              </div>
              <div class="stat-item" v-if="operation.progress.speedBytesPerSec > 0">
                <span class="stat-label">Speed:</span>
                <span class="stat-value">{{ formatBytes(operation.progress.speedBytesPerSec) }}/s</span>
              </div>
              <div class="stat-item" v-if="operation.progress.etaSeconds !== null">
                <span class="stat-label">ETA:</span>
                <span class="stat-value">{{ formatTime(operation.progress.etaSeconds) }}</span>
              </div>
            </div>

            <div v-if="operation.progress.errorMessage" class="operation-error">
              {{ operation.progress.errorMessage }}
            </div>
          </div>
        </TransitionGroup>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useFileOperationsProgress } from '../composables/useFileOperationsProgress';
import type { FileOperation } from '../types';

const {
  activeOperations,
  hasActiveOperations,
  cancelOperation,
  pauseOperation,
  resumeOperation,
  removeOperation,
  formatBytes,
  formatTime,
  cleanup,
} = useFileOperationsProgress();

const getOperationTitle = (operation: FileOperation): string => {
  const typeLabels = {
    copy: 'Copying',
    move: 'Moving',
    delete: 'Deleting',
  };

  const statusLabels = {
    running: '',
    paused: 'Paused',
    completed: 'Completed',
    cancelled: 'Cancelled',
    failed: 'Failed',
  };

  const typeLabel = typeLabels[operation.type] || operation.type;
  const statusLabel = statusLabels[operation.status] || '';

  if (statusLabel) {
    return `${typeLabel} - ${statusLabel}`;
  }

  return `${typeLabel} ${operation.progress.totalItems} item(s)`;
};

const getProgressPercent = (operation: FileOperation): number => {
  const { totalBytes, currentBytes } = operation.progress;
  if (totalBytes === 0) return 0;
  return (currentBytes / totalBytes) * 100;
};

const handlePauseResume = (operation: FileOperation) => {
  if (operation.status === 'running') {
    pauseOperation(operation.id);
  } else if (operation.status === 'paused') {
    resumeOperation(operation.id);
  }
};

const handleCloseOrCancel = (operation: FileOperation) => {
  if (operation.status === 'running' || operation.status === 'paused') {
    // Cancel running or paused operation
    cancelOperation(operation.id);
  } else {
    // Close completed/failed/cancelled operation
    removeOperation(operation.id);
  }
};

onMounted(() => {
  // Listener is initialized automatically when needed
});

onUnmounted(() => {
  cleanup();
});
</script>

<style scoped>
.operations-progress {
  position: fixed;
  bottom: 0;
  right: 0;
  width: 450px;
  max-height: 80vh;
  z-index: 1000;
  pointer-events: none;
}

.operations-container {
  background: #2d2d2d;
  border-radius: 8px 8px 0 0;
  box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.3);
  overflow: hidden;
  pointer-events: all;
}

.operations-list {
  max-height: calc(80vh - 40px);
  overflow-y: auto;
}

.operation-item {
  padding: 16px;
  border-bottom: 1px solid #3d3d3d;
  background: #2d2d2d;
}

.operation-item:last-child {
  border-bottom: none;
}

.operation-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.operation-info {
  display: flex;
  gap: 12px;
  flex: 1;
  min-width: 0;
}

.operation-icon {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  color: #4a9eff;
}

.operation-icon svg {
  width: 100%;
  height: 100%;
}

.operation-copy .operation-icon {
  color: #4a9eff;
}

.operation-move .operation-icon {
  color: #ffa500;
}

.operation-delete .operation-icon {
  color: #ff4444;
}

.operation-text {
  flex: 1;
  min-width: 0;
}

.operation-title {
  font-size: 14px;
  font-weight: 600;
  color: #e0e0e0;
  margin-bottom: 4px;
}

.operation-file {
  font-size: 12px;
  color: #999;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.operation-buttons {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.action-btn {
  background: transparent;
  border: none;
  color: #999;
  cursor: pointer;
  padding: 4px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.pause-btn:hover {
  background: #ffa500;
  color: white;
}

.cancel-btn:hover {
  background: #ff4444;
  color: white;
}

.status-completed .cancel-btn:hover,
.status-cancelled .cancel-btn:hover {
  background: #4a9eff;
  color: white;
}

.action-btn svg {
  width: 16px;
  height: 16px;
}

.progress-bar-container {
  margin-bottom: 12px;
}

.progress-bar {
  height: 6px;
  background: #3d3d3d;
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #4a9eff 0%, #6bb6ff 100%);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.operation-copy .progress-fill {
  background: linear-gradient(90deg, #4a9eff 0%, #6bb6ff 100%);
}

.operation-move .progress-fill {
  background: linear-gradient(90deg, #ffa500 0%, #ffb733 100%);
}

.operation-delete .progress-fill {
  background: linear-gradient(90deg, #ff4444 0%, #ff6666 100%);
}

.operation-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  font-size: 12px;
}

.stat-item {
  display: flex;
  gap: 4px;
}

.stat-label {
  color: #999;
}

.stat-value {
  color: #e0e0e0;
  font-weight: 500;
}

.operation-error {
  margin-top: 12px;
  padding: 8px;
  background: rgba(255, 68, 68, 0.1);
  border-left: 3px solid #ff4444;
  color: #ff6666;
  font-size: 12px;
  border-radius: 4px;
}

.status-paused .progress-fill {
  background: linear-gradient(90deg, #ffa500 0%, #ffb733 100%);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.status-completed .progress-fill {
  background: linear-gradient(90deg, #44ff44 0%, #66ff66 100%);
}

.status-cancelled .progress-fill {
  background: linear-gradient(90deg, #999 0%, #bbb 100%);
}

.status-failed .progress-fill {
  background: linear-gradient(90deg, #ff4444 0%, #ff6666 100%);
}

/* Transitions */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(100%);
  opacity: 0;
}

.operation-list-enter-active,
.operation-list-leave-active {
  transition: all 0.3s ease;
}

.operation-list-enter-from,
.operation-list-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

.operation-list-move {
  transition: transform 0.3s ease;
}

/* Scrollbar styling */
.operations-list::-webkit-scrollbar {
  width: 8px;
}

.operations-list::-webkit-scrollbar-track {
  background: #1d1d1d;
}

.operations-list::-webkit-scrollbar-thumb {
  background: #4d4d4d;
  border-radius: 4px;
}

.operations-list::-webkit-scrollbar-thumb:hover {
  background: #5d5d5d;
}
</style>
