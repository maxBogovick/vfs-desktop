import { ref, computed } from 'vue';
import type { FileItem, SearchFilter, FileType, SortBy, SortOrder } from '../types';

export function useSearch() {
  const searchQuery = ref('');
  const activeFilters = ref<SearchFilter>({
    query: '',
    fileTypes: [],
    dateRange: undefined,
    sizeRange: undefined,
    tags: [],
  });

  const sortBy = ref<SortBy>('name');
  const sortOrder = ref<SortOrder>('asc');

  // Filter files based on search query and filters
  const filterFiles = (files: FileItem[]): FileItem[] => {
    let filtered = [...files];

    // Text search
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      filtered = filtered.filter(file =>
        file.name.toLowerCase().includes(query)
      );
    }

    // File type filter
    if (activeFilters.value.fileTypes.length > 0) {
      filtered = filtered.filter(file =>
        activeFilters.value.fileTypes.includes(file.type)
      );
    }

    // Date range filter
    if (activeFilters.value.dateRange) {
      filtered = filtered.filter(file => {
        if (!file.modified) return false;
        const fileDate = new Date(file.modified);
        const { from, to } = activeFilters.value.dateRange!;
        return fileDate >= from && fileDate <= to;
      });
    }

    // Size range filter
    if (activeFilters.value.sizeRange) {
      filtered = filtered.filter(file => {
        if (!file.size) return false;
        const { min, max } = activeFilters.value.sizeRange!;
        return file.size >= min && file.size <= max;
      });
    }

    // Tags filter
    if (activeFilters.value.tags && activeFilters.value.tags.length > 0) {
      filtered = filtered.filter(file => {
        if (!file.tags || file.tags.length === 0) return false;
        return activeFilters.value.tags!.some(tag => file.tags!.includes(tag));
      });
    }

    return filtered;
  };

  // Sort files
  const sortFiles = (files: FileItem[]): FileItem[] => {
    const sorted = [...files];

    sorted.sort((a, b) => {
      let comparison = 0;

      switch (sortBy.value) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'date':
          const dateA = a.modified ? new Date(a.modified).getTime() : 0;
          const dateB = b.modified ? new Date(b.modified).getTime() : 0;
          comparison = dateA - dateB;
          break;
        case 'size':
          comparison = (a.size || 0) - (b.size || 0);
          break;
        case 'type':
          comparison = a.type.localeCompare(b.type);
          break;
      }

      return sortOrder.value === 'asc' ? comparison : -comparison;
    });

    // Always put folders first
    return sorted.sort((a, b) => {
      if (a.type === 'folder' && b.type !== 'folder') return -1;
      if (a.type !== 'folder' && b.type === 'folder') return 1;
      return 0;
    });
  };

  // Apply both filtering and sorting
  const processFiles = (files: FileItem[]): FileItem[] => {
    const filtered = filterFiles(files);
    return sortFiles(filtered);
  };

  // Set search query
  const setSearchQuery = (query: string) => {
    searchQuery.value = query;
    activeFilters.value.query = query;
  };

  // Add file type filter
  const addFileTypeFilter = (type: FileType) => {
    if (!activeFilters.value.fileTypes.includes(type)) {
      activeFilters.value.fileTypes.push(type);
    }
  };

  // Remove file type filter
  const removeFileTypeFilter = (type: FileType) => {
    activeFilters.value.fileTypes = activeFilters.value.fileTypes.filter(t => t !== type);
  };

  // Clear file type filters
  const clearFileTypeFilters = () => {
    activeFilters.value.fileTypes = [];
  };

  // Set date range filter
  const setDateRangeFilter = (from: Date, to: Date) => {
    activeFilters.value.dateRange = { from, to };
  };

  // Clear date range filter
  const clearDateRangeFilter = () => {
    activeFilters.value.dateRange = undefined;
  };

  // Set size range filter (in bytes)
  const setSizeRangeFilter = (min: number, max: number) => {
    activeFilters.value.sizeRange = { min, max };
  };

  // Clear size range filter
  const clearSizeRangeFilter = () => {
    activeFilters.value.sizeRange = undefined;
  };

  // Add tag filter
  const addTagFilter = (tag: string) => {
    if (!activeFilters.value.tags) {
      activeFilters.value.tags = [];
    }
    if (!activeFilters.value.tags.includes(tag)) {
      activeFilters.value.tags.push(tag);
    }
  };

  // Remove tag filter
  const removeTagFilter = (tag: string) => {
    if (activeFilters.value.tags) {
      activeFilters.value.tags = activeFilters.value.tags.filter(t => t !== tag);
    }
  };

  // Clear tag filters
  const clearTagFilters = () => {
    activeFilters.value.tags = [];
  };

  // Clear all filters
  const clearAllFilters = () => {
    activeFilters.value = {
      query: searchQuery.value,
      fileTypes: [],
      dateRange: undefined,
      sizeRange: undefined,
      tags: [],
    };
  };

  // Has active filters
  const hasActiveFilters = computed(() => {
    return (
      activeFilters.value.fileTypes.length > 0 ||
      activeFilters.value.dateRange !== undefined ||
      activeFilters.value.sizeRange !== undefined ||
      (activeFilters.value.tags && activeFilters.value.tags.length > 0)
    );
  });

  // Set sort
  const setSorting = (by: SortBy, order?: SortOrder) => {
    sortBy.value = by;
    if (order) {
      sortOrder.value = order;
    }
  };

  // Toggle sort order
  const toggleSortOrder = () => {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc';
  };

  return {
    searchQuery,
    activeFilters,
    sortBy,
    sortOrder,
    hasActiveFilters,
    filterFiles,
    sortFiles,
    processFiles,
    setSearchQuery,
    addFileTypeFilter,
    removeFileTypeFilter,
    clearFileTypeFilters,
    setDateRangeFilter,
    clearDateRangeFilter,
    setSizeRangeFilter,
    clearSizeRangeFilter,
    addTagFilter,
    removeTagFilter,
    clearTagFilters,
    clearAllFilters,
    setSorting,
    toggleSortOrder,
  };
}
