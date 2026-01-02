import {ref} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import type {UIState} from '../types';

// ВАЖНО: Создаем refs на уровне модуля, чтобы они были общими для всех вызовов useUIState()
// Это гарантирует, что App.vue и Sidebar.vue используют ОДНИ И ТЕ ЖЕ реактивные объекты
const sidebarWidth = ref(240);
const previewWidth = ref(300);
const isLoaded = ref(false);
const editModeEnabled = ref(false);

// Sidebar state
const expandedFolders = ref<string[]>([]);
const sidebarSectionsExpanded = ref({
  quickAccess: true,
  folderTree: true,
  favorites: false,
});

export function useUIState() {
  // Load UI state from backend
  const loadUIState = async (): Promise<UIState | null> => {
    try {
      const state = await invoke<UIState>('get_ui_state');
      console.log('[useUIState] Loaded state:', state);

      if (state) {
        sidebarWidth.value = state.sidebar_width;
        previewWidth.value = state.preview_width;
        editModeEnabled.value = state.edit_mode_enabled ?? false;

        // Restore sidebar state
        if (state.sidebar) {
          expandedFolders.value = state.sidebar.expanded_folders || [];
          sidebarSectionsExpanded.value = {
            quickAccess: state.sidebar.quick_access_expanded ?? true,
            folderTree: state.sidebar.folder_tree_expanded ?? true,
            favorites: state.sidebar.favorites_expanded ?? false,
          };

          console.log('[useUIState] ✅ Restored sidebar state:');
          console.log('  - expanded_folders:', expandedFolders.value.length);
          console.log('  - quickAccess:', sidebarSectionsExpanded.value.quickAccess);
          console.log('  - folderTree:', sidebarSectionsExpanded.value.folderTree);
          console.log('  - favorites:', sidebarSectionsExpanded.value.favorites);
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

  // ПРИМЕЧАНИЕ: Все сохранение теперь происходит через watch в App.vue
  // Этот composable только загружает состояние и предоставляет реактивные refs

  return {
    sidebarWidth,
    previewWidth,
    isLoaded,
    editModeEnabled,
    expandedFolders,
    sidebarSectionsExpanded,
    loadUIState,
  };
}
