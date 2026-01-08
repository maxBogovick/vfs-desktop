<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import BaseWidget from './BaseWidget.vue';

defineProps<{
  visible: boolean;
}>();

defineEmits<{
  (e: 'close'): void;
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
    title="Quick Notes"
    width="w-64"
    height="h-64"
    :initial-position="{ x: 600, y: 150 }"
    @close="$emit('close')"
  >
    <div class="flex-1 p-0 bg-[var(--vf-bg-primary)]">
      <textarea
        v-model="noteContent"
        @keydown.esc.stop="$emit('close')"
        class="w-full h-full p-2 bg-transparent resize-none focus:outline-none text-[var(--vf-text-primary)] font-mono text-xs placeholder-[var(--vf-text-tertiary)]"
        placeholder="Type your notes here..."
      ></textarea>
    </div>
  </BaseWidget>
</template>
