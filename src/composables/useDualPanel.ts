import {computed, ref} from 'vue';
import type {ActivePanel, DualPanelConfig, FileItem, PanelMode, Tab} from '../types';

// Module-level shared state for dual panel mode
const panelMode = ref<PanelMode>('single');
const leftPanelWidthPercent = ref<number>(50);
const activePanel = ref<ActivePanel>('left');

// Left panel state
const leftPanelTabs = ref<Tab[]>([]);
const leftPanelActiveTabId = ref<number | undefined>(undefined);

// Right panel state
const rightPanelTabs = ref<Tab[]>([]);
const rightPanelActiveTabId = ref<number | undefined>(undefined);

// Active panel methods (for keyboard shortcuts in dual mode)
export interface ActivePanelMethods {
  getFiles: () => FileItem[];
  getSelectedItems: () => FileItem[];
  selectAll: () => void;
  clearSelection: () => void;
  handleCopy: () => void;
  handleCut: () => void;
  handlePaste: () => void;
  handleDelete: () => void;
  handleRename: () => void;
  handleRefresh: () => void;
  handleNewFolder: () => void;
  handleNewFile: () => void;
  addTab: () => void;
  closeTab: () => void;
  goUp: () => void;
  goBack: () => void;
  goForward: () => void;
  canGoBack: () => boolean;
  canGoForward: () => boolean;
  moveFocusUp: () => void;
  moveFocusDown: () => void;
  moveFocusToFirst: () => void;
  moveFocusToLast: () => void;
  selectFocused: () => void;
  toggleFocusedSelection: () => void;
  openFocusedItem: () => void;
}

const activePanelMethods = ref<ActivePanelMethods | null>(null);

export function registerActivePanelMethods(methods: ActivePanelMethods) {
  activePanelMethods.value = methods;
}

export function getActivePanelMethods(): ActivePanelMethods | null {
  return activePanelMethods.value;
}

export function useDualPanel() {
  // Computed properties
  const isDualMode = computed(() => panelMode.value === 'dual');

  const activePanelTabs = computed(() => {
    return activePanel.value === 'left' ? leftPanelTabs.value : rightPanelTabs.value;
  });

  const activePanelTabId = computed(() => {
    return activePanel.value === 'left' ? leftPanelActiveTabId.value : rightPanelActiveTabId.value;
  });

  const activePanelPath = computed(() => {
    const tabs = activePanelTabs.value;
    const tabId = activePanelTabId.value;
    const activeTab = tabs.find(tab => tab.id === tabId);
    return activeTab?.path || [];
  });

  // Methods
  const togglePanelMode = () => {
    if (panelMode.value === 'single') {
      panelMode.value = 'dual';
      // Initialize dual panels if empty
      if (leftPanelTabs.value.length === 0) {
        const defaultTab: Tab = {
          id: Date.now(),
          path: [''],
          name: 'Home',
          history: [['']],
          historyIndex: 0,
        };
        leftPanelTabs.value = [defaultTab];
        leftPanelActiveTabId.value = defaultTab.id;
      }
      if (rightPanelTabs.value.length === 0) {
        const defaultTab: Tab = {
          id: Date.now() + 1,
          path: [''],
          name: 'Home',
          history: [['']],
          historyIndex: 0,
        };
        rightPanelTabs.value = [defaultTab];
        rightPanelActiveTabId.value = defaultTab.id;
      }
    } else {
      panelMode.value = 'single';
    }
  };

  const switchActivePanel = (panel: ActivePanel) => {
    activePanel.value = panel;
  };

  const setPanelSplit = (percent: number) => {
    // Ограничение 20-80% для предотвращения схлопывания
    leftPanelWidthPercent.value = Math.max(20, Math.min(80, percent));
  };

  const loadDualPanelState = (config: DualPanelConfig) => {
    leftPanelWidthPercent.value = config.left_panel_width_percent;
    activePanel.value = config.active_panel;

    // Load left panel tabs
    if (config.left_panel.tabs && config.left_panel.tabs.length > 0) {
      leftPanelTabs.value = config.left_panel.tabs.map(tabState => ({
        id: tabState.id,
        path: tabState.path,
        name: tabState.name,
        history: [tabState.path],
        historyIndex: 0,
      }));
      leftPanelActiveTabId.value = config.left_panel.active_tab_id;
    }

    // Load right panel tabs
    if (config.right_panel.tabs && config.right_panel.tabs.length > 0) {
      rightPanelTabs.value = config.right_panel.tabs.map(tabState => ({
        id: tabState.id,
        path: tabState.path,
        name: tabState.name,
        history: [tabState.path],
        historyIndex: 0,
      }));
      rightPanelActiveTabId.value = config.right_panel.active_tab_id;
    }
  };

  const serializeDualPanelState = (): DualPanelConfig => {
    return {
      left_panel_width_percent: leftPanelWidthPercent.value,
      left_panel: {
        tabs: leftPanelTabs.value.map(tab => ({
          id: tab.id,
          path: tab.path,
          name: tab.name,
        })),
        active_tab_id: leftPanelActiveTabId.value,
      },
      right_panel: {
        tabs: rightPanelTabs.value.map(tab => ({
          id: tab.id,
          path: tab.path,
          name: tab.name,
        })),
        active_tab_id: rightPanelActiveTabId.value,
      },
      active_panel: activePanel.value,
    };
  };

  return {
    // State
    panelMode,
    leftPanelWidthPercent,
    activePanel,
    leftPanelTabs,
    leftPanelActiveTabId,
    rightPanelTabs,
    rightPanelActiveTabId,

    // Computed
    isDualMode,
    activePanelTabs,
    activePanelTabId,
    activePanelPath,

    // Methods
    togglePanelMode,
    switchActivePanel,
    setPanelSplit,
    loadDualPanelState,
    serializeDualPanelState,
  };
}
