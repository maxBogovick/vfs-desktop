import { onMounted, onUnmounted } from 'vue';

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean;
  callback: () => void;
  description: string;
}

export function useKeyboard(shortcuts: KeyboardShortcut[]) {
  const handleKeyDown = (event: KeyboardEvent) => {
    const matchingShortcut = shortcuts.find(shortcut => {
      const keyMatch = shortcut.key.toLowerCase() === event.key.toLowerCase();
      if (!keyMatch) return false;

      // Check if this shortcut requires any modifiers
      const requiresModifiers = shortcut.ctrl || shortcut.shift || shortcut.alt || shortcut.meta;

      if (requiresModifiers) {
        // For shortcuts WITH modifiers: required modifiers MUST be pressed, others don't matter
        const ctrlMatch = !shortcut.ctrl || event.ctrlKey;
        const shiftMatch = !shortcut.shift || event.shiftKey;
        const altMatch = !shortcut.alt || event.altKey;
        const metaMatch = !shortcut.meta || event.metaKey;
        return ctrlMatch && shiftMatch && altMatch && metaMatch;
      } else {
        // For shortcuts WITHOUT modifiers: NO modifiers should be pressed
        const noModifiers = !event.ctrlKey && !event.shiftKey && !event.altKey && !event.metaKey;
        return noModifiers;
      }
    });

    if (matchingShortcut) {
      // Ignore shortcuts without modifiers when typing in input/textarea/select elements
      const target = event.target as HTMLElement;
      const isEditableElement =
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT' ||
        target.isContentEditable;

      const hasModifiers = matchingShortcut.ctrl || matchingShortcut.shift || matchingShortcut.alt || matchingShortcut.meta;

      // Only block shortcuts without modifiers in editable elements
      if (isEditableElement && !hasModifiers) {
        return;
      }

      event.preventDefault();
      matchingShortcut.callback();
    }
  };

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });

  return {
    shortcuts,
  };
}
