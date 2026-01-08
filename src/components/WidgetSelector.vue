<script setup lang="ts">
import { useWidgets } from '../composables/useWidgets';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const { widgets, toggleWidget } = useWidgets();
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[100] flex items-center justify-center bg-black bg-opacity-50" @click.self="$emit('close')">
    <div class="bg-[var(--vf-bg-surface)] border border-[var(--vf-border-default)] rounded-lg shadow-2xl w-[400px] flex flex-col max-h-[80vh]">
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--vf-border-default)] bg-[var(--vf-bg-secondary)]">
        <h3 class="font-semibold text-[var(--vf-text-primary)]">Widget Library</h3>
        <button
          @click="$emit('close')"
          class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] text-xl leading-none"
        >
          ×
        </button>
      </div>

      <!-- Content -->
      <div class="p-4 overflow-y-auto bg-[var(--vf-bg-primary)]">
        <div class="space-y-3">
          <div
            v-for="widget in widgets"
            :key="widget.id"
            class="flex items-start gap-3 p-3 border border-[var(--vf-border-default)] rounded hover:bg-[var(--vf-surface-hover)] transition-colors"
          >
            <input
              type="checkbox"
              :id="`widget-${widget.id}`"
              :checked="widget.active"
              @change="toggleWidget(widget.id)"
              class="mt-1"
            />
            <label :for="`widget-${widget.id}`" class="flex-1 cursor-pointer select-none">
              <div class="font-medium text-[var(--vf-text-primary)]">{{ widget.name }}</div>
              <div class="text-xs text-[var(--vf-text-secondary)] mt-0.5">{{ widget.description }}</div>
            </label>
          </div>
        </div>

        <div v-if="widgets.length === 0" class="text-center text-[var(--vf-text-tertiary)] py-8">
          No widgets available.
        </div>
      </div>
      
      <!-- Footer -->
      <div class="px-4 py-3 border-t border-[var(--vf-border-default)] bg-[var(--vf-bg-secondary)] flex justify-end">
        <button
          @click="$emit('close')"
          class="px-4 py-1.5 bg-[var(--vf-surface-default)] hover:bg-[var(--vf-surface-hover)] border border-[var(--vf-border-default)] rounded text-sm text-[var(--vf-text-primary)] transition-colors"
        >
          Done
        </button>
      </div>
    </div>
  </div>
</template>
