<script setup lang="ts">
import type { FileItem } from '../types';

interface Props {
  isOpen: boolean;
  file: FileItem | null;
}

interface Emits {
  (e: 'close'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const getFileIcon = (item: FileItem) => {
  const icons: Record<string, string> = {
    drive: '💾',
    folder: '📁',
    image: '🖼️',
    pdf: '📄',
    code: '📜',
    file: '📄',
    system: '⚙️',
    video: '🎬',
    audio: '🎵',
    archive: '📦',
  };
  return icons[item.type] || '📄';
};

const getFileTypeName = (type: string) => {
  const names: Record<string, string> = {
    folder: 'File Folder',
    drive: 'Local Disk',
    image: 'Image File',
    pdf: 'PDF Document',
    code: 'Source Code',
    file: 'File',
    system: 'System Folder',
    video: 'Video File',
    audio: 'Audio File',
    archive: 'Archive',
  };
  return names[type] || 'File';
};
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen && file"
      @click="emit('close')"
      class="fixed inset-0 bg-black/30 z-[60] flex items-center justify-center"
    >
      <div
        @click.stop
        class="bg-[#ECE9D8] rounded border-2 border-[#0054E3] shadow-2xl w-[450px] overflow-hidden animate-pop-in"
      >
        <!-- Title Bar -->
        <div class="bg-gradient-to-r from-[#0054E3] to-[#0A246A] h-7 flex items-center px-2 gap-2">
          <div class="w-4 h-4 flex items-center justify-center text-xs">
            {{ getFileIcon(file) }}
          </div>
          <div class="flex-1 text-white font-bold text-xs truncate">{{ file.name }} Properties</div>
          <button
            @click="emit('close')"
            class="w-5 h-5 bg-[#C1D2EE] hover:bg-[#FF4444] flex items-center justify-center text-[10px] font-bold border border-white/30"
          >
            ✕
          </button>
        </div>

        <!-- Tabs -->
        <div class="bg-[#F1EFE2] border-b border-[#919B9C] px-2 pt-1">
          <div class="bg-white border-t-2 border-l-2 border-r-2 border-[#919B9C] px-4 py-1 inline-block rounded-t text-xs font-bold">
            General
          </div>
        </div>

        <!-- Content -->
        <div class="p-4 space-y-4">
          <!-- Icon and Name -->
          <div class="flex items-start gap-4 pb-4 border-b border-[#919B9C]">
            <div class="w-16 h-16 flex items-center justify-center text-5xl bg-white border border-[#919B9C] rounded">
              {{ getFileIcon(file) }}
            </div>
            <div class="flex-1 pt-2">
              <input
                type="text"
                :value="file.name"
                readonly
                class="w-full px-2 py-1 border border-[#7F9DB9] text-sm font-bold bg-white"
              />
            </div>
          </div>

          <!-- Properties Table -->
          <div class="space-y-2">
            <!-- Type -->
            <div class="flex items-center gap-2">
              <div class="w-24 text-xs text-gray-600">Type:</div>
              <div class="flex-1 text-xs font-bold">{{ getFileTypeName(file.type) }}</div>
            </div>

            <!-- Location -->
            <div class="flex items-center gap-2">
              <div class="w-24 text-xs text-gray-600">Location:</div>
              <div class="flex-1 text-xs break-all">{{ file.path }}</div>
            </div>

            <!-- Size -->
            <div v-if="file.sizeFormatted" class="flex items-center gap-2">
              <div class="w-24 text-xs text-gray-600">Size:</div>
              <div class="flex-1 text-xs">
                {{ file.sizeFormatted }}
                <span v-if="file.size" class="text-gray-500">({{ file.size.toLocaleString() }} bytes)</span>
              </div>
            </div>

            <div class="border-t border-[#919B9C] my-2"></div>

            <!-- Created -->
            <div v-if="file.created" class="flex items-center gap-2">
              <div class="w-24 text-xs text-gray-600">Created:</div>
              <div class="flex-1 text-xs">{{ file.created }}</div>
            </div>

            <!-- Modified -->
            <div v-if="file.modified" class="flex items-center gap-2">
              <div class="w-24 text-xs text-gray-600">Modified:</div>
              <div class="flex-1 text-xs">{{ file.modified }}</div>
            </div>

            <!-- Accessed -->
            <div v-if="file.accessed" class="flex items-center gap-2">
              <div class="w-24 text-xs text-gray-600">Accessed:</div>
              <div class="flex-1 text-xs">{{ file.accessed }}</div>
            </div>

            <div v-if="file.permissions" class="border-t border-[#919B9C] my-2"></div>

            <!-- Permissions -->
            <div v-if="file.permissions" class="space-y-1">
              <div class="text-xs text-gray-600 mb-1">Permissions:</div>
              <div class="flex gap-4 text-xs ml-24">
                <label class="flex items-center gap-1">
                  <input type="checkbox" :checked="file.permissions.readable" disabled class="w-3 h-3" />
                  Read
                </label>
                <label class="flex items-center gap-1">
                  <input type="checkbox" :checked="file.permissions.writable" disabled class="w-3 h-3" />
                  Write
                </label>
                <label class="flex items-center gap-1">
                  <input type="checkbox" :checked="file.permissions.executable" disabled class="w-3 h-3" />
                  Execute
                </label>
              </div>
            </div>

            <!-- Tags -->
            <div v-if="file.tags && file.tags.length > 0" class="border-t border-[#919B9C] my-2"></div>
            <div v-if="file.tags && file.tags.length > 0" class="flex items-start gap-2">
              <div class="w-24 text-xs text-gray-600">Tags:</div>
              <div class="flex-1 flex flex-wrap gap-1">
                <span
                  v-for="tag in file.tags"
                  :key="tag"
                  class="px-2 py-0.5 bg-blue-100 text-blue-800 rounded text-[10px] font-medium"
                >
                  {{ tag }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer Buttons -->
        <div class="bg-[#F1EFE2] border-t border-[#919B9C] p-3 flex justify-end gap-2">
          <button
            @click="emit('close')"
            class="px-4 py-1.5 bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] rounded text-xs font-['Tahoma'] min-w-[75px]"
          >
            OK
          </button>
          <button
            @click="emit('close')"
            class="px-4 py-1.5 bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] rounded text-xs font-['Tahoma'] min-w-[75px]"
          >
            Cancel
          </button>
          <button
            class="px-4 py-1.5 bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] rounded text-xs font-['Tahoma'] min-w-[75px]"
          >
            Apply
          </button>
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
  animation: popIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes popIn {
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
