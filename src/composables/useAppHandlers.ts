import { computed, type Ref } from 'vue';
import type { FileItem } from '../types';
import { useDualPanel, getActivePanelMethods } from './useDualPanel';
import { useNavigation } from './useNavigation';
import { useFileOperations } from './useFileOperations';
import { useFileSystem } from './useFileSystem';
import { useSelection } from './useSelection';

/**
 * Параметры для создания обработчиков приложения
 */
export interface AppHandlersParams {
  isDualMode: Ref<boolean>;
  currentPath: Ref<string[]>;
  files: Ref<FileItem[]>;
  selectedIds: Ref<Set<string>>;
  editModeEnabled: Ref<boolean>;
  onRefresh: () => Promise<void>;
  onPreviewFile: (file: FileItem) => void;
  onEditFile: (file: FileItem, panelFs?: string) => void;
  onOpenInlineCreator: (mode: 'file' | 'folder') => void;
  onSelectAll: () => void;
  onInvertSelection: () => void;
}

/**
 * Composable для основных обработчиков приложения
 * Навигация, Toolbar, Drag&Drop, Item actions
 */
export function useAppHandlers(params: AppHandlersParams) {
  const { goBack, goForward, goUp, goHome, addTab, navigateTo } = useNavigation();
  const { panelMode, activePanelPath, activePanel, leftPanelTabs, leftPanelActiveTabId, rightPanelTabs, rightPanelActiveTabId } = useDualPanel();
  const fileOps = useFileOperations(params.onRefresh);
  const { normalizePath, getHomeDirectory } = useFileSystem();
  const { clearSelection } = useSelection();

  // Navigation handlers (работают в single и dual режимах)
  const handleGoBack = () => {
    if (params.isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) methods.goBack();
    } else {
      goBack();
    }
  };

  const handleGoForward = () => {
    if (params.isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) methods.goForward();
    } else {
      goForward();
    }
  };

  const handleGoUp = () => {
    if (params.isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) methods.goUp();
    } else {
      goUp();
    }
  };

  const handleAddTab = () => {
    if (params.isDualMode.value) {
      const methods = getActivePanelMethods();
      if (methods) methods.addTab();
    } else {
      addTab();
    }
  };

  const handleGoHome = async () => {
    if (params.isDualMode.value) {
      const homeDir = await getHomeDirectory();
      const pathParts = homeDir.replace(/^\//, '').split('/').filter(p => p);

      if (activePanel.value === 'left') {
        const activeTab = leftPanelTabs.value.find(t => t.id === leftPanelActiveTabId.value);
        if (activeTab) {
          const tabIndex = leftPanelTabs.value.findIndex(t => t.id === leftPanelActiveTabId.value);
          const updatedTabs = [...leftPanelTabs.value];
          const newHistory = activeTab.history.slice(0, activeTab.historyIndex + 1);
          newHistory.push(pathParts);
          updatedTabs[tabIndex] = {
            ...activeTab,
            path: pathParts,
            name: 'Home',
            history: newHistory,
            historyIndex: newHistory.length - 1,
          };
          leftPanelTabs.value = updatedTabs;
        }
      } else {
        const activeTab = rightPanelTabs.value.find(t => t.id === rightPanelActiveTabId.value);
        if (activeTab) {
          const tabIndex = rightPanelTabs.value.findIndex(t => t.id === rightPanelActiveTabId.value);
          const updatedTabs = [...rightPanelTabs.value];
          const newHistory = activeTab.history.slice(0, activeTab.historyIndex + 1);
          newHistory.push(pathParts);
          updatedTabs[tabIndex] = {
            ...activeTab,
            path: pathParts,
            name: 'Home',
            history: newHistory,
            historyIndex: newHistory.length - 1,
          };
          rightPanelTabs.value = updatedTabs;
        }
      }
    } else {
      goHome();
    }
  };

  const handleNavigateToPath = async (path: string) => {
    try {
      const normalizedPath = await normalizePath(path);
      const pathArray = normalizedPath.replace(/^\//, '').split('/').filter(p => p);
      navigateTo(pathArray);
    } catch (err) {
      const { error } = await import('./useNotifications');
      error('Invalid path', err instanceof Error ? err.message : 'Path not found');
    }
  };

  // Item handlers
  const handleItemDoubleClick = (item: FileItem) => {
    if (item.type === 'folder' || item.type === 'drive' || item.type === 'system' || item.type === 'archive') {
      const pathParts = item.path.split('/').filter(p => p);
      navigateTo(pathParts);
    } else {
      const isEditableFile = item.type === 'file' || item.type === 'code';
      if (params.editModeEnabled.value && isEditableFile) {
        params.onEditFile(item);
      } else {
        params.onPreviewFile(item);
      }
    }
  };

  // Drag handlers
  const handleDragStart = (item: FileItem, event: DragEvent, hasSelection: boolean, isSelected: (id: string) => boolean) => {
    const items = hasSelection && isSelected(item.id)
      ? params.files.value.filter(f => params.selectedIds.value.has(f.id))
      : [item];

    console.log('[App] Starting drag with items:', items.length, items.map(i => i.name));

    // This should use useDragDrop startDrag
    // but we can't call it from here without passing it
    // Instead, return items for external handling
    return items;
  };

  // Toolbar handlers (for single panel mode)
  const handleSort = (field: 'name' | 'size' | 'modified' | 'type', order: 'asc' | 'desc', sortBy: Ref<any>, sortOrder: Ref<any>) => {
    sortBy.value = field;
    sortOrder.value = order;
  };

  const handleSelectAll = () => {
    params.onSelectAll();
  };

  const handleInvertSelection = () => {
    params.onInvertSelection();
  };

  const handleRefresh = async () => {
    await params.onRefresh();
  };

  // Inline file creator handlers
  const handleCreateFile = async (payload: { name: string; isFolder: boolean; templateId?: string }) => {
    try {
      if (payload.isFolder) {
        await fileOps.handleNewFile(params.currentPath.value, payload.name);
      } else {
        await fileOps.handleNewFile(params.currentPath.value, payload.name, payload.templateId);
      }
    } catch (err) {
      console.error('Failed to create file:', err);
    }
  };

  const handleBatchCreateFiles = async (names: string[]) => {
    try {
      const files = names.map(name => ({ name }));
      await fileOps.handleBatchCreate(params.currentPath.value, files);
    } catch (err) {
      console.error('Failed to batch create files:', err);
    }
  };

  // Computed values
  const canGoBackComputed = computed(() => {
    if (params.isDualMode.value) {
      const methods = getActivePanelMethods();
      return methods ? methods.canGoBack() : false;
    }
    // This should return canGoBack from useNavigation
    // but we need to pass it as parameter
    return false;
  });

  const canGoForwardComputed = computed(() => {
    if (params.isDualMode.value) {
      const methods = getActivePanelMethods();
      return methods ? methods.canGoForward() : false;
    }
    return false;
  });

  return {
    // Navigation
    handleGoBack,
    handleGoForward,
    handleGoUp,
    handleGoHome,
    handleAddTab,
    handleNavigateToPath,

    // Items
    handleItemDoubleClick,
    handleDragStart,

    // Toolbar
    handleSort,
    handleSelectAll,
    handleInvertSelection,
    handleRefresh,

    // File creation
    handleCreateFile,
    handleBatchCreateFiles,

    // Computed
    canGoBackComputed,
    canGoForwardComputed,
  };
}
