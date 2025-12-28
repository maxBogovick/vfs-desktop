<script setup lang="ts">
import { ref, computed } from 'vue';
import type { FileConflict, ConflictAction, ConflictResolution } from '../types';

interface Props {
  isOpen: boolean;
  conflict: FileConflict | null;
}

interface Emits {
  (e: 'resolve', resolution: ConflictResolution): void;
  (e: 'cancel'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const selectedAction = ref<ConflictAction>('skip');
const applyToAll = ref(false);
const newName = ref('');

// Format file size
const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
};

// Format date
const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
};

// Determine which file is newer
const newerFile = computed(() => {
  if (!props.conflict) return null;
  return props.conflict.sourceFile.modified > props.conflict.destinationFile.modified
    ? 'source'
    : 'destination';
});

// Initialize newName when rename is selected
const handleActionChange = (action: ConflictAction) => {
  selectedAction.value = action;
  if (action === 'rename' && props.conflict) {
    // Generate default new name (add number suffix)
    const name = props.conflict.sourceFile.name;
    const dotIndex = name.lastIndexOf('.');
    if (dotIndex > 0) {
      const baseName = name.substring(0, dotIndex);
      const extension = name.substring(dotIndex);
      newName.value = `${baseName} (copy)${extension}`;
    } else {
      newName.value = `${name} (copy)`;
    }
  }
};

// Handle confirm
const handleConfirm = () => {
  const resolution: ConflictResolution = {
    action: selectedAction.value,
    applyToAll: applyToAll.value,
  };

  if (selectedAction.value === 'rename') {
    resolution.newName = newName.value || undefined;
  }

  emit('resolve', resolution);

  // Reset state
  selectedAction.value = 'skip';
  applyToAll.value = false;
  newName.value = '';
};

// Handle cancel
const handleCancel = () => {
  emit('cancel');

  // Reset state
  selectedAction.value = 'skip';
  applyToAll.value = false;
  newName.value = '';
};
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen && conflict"
      @click="handleCancel"
      class="fixed inset-0 bg-black/30 z-[60] flex items-center justify-center"
    >
      <div
        @click.stop
        class="bg-[#ECE9D8] rounded border-2 border-[#0054E3] shadow-2xl w-[600px] overflow-hidden animate-pop-in"
      >
        <!-- Title Bar -->
        <div class="bg-gradient-to-r from-[#0054E3] to-[#0A246A] h-7 flex items-center px-2 gap-2">
          <div class="w-4 h-4 flex items-center justify-center text-xs">⚠️</div>
          <div class="flex-1 text-white font-bold text-xs">File Conflict</div>
          <button
            @click="handleCancel"
            class="w-5 h-5 bg-[#C1D2EE] hover:bg-[#FF4444] flex items-center justify-center text-[10px] font-bold border border-white/30"
          >
            ✕
          </button>
        </div>

        <!-- Content -->
        <div class="p-4">
          <!-- Message -->
          <div class="mb-4">
            <p class="text-sm font-bold mb-2">A file with this name already exists:</p>
            <p class="text-sm text-gray-700 font-mono bg-white p-2 rounded border border-gray-300">
              {{ conflict.destinationFile.name }}
            </p>
          </div>

          <!-- File Comparison -->
          <div class="grid grid-cols-2 gap-4 mb-4">
            <!-- Source File -->
            <div class="bg-white border-2 border-blue-400 rounded p-3">
              <div class="flex items-center gap-2 mb-2">
                <div class="text-2xl">📄</div>
                <div class="flex-1">
                  <div class="text-xs font-bold text-blue-600">New File</div>
                  <div
                    v-if="newerFile === 'source'"
                    class="text-[10px] text-green-600 font-bold"
                  >
                    ⬆ Newer
                  </div>
                </div>
              </div>
              <div class="space-y-1 text-xs">
                <div>
                  <span class="text-gray-600">Size:</span>
                  <span class="ml-1 font-mono">{{ formatSize(conflict.sourceFile.size) }}</span>
                </div>
                <div>
                  <span class="text-gray-600">Modified:</span>
                  <span class="ml-1 text-[11px]">{{ formatDate(conflict.sourceFile.modified) }}</span>
                </div>
              </div>
            </div>

            <!-- Destination File -->
            <div class="bg-white border-2 border-gray-400 rounded p-3">
              <div class="flex items-center gap-2 mb-2">
                <div class="text-2xl">📄</div>
                <div class="flex-1">
                  <div class="text-xs font-bold text-gray-600">Existing File</div>
                  <div
                    v-if="newerFile === 'destination'"
                    class="text-[10px] text-green-600 font-bold"
                  >
                    ⬆ Newer
                  </div>
                </div>
              </div>
              <div class="space-y-1 text-xs">
                <div>
                  <span class="text-gray-600">Size:</span>
                  <span class="ml-1 font-mono">{{ formatSize(conflict.destinationFile.size) }}</span>
                </div>
                <div>
                  <span class="text-gray-600">Modified:</span>
                  <span class="ml-1 text-[11px]">{{ formatDate(conflict.destinationFile.modified) }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Actions -->
          <div class="mb-4 space-y-2">
            <p class="text-sm font-bold mb-2">Choose an action:</p>

            <!-- Skip -->
            <label class="flex items-center gap-2 p-2 bg-white rounded border border-gray-300 hover:border-blue-400 cursor-pointer">
              <input
                type="radio"
                name="action"
                value="skip"
                v-model="selectedAction"
                class="w-4 h-4"
              />
              <div class="flex-1">
                <div class="text-sm font-bold">Skip this file</div>
                <div class="text-xs text-gray-600">Don't copy, keep existing file</div>
              </div>
            </label>

            <!-- Replace -->
            <label class="flex items-center gap-2 p-2 bg-white rounded border border-gray-300 hover:border-blue-400 cursor-pointer">
              <input
                type="radio"
                name="action"
                value="replace"
                v-model="selectedAction"
                class="w-4 h-4"
              />
              <div class="flex-1">
                <div class="text-sm font-bold">Replace existing file</div>
                <div class="text-xs text-gray-600">Overwrite with new file</div>
              </div>
            </label>

            <!-- Rename -->
            <label class="flex items-start gap-2 p-2 bg-white rounded border border-gray-300 hover:border-blue-400 cursor-pointer">
              <input
                type="radio"
                name="action"
                value="rename"
                @change="handleActionChange('rename')"
                v-model="selectedAction"
                class="w-4 h-4 mt-0.5"
              />
              <div class="flex-1">
                <div class="text-sm font-bold mb-1">Keep both (rename new file)</div>
                <div v-if="selectedAction === 'rename'" class="mt-2">
                  <input
                    type="text"
                    v-model="newName"
                    @click.stop
                    placeholder="Enter new name"
                    class="w-full px-2 py-1 text-xs border border-gray-400 rounded focus:outline-none focus:border-blue-500"
                  />
                </div>
                <div v-else class="text-xs text-gray-600">Copy file with a different name</div>
              </div>
            </label>

            <!-- Compare (future feature) -->
            <label class="flex items-center gap-2 p-2 bg-gray-100 rounded border border-gray-300 opacity-50 cursor-not-allowed">
              <input
                type="radio"
                name="action"
                value="compare"
                disabled
                class="w-4 h-4"
              />
              <div class="flex-1">
                <div class="text-sm font-bold">Compare files</div>
                <div class="text-xs text-gray-600">View differences side-by-side (coming soon)</div>
              </div>
            </label>
          </div>

          <!-- Apply to all checkbox -->
          <div class="mb-4 p-2 bg-yellow-50 border border-yellow-300 rounded">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                v-model="applyToAll"
                class="w-4 h-4"
              />
              <span class="text-sm font-bold">Apply this action to all conflicts</span>
            </label>
            <p class="text-xs text-gray-600 ml-6">
              This will automatically apply your choice to all remaining file conflicts
            </p>
          </div>

          <!-- Buttons -->
          <div class="flex justify-end gap-2">
            <button
              @click="handleCancel"
              class="px-4 py-1.5 bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] rounded text-xs min-w-[75px]"
            >
              Cancel
            </button>
            <button
              @click="handleConfirm"
              :disabled="selectedAction === 'rename' && !newName.trim()"
              :class="[
                'px-4 py-1.5 border-2 rounded text-xs font-bold min-w-[75px]',
                selectedAction === 'rename' && !newName.trim()
                  ? 'bg-gray-300 border-gray-400 cursor-not-allowed'
                  : 'bg-gradient-to-b from-[#EFF3FF] to-[#C1D2EE] border-[#003C74] hover:from-[#C1D2EE] hover:to-[#0054E3] hover:text-white active:bg-[#0A246A]',
              ]"
            >
              Continue
            </button>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.animate-pop-in {
  animation: pop-in 0.2s ease-out;
}

@keyframes pop-in {
  0% {
    transform: scale(0.9);
    opacity: 0;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
