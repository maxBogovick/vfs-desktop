import type { KeyboardShortcut } from '../composables/useKeyboard';
import type { FileItem } from '../types';

interface ShortcutHandlers {
  openCommandPalette: () => void;
  closeDialogs: () => void;
  selectAll: (files: FileItem[]) => void;
  addTab: () => void;
  closeTab: (canClose: boolean) => void;
  goUp: (canGoUp: boolean) => void;
  handleCopy: () => void;
  handleCut: () => void;
  handlePaste: () => void;
  handleDelete: () => void;
  handleRename: () => void;
  handleRefresh: () => void;
  handleNewFolder: () => void;
  openSettings: () => void;
  // Keyboard navigation
  moveFocusUp: () => void;
  moveFocusDown: () => void;
  moveFocusToFirst: () => void;
  moveFocusToLast: () => void;
  selectFocused: () => void;
  toggleFocusedSelection: () => void;
  openFocusedItem: () => void;
}

export function createKeyboardShortcuts(
  handlers: ShortcutHandlers,
  files: () => FileItem[]
): KeyboardShortcut[] {
  return [
    {
      key: 'k',
      ctrl: true,
      description: 'Open command palette',
      callback: handlers.openCommandPalette,
    },
    {
      key: 'Escape',
      description: 'Close dialogs',
      callback: handlers.closeDialogs,
    },
    {
      key: 'a',
      ctrl: true,
      description: 'Select all',
      callback: () => handlers.selectAll(files()),
    },
    {
      key: 't',
      ctrl: true,
      description: 'New tab',
      callback: handlers.addTab,
    },
    {
      key: 'w',
      ctrl: true,
      description: 'Close tab',
      callback: () => handlers.closeTab(true),
    },
    {
      key: 'Backspace',
      description: 'Go up',
      callback: () => handlers.goUp(true),
    },
    {
      key: 'c',
      ctrl: true,
      description: 'Copy',
      callback: handlers.handleCopy,
    },
    {
      key: 'x',
      ctrl: true,
      description: 'Cut',
      callback: handlers.handleCut,
    },
    {
      key: 'v',
      ctrl: true,
      description: 'Paste',
      callback: handlers.handlePaste,
    },
    {
      key: 'Delete',
      description: 'Delete',
      callback: handlers.handleDelete,
    },
    {
      key: 'F2',
      description: 'Rename',
      callback: handlers.handleRename,
    },
    {
      key: 'F5',
      description: 'Refresh',
      callback: handlers.handleRefresh,
    },
    {
      key: 'n',
      ctrl: true,
      shift: true,
      description: 'New folder',
      callback: handlers.handleNewFolder,
    },
    {
      key: ',',
      ctrl: true,
      description: 'Open settings',
      callback: handlers.openSettings,
    },
    // Keyboard navigation
    {
      key: 'ArrowUp',
      description: 'Move focus up',
      callback: handlers.moveFocusUp,
    },
    {
      key: 'ArrowDown',
      description: 'Move focus down',
      callback: handlers.moveFocusDown,
    },
    {
      key: 'Home',
      description: 'Move focus to first',
      callback: handlers.moveFocusToFirst,
    },
    {
      key: 'End',
      description: 'Move focus to last',
      callback: handlers.moveFocusToLast,
    },
    {
      key: ' ',
      description: 'Toggle selection',
      callback: handlers.toggleFocusedSelection,
    },
    {
      key: 'Enter',
      description: 'Open item',
      callback: handlers.openFocusedItem,
    },
  ];
}
