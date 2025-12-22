import { ref, computed } from 'vue';
import type { FileItem } from '../types';

export function useSelection() {
  const selectedIds = ref<Set<string>>(new Set());
  const lastSelectedId = ref<string | null>(null);
  const rangeAnchorId = ref<string | null>(null);

  const selectedCount = computed(() => selectedIds.value.size);

  const hasSelection = computed(() => selectedIds.value.size > 0);

  const isSingleSelection = computed(() => selectedIds.value.size === 1);

  // Get all selected IDs as array
  const selectedIdsArray = computed(() => Array.from(selectedIds.value));

  // Toggle selection for an item
  const toggleSelection = (id: string, multi: boolean = false) => {
    if (!multi) {
      selectedIds.value.clear();
    }

    if (selectedIds.value.has(id)) {
      selectedIds.value.delete(id);
    } else {
      selectedIds.value.add(id);
    }

    lastSelectedId.value = id;
    rangeAnchorId.value = id;
  };

  // Select single item (clear others)
  const selectSingle = (id: string) => {
    selectedIds.value.clear();
    selectedIds.value.add(id);
    lastSelectedId.value = id;
    rangeAnchorId.value = id;
  };

  // Add to selection (multi-select)
  const addToSelection = (id: string) => {
    selectedIds.value.add(id);
    lastSelectedId.value = id;
  };

  // Remove from selection
  const removeFromSelection = (id: string) => {
    selectedIds.value.delete(id);
  };

  // Select range between anchor and target
  const selectRange = (items: FileItem[], targetId: string) => {
    if (!rangeAnchorId.value) {
      selectSingle(targetId);
      return;
    }

    const anchorIndex = items.findIndex(item => item.id === rangeAnchorId.value);
    const targetIndex = items.findIndex(item => item.id === targetId);

    if (anchorIndex === -1 || targetIndex === -1) return;

    const start = Math.min(anchorIndex, targetIndex);
    const end = Math.max(anchorIndex, targetIndex);

    selectedIds.value.clear();
    for (let i = start; i <= end; i++) {
      selectedIds.value.add(items[i].id);
    }

    lastSelectedId.value = targetId;
  };

  // Select all items
  const selectAll = (items: FileItem[]) => {
    selectedIds.value.clear();
    items.forEach(item => selectedIds.value.add(item.id));
    lastSelectedId.value = items.length > 0 ? items[items.length - 1].id : null;
  };

  // Clear selection
  const clearSelection = () => {
    selectedIds.value.clear();
    lastSelectedId.value = null;
    rangeAnchorId.value = null;
  };

  // Check if item is selected
  const isSelected = (id: string) => {
    return selectedIds.value.has(id);
  };

  // Handle item click with modifiers
  const handleItemClick = (item: FileItem, items: FileItem[], event: MouseEvent) => {
    if (event.shiftKey) {
      // Range selection
      selectRange(items, item.id);
    } else if (event.ctrlKey || event.metaKey) {
      // Multi-selection toggle
      toggleSelection(item.id, true);
    } else {
      // Single selection
      selectSingle(item.id);
    }
  };

  // Get selected items
  const getSelectedItems = (items: FileItem[]): FileItem[] => {
    return items.filter(item => selectedIds.value.has(item.id));
  };

  // Invert selection
  const invertSelection = (items: FileItem[]) => {
    const newSelection = new Set<string>();
    items.forEach(item => {
      if (!selectedIds.value.has(item.id)) {
        newSelection.add(item.id);
      }
    });
    selectedIds.value = newSelection;
  };

  return {
    selectedIds,
    lastSelectedId,
    rangeAnchorId,
    selectedCount,
    hasSelection,
    isSingleSelection,
    selectedIdsArray,
    toggleSelection,
    selectSingle,
    addToSelection,
    removeFromSelection,
    selectRange,
    selectAll,
    clearSelection,
    isSelected,
    handleItemClick,
    getSelectedItems,
    invertSelection,
  };
}
