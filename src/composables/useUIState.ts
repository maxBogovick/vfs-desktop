import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Tab, TabState, WindowState, SidebarState, UIState } from '../types';

// Debounce utility
function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return (...args: Parameters<T>) => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(() => {
      fn(...args);
    }, delay);
  };
}

export function useUIState() {
  const sidebarWidth = ref(240);
  const previewWidth = ref(300);
  const isLoaded = ref(false);

  // Sidebar state
  const expandedFolders = ref<string[]>([]);
  const sidebarSectionsExpanded = ref({
    quickAccess: true,
    folderTree: true,
    favorites: false,
  });

  // Load UI state from backend
  const loadUIState = async (): Promise<UIState | null> => {
    try {
      const state = await invoke<UIState>('get_ui_state');

      if (state) {
        sidebarWidth.value = state.sidebar_width;
        previewWidth.value = state.preview_width;

        // Restore sidebar state
        if (state.sidebar) {
          expandedFolders.value = state.sidebar.expanded_folders || [];
          sidebarSectionsExpanded.value = {
            quickAccess: state.sidebar.quick_access_expanded ?? true,
            folderTree: state.sidebar.folder_tree_expanded ?? true,
            favorites: state.sidebar.favorites_expanded ?? false,
          };
        }

        isLoaded.value = true;
        return state;
      }
    } catch (error) {
      console.error('Failed to load UI state:', error);
    }

    isLoaded.value = true;
    return null;
  };

  // Save UI state to backend
  const saveUIState = async (state: UIState) => {
    try {
      await invoke('save_ui_state', { uiState: state });
    } catch (error) {
      console.error('Failed to save UI state:', error);
    }
  };

  // Debounced save function (wait 500ms after last change)
  const debouncedSave = debounce(saveUIState, 500);

  // Save sidebar width
  const saveSidebarWidth = (width: number) => {
    sidebarWidth.value = width;
  };

  // Save preview width
  const savePreviewWidth = (width: number) => {
    previewWidth.value = width;
  };

  // Save tabs state
  const saveTabsState = async (tabs: Tab[], activeTabId: number) => {
    if (!isLoaded.value) return;

    const tabStates: TabState[] = tabs.map(tab => ({
      id: tab.id,
      path: tab.path,
      name: tab.name,
    }));

    const state: UIState = {
      sidebar_width: sidebarWidth.value,
      preview_width: previewWidth.value,
      tabs: tabStates,
      active_tab_id: activeTabId,
      window: {
        maximized: false,
      },
      sidebar: {
        expanded_folders: expandedFolders.value,
        quick_access_expanded: sidebarSectionsExpanded.value.quickAccess,
        folder_tree_expanded: sidebarSectionsExpanded.value.folderTree,
        favorites_expanded: sidebarSectionsExpanded.value.favorites,
      },
    };

    await debouncedSave(state);
  };

  // Save last path
  const saveLastPath = async (path: string[]) => {
    if (!isLoaded.value) return;

    const state: UIState = {
      sidebar_width: sidebarWidth.value,
      preview_width: previewWidth.value,
      tabs: [],
      last_path: path,
      window: {
        maximized: false,
      },
      sidebar: {
        expanded_folders: expandedFolders.value,
        quick_access_expanded: sidebarSectionsExpanded.value.quickAccess,
        folder_tree_expanded: sidebarSectionsExpanded.value.folderTree,
        favorites_expanded: sidebarSectionsExpanded.value.favorites,
      },
    };

    await debouncedSave(state);
  };

  // Save window state
  const saveWindowState = async (windowState: WindowState) => {
    if (!isLoaded.value) return;

    const state: UIState = {
      sidebar_width: sidebarWidth.value,
      preview_width: previewWidth.value,
      tabs: [],
      window: windowState,
      sidebar: {
        expanded_folders: expandedFolders.value,
        quick_access_expanded: sidebarSectionsExpanded.value.quickAccess,
        folder_tree_expanded: sidebarSectionsExpanded.value.folderTree,
        favorites_expanded: sidebarSectionsExpanded.value.favorites,
      },
    };

    await debouncedSave(state);
  };

  // Save complete UI state
  const saveCompleteState = async (
    tabs: Tab[],
    activeTabId: number,
    lastPath?: string[],
    windowState?: WindowState
  ) => {
    if (!isLoaded.value) return;

    const tabStates: TabState[] = tabs.map(tab => ({
      id: tab.id,
      path: tab.path,
      name: tab.name,
    }));

    const state: UIState = {
      sidebar_width: sidebarWidth.value,
      preview_width: previewWidth.value,
      tabs: tabStates,
      active_tab_id: activeTabId,
      last_path: lastPath,
      window: windowState || {
        maximized: false,
      },
      sidebar: {
        expanded_folders: expandedFolders.value,
        quick_access_expanded: sidebarSectionsExpanded.value.quickAccess,
        folder_tree_expanded: sidebarSectionsExpanded.value.folderTree,
        favorites_expanded: sidebarSectionsExpanded.value.favorites,
      },
    };

    await debouncedSave(state);
  };

  // Toggle folder expansion
  const toggleFolderExpansion = (folderPath: string) => {
    const index = expandedFolders.value.indexOf(folderPath);
    if (index > -1) {
      expandedFolders.value.splice(index, 1);
    } else {
      expandedFolders.value.push(folderPath);
    }
  };

  // Save sidebar state
  const saveSidebarState = async () => {
    if (!isLoaded.value) return;

    const state: UIState = {
      sidebar_width: sidebarWidth.value,
      preview_width: previewWidth.value,
      tabs: [],
      window: {
        maximized: false,
      },
      sidebar: {
        expanded_folders: expandedFolders.value,
        quick_access_expanded: sidebarSectionsExpanded.value.quickAccess,
        folder_tree_expanded: sidebarSectionsExpanded.value.folderTree,
        favorites_expanded: sidebarSectionsExpanded.value.favorites,
      },
    };

    await debouncedSave(state);
  };

  // Watch for changes in sidebar width
  watch(sidebarWidth, async (newWidth) => {
    if (!isLoaded.value) return;

    const state: UIState = {
      sidebar_width: newWidth,
      preview_width: previewWidth.value,
      tabs: [],
      window: {
        maximized: false,
      },
      sidebar: {
        expanded_folders: expandedFolders.value,
        quick_access_expanded: sidebarSectionsExpanded.value.quickAccess,
        folder_tree_expanded: sidebarSectionsExpanded.value.folderTree,
        favorites_expanded: sidebarSectionsExpanded.value.favorites,
      },
    };

    await debouncedSave(state);
  });

  // Watch for changes in preview width
  watch(previewWidth, async (newWidth) => {
    if (!isLoaded.value) return;

    const state: UIState = {
      sidebar_width: sidebarWidth.value,
      preview_width: newWidth,
      tabs: [],
      window: {
        maximized: false,
      },
      sidebar: {
        expanded_folders: expandedFolders.value,
        quick_access_expanded: sidebarSectionsExpanded.value.quickAccess,
        folder_tree_expanded: sidebarSectionsExpanded.value.folderTree,
        favorites_expanded: sidebarSectionsExpanded.value.favorites,
      },
    };

    await debouncedSave(state);
  });

  // Watch for changes in expanded folders
  watch(expandedFolders, () => {
    saveSidebarState();
  }, { deep: true });

  // Watch for changes in sidebar sections
  watch(sidebarSectionsExpanded, () => {
    saveSidebarState();
  }, { deep: true });

  return {
    sidebarWidth,
    previewWidth,
    isLoaded,
    expandedFolders,
    sidebarSectionsExpanded,
    loadUIState,
    saveUIState,
    saveSidebarWidth,
    savePreviewWidth,
    saveTabsState,
    saveLastPath,
    saveWindowState,
    saveCompleteState,
    toggleFolderExpansion,
    saveSidebarState,
  };
}
