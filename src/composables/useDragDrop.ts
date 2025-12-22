import { ref, computed } from 'vue';
import type { FileItem } from '../types';

export function useDragDrop() {
  const isDragging = ref(false);
  const draggedItems = ref<FileItem[]>([]);
  const dropTarget = ref<FileItem | null>(null);
  const dragOverId = ref<string | null>(null);

  const hasDraggedItems = computed(() => draggedItems.value.length > 0);

  // Start dragging
  const startDrag = (items: FileItem[], event: DragEvent) => {
    isDragging.value = true;
    draggedItems.value = items;

    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'copyMove';
      event.dataTransfer.setData('text/plain', JSON.stringify(items.map(i => i.path)));
    }
  };

  // Handle drag over
  const handleDragOver = (item: FileItem | null, event: DragEvent) => {
    event.preventDefault();

    // Only allow drop on folders
    if (item && item.type === 'folder') {
      dropTarget.value = item;
      dragOverId.value = item.id;

      if (event.dataTransfer) {
        event.dataTransfer.dropEffect = event.ctrlKey || event.metaKey ? 'copy' : 'move';
      }
    } else {
      dropTarget.value = null;
      dragOverId.value = null;
    }
  };

  // Handle drag leave
  const handleDragLeave = (item: FileItem) => {
    if (dragOverId.value === item.id) {
      dropTarget.value = null;
      dragOverId.value = null;
    }
  };

  // Handle drop
  const handleDrop = async (
    item: FileItem | null,
    event: DragEvent,
    onMove: (sources: string[], destination: string) => Promise<void>,
    onCopy: (sources: string[], destination: string) => Promise<void>
  ) => {
    event.preventDefault();

    if (!item || item.type !== 'folder') {
      endDrag();
      return;
    }

    const isCopy = event.ctrlKey || event.metaKey;
    const sources = draggedItems.value.map(i => i.path);
    const destination = item.path;

    try {
      if (isCopy) {
        await onCopy(sources, destination);
      } else {
        await onMove(sources, destination);
      }
    } catch (error) {
      console.error('Drop failed:', error);
    } finally {
      endDrag();
    }
  };

  // End dragging
  const endDrag = () => {
    isDragging.value = false;
    draggedItems.value = [];
    dropTarget.value = null;
    dragOverId.value = null;
  };

  // Check if item is drag target
  const isDragTarget = (id: string) => {
    return dragOverId.value === id;
  };

  // Check if item is being dragged
  const isBeingDragged = (id: string) => {
    return draggedItems.value.some(item => item.id === id);
  };

  return {
    isDragging,
    draggedItems,
    dropTarget,
    dragOverId,
    hasDraggedItems,
    startDrag,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    endDrag,
    isDragTarget,
    isBeingDragged,
  };
}
