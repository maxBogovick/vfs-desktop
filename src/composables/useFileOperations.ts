import type {FileItem} from '../types';
import {useFileSystem} from './useFileSystem';
import {useClipboard} from './useClipboard';
import {useNotifications} from './useNotifications';
import {useFileContentCache} from './useFileContentCache';
import {useFileOperationsProgress} from './useFileOperationsProgress';
import {useConflictResolution} from './useConflictResolution';

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

  // Paste items from clipboard with conflict resolution
  const handlePaste = async (currentPath: string[]) => {
    if (!hasClipboardItems.value) {
      warning('Nothing to paste', 'Clipboard is empty');
      return;
    }

    const {
      checkConflict,
      requestConflictResolution,
      resetSavedResolution,
    } = useConflictResolution();

    const { clipboardItems, operation } = useClipboard();

    try {
      const currentDir = await getCurrentDirectory(currentPath);

      // Reset any saved "apply to all" resolution from previous operations
      resetSavedResolution();

      // Check for conflicts
      const itemsToProcess: Array<{
        sourcePath: string;
        targetPath: string;
        action: 'copy' | 'move' | 'rename';
        newName?: string;
      }> = [];

      const itemsToSkip: string[] = [];

      for (const item of clipboardItems.value) {
        const conflict = await checkConflict(item.path, currentDir);

        if (conflict) {
          // File exists, ask user what to do
          try {
            const resolution = await requestConflictResolution(conflict);

            if (resolution.action === 'skip') {
              itemsToSkip.push(item.path);
              continue;
            }

            if (resolution.action === 'replace') {
              // Just copy/move normally, will overwrite
              itemsToProcess.push({
                sourcePath: item.path,
                targetPath: currentDir,
                action: operation.value === 'copy' ? 'copy' : 'move',
                newName: undefined,
              });
            } else if (resolution.action === 'rename') {
              // Copy/move with new name
              if (resolution.newName) {
                // For rename, we always use copy_file_with_custom_name
                // and then delete source if operation is 'cut'
                itemsToProcess.push({
                  sourcePath: item.path,
                  targetPath: currentDir,
                  action: 'rename',
                  newName: resolution.newName,
                });
              }
            }
            // 'compare' action is not implemented yet
          } catch (err) {
            // User cancelled conflict resolution
            warning('Operation cancelled', 'File operation was cancelled');
            resetSavedResolution();
            return;
          }
        } else {
          // No conflict, proceed normally
          itemsToProcess.push({
            sourcePath: item.path,
            targetPath: item.path,
            action: operation.value === 'copy' ? 'copy' : 'move',
            newName: undefined,
          });
        }
      }

      // Now perform the operations
      if (itemsToProcess.length === 0) {
        warning('No files to paste', 'All files were skipped');
        resetSavedResolution();
        return;
      }

      // Group by action
      const sourcesToCopy = itemsToProcess
        .filter(i => i.action === 'copy')
        .map(i => i.sourcePath);
      const sourcesToMove = itemsToProcess
        .filter(i => i.action === 'move')
        .map(i => i.sourcePath);
      const itemsToRename = itemsToProcess
        .filter(i => i.action === 'rename');

      if (sourcesToCopy.length > 0) {
        await copyItemsWithProgress(sourcesToCopy, currentDir);
      }

      if (sourcesToMove.length > 0) {
        await moveItemsWithProgress(sourcesToMove, currentDir);
      }

      // Handle rename actions separately (copy with custom name)
      const { invoke } = await import('@tauri-apps/api/core');
      for (const item of itemsToRename) {
        if (item.newName) {
          // Copy file with custom name
          await invoke('copy_file_with_custom_name', {
            sourcePath: item.sourcePath,
            destinationDir: item.targetPath,
            newName: item.newName,
          });

          // If original operation was 'cut', delete the source file
          if (operation.value === 'cut') {
            await deleteItemsWithProgress([item.sourcePath]);
          }
        }
      }

      // Report results
      const skippedCount = itemsToSkip.length;
      const processedCount = itemsToProcess.length;

      if (skippedCount > 0) {
        warning(
          'Some files skipped',
          `${processedCount} file(s) pasted, ${skippedCount} skipped`
        );
      } else {
        success('Pasted', `${processedCount} file(s) pasted successfully`);
      }

      // Refresh directory after paste
      await refreshDirectory(currentPath);

      // Reset saved resolution
      resetSavedResolution();
    } catch (err) {
      showError('Paste failed', err instanceof Error ? err.message : 'Unknown error');
      resetSavedResolution();
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
