<script setup lang="ts">
import { useWidgets } from '../composables/useWidgets';
import CurrencyWidget from './CurrencyWidget.vue';
import QuickNotesWidget from './QuickNotesWidget.vue';
import ResourceMonitor from './ResourceMonitor.vue';
import CalculatorWidget from './CalculatorWidget.vue';
import type { Component } from 'vue';

const { widgets, toggleWidget } = useWidgets();

// Registry of available widgets
const componentRegistry: Record<string, Component> = {
  CurrencyWidget,
  QuickNotesWidget,
  ResourceMonitor,
  CalculatorWidget
};
</script>

<template>
  <div class="widget-layer absolute inset-0 pointer-events-none z-50 overflow-hidden">
    <template v-for="widget in widgets" :key="widget.id">
      <component
        v-if="widget.active && componentRegistry[widget.componentName]"
        :is="componentRegistry[widget.componentName]"
        :visible="true"
        class="pointer-events-auto"
        @close="toggleWidget(widget.id)"
      />
    </template>
  </div>
</template>
