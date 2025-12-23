import type { FileItem } from '../types';
import { useNotifications } from './useNotifications';

interface CommandHandlers {
  onNewFolder: () => void;
  onNewFile: () => void;
  onSearch: () => void;
  onGoto: () => void;
  onRefresh: () => Promise<void>;
  onCopyPath: (selectedItems: FileItem[]) => void;
  onSelectAll: (allFiles: FileItem[]) => void;
  onNewTab: () => void;
  onCloseTab: () => void;
  onSettings: () => void;
}

export function useCommands(handlers: CommandHandlers) {
  const { success, warning, info } = useNotifications();

  const executeCommand = (cmd: { id: string }) => {
    const commandHandlers: Record<string, () => void> = {
      'new-folder': handlers.onNewFolder,
      'new-file': () => {
        handlers.onNewFile();
        warning('Not implemented', 'File creation is not yet implemented');
      },
      'search': () => {
        handlers.onSearch();
        info('Search', 'Use the search bar in the toolbar');
      },
      'goto': handlers.onGoto,
      'refresh': handlers.onRefresh,
      'copy-path': () => {
        // This will be called with selectedItems from the caller
      },
      'select-all': () => {
        // This will be called with allFiles from the caller
      },
      'new-tab': handlers.onNewTab,
      'close-tab': handlers.onCloseTab,
      'settings': () => {
        handlers.onSettings();
        info('Settings', 'Settings panel coming soon');
      },
    };

    const handler = commandHandlers[cmd.id];
    if (handler) {
      handler();
    }
  };

  // Specific command implementations
  const copyPathCommand = (selectedItems: FileItem[]) => {
    if (selectedItems.length > 0) {
      const paths = selectedItems.map(item => item.path).join('\n');
      navigator.clipboard.writeText(paths);
      success('Copied path', `${selectedItems.length} path(s) copied to clipboard`);
    } else {
      warning('No selection', 'Please select files to copy their paths');
    }
  };

  const selectAllCommand = (allFiles: FileItem[], selectAll: (files: FileItem[]) => void) => {
    selectAll(allFiles);
    success('Selected all', `${allFiles.length} items selected`);
  };

  const closeTabCommand = (tabsCount: number, closeTab: (tabId: string) => void, activeTabId: string) => {
    if (tabsCount > 1) {
      closeTab(activeTabId);
      success('Tab closed', 'Tab closed successfully');
    } else {
      warning('Cannot close', 'Cannot close the last tab');
    }
  };

  return {
    executeCommand,
    copyPathCommand,
    selectAllCommand,
    closeTabCommand,
  };
}
