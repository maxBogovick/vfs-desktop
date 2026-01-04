import { ref, computed } from 'vue';
import type { FileItem, FileSystemBackend } from '../types';
import { useDualPanel } from './useDualPanel';
import { useNotifications } from './useNotifications';

// GLOBAL STATE - создаём один раз и переиспользуем
const isDragging = ref(false);
const draggedItems = ref<FileItem[]>([]);
const dropTarget = ref<FileItem | null>(null);
const dragOverId = ref<string | null>(null);

export function useDragDrop() {
  // Используем глобальное состояние вместо создания нового

  const hasDraggedItems = computed(() => draggedItems.value.length > 0);

  // Start dragging
  const startDrag = (items: FileItem[], event: DragEvent) => {
    console.log('[useDragDrop] startDrag called with', items.length, 'items');
    isDragging.value = true;
    draggedItems.value = items;
    console.log('[useDragDrop] State updated:', { isDragging: isDragging.value, count: draggedItems.value.length });

    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'copyMove';
      event.dataTransfer.setData('text/plain', JSON.stringify(items.map(i => i.path)));
    }
  };

  // Handle drag over
  const handleDragOver = (item: FileItem | null, event: DragEvent) => {
    event.preventDefault(); // Обязательно
    // event.stopPropagation(); // Можно добавить, чтобы не всплывало к фону

    if (item && item.type === 'folder') {
      dropTarget.value = item;
      dragOverId.value = item.id;
      if (event.dataTransfer) {
        event.dataTransfer.dropEffect = event.ctrlKey || event.metaKey ? 'copy' : 'move';
      }
    } else {
      // Если навели на файл (не папку), сброс "внутрь" запрещен,
      // но мы должны очистить подсветку
      dropTarget.value = null;
      dragOverId.value = null;
      // Важно: здесь мы НЕ запрещаем dropEffect, чтобы событие могло всплыть к контейнеру,
      // либо можно установить 'none', если хотим запретить сброс НА файл.
      if (event.dataTransfer) {
        event.dataTransfer.dropEffect = 'none';
      }
    }
  };

  // 2. Добавляем специальный handler для фона (пустого места)
  const handleDragOverBackground = (event: DragEvent) => {
    event.preventDefault(); // Обязательно! Без этого drop не сработает

    // Сбрасываем подсветку конкретных папок, так как мы на фоне
    dropTarget.value = null;
    dragOverId.value = null;

    if (event.dataTransfer) {
      // ЯВНО разрешаем сброс
      event.dataTransfer.dropEffect = event.ctrlKey || event.metaKey ? 'copy' : 'move';
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
    onCopy: (sources: string[], destination: string) => Promise<void>,
    destinationFs?: FileSystemBackend
  ) => {
    event.preventDefault();

    // Validate filesystem compatibility
    const { isDualMode, activePanel, leftPanelFilesystem, rightPanelFilesystem } = useDualPanel();
    
    if (isDualMode.value && destinationFs) {
      const sourceFs = activePanel.value === 'left' ? leftPanelFilesystem.value : rightPanelFilesystem.value;
      
      if (sourceFs !== destinationFs) {
        const { error } = useNotifications();
        error('Invalid operation', 'Cannot move/copy files between different filesystem types');
        endDrag();
        return;
      }
    }

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
    console.log('[useDragDrop] endDrag called');
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
    handleDragOverBackground,
    handleDragLeave,
    handleDrop,
    endDrag,
    isDragTarget,
    isBeingDragged,
  };
}
