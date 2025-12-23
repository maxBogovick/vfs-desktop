<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue';
import type { FileItem } from '../types';
import { useFileContentCache } from '../composables/useFileContentCache';

interface Props {
  file: FileItem | null;
}

interface Emits {
  (e: 'close'): void;
  (e: 'open', file: FileItem): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const { getFileContent } = useFileContentCache();

const fileContent = ref<string | null>(null);
const isLoadingContent = ref(false);
const contentError = ref<string | null>(null);

// Функция очистки контента для освобождения памяти
const cleanupContent = () => {
  fileContent.value = null;
  contentError.value = null;
  isLoadingContent.value = false;
};

// Очищаем при unmount
onUnmounted(() => {
  cleanupContent();
});

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

const getTagColor = (tag: string) => {
  const colors: Record<string, string> = {
    work: 'bg-blue-400',
    urgent: 'bg-red-400',
    finance: 'bg-green-500',
    dev: 'bg-purple-400',
    design: 'bg-pink-400',
  };
  return colors[tag] || 'bg-gray-400';
};

const canPreview = (item: FileItem | null): boolean => {
  if (!item) return false;
  return item.type === 'image' || item.type === 'code' || item.type === 'file';
};

const getImageMimeType = (fileName: string): string => {
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  const mimeTypes: Record<string, string> = {
    jpg: 'image/jpeg',
    jpeg: 'image/jpeg',
    png: 'image/png',
    gif: 'image/gif',
    webp: 'image/webp',
    bmp: 'image/bmp',
    svg: 'image/svg+xml',
  };
  return mimeTypes[ext] || 'image/jpeg';
};

watch(() => props.file, async (newFile, oldFile) => {
  // Очищаем предыдущий контент для освобождения памяти
  cleanupContent();

  if (!newFile || !canPreview(newFile)) {
    return;
  }

  isLoadingContent.value = true;

  try {
    // Используем кеш для загрузки файла
    const content = await getFileContent(newFile.path, 5_000_000); // 5MB limit
    fileContent.value = content;
  } catch (err) {
    contentError.value = err instanceof Error ? err.message : 'Failed to load file content';
  } finally {
    isLoadingContent.value = false;
  }
}, { immediate: true });
</script>

<template>
  <transition name="slide-right">
    <div
      v-if="file"
      class="w-72 bg-gradient-to-b from-[#C1D9F4] to-[#A5C8E1] border-l border-[#919B9C] flex flex-col overflow-hidden"
    >
      <!-- Header -->
      <div class="flex justify-between items-center p-3 border-b border-[#8BA5C7]">
        <div class="text-[12px] font-bold text-[#003D7A]">File Preview</div>
        <button
          @click="emit('close')"
          class="text-[#003D7A] hover:text-[#0054E3] text-lg leading-none w-5 h-5 flex items-center justify-center hover:bg-white/30 rounded"
        >
          ✕
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-3">
        <!-- File Content Preview -->
        <div class="bg-white rounded-lg p-4 mb-3 shadow-inner">
          <!-- Loading state -->
          <div v-if="isLoadingContent" class="w-full h-32 flex items-center justify-center">
            <div class="text-center">
              <div class="text-3xl mb-2">⏳</div>
              <div class="text-xs text-gray-500">Loading preview...</div>
            </div>
          </div>

          <!-- Error state -->
          <div v-else-if="contentError" class="w-full h-32 flex items-center justify-center">
            <div class="text-center">
              <div class="text-3xl mb-2">⚠️</div>
              <div class="text-xs text-red-600">{{ contentError }}</div>
            </div>
          </div>

          <!-- Image preview -->
          <div v-else-if="file.type === 'image' && fileContent" class="w-full">
            <img
              :src="`data:${getImageMimeType(file.name)};base64,${fileContent}`"
              :alt="file.name"
              class="w-full h-auto rounded"
            />
          </div>

          <!-- Text/Code preview -->
          <div v-else-if="(file.type === 'code' || file.type === 'file') && fileContent" class="w-full">
            <pre class="text-[10px] font-mono whitespace-pre-wrap break-words max-h-64 overflow-y-auto bg-gray-50 p-2 rounded border border-gray-200">{{ fileContent }}</pre>
          </div>

          <!-- Icon fallback (for non-previewable files) -->
          <div v-else class="w-full h-32 flex items-center justify-center text-6xl">
            {{ getFileIcon(file) }}
          </div>
        </div>

        <!-- File Info -->
        <div class="space-y-3">
          <!-- Name -->
          <div class="bg-white/20 rounded p-2">
            <div class="text-[9px] text-[#003D7A] font-bold mb-1 uppercase tracking-wide">Name</div>
            <div class="text-[11px] text-[#003D7A] font-bold break-words">{{ file.name }}</div>
          </div>

          <!-- Type -->
          <div class="bg-white/20 rounded p-2">
            <div class="text-[9px] text-[#003D7A] font-bold mb-1 uppercase tracking-wide">Type</div>
            <div class="text-[11px] text-[#003D7A]">{{ file.type }}</div>
          </div>

          <!-- Size -->
          <div v-if="file.sizeFormatted" class="bg-white/20 rounded p-2">
            <div class="text-[9px] text-[#003D7A] font-bold mb-1 uppercase tracking-wide">Size</div>
            <div class="text-[11px] text-[#003D7A]">{{ file.sizeFormatted }}</div>
          </div>

          <!-- Modified Date -->
          <div v-if="file.modified" class="bg-white/20 rounded p-2">
            <div class="text-[9px] text-[#003D7A] font-bold mb-1 uppercase tracking-wide">Modified</div>
            <div class="text-[11px] text-[#003D7A]">{{ file.modified }}</div>
          </div>

          <!-- Created Date -->
          <div v-if="file.created" class="bg-white/20 rounded p-2">
            <div class="text-[9px] text-[#003D7A] font-bold mb-1 uppercase tracking-wide">Created</div>
            <div class="text-[11px] text-[#003D7A]">{{ file.created }}</div>
          </div>

          <!-- Path -->
          <div class="bg-white/20 rounded p-2">
            <div class="text-[9px] text-[#003D7A] font-bold mb-1 uppercase tracking-wide">Location</div>
            <div class="text-[10px] text-[#003D7A] break-all">{{ file.path }}</div>
          </div>

          <!-- Tags -->
          <div v-if="file.tags && file.tags.length > 0" class="bg-white/20 rounded p-2">
            <div class="text-[9px] text-[#003D7A] font-bold mb-2 uppercase tracking-wide">Tags</div>
            <div class="flex flex-wrap gap-1.5">
              <span
                v-for="tag in file.tags"
                :key="tag"
                :class="`${getTagColor(tag)} text-white px-2 py-1 rounded-full text-[10px] shadow-sm font-medium`"
              >
                {{ tag }}
              </span>
            </div>
          </div>

          <!-- Permissions -->
          <div v-if="file.permissions" class="bg-white/20 rounded p-2">
            <div class="text-[9px] text-[#003D7A] font-bold mb-2 uppercase tracking-wide">Permissions</div>
            <div class="flex gap-2 text-[10px]">
              <span :class="file.permissions.readable ? 'text-green-700' : 'text-gray-400'">
                {{ file.permissions.readable ? '✓' : '✗' }} Read
              </span>
              <span :class="file.permissions.writable ? 'text-green-700' : 'text-gray-400'">
                {{ file.permissions.writable ? '✓' : '✗' }} Write
              </span>
              <span :class="file.permissions.executable ? 'text-green-700' : 'text-gray-400'">
                {{ file.permissions.executable ? '✓' : '✗' }} Execute
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="p-3 border-t border-[#8BA5C7] bg-white/10">
        <div class="grid grid-cols-2 gap-2">
          <button
            @click="emit('open', file)"
            class="bg-[#0A246A] text-white py-2 rounded text-xs font-bold hover:bg-[#0054E3] shadow-md transition-colors"
          >
            Open
          </button>
          <button
            class="bg-white/90 border border-[#8BA5C7] text-[#003D7A] py-2 rounded text-xs font-bold hover:bg-white transition-colors"
          >
            Properties
          </button>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.slide-right-enter-active,
.slide-right-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-right-enter-from,
.slide-right-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
