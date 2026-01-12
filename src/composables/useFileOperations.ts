import type {FileItem} from '../types';
import {useFileSystem} from './useFileSystem';
import {useClipboard} from './useClipboard';
import {useNotifications} from './useNotifications';
import {useFileContentCache} from './useFileContentCache';
import {useFileOperationsProgress} from './useFileOperationsProgress';
import {useConflictResolution} from './useConflictResolution';
import {useTemplates} from './useTemplates';

export function useFileOperations(refreshCallback?: () => Promise<void>) {
  const {
    loadDirectory,
    renameItem,
    createFolder,
    createFile,
    createFilesBatch,
    openFile: openFileInApp,
    revealInFinder,
    getHomeDirectory,
  } = useFileSystem();

  const {
    copyItemsWithProgress,
    moveItemsWithProgress,
    deleteItemsWithProgress,
  } = useFileOperationsProgress();

  const {
    hasClipboardItems,
    clipboardItems,
    operation,
    sourceFilesystem,
    copy: copyToClipboard,
    cut: cutToClipboard,
  } = useClipboard();

  const { success, error: showError, warning } = useNotifications();
  const { invalidate: invalidateCache } = useFileContentCache();
  const { getTemplateContent } = useTemplates();

  // Функция для обновления текущей директории
  const refreshDirectory = async (currentPath: string[], panelFs?: string) => {
    if (refreshCallback) {
      await refreshCallback();
    } else {
      const currentDir = await getCurrentDirectory(currentPath, panelFs);
      await loadDirectory(currentDir, panelFs);
    }
  };

  // Helper to get current directory path with proper leading slash
  const getCurrentDirectory = async (currentPath: string[], panelFs?: string): Promise<string> => {
    let pathString = currentPath.join('/');

    // Add leading slash if path is not empty and doesn't start with slash
    if (pathString && !pathString.startsWith('/')) {
      pathString = '/' + pathString;
    }

    if (!pathString) {
      return await getHomeDirectory(panelFs);
    }

    return pathString;
  };

  // Copy selected items to clipboard
  const handleCopy = (selectedItems: FileItem[], panelFs?: string) => {
    if (selectedItems.length > 0) {
      copyToClipboard(selectedItems, panelFs);
      success('Copied', `${selectedItems.length} item(s) copied to clipboard`);
    }
  };

  // Cut selected items to clipboard
  const handleCut = (selectedItems: FileItem[], panelFs?: string) => {
    if (selectedItems.length > 0) {
      cutToClipboard(selectedItems, panelFs);
      warning('Cut', `${selectedItems.length} item(s) cut to clipboard`);
    }
  };

  // Generic handler for file transfer (copy/move) with conflict resolution
  const handleTransfer = async (
    sources: string[],
    destinationPath: string,
    operationType: 'copy' | 'move',
    sourceFs?: string,
    destFs?: string
  ) => {
    const {
      checkConflict,
      requestConflictResolution,
      resetSavedResolution,
    } = useConflictResolution();

    try {
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

      for (const sourcePath of sources) {
        const conflict = await checkConflict(sourcePath, destinationPath, sourceFs, destFs);

        if (conflict) {
          // File exists, ask user what to do
          try {
            const resolution = await requestConflictResolution(conflict);

            if (resolution.action === 'skip') {
              itemsToSkip.push(sourcePath);
              continue;
            }

            if (resolution.action === 'replace') {
              // Just copy/move normally, will overwrite
              itemsToProcess.push({
                sourcePath: sourcePath,
                targetPath: destinationPath,
                action: operationType,
                newName: undefined,
              });
            } else if (resolution.action === 'rename') {
              // Copy/move with new name
              if (resolution.newName) {
                // For rename, we always use copy_file_with_custom_name
                // and then delete source if operation is 'move'
                itemsToProcess.push({
                  sourcePath: sourcePath,
                  targetPath: destinationPath,
                  action: 'rename',
                  newName: resolution.newName,
                });
              }
            }
          } catch (err) {
            // User cancelled conflict resolution
            warning('Operation cancelled', 'File operation was cancelled');
            resetSavedResolution();
            return;
          }
        } else {
          // No conflict, proceed normally
          itemsToProcess.push({
            sourcePath: sourcePath,
            targetPath: destinationPath, // Target directory
            action: operationType,
            newName: undefined,
          });
        }
      }

      // Now perform the operations
      if (itemsToProcess.length === 0) {
        if (itemsToSkip.length > 0) {
           warning('No files transferred', 'All files were skipped');
        }
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
        await copyItemsWithProgress(sourcesToCopy, destinationPath, sourceFs, destFs);
      }

      if (sourcesToMove.length > 0) {
        await moveItemsWithProgress(sourcesToMove, destinationPath, sourceFs, destFs);
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
            sourceFileSystem: sourceFs || null,
            destinationFileSystem: destFs || null,
          });

          // If original operation was 'move', delete the source file
          if (operationType === 'move') {
            await deleteItemsWithProgress([item.sourcePath], sourceFs);
          }
        }
      }

      // Report results
      const skippedCount = itemsToSkip.length;
      const processedCount = itemsToProcess.length;

      if (skippedCount > 0) {
        warning(
          'Some files skipped',
          `${processedCount} file(s) transferred, ${skippedCount} skipped`
        );
      } else {
        success('Transfer complete', `${processedCount} file(s) transferred successfully`);
      }

      // Refresh directory after transfer
      // We assume destination is currentPath if this is drag & drop to current view
      // But handleTransfer receives destinationPath string.
      // We can try to refresh if it matches current context, or caller handles refresh.
      // But useFileOperations has `refreshDirectory`.
      // If we are in FilePanel, we might need to refresh that panel.
      // For now, we will just call refreshDirectory if destinationPath matches currentPath logic?
      // Actually, handlePaste calls refreshDirectory.
      // Here we can't easily know if destinationPath == currentPath array.
      // We will leave refresh to the caller or try to refresh if possible.
      // BUT `handlePaste` refreshes.
      
      // Let's return true/false or something to indicate success so caller can refresh?
      // Or just call refreshCallback if defined?
      if (refreshCallback) {
          await refreshCallback();
      }

      // Reset saved resolution
      resetSavedResolution();
    } catch (err) {
      showError('Transfer failed', err instanceof Error ? err.message : 'Unknown error');
      resetSavedResolution();
    }
  };

  // Paste items from clipboard with conflict resolution
  const handlePaste = async (currentPath: string[], panelFs?: string) => {
    if (!hasClipboardItems.value) {
      warning('Nothing to paste', 'Clipboard is empty');
      return;
    }

    try {
      const currentDir = await getCurrentDirectory(currentPath, panelFs);
      const destinationFs = panelFs;
      const sourceFs = sourceFilesystem.value;
      const sources = clipboardItems.value.map(i => i.path);
      const opType = operation.value === 'cut' ? 'move' : 'copy';

      await handleTransfer(sources, currentDir, opType, sourceFs, destinationFs);
      
      // Refresh directory
      await refreshDirectory(currentPath, panelFs);
    } catch (err) {
      showError('Paste failed', err instanceof Error ? err.message : 'Unknown error');
    }
  };

  // Delete items with confirmation
  const handleDelete = async (
    selectedItems: FileItem[],
    currentPath: string[],
    clearSelection: () => void,
    showConfirm: (title: string, message: string, onConfirm: () => void, type?: 'warning' | 'danger' | 'info') => void,
    panelFs?: string
  ) => {
    if (selectedItems.length === 0) return;

    showConfirm(
      'Confirm Delete',
      `Are you sure you want to permanently delete ${selectedItems.length} item(s)?`,
      async () => {
        try {
          const paths = selectedItems.map(item => item.path);
          await deleteItemsWithProgress(paths, panelFs);
          // Инвалидируем кеш для удаленных файлов
          selectedItems.forEach(item => invalidateCache(item.path));
          // Refresh directory
          await refreshDirectory(currentPath, panelFs);
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
    showInput: (title: string, label: string, onConfirm: (value: string) => void, defaultValue?: string, placeholder?: string, inputType?: string) => void,
    panelFs?: string
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
          await renameItem(item.path, newName, panelFs);
          // Инвалидируем кеш для старого пути
          invalidateCache(item.path);
          // Refresh directory
          await refreshDirectory(currentPath, panelFs);
          success('Renamed', `Renamed to ${newName}`);
        } catch (err) {
          showError('Rename failed', err instanceof Error ? err.message : 'Unknown error');
        }
      },
      item.name,
      'New name',
      'text'
    );
  };

  // Create new folder
  const handleNewFolder = (
    currentPath: string[],
    showInput: (title: string, label: string, onConfirm: (value: string) => void, defaultValue?: string, placeholder?: string, inputType?: string) => void,
    panelFs?: string
  ) => {
    showInput(
      'New Folder',
      'Enter folder name:',
      async (name: string) => {
        if (!name) {
          return;
        }

        try {
          const currentDir = await getCurrentDirectory(currentPath, panelFs);
          await createFolder(currentDir, name, panelFs);
          // Refresh directory
          await refreshDirectory(currentPath, panelFs);
          success('Folder created', `Created ${name}`);
        } catch (err) {
          showError('Create folder failed', err instanceof Error ? err.message : 'Unknown error');
        }
      },
      '',
      'New Folder',
      'text'
    );
  };

  // Create new file (inline mode)
  const handleNewFile = async (
    currentPath: string[],
    name: string,
    templateId?: string,
    panelFs?: string
  ) => {
    if (!name) {
      warning('Invalid name', 'File name cannot be empty');
      return;
    }

    try {
      const currentDir = await getCurrentDirectory(currentPath, panelFs);

      // Get template content if templateId is provided
      let content: string | undefined;
      if (templateId) {
        content = await getTemplateContent(templateId);
      }

      await createFile(currentDir, name, content, panelFs);

      // Refresh directory
      await refreshDirectory(currentPath, panelFs);
      success('File created', `Created ${name}`);
    } catch (err) {
      showError('Create file failed', err instanceof Error ? err.message : 'Unknown error');
      throw err;
    }
  };

  // Batch create files
  const handleBatchCreate = async (
    currentPath: string[],
    files: Array<{ name: string; content?: string; templateId?: string }>,
    panelFs?: string
  ) => {
    if (files.length === 0) {
      warning('No files', 'No files to create');
      return;
    }

    try {
      const currentDir = await getCurrentDirectory(currentPath, panelFs);

      // Prepare files with template content if needed
      const filesWithContent = await Promise.all(
        files.map(async (file) => {
          let content = file.content;

          // Get template content if templateId is provided
          if (file.templateId && !content) {
            content = await getTemplateContent(file.templateId);
          }

          return {
            name: file.name,
            content,
          };
        })
      );

      const result = await createFilesBatch(currentDir, filesWithContent, panelFs);

      // Refresh directory
      await refreshDirectory(currentPath, panelFs);

      // Show result
      if (result.failed && result.failed.length > 0) {
        warning(
          'Some files failed',
          `Created ${result.created.length} file(s), ${result.failed.length} failed`
        );
      } else {
        success('Files created', `Created ${result.created.length} file(s)`);
      }

      return result;
    } catch (err) {
      showError('Batch create failed', err instanceof Error ? err.message : 'Unknown error');
      throw err;
    }
  };

  // Open file in default application
  const handleOpenFile = async (file: FileItem, panelFs?: string) => {
    try {
      await openFileInApp(file.path, panelFs);
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
  const handleRevealInFinder = async (selectedItems: FileItem[], panelFs?: string) => {
    if (selectedItems.length !== 1) return;

    try {
      await revealInFinder(selectedItems[0].path, panelFs);
    } catch (err) {
      showError('Failed to reveal', err instanceof Error ? err.message : 'Unknown error');
    }
  };

  // Refresh directory
  const handleRefresh = async (currentPath: string[], panelFs?: string) => {
    try {
      await refreshDirectory(currentPath, panelFs);
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
    handleNewFile,
    handleBatchCreate,
    handleOpenFile,
    handleProperties,
    handleRevealInFinder,
    handleRefresh,
    handleTransfer,
  };
}