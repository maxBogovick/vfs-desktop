import type { FileItem } from '../types';
import { useFileSystem } from './useFileSystem';
import { useClipboard } from './useClipboard';
import { useNotifications } from './useNotifications';
import { useFileContentCache } from './useFileContentCache';
import { useFileOperationsProgress } from './useFileOperationsProgress';

export function useFileOperations(refreshCallback?: () => Promise<void>) {
  const {
    loadDirectory,
    renameItem,
    createFolder,
    openFile: openFileInApp,
    revealInFinder,
  } = useFileSystem();

  const {
    copyItemsWithProgress,
    moveItemsWithProgress,
    deleteItemsWithProgress,
  } = useFileOperationsProgress();

  const {
    hasClipboardItems,
    copy: copyToClipboard,
    cut: cutToClipboard,
    paste: pasteFromClipboard,
  } = useClipboard();

  const { success, error: showError, warning } = useNotifications();
  const { invalidate: invalidateCache } = useFileContentCache();

  // Функция для обновления текущей директории
  const refreshDirectory = async (currentPath: string[]) => {
    if (refreshCallback) {
      await refreshCallback();
    } else {
      const currentDir = await getCurrentDirectory(currentPath);
      await loadDirectory(currentDir);
    }
  };

  // Helper to get current directory path with proper leading slash
  const getCurrentDirectory = async (currentPath: string[]): Promise<string> => {
    let pathString = currentPath.join('/');

    // Add leading slash if path is not empty and doesn't start with slash
    if (pathString && !pathString.startsWith('/')) {
      pathString = '/' + pathString;
    }

    if (!pathString) {
      const { getHomeDirectory } = useFileSystem();
      return await getHomeDirectory();
    }

    return pathString;
  };

  // Copy selected items to clipboard
  const handleCopy = (selectedItems: FileItem[]) => {
    if (selectedItems.length > 0) {
      copyToClipboard(selectedItems);
      success('Copied', `${selectedItems.length} item(s) copied to clipboard`);
    }
  };

  // Cut selected items to clipboard
  const handleCut = (selectedItems: FileItem[]) => {
    if (selectedItems.length > 0) {
      cutToClipboard(selectedItems);
      warning('Cut', `${selectedItems.length} item(s) cut to clipboard`);
    }
  };

  // Paste items from clipboard
  const handlePaste = async (currentPath: string[]) => {
    if (!hasClipboardItems.value) {
      warning('Nothing to paste', 'Clipboard is empty');
      return;
    }

    try {
      const currentDir = await getCurrentDirectory(currentPath);
      await pasteFromClipboard(currentDir, copyItemsWithProgress, moveItemsWithProgress);
      // Refresh directory after paste
      await refreshDirectory(currentPath);
    } catch (err) {
      showError('Paste failed', err instanceof Error ? err.message : 'Unknown error');
    }
  };

  // Delete items with confirmation
  const handleDelete = async (
    selectedItems: FileItem[],
    currentPath: string[],
    clearSelection: () => void,
    showConfirm: (title: string, message: string, onConfirm: () => void, type?: 'warning' | 'danger' | 'info') => void
  ) => {
    if (selectedItems.length === 0) return;

    showConfirm(
      'Confirm Delete',
      `Are you sure you want to permanently delete ${selectedItems.length} item(s)?`,
      async () => {
        try {
          const paths = selectedItems.map(item => item.path);
          await deleteItemsWithProgress(paths);
          // Инвалидируем кеш для удаленных файлов
          selectedItems.forEach(item => invalidateCache(item.path));
          // Refresh directory
          await refreshDirectory(currentPath);
          clearSelection();
        } catch (err) {
          showError('Delete failed', err instanceof Error ? err.message : 'Unknown error');
        }
      },
      'danger'
    );
  };

  // Rename item
  const handleRename = (
    selectedItems: FileItem[],
    currentPath: string[],
    showInput: (title: string, label: string, onConfirm: (value: string) => void, defaultValue?: string, placeholder?: string) => void
  ) => {
    if (selectedItems.length !== 1) {
      warning('Invalid selection', 'Please select exactly one item to rename');
      return;
    }

    const item = selectedItems[0];

    showInput(
      'Rename',
      'Enter new name:',
      async (newName: string) => {
        if (!newName || newName === item.name) {
          return;
        }

        try {
          await renameItem(item.path, newName);
          // Инвалидируем кеш для старого пути
          invalidateCache(item.path);
          // Refresh directory
          await refreshDirectory(currentPath);
          success('Renamed', `Renamed to ${newName}`);
        } catch (err) {
          showError('Rename failed', err instanceof Error ? err.message : 'Unknown error');
        }
      },
      item.name,
      'New name'
    );
  };

  // Create new folder
  const handleNewFolder = (
    currentPath: string[],
    showInput: (title: string, label: string, onConfirm: (value: string) => void, defaultValue?: string, placeholder?: string) => void
  ) => {
    showInput(
      'New Folder',
      'Enter folder name:',
      async (name: string) => {
        if (!name) {
          return;
        }

        try {
          const currentDir = await getCurrentDirectory(currentPath);
          await createFolder(currentDir, name);
          // Refresh directory
          await refreshDirectory(currentPath);
          success('Folder created', `Created ${name}`);
        } catch (err) {
          showError('Create folder failed', err instanceof Error ? err.message : 'Unknown error');
        }
      },
      '',
      'New Folder'
    );
  };

  // Open file in default application
  const handleOpenFile = async (file: FileItem) => {
    try {
      await openFileInApp(file.path);
      success('File opened', `${file.name} opened successfully`);
    } catch (err) {
      showError('Failed to open file', err instanceof Error ? err.message : 'Unknown error');
    }
  };

  // Show properties
  const handleProperties = (
    selectedItems: FileItem[],
    showProperties: (file: FileItem) => void
  ) => {
    if (selectedItems.length !== 1) {
      warning('Invalid selection', 'Please select exactly one item to view properties');
      return;
    }

    showProperties(selectedItems[0]);
  };

  // Reveal in Finder
  const handleRevealInFinder = async (selectedItems: FileItem[]) => {
    if (selectedItems.length !== 1) return;

    try {
      await revealInFinder(selectedItems[0].path);
    } catch (err) {
      showError('Failed to reveal', err instanceof Error ? err.message : 'Unknown error');
    }
  };

  // Refresh directory
  const handleRefresh = async (currentPath: string[]) => {
    try {
      await refreshDirectory(currentPath);
      success('Refreshed', 'Directory refreshed');
    } catch (err) {
      showError('Refresh failed', err instanceof Error ? err.message : 'Unknown error');
    }
  };

  return {
    getCurrentDirectory,
    handleCopy,
    handleCut,
    handlePaste,
    handleDelete,
    handleRename,
    handleNewFolder,
    handleOpenFile,
    handleProperties,
    handleRevealInFinder,
    handleRefresh,
  };
}
