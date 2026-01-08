<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import BaseWidget from '../../components/BaseWidget.vue';
import type { WidgetLayout } from '../../composables/useWidgets';

defineProps<{
  visible: boolean;
  id: string;
  layout: WidgetLayout;
}>();

defineEmits<{
  (e: 'close'): void;
  (e: 'update:layout', layout: Partial<WidgetLayout>): void;
}>();

const noteContent = ref('');

onMounted(() => {
  const saved = localStorage.getItem('vfdir-quick-notes');
  if (saved) {
    noteContent.value = saved;
  }
});

watch(noteContent, (newVal) => {
  localStorage.setItem('vfdir-quick-notes', newVal);
});
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="Quick Notes"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <div class="flex-1 p-0 bg-[var(--vf-bg-primary)] h-full">
      <textarea
        v-model="noteContent"
        @keydown.esc.stop="$emit('close')"
        class="w-full h-full p-2 bg-transparent resize-none focus:outline-none text-[var(--vf-text-primary)] font-mono text-xs placeholder-[var(--vf-text-tertiary)]"
        placeholder="Type your notes here..."
      ></textarea>
    </div>
  </BaseWidget>
</template>