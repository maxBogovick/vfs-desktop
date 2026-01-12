import { invoke } from '@tauri-apps/api/core';
import type { FileItem } from '../types';
import { useContextMenu } from './useContextMenu';
import { useFileOperations } from './useFileOperations';
import { useNotifications } from './useNotifications';
import { useDialogs } from './useDialogs';
import { useBookmarks } from './useBookmarks';
import { useFileSystem } from './useFileSystem';
import { useArchiveOperations } from './useArchiveOperations';
import { useOperationsQueue } from './useOperationsQueue';

/**
 * Параметры для создания обработчиков контекстного меню
 */
export interface ContextMenuActionsParams {
  isDualMode: () => boolean;
  getActivePanelMethods: () => any;
  getActivePanelPath: () => string[];
  getCurrentPath: () => string[];
  getSelectedItems: () => FileItem[];
  clearSelection: () => void;
  refreshCurrentDirectory: () => Promise<void>;
  openEditor: (item: FileItem, panelFs?: string) => void;
  openBatchRename: (files: FileItem[]) => void;
  openBatchAttribute: (files: FileItem[]) => void;
  getActiveFilesystem: () => string | undefined;
}

/**
 * Composable для создания обработчиков контекстного меню
 * Централизует всю логику действий из контекстного меню
 */
export function useContextMenuActions(params: ContextMenuActionsParams) {
  const { contextMenu } = useContextMenu();
  const { handleCopy, handleCut, handlePaste, handleDelete, handleRename, handleNewFolder, handleOpenFile } = useFileOperations(params.refreshCurrentDirectory);
  const { success, error } = useNotifications();
  const { showInput, showConfirm, propertiesDialog } = useDialogs();
  const { addBookmark } = useBookmarks();
  const { openTerminal } = useFileSystem();
  const { extractHere, extractToFolder, compress } = useArchiveOperations();
  const { addOperation } = useOperationsQueue();

  // Helper: Get current directory path as string
  const getCurrentDirPath = (): string => {
    const pathArray = params.isDualMode()
      ? params.getActivePanelPath()
      : params.getCurrentPath();
    return '/' + pathArray.join('/');
  };

  // Helper: Handle queue operation
  const handleQueueOperation = async (operationType: 'copy' | 'move' | 'delete' | 'archive' | 'extract') => {
    try {
      let sourceFs: string | undefined = params.getActiveFilesystem();
      let destinationFs: string | undefined = sourceFs;

      const selectedItems = params.getSelectedItems();
      if (selectedItems.length === 0) return;

      // For Copy and Move, we need to ask for destination
      if (operationType === 'copy' || operationType === 'move') {
        showInput(
          operationType === 'copy' ? 'Copy to...' : 'Move to...',
          'Enter destination path:',
          async (destinationPath: string) => {
            if (destinationPath) {
              await executeQueueOperation(operationType, selectedItems, destinationPath, sourceFs, destinationFs);
            }
          },
          '/',
          '/path/to/destination'
        );
        return;
      }

      // For Delete, Extract, Archive - execute directly
      await executeQueueOperation(operationType, selectedItems, undefined, sourceFs, destinationFs);
    } catch (err) {
      error('Failed to add to queue', err instanceof Error ? err.message : String(err));
    }
  };

  // Execute queue operation
  const executeQueueOperation = async (
    operationType: 'copy' | 'move' | 'delete' | 'archive' | 'extract',
    items: FileItem[],
    destination?: string,
    sourceFs?: string,
    destFs?: string
  ) => {
    try {
      for (const item of items) {
        let params: any;
        let description = '';

        switch (operationType) {
          case 'copy':
            params = {
              type: 'Copy',
              sources: [item.path],
              destination: destination || '',
              sourceFs: sourceFs,
              destFs: destFs,
            };
            description = `Copy ${item.name} to ${destination}`;
            break;

          case 'move':
            params = {
              type: 'Move',
              sources: [item.path],
              destination: destination || '',
              sourceFs: sourceFs,
              destFs: destFs,
            };
            description = `Move ${item.name} to ${destination}`;
            break;

          case 'delete':
            params = {
              type: 'Delete',
              paths: [item.path],
              panelFs: sourceFs,
              sourceFs: sourceFs,
            };
            description = `Delete ${item.name}`;
            break;

          case 'archive':
            const currentDir = getCurrentDirPath();
            const archiveName = `${item.name}.zip`;
            const archivePathVal = currentDir.endsWith('/') ? `${currentDir}${archiveName}` : `${currentDir}/${archiveName}`;

            params = {
              type: 'Archive',
              sources: [item.path],
              archivePath: archivePathVal,
              format: 'zip',
              sourceFs: sourceFs,
              destFs: destFs,
            };
            description = `Create archive ${archiveName}`;
            break;

          case 'extract':
            const extractDir = getCurrentDirPath();
            params = {
              type: 'Extract',
              archivePath: item.path,
              destination: extractDir,
              sourceFs: sourceFs,
              destFs: destFs,
            };
            description = `Extract ${item.name}`;
            break;

          default:
            throw new Error(`Unknown operation type: ${operationType}`);
        }

        await addOperation(operationType, params, {
          priority: 'normal',
          description,
          tags: ['context-menu'],
        });
      }

      success('Added to Queue', `${items.length} item(s) added to operations queue`);
    } catch (err) {
      error('Failed to add to queue', err instanceof Error ? err.message : String(err));
    }
  };

  // Context menu action handlers
  const handlers = {
    open: () => {
      if (contextMenu.value?.item) {
        handleOpenFile(contextMenu.value.item);
      }
    },

    edit: () => {
      if (contextMenu.value?.item) {
        if (params.isDualMode()) {
          const methods = params.getActivePanelMethods();
          if (methods) methods.handleEditFile();
        } else {
          params.openEditor(contextMenu.value.item);
        }
      }
    },

    copy: () => {
      if (params.isDualMode()) {
        const methods = params.getActivePanelMethods();
        if (methods) methods.handleCopy();
      } else {
        handleCopy(params.getSelectedItems());
      }
    },

    cut: () => {
      if (params.isDualMode()) {
        const methods = params.getActivePanelMethods();
        if (methods) methods.handleCut();
      } else {
        handleCut(params.getSelectedItems());
      }
    },

    paste: () => {
      if (params.isDualMode()) {
        const methods = params.getActivePanelMethods();
        if (methods) methods.handlePaste();
      } else {
        handlePaste(params.getCurrentPath());
      }
    },

    rename: () => {
      if (params.isDualMode()) {
        const methods = params.getActivePanelMethods();
        if (methods) methods.handleRename();
      } else {
        handleRename(params.getSelectedItems(), params.getCurrentPath(), showInput);
      }
    },

    delete: () => {
      if (params.isDualMode()) {
        const methods = params.getActivePanelMethods();
        if (methods) methods.handleDelete();
      } else {
        handleDelete(params.getSelectedItems(), params.getCurrentPath(), params.clearSelection, showConfirm);
      }
    },

    addToFavorites: async () => {
      if (!contextMenu.value?.item) return;
      const item = contextMenu.value.item;

      if (item.type !== 'folder') {
        error('Cannot bookmark', 'Only folders can be added to favorites');
        return;
      }

      const bookmark = await addBookmark(item.path, item.name);
      if (bookmark) {
        success('Added to Favorites', `Added: ${bookmark.name}`);
      } else {
        error('Failed to add bookmark', 'This folder may already be bookmarked');
      }
    },

    openTerminal: async () => {
      if (!contextMenu.value?.item) return;
      const item = contextMenu.value.item;

      try {
        await openTerminal(item.path);
        success('Terminal opened', `Opened terminal in ${item.name}`);
      } catch (err) {
        error('Failed to open terminal', err instanceof Error ? err.message : 'Unknown error');
      }
    },

    extractHere: async () => {
      if (!contextMenu.value?.item) return;
      const currentDir = getCurrentDirPath();
      await extractHere(contextMenu.value.item, currentDir, async () => {
        if (params.isDualMode()) {
          params.getActivePanelMethods()?.handleRefresh();
        } else {
          await params.refreshCurrentDirectory();
        }
      });
    },

    extractToFolder: async () => {
      if (!contextMenu.value?.item) return;
      const currentDir = getCurrentDirPath();
      await extractToFolder(contextMenu.value.item, currentDir, async () => {
        if (params.isDualMode()) {
          params.getActivePanelMethods()?.handleRefresh();
        } else {
          await params.refreshCurrentDirectory();
        }
      });
    },

    compressToZip: async () => {
      const currentDir = getCurrentDirPath();
      await compress(params.getSelectedItems(), 'zip', currentDir, async () => {
        if (params.isDualMode()) {
          params.getActivePanelMethods()?.handleRefresh();
        } else {
          await params.refreshCurrentDirectory();
        }
      });
    },

    compressToTar: async () => {
      const currentDir = getCurrentDirPath();
      await compress(params.getSelectedItems(), 'tar', currentDir, async () => {
        if (params.isDualMode()) {
          params.getActivePanelMethods()?.handleRefresh();
        } else {
          await params.refreshCurrentDirectory();
        }
      });
    },

    compressToTarGz: async () => {
      const currentDir = getCurrentDirPath();
      await compress(params.getSelectedItems(), 'tar.gz', currentDir, async () => {
        if (params.isDualMode()) {
          params.getActivePanelMethods()?.handleRefresh();
        } else {
          await params.refreshCurrentDirectory();
        }
      });
    },

    properties: () => {
      const selected = params.getSelectedItems();
      if (selected.length === 1) {
        propertiesDialog.value = { isOpen: true, file: selected[0] };
      }
    },

    batchRename: () => {
      const selected = params.getSelectedItems();
      if (selected.length > 0) {
        params.openBatchRename(selected);
      }
    },

    batchAttributes: () => {
      const selected = params.getSelectedItems();
      if (selected.length > 0) {
        params.openBatchAttribute(selected);
      }
    },

    refresh: () => {
      if (params.isDualMode()) {
        params.getActivePanelMethods()?.handleRefresh();
      } else {
        params.refreshCurrentDirectory();
      }
    },

    newFolder: () => {
      if (params.isDualMode()) {
        params.getActivePanelMethods()?.handleNewFolder();
      } else {
        handleNewFolder(params.getCurrentPath(), showInput);
      }
    },

    newFile: () => {
      if (params.isDualMode()) {
        params.getActivePanelMethods()?.handleNewFile();
      } else {
        // This should be handled in App.vue by opening inline creator
        // but we can't do it from here without circular dependency
        console.warn('newFile from context menu not implemented in composable');
      }
    },

    selectAll: () => {
      if (params.isDualMode()) {
        params.getActivePanelMethods()?.selectAll();
      } else {
        // This should be handled in App.vue
        console.warn('selectAll from context menu not implemented in composable');
      }
    },

    queueCopy: async () => {
      await handleQueueOperation('copy');
    },

    queueMove: async () => {
      await handleQueueOperation('move');
    },

    queueDelete: async () => {
      await handleQueueOperation('delete');
    },

    queueArchive: async () => {
      await handleQueueOperation('archive');
    },

    queueExtract: async () => {
      await handleQueueOperation('extract');
    },

    share: async () => {
      if (!contextMenu.value?.item) return;

      let currentFs: string | undefined = params.getActiveFilesystem();

      try {
        const result = await invoke<{ url: string; qr_svg: string; filename: string }>('share_file', {
          path: contextMenu.value.item.path,
          filesystem: currentFs,
        });

        // This should be handled in App.vue by showing the share dialog
        // Return result for external handling
        return result;
      } catch (err) {
        error('Share failed', err instanceof Error ? err.message : String(err));
      }
    },

    hideTo: async () => {
      if (!contextMenu.value?.item) return;
      const item = contextMenu.value.item;

      try {
        // 1. Select Host File (Container)
        // We use vault_select_file to pick an existing media file
        const hostPath = await invoke<string | null>('vault_select_file');
        if (!hostPath) return;

        // 2. Select Output File
        // Where to save the resulting stego-file
        const outputPath = await invoke<string | null>('vault_save_file_dialog');
        if (!outputPath) return;

        // 3. Prompt for Password
        showInput(
          'Encrypt Hidden Data',
          'Enter password to encrypt the hidden data:',
          async (password) => {
            if (!password) return;

            try {
              // 4. Execute Backend Command
              await invoke('vault_hide_path_in_container', {
                sourcePath: item.path,
                hostPath: hostPath,
                outputPath: outputPath,
                password: password
              });

              success('Steganography Success', `Hidden '${item.name}' inside '${outputPath}'`);
            } catch (err) {
              error('Failed to hide file', err instanceof Error ? err.message : String(err));
            }
          },
          '',
          'Password',
          'password'
        );
      } catch (err) {
        error('Failed to hide file', err instanceof Error ? err.message : String(err));
      }
    },

    extractHidden: async () => {
      if (!contextMenu.value?.item) return;
      const item = contextMenu.value.item;

      // Determine output directory: current_dir/filename_extracted
      const currentDir = getCurrentDirPath();
      const outputDirName = `${item.name}_extracted`;
      const outputPath = currentDir.endsWith('/') ? `${currentDir}${outputDirName}` : `${currentDir}/${outputDirName}`;

      showInput(
        'Extract Hidden Data',
        'Enter password to decrypt:',
        async (password) => {
          if (!password) return;

          try {
            await invoke('vault_extract_from_container', {
              containerPath: item.path,
              outputPath: outputPath,
              password: password
            });

            success('Extraction Success', `Extracted to '${outputDirName}'`);
            
            // Refresh view to show new folder
            if (params.isDualMode()) {
              params.getActivePanelMethods()?.handleRefresh();
            } else {
              await params.refreshCurrentDirectory();
            }
          } catch (err) {
            error('Failed to extract', err instanceof Error ? err.message : String(err));
          }
        },
        '',
        'Password',
        'password'
      );
    },
  };

  return handlers;
}
