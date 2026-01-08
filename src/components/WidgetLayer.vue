<script setup lang="ts">
import { shallowRef, onMounted } from 'vue';
import { useWidgets, type WidgetLayout } from '../composables/useWidgets';
import type { Component } from 'vue';

const { widgets, toggleWidget, setWidgetLayout } = useWidgets();

// Registry of available widgets
const componentRegistry = shallowRef<Record<string, Component>>({});

onMounted(async () => {
  // Dynamically import all Widget.vue files from src/widgets/*
  const components = import.meta.glob('../widgets/*/Widget.vue');

  for (const path in components) {
    // Extract ID from path: ../widgets/<id>/Widget.vue
    const match = path.match(/\.\.\/widgets\/(.+)\/Widget\.vue/);
    if (match && match[1]) {
      const id = match[1];
      const module: any = await components[path]();
      componentRegistry.value[id] = module.default;
    }
  }
});

const handleLayoutUpdate = (id: string, newLayout: Partial<WidgetLayout>) => {
  setWidgetLayout(id, newLayout);
};
</script>

<template>
  <!-- Main Widget Layer -->
  <div class="widget-layer absolute inset-0 pointer-events-none z-50 overflow-hidden">
    <template v-for="widget in widgets" :key="widget.id">
      <component
        v-if="widget.active && componentRegistry[widget.componentName]"
        :is="componentRegistry[widget.componentName]"
        :visible="true"
        :id="widget.id"
        :layout="widget.layout"
        class="pointer-events-auto"
        @close="toggleWidget(widget.id)"
        @update:layout="(l: Partial<WidgetLayout>) => handleLayoutUpdate(widget.id, l)"
      />
    </template>
  </div>

  <!-- Footer Dock for Minimized Widgets -->
  <div class="fixed bottom-6 left-0 right-0 h-8 flex justify-center items-end pointer-events-none z-[60]">
    <div class="flex gap-2 px-4 py-1 bg-[var(--vf-bg-secondary)]/80 backdrop-blur border border-[var(--vf-border-default)] rounded-t-lg pointer-events-auto" v-if="widgets.some(w => w.active && w.layout.minimized)">
      <button
        v-for="widget in widgets.filter(w => w.active && w.layout.minimized)"
        :key="widget.id"
        @click="handleLayoutUpdate(widget.id, { minimized: false })"
        class="px-3 py-0.5 text-xs bg-[var(--vf-surface-default)] border border-[var(--vf-border-default)] rounded hover:bg-[var(--vf-surface-hover)] truncate max-w-[120px] shadow-sm flex items-center gap-1"
      >
        <span class="font-bold text-[var(--vf-accent-primary)]">_</span>
        {{ widget.name }}
      </button>
    </div>
  </div>
</template>
