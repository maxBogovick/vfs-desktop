<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

interface Props {
  isOpen: boolean;
  title: string;
  label: string;
  defaultValue?: string;
  placeholder?: string;
}

interface Emits {
  (e: 'confirm', value: string): void;
  (e: 'cancel'): void;
}

const props = withDefaults(defineProps<Props>(), {
  defaultValue: '',
  placeholder: '',
});

const emit = defineEmits<Emits>();

const inputValue = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

// Reset and focus when dialog opens
watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    inputValue.value = props.defaultValue;
    await nextTick();
    inputRef.value?.focus();
    inputRef.value?.select();
  }
});

const handleConfirm = () => {
  if (inputValue.value.trim()) {
    emit('confirm', inputValue.value.trim());
  }
};

const handleCancel = () => {
  emit('cancel');
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    handleConfirm();
  } else if (event.key === 'Escape') {
    handleCancel();
  }
};
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-gray-400/30 flex items-center justify-center z-50"
    @click.self="handleCancel"
  >
    <div
      class="bg-[var(--vf-bg-primary)] border-2 border-t-white border-l-white border-r-[#404040] border-b-[#404040] shadow-lg"
      style="width: 400px"
      @click.stop
    >
      <!-- Title Bar -->
      <div class="bg-gradient-to-r from-[var(--vf-accent-hover)] to-[var(--vf-surface-hover)] px-2 py-1 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-white text-[11px] font-bold">{{ title }}</span>
        </div>
        <button
          @click="handleCancel"
          class="w-4 h-4 bg-[var(--vf-bg-primary)] border border-t-white border-l-white border-r-[#404040] border-b-[#404040] flex items-center justify-center text-[10px] hover:bg-[var(--vf-surface-hover)] active:border-t-[#404040] active:border-l-[#404040] active:border-r-white active:border-b-white"
        >
          ✕
        </button>
      </div>

      <!-- Content -->
      <div class="p-4">
        <label class="block text-[11px] mb-2">{{ label }}</label>
        <input
          ref="inputRef"
          v-model="inputValue"
          type="text"
          :placeholder="placeholder"
          @keydown="handleKeydown"
          class="w-full px-2 py-1 text-[11px] border border-[var(--vf-border-accent)] focus:outline-none focus:border-[var(--vf-accent-hover)]"
        />
      </div>

      <!-- Buttons -->
      <div class="px-4 pb-4 flex justify-end gap-2">
        <button
          @click="handleConfirm"
          class="px-4 py-1 text-[11px] bg-[var(--vf-bg-primary)] border-2 border-t-white border-l-white border-r-[#404040] border-b-[#404040] hover:bg-[var(--vf-surface-hover)] active:border-t-[#404040] active:border-l-[#404040] active:border-r-white active:border-b-white min-w-[75px]"
        >
          OK
        </button>
        <button
          @click="handleCancel"
          class="px-4 py-1 text-[11px] bg-[var(--vf-bg-primary)] border-2 border-t-white border-l-white border-r-[#404040] border-b-[#404040] hover:bg-[var(--vf-surface-hover)] active:border-t-[#404040] active:border-l-[#404040] active:border-r-white active:border-b-white min-w-[75px]"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
</template>
