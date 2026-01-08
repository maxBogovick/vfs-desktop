<script setup lang="ts">
import { ref, watch } from 'vue';

interface Props {
  visible: boolean;
  title: string;
  width?: string; // Tailwind class or arbitrary value e.g. "w-64"
  height?: string; // Tailwind class or arbitrary value e.g. "h-64" (optional)
  initialPosition?: { x: number; y: number };
}

const props = withDefaults(defineProps<Props>(), {
  width: 'w-64',
  initialPosition: () => ({ x: 200, y: 150 })
});

defineEmits<{
  (e: 'close'): void;
}>();

const position = ref(props.initialPosition);
const dragging = ref(false);
const offset = ref({ x: 0, y: 0 });

// Dragging logic
const startDrag = (e: MouseEvent) => {
  dragging.value = true;
  offset.value = {
    x: e.clientX - position.value.x,
    y: e.clientY - position.value.y
  };
  window.addEventListener('mousemove', onDrag);
  window.addEventListener('mouseup', stopDrag);
};

const onDrag = (e: MouseEvent) => {
  if (dragging.value) {
    position.value = {
      x: e.clientX - offset.value.x,
      y: e.clientY - offset.value.y
    };
  }
};

const stopDrag = () => {
  dragging.value = false;
  window.removeEventListener('mousemove', onDrag);
  window.removeEventListener('mouseup', stopDrag);
};
</script>

<template>
  <div
    v-if="visible"
    class="fixed z-50 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded shadow-xl flex flex-col text-sm font-sans"
    :class="[width, height]"
    :style="{ top: `${position.y}px`, left: `${position.x}px` }"
  >
    <!-- Header -->
    <div
      class="flex justify-between items-center px-3 py-2 bg-[var(--vf-surface-default)] cursor-move border-b border-[var(--vf-border-default)] rounded-t select-none"
      @mousedown="startDrag"
    >
      <span class="font-bold text-[var(--vf-text-primary)]">{{ title }}</span>
      <div class="flex gap-2">
        <!-- Optional Actions Slot -->
        <slot name="actions"></slot>
        
        <!-- Close Button -->
        <button 
          @click="$emit('close')" 
          class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-accent-danger)] transition-colors"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Content -->
    <slot></slot>
  </div>
</template>
