<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue';
import type { FileItem } from '../types';
import { useFileContentCache } from '../composables/useFileContentCache';

interface Props {
  file: FileItem | null;
  isOpen: boolean;
}

interface Emits {
  (e: 'close'): void;
  (e: 'save', content: string): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const { getFileContent } = useFileContentCache();
const editorContainer = ref<HTMLElement | null>(null);

const fileContent = ref<string>('');
const isLoading = ref(false);
const isSaving = ref(false);
const error = ref<string | null>(null);
const hasChanges = ref(false);
const originalContent = ref<string>('');

// Load file content when file changes
watch(() => props.file, async (newFile) => {
  if (!newFile) {
    fileContent.value = '';
    originalContent.value = '';
    hasChanges.value = false;
    error.value = null;
    return;
  }

  isLoading.value = true;
  error.value = null;

  try {
    const content = await getFileContent(newFile.path, 10_000_000); // 10MB limit
    fileContent.value = content;
    originalContent.value = content;
    hasChanges.value = false;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load file content';
    fileContent.value = '';
    originalContent.value = '';
  } finally {
    isLoading.value = false;
  }
}, { immediate: true });

// Track content changes
watch(fileContent, (newContent) => {
  if (!isLoading.value) {
    hasChanges.value = newContent !== originalContent.value;
  }
});

// Focus container when opened
watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    await nextTick();
    editorContainer.value?.focus();
  }
});

const handleSave = () => {
  emit('save', fileContent.value);
};

const handleClose = () => {
  if (hasChanges.value) {
    const confirmed = confirm('You have unsaved changes. Are you sure you want to close?');
    if (!confirmed) return;
  }
  emit('close');
};

// Handle keyboard shortcuts
const handleKeyDown = (event: KeyboardEvent) => {
  // Ctrl+S / Cmd+S to save
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault();
    handleSave();
  }
  // Escape to close
  if (event.key === 'Escape') {
    event.preventDefault();
    handleClose();
  }
};
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen && file"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="handleClose"
      @keydown.esc="handleClose"
    >
      <div
        ref="editorContainer"
        tabindex="-1"
        class="bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] rounded-lg shadow-2xl w-[90vw] h-[90vh] flex flex-col outline-none"
        @keydown="handleKeyDown"
      >
        <!-- Header -->
        <div class="flex items-center justify-between p-3 border-b border-[var(--vf-border-default)] bg-[var(--vf-bg-secondary)]">
          <div class="flex items-center gap-2">
            <span class="text-xl">📝</span>
            <div>
              <div class="font-semibold text-sm text-[var(--vf-text-primary)]">
                {{ file.name }}
                <span v-if="hasChanges" class="text-orange-500 ml-2">●</span>
              </div>
              <div class="text-[10px] text-[var(--vf-text-secondary)]">{{ file.path }}</div>
            </div>
          </div>
          <button
            @click="handleClose"
            class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] text-2xl leading-none w-8 h-8 flex items-center justify-center hover:bg-[var(--vf-surface-hover)] rounded"
          >
            ×
          </button>
        </div>

        <!-- Editor Content -->
        <div class="flex-1 overflow-hidden flex flex-col p-3">
          <!-- Loading State -->
          <div v-if="isLoading" class="flex-1 flex items-center justify-center">
            <div class="text-center">
              <div class="text-4xl mb-3">⏳</div>
              <div class="text-sm text-[var(--vf-text-secondary)]">Loading file...</div>
            </div>
          </div>

          <!-- Error State -->
          <div v-else-if="error" class="flex-1 flex items-center justify-center">
            <div class="text-center">
              <div class="text-4xl mb-3">⚠️</div>
              <div class="text-sm text-red-600 mb-2">{{ error }}</div>
              <button
                @click="handleClose"
                class="px-4 py-2 bg-[var(--vf-accent-primary)] text-white rounded text-sm hover:bg-[var(--vf-accent-hover)]"
              >
                Close
              </button>
            </div>
          </div>

          <!-- Textarea -->
          <textarea
            v-else
            v-model="fileContent"
            class="flex-1 w-full p-4 border border-[var(--vf-border-default)] rounded bg-[var(--vf-surface-default)] text-[var(--vf-text-primary)] font-mono text-sm resize-none focus:outline-none focus:ring-2 focus:ring-[var(--vf-accent-primary)]"
            spellcheck="false"
            :disabled="isSaving"
          />
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-between p-3 border-t border-[var(--vf-border-default)] bg-[var(--vf-bg-secondary)]">
          <div class="text-[11px] text-[var(--vf-text-secondary)]">
            <span v-if="hasChanges" class="text-orange-500">● Unsaved changes</span>
            <span v-else class="text-green-600">✓ No changes</span>
            <span class="ml-4">Press Ctrl+S to save, Esc to close</span>
          </div>
          <div class="flex gap-2">
            <button
              @click="handleClose"
              class="px-4 py-2 border border-[var(--vf-border-default)] rounded text-sm hover:bg-[var(--vf-surface-hover)] text-[var(--vf-text-primary)]"
              :disabled="isSaving"
            >
              Cancel
            </button>
            <button
              @click="handleSave"
              class="px-4 py-2 bg-[var(--vf-accent-primary)] text-white rounded text-sm hover:bg-[var(--vf-accent-hover)] disabled:opacity-50 disabled:cursor-not-allowed"
              :disabled="isSaving || !hasChanges"
            >
              <span v-if="isSaving">Saving...</span>
              <span v-else>Save</span>
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
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
