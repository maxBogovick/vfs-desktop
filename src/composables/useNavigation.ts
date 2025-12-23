import { ref, computed } from 'vue';
import type { Tab } from '../types';

export function useNavigation() {
  const tabs = ref<Tab[]>([
    {
      id: Date.now(),
      path: [], // Empty path will load home directory
      name: 'Home',
      history: [[]],
      historyIndex: 0,
    },
  ]);

  const activeTabId = ref(tabs.value[0].id);

  const activeTab = computed(() => {
    return tabs.value.find(t => t.id === activeTabId.value);
  });

  const currentPath = computed(() => {
    return activeTab.value?.path || [];
  });

  const currentPathString = computed(() => {
    return currentPath.value.join('/');
  });

  // Navigate to a specific path
  const navigateTo = (path: string[]) => {
    if (!activeTab.value) return;

    const tab = activeTab.value;

    // Add to history
    const newHistory = tab.history.slice(0, tab.historyIndex + 1);
    newHistory.push(path);

    // Ограничиваем размер истории (последние 50 записей)
    const MAX_HISTORY_SIZE = 50;
    if (newHistory.length > MAX_HISTORY_SIZE) {
      newHistory.splice(0, newHistory.length - MAX_HISTORY_SIZE);
    }

    // Update tab
    const tabIndex = tabs.value.findIndex(t => t.id === activeTabId.value);
    if (tabIndex !== -1) {
      tabs.value[tabIndex] = {
        ...tab,
        path,
        name: path[path.length - 1] || 'Home',
        history: newHistory,
        historyIndex: newHistory.length - 1,
      };
    }
  };

  // Navigate to specific index in breadcrumb
  const navigateToBreadcrumb = (index: number) => {
    const newPath = currentPath.value.slice(0, index + 1);
    navigateTo(newPath);
  };

  // Navigate into a folder
  const navigateInto = (folderName: string) => {
    const newPath = [...currentPath.value, folderName];
    navigateTo(newPath);
  };

  // Go back in history
  const goBack = () => {
    if (!activeTab.value) return;

    const tab = activeTab.value;
    if (tab.historyIndex > 0) {
      const tabIndex = tabs.value.findIndex(t => t.id === activeTabId.value);
      if (tabIndex !== -1) {
        const newIndex = tab.historyIndex - 1;
        const newPath = tab.history[newIndex];
        tabs.value[tabIndex] = {
          ...tab,
          path: newPath,
          name: newPath[newPath.length - 1] || 'Home',
          historyIndex: newIndex,
        };
      }
    }
  };

  // Go forward in history
  const goForward = () => {
    if (!activeTab.value) return;

    const tab = activeTab.value;
    if (tab.historyIndex < tab.history.length - 1) {
      const tabIndex = tabs.value.findIndex(t => t.id === activeTabId.value);
      if (tabIndex !== -1) {
        const newIndex = tab.historyIndex + 1;
        const newPath = tab.history[newIndex];
        tabs.value[tabIndex] = {
          ...tab,
          path: newPath,
          name: newPath[newPath.length - 1] || 'Home',
          historyIndex: newIndex,
        };
      }
    }
  };

  // Go up one directory level
  const goUp = () => {
    if (currentPath.value.length > 0) {
      if (currentPath.value.length === 1) {
        // If we're one level deep, go to home
        navigateTo([]);
      } else {
        // Otherwise go up one level
        navigateToBreadcrumb(currentPath.value.length - 2);
      }
    }
  };

  // Go to home/root
  const goHome = () => {
    navigateTo([]); // Empty path will load home directory
  };

  // Can go back?
  const canGoBack = computed(() => {
    return activeTab.value ? activeTab.value.historyIndex > 0 : false;
  });

  // Can go forward?
  const canGoForward = computed(() => {
    if (!activeTab.value) return false;
    return activeTab.value.historyIndex < activeTab.value.history.length - 1;
  });

  // Can go up?
  const canGoUp = computed(() => {
    return currentPath.value.length > 0;
  });

  // Add new tab
  const addTab = (path: string[] = []) => {
    const newTab: Tab = {
      id: Date.now(),
      path,
      name: path[path.length - 1] || 'Home',
      history: [path],
      historyIndex: 0,
    };
    tabs.value.push(newTab);
    activeTabId.value = newTab.id;
  };

  // Close tab
  const closeTab = (tabId: number) => {
    if (tabs.value.length <= 1) return;

    const tabIndex = tabs.value.findIndex(t => t.id === tabId);

    // Очищаем историю таба перед удалением для освобождения памяти
    const tabToClose = tabs.value[tabIndex];
    if (tabToClose) {
      tabToClose.history = [];
      tabToClose.path = [];
    }

    // Удаляем таб
    tabs.value = tabs.value.filter(t => t.id !== tabId);

    // Switch to another tab if the closed tab was active
    if (activeTabId.value === tabId) {
      const newActiveTab = tabs.value[Math.max(0, tabIndex - 1)];
      activeTabId.value = newActiveTab.id;
    }
  };

  // Switch to tab
  const switchTab = (tabId: number) => {
    activeTabId.value = tabId;
  };

  return {
    tabs,
    activeTabId,
    activeTab,
    currentPath,
    currentPathString,
    canGoBack,
    canGoForward,
    canGoUp,
    navigateTo,
    navigateToBreadcrumb,
    navigateInto,
    goBack,
    goForward,
    goUp,
    goHome,
    addTab,
    closeTab,
    switchTab,
  };
}
