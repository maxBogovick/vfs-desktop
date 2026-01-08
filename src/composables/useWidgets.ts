import { ref } from 'vue';

export interface Widget {
  id: string;
  name: string;
  description: string;
  active: boolean;
  componentName: string; // The Vue component name
}

const widgets = ref<Widget[]>([
  {
    id: 'currency',
    name: 'Currency Rates',
    description: 'Displays current exchange rates (USD base)',
    active: false,
    componentName: 'CurrencyWidget'
  },
  {
    id: 'notes',
    name: 'Quick Notes',
    description: 'A simple scratchpad for temporary notes',
    active: false,
    componentName: 'QuickNotesWidget'
  },
  {
    id: 'resource-monitor',
    name: 'System Monitor',
    description: 'Track CPU and Memory usage',
    active: false,
    componentName: 'ResourceMonitor'
  },
  {
    id: 'calculator',
    name: 'Calculator',
    description: 'Programmer calculator (DEC/HEX/BIN)',
    active: false,
    componentName: 'CalculatorWidget'
  }
]);

const showWidgetSelector = ref(false);

export function useWidgets() {
  const toggleWidget = (id: string) => {
    const widget = widgets.value.find(w => w.id === id);
    if (widget) {
      widget.active = !widget.active;
    }
  };

  const isWidgetActive = (id: string) => {
    return widgets.value.find(w => w.id === id)?.active || false;
  };

  const openWidgetSelector = () => {
    showWidgetSelector.value = true;
  };

  const closeWidgetSelector = () => {
    showWidgetSelector.value = false;
  };

  return {
    widgets,
    showWidgetSelector,
    toggleWidget,
    isWidgetActive,
    openWidgetSelector,
    closeWidgetSelector
  };
}
