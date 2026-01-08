<script setup lang="ts">
import { ref, computed } from 'vue';
import type { WidgetLayout } from '../composables/useWidgets';

const props = defineProps<{
  id: string; // Widget ID for saving state
  title: string;
  layout: WidgetLayout;
  visible: boolean; // Managed by parent (WidgetLayer) based on active state
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'update:layout', layout: Partial<WidgetLayout>): void;
}>();

const dragging = ref(false);
const resizing = ref(false);
const dragOffset = ref({ x: 0, y: 0 });
const resizeStart = ref({ w: 0, h: 0, x: 0, y: 0 });

// --- Dragging ---
const startDrag = (e: MouseEvent) => {
  if (resizing.value) return; // Don't drag if resizing
  dragging.value = true;
  dragOffset.value = {
    x: e.clientX - props.layout.x,
    y: e.clientY - props.layout.y
  };
  window.addEventListener('mousemove', onDrag);
  window.addEventListener('mouseup', stopDrag);
};

const onDrag = (e: MouseEvent) => {
  if (dragging.value) {
    emit('update:layout', {
      x: e.clientX - dragOffset.value.x,
      y: e.clientY - dragOffset.value.y
    });
  }
};

const stopDrag = () => {
  dragging.value = false;
  window.removeEventListener('mousemove', onDrag);
  window.removeEventListener('mouseup', stopDrag);
};

// --- Resizing ---
const startResize = (e: MouseEvent) => {
  e.stopPropagation(); // Prevent drag start
  resizing.value = true;
  resizeStart.value = {
    w: props.layout.width,
    h: props.layout.height,
    x: e.clientX,
    y: e.clientY
  };
  window.addEventListener('mousemove', onResize);
  window.addEventListener('mouseup', stopResize);
};

const onResize = (e: MouseEvent) => {
  if (resizing.value) {
    const deltaX = e.clientX - resizeStart.value.x;
    const deltaY = e.clientY - resizeStart.value.y;
    
    emit('update:layout', {
      width: Math.max(200, resizeStart.value.w + deltaX), // Min width 200
      height: Math.max(150, resizeStart.value.h + deltaY) // Min height 150
    });
  }
};

const stopResize = () => {
  resizing.value = false;
  window.removeEventListener('mousemove', onResize);
  window.removeEventListener('mouseup', stopResize);
};

const toggleMinimize = () => {
  emit('update:layout', { minimized: !props.layout.minimized });
};
</script>

<template>
  <div
    v-show="visible && !layout.minimized"
    class="fixed z-50 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded shadow-xl flex flex-col text-sm font-sans overflow-hidden"
    :style="{ 
      top: `${layout.y}px`, 
      left: `${layout.x}px`,
      width: `${layout.width}px`,
      height: `${layout.height}px`
    }"
  >
    <!-- Header -->
    <div
      class="flex justify-between items-center px-3 py-2 bg-[var(--vf-surface-default)] cursor-move border-b border-[var(--vf-border-default)] select-none shrink-0"
      @mousedown="startDrag"
    >
      <span class="font-bold text-[var(--vf-text-primary)] truncate mr-2">{{ title }}</span>
      <div class="flex gap-1 items-center">
        <!-- Optional Actions Slot -->
        <slot name="actions"></slot>
        
        <!-- Minimize Button -->
        <button 
          @click.stop="toggleMinimize" 
          class="w-5 h-5 flex items-center justify-center text-[var(--vf-text-secondary)] hover:bg-[var(--vf-surface-hover)] rounded transition-colors"
          title="Minimize"
        >
          _
        </button>

        <!-- Close Button -->
        <button 
          @click.stop="$emit('close')" 
          class="w-5 h-5 flex items-center justify-center text-[var(--vf-text-secondary)] hover:bg-red-100 hover:text-red-500 rounded transition-colors"
          title="Close"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-auto relative">
      <slot></slot>
    </div>

    <!-- Resize Handle -->
    <div 
      class="absolute bottom-0 right-0 w-4 h-4 cursor-nwse-resize flex items-end justify-end p-0.5 opacity-50 hover:opacity-100 z-10"
      @mousedown="startResize"
    >
      <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
        <path d="M8 8H0L8 0V8Z" fill="var(--vf-border-default)"/>
      </svg>
    </div>
  </div>
</template>