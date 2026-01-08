<script setup lang="ts">
import { ref, computed } from 'vue';
import { useWidgets } from '../composables/useWidgets';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const { widgets, toggleWidget } = useWidgets();
const searchQuery = ref('');

const filteredWidgets = computed(() => {
  if (!searchQuery.value) return widgets.value;
  const query = searchQuery.value.toLowerCase();
  return widgets.value.filter(w => 
    w.name.toLowerCase().includes(query) || 
    w.description.toLowerCase().includes(query)
  );
});
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm" @click.self="$emit('close')">
    <div class="bg-[var(--vf-bg-surface)] border border-[var(--vf-border-default)] rounded-xl shadow-2xl w-[600px] flex flex-col max-h-[85vh] overflow-hidden animate-in fade-in zoom-in-95 duration-200">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-[var(--vf-border-default)] bg-[var(--vf-bg-secondary)] flex items-center justify-between">
        <div>
          <h3 class="font-bold text-lg text-[var(--vf-text-primary)]">Widgets</h3>
          <p class="text-xs text-[var(--vf-text-secondary)] mt-0.5">Customize your workspace tools</p>
        </div>
        <button
          @click="$emit('close')"
          class="w-8 h-8 flex items-center justify-center rounded-full hover:bg-[var(--vf-surface-hover)] text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] transition-colors"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>

      <!-- Search Bar -->
      <div class="px-5 py-3 bg-[var(--vf-bg-primary)] border-b border-[var(--vf-border-subtle)]">
        <div class="relative">
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--vf-text-tertiary)]" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search widgets..."
            class="w-full pl-10 pr-4 py-2 bg-[var(--vf-bg-surface)] border border-[var(--vf-border-default)] rounded-lg text-sm text-[var(--vf-text-primary)] placeholder-[var(--vf-text-disabled)] focus:outline-none focus:border-[var(--vf-accent-primary)] transition-colors"
            autoFocus
          />
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto bg-[var(--vf-bg-primary)] p-5">
        <div v-if="filteredWidgets.length === 0" class="flex flex-col items-center justify-center py-12 text-[var(--vf-text-tertiary)]">
          <svg class="mb-3 opacity-50" xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
          <p>No widgets found matching "{{ searchQuery }}"</p>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div
            v-for="widget in filteredWidgets"
            :key="widget.id"
            @click="toggleWidget(widget.id)"
            class="group relative flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-all duration-200"
            :class="[
              widget.active 
                ? 'bg-[var(--vf-surface-selected)] border-[var(--vf-accent-primary)] ring-1 ring-[var(--vf-accent-primary)] shadow-sm' 
                : 'bg-[var(--vf-surface-default)] border-[var(--vf-border-default)] hover:border-[var(--vf-accent-primary)] hover:shadow-md'
            ]"
          >
            <!-- Toggle Checkbox (Visual) -->
            <div 
              class="mt-1 flex-shrink-0 w-5 h-5 rounded border flex items-center justify-center transition-colors"
              :class="widget.active ? 'bg-[var(--vf-accent-primary)] border-[var(--vf-accent-primary)]' : 'bg-[var(--vf-bg-surface)] border-[var(--vf-border-default)]'"
            >
              <svg v-if="widget.active" class="text-white w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
            </div>

            <!-- Info -->
            <div class="flex-1 min-w-0">
              <div class="font-semibold text-sm text-[var(--vf-text-primary)] mb-0.5 truncate">{{ widget.name }}</div>
              <div class="text-xs text-[var(--vf-text-secondary)] line-clamp-2 leading-relaxed">{{ widget.description }}</div>
            </div>
            
            <!-- Status Indicator -->
            <div 
              class="absolute top-3 right-3 w-2 h-2 rounded-full"
              :class="widget.active ? 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]' : 'bg-transparent'"
            ></div>
          </div>
        </div>
      </div>
      
      <!-- Footer -->
      <div class="px-5 py-3 border-t border-[var(--vf-border-default)] bg-[var(--vf-bg-secondary)] flex justify-between items-center text-xs text-[var(--vf-text-tertiary)]">
        <span>{{ widgets.filter(w => w.active).length }} active widgets</span>
        <button
          @click="$emit('close')"
          class="px-6 py-2 bg-[var(--vf-accent-primary)] hover:bg-[var(--vf-accent-hover)] text-white rounded font-medium transition-colors shadow-sm"
        >
          Done
        </button>
      </div>
    </div>
  </div>
</template>