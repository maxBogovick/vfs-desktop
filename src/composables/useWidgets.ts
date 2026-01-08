import { ref, watch, reactive } from 'vue';

export interface WidgetLayout {
  x: number;
  y: number;
  width: number;
  height: number;
  minimized: boolean;
}

export interface Widget {
  id: string;
  name: string;
  description: string;
  active: boolean;
  componentName: string;
  layout: WidgetLayout;
}

// Default layout if none saved
const DEFAULT_LAYOUT: WidgetLayout = {
  x: 100,
  y: 100,
  width: 300,
  height: 200,
  minimized: false
};

const widgets = ref<Widget[]>([]);
const showWidgetSelector = ref(false);
const isInitialized = ref(false);

export function useWidgets() {
  
  if (!isInitialized.value) {
    // 1. Load basic registry from files
    const configs = import.meta.glob('../widgets/*/config.json', { eager: true });
    
    // 2. Load saved states from LocalStorage
    const savedStatesStr = localStorage.getItem('vfdir-widgets-state');
    const savedStates: Record<string, Partial<Widget>> = savedStatesStr ? JSON.parse(savedStatesStr) : {};

    const loadedWidgets: Widget[] = [];

    for (const path in configs) {
      // @ts-ignore
      const config = configs[path].default || configs[path];
      const saved = savedStates[config.id] || {};
      
      loadedWidgets.push({
        id: config.id,
        name: config.name,
        description: config.description,
        componentName: config.id,
        active: saved.active !== undefined ? saved.active : (config.defaultActive || false),
        layout: {
          x: saved.layout?.x ?? config.defaultX ?? DEFAULT_LAYOUT.x,
          y: saved.layout?.y ?? config.defaultY ?? DEFAULT_LAYOUT.y,
          width: saved.layout?.width ?? config.defaultWidth ?? DEFAULT_LAYOUT.width,
          height: saved.layout?.height ?? config.defaultHeight ?? DEFAULT_LAYOUT.height,
          minimized: saved.layout?.minimized ?? false
        }
      });
    }

    widgets.value = loadedWidgets;
    isInitialized.value = true;

    // 3. Setup auto-save
    watch(widgets, (newWidgets) => {
      const stateToSave: Record<string, Partial<Widget>> = {};
      for (const w of newWidgets) {
        stateToSave[w.id] = {
          active: w.active,
          layout: w.layout
        };
      }
      localStorage.setItem('vfdir-widgets-state', JSON.stringify(stateToSave));
    }, { deep: true });
  }

  const toggleWidget = (id: string) => {
    const widget = widgets.value.find(w => w.id === id);
    if (widget) {
      widget.active = !widget.active;
      // If activating, ensure it's not minimized initially if desired, 
      // or keep last state. Let's un-minimize on open to ensure visibility.
      if (widget.active) {
        widget.layout.minimized = false;
      }
    }
  };

  const setWidgetLayout = (id: string, layout: Partial<WidgetLayout>) => {
    const widget = widgets.value.find(w => w.id === id);
    if (widget) {
      Object.assign(widget.layout, layout);
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
    setWidgetLayout,
    isWidgetActive,
    openWidgetSelector,
    closeWidgetSelector
  };
}
