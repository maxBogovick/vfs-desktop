import { ref } from 'vue';
import type { FileItem } from '../types';

// Module-level shared context menu state (singleton)
const contextMenu = ref<{ x: number; y: number; item: FileItem } | null>(null);

export function useContextMenu() {
  const showContextMenu = (item: FileItem, event: MouseEvent) => {
    event.preventDefault();
    contextMenu.value = { x: event.clientX, y: event.clientY, item };
  };

  const closeContextMenu = () => {
    contextMenu.value = null;
  };

  return {
    contextMenu,
    showContextMenu,
    closeContextMenu,
  };
}
