import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { FileItem, ViewMode } from '../types';

/**
 * Composable для управления состоянием UI приложения
 * Централизованное хранилище для всех dialogs, panels и UI флагов
 */
export function useAppUIState() {
  // View and sorting state
  const viewMode = ref<ViewMode>('list');
  const sortBy = ref<'name' | 'size' | 'modified' | 'type'>('name');
  const sortOrder = ref<'asc' | 'desc'>('asc');
  const showHidden = ref(false);

  // Dialogs
  const isCommandPaletteOpen = ref(false);
  const showSettings = ref(false);
  const settingsInitialTab = ref<'general' | 'colors'>('general');

  // Batch operations
  const showBatchRenameDialog = ref(false);
  const showBatchAttributeDialog = ref(false);
  const showBatchQueue = ref(false);
  const batchOperationFiles = ref<FileItem[]>([]);

  // Queue operations
  const showOperationsQueue = ref(false);
  const showQueueSettings = ref(false);

  // Panels
  const previewFile = ref<FileItem | null>(null);
  const showDashboard = ref(false);
  const dashboardWidth = ref(400);

  // Share
  const showShareDialog = ref(false);
  const shareInfo = ref<{ url: string; qr_svg: string; filename: string } | null>(null);

  // Inline file creator
  const showInlineCreator = ref(false);
  const inlineCreatorMode = ref<'file' | 'folder'>('file');

  // Filesystem backend
  const currentFilesystemBackend = ref<string>('real');

  // System stats
  const systemStats = ref({ memory_mb: 0, cpu_percent: 0 });

  // Actions
  const openCommandPalette = () => {
    isCommandPaletteOpen.value = true;
  };

  const closeCommandPalette = () => {
    isCommandPaletteOpen.value = false;
  };

  const openSettings = (tab: 'general' | 'colors' = 'general') => {
    settingsInitialTab.value = tab;
    showSettings.value = true;
  };

  const closeSettings = () => {
    showSettings.value = false;
    settingsInitialTab.value = 'general';
  };

  const toggleDashboard = () => {
    showDashboard.value = !showDashboard.value;
    // Close preview when opening dashboard
    if (showDashboard.value) {
      previewFile.value = null;
    }
  };

  const openPreview = (file: FileItem) => {
    previewFile.value = file;
    showDashboard.value = false;
  };

  const closePreview = () => {
    previewFile.value = null;
  };

  const openBatchRename = (files: FileItem[]) => {
    batchOperationFiles.value = files;
    showBatchRenameDialog.value = true;
  };

  const openBatchAttribute = (files: FileItem[]) => {
    batchOperationFiles.value = files;
    showBatchAttributeDialog.value = true;
  };

  const closeBatchDialogs = () => {
    showBatchRenameDialog.value = false;
    showBatchAttributeDialog.value = false;
    batchOperationFiles.value = [];
  };

  const openInlineCreator = (mode: 'file' | 'folder') => {
    inlineCreatorMode.value = mode;
    showInlineCreator.value = true;
  };

  const closeInlineCreator = () => {
    showInlineCreator.value = false;
  };

  const toggleOperationsQueue = () => {
    showOperationsQueue.value = !showOperationsQueue.value;
  };

  const updateSystemStats = (stats: { memory_mb: number; cpu_percent: number }) => {
    systemStats.value = stats;
  };

  // Auto-save filesystem backend changes to config
  watch(currentFilesystemBackend, async (newBackend) => {
    try {
      const config = await invoke<any>('get_config');
      config.filesystem_backend = newBackend;
      await invoke('save_config', { config });
      console.log('[useAppUIState] Saved filesystem backend:', newBackend);
    } catch (error) {
      console.error('[useAppUIState] Failed to save filesystem backend:', error);
    }
  });

  // Load initial filesystem backend from config
  const loadFilesystemBackend = async () => {
    try {
      const config = await invoke<any>('get_config');
      const isVirtualFS = config.filesystem_backend === 'virtual';
      currentFilesystemBackend.value = isVirtualFS ? 'virtual' : 'real';
      console.log('[useAppUIState] Loaded filesystem backend:', currentFilesystemBackend.value);
    } catch (error) {
      console.error('[useAppUIState] Failed to load filesystem backend:', error);
    }
  };

  return {
    // View state
    viewMode,
    sortBy,
    sortOrder,
    showHidden,

    // Dialogs
    isCommandPaletteOpen,
    showSettings,
    settingsInitialTab,

    // Batch operations
    showBatchRenameDialog,
    showBatchAttributeDialog,
    showBatchQueue,
    batchOperationFiles,

    // Queue
    showOperationsQueue,
    showQueueSettings,

    // Panels
    previewFile,
    showDashboard,
    dashboardWidth,

    // Share
    showShareDialog,
    shareInfo,

    // Inline creator
    showInlineCreator,
    inlineCreatorMode,

    // Backend
    currentFilesystemBackend,

    // System
    systemStats,

    // Actions
    openCommandPalette,
    closeCommandPalette,
    openSettings,
    closeSettings,
    toggleDashboard,
    openPreview,
    closePreview,
    openBatchRename,
    openBatchAttribute,
    closeBatchDialogs,
    openInlineCreator,
    closeInlineCreator,
    toggleOperationsQueue,
    updateSystemStats,
    loadFilesystemBackend,
  };
}
