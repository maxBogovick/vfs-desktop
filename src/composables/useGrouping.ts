import {ref} from 'vue';
import type {FileItem} from '../types';

export type GroupBy = 'none' | 'type' | 'date' | 'size' | 'extension' | 'name';

export interface FileGroup {
  id: string;
  name: string;
  icon: string;
  color: string;
  items: FileItem[];
  sortOrder: number;
}

function extractDateFrom(file: FileItem) {
  if (file.modified) {
    return file.modified;
  } else if(file.created) {
    return file.created;
  }
  return "01.01.1970";
}

export function useGrouping() {
  const groupBy = ref<GroupBy>('none');

  const groupByOptions = [
    { value: 'none', label: 'No Grouping', icon: '▪️' },
    { value: 'type', label: 'Type', icon: '📁' },
    { value: 'date', label: 'Date Modified', icon: '📅' },
    { value: 'size', label: 'Size', icon: '📏' },
    { value: 'extension', label: 'Extension', icon: '🏷️' },
    { value: 'name', label: 'Name', icon: '🔤' },
  ] as const;

  const getSizeCategory = (size: number): { name: string; order: number } => {
    if (size === 0) return { name: 'Empty', order: 0 };
    if (size < 1024 * 10) return { name: 'Tiny (< 10 KB)', order: 1 };
    if (size < 1024 * 100) return { name: 'Small (< 100 KB)', order: 2 };
    if (size < 1024 * 1024) return { name: 'Medium (< 1 MB)', order: 3 };
    if (size < 1024 * 1024 * 10) return { name: 'Large (< 10 MB)', order: 4 };
    if (size < 1024 * 1024 * 100) return { name: 'Very Large (< 100 MB)', order: 5 };
    return { name: 'Huge (> 100 MB)', order: 6 };
  };

  const getDateCategory = (dateStr: string): { name: string; order: number } => {
    if (!dateStr) return { name: 'Unknown', order: 99 };

    try {
      // Parse date string - handle formats like "12/28/2025", "Dec 28, 2025", "2025-12-28"
      let date = new Date(dateStr);

      // Validate parsed date
      if (isNaN(date.getTime())) {
        const [day, month, year] = dateStr.split('.').map(Number);

// Создаём объект Date, учитывая, что месяцы в JavaScript нумеруются с 0
         date = new Date(year, month - 1, day);
         if (isNaN(date.getTime())) {
           console.warn('[Grouping] Invalid date format:', dateStr);
           return {name: 'Unknown', order: 99};
         }
      }

      const now = new Date();

      // Set time to midnight for accurate day comparison
      const dateOnly = new Date(date.getFullYear(), date.getMonth(), date.getDate());
      const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      const yesterday = new Date(today);
      yesterday.setDate(yesterday.getDate() - 1);
      const thisWeekStart = new Date(today);
      thisWeekStart.setDate(thisWeekStart.getDate() - today.getDay());
      const thisMonthStart = new Date(now.getFullYear(), now.getMonth(), 1);
      const thisYearStart = new Date(now.getFullYear(), 0, 1);

      if (dateOnly.getTime() >= today.getTime()) return { name: 'Today', order: 0 };
      if (dateOnly.getTime() >= yesterday.getTime()) return { name: 'Yesterday', order: 1 };
      if (dateOnly.getTime() >= thisWeekStart.getTime()) return { name: 'This Week', order: 2 };
      if (dateOnly.getTime() >= thisMonthStart.getTime()) return { name: 'This Month', order: 3 };
      if (dateOnly.getTime() >= thisYearStart.getTime()) return { name: 'This Year', order: 4 };

      const year = date.getFullYear();
      return { name: `${year}`, order: 5 + (now.getFullYear() - year) };
    } catch (error) {
      console.error('[Grouping] Date parsing error:', error, 'for date:', dateStr);
      return { name: 'Unknown', order: 99 };
    }
  };

  const getTypeInfo = (type: string): { name: string; icon: string; color: string; order: number } => {
    const typeMap: Record<string, { name: string; icon: string; color: string; order: number }> = {
      folder: { name: 'Folders', icon: '📁', color: '#F59E0B', order: 0 },
      drive: { name: 'Drives', icon: '💾', color: '#8B5CF6', order: 1 },
      image: { name: 'Images', icon: '🖼️', color: '#10B981', order: 2 },
      video: { name: 'Videos', icon: '🎬', color: '#EF4444', order: 3 },
      audio: { name: 'Audio', icon: '🎵', color: '#EC4899', order: 4 },
      pdf: { name: 'PDF Documents', icon: '📄', color: '#DC2626', order: 5 },
      code: { name: 'Code Files', icon: '📜', color: '#3B82F6', order: 6 },
      archive: { name: 'Archives', icon: '📦', color: '#F97316', order: 7 },
      system: { name: 'System Files', icon: '⚙️', color: '#6B7280', order: 8 },
      file: { name: 'Other Files', icon: '📄', color: '#9CA3AF', order: 9 },
    };
    return typeMap[type] || typeMap.file;
  };

  const getExtensionInfo = (fileName: string): { ext: string; order: number } => {
    const parts = fileName.split('.');
    if (parts.length < 2) return { ext: 'No Extension', order: 999 };
    const ext = parts[parts.length - 1].toLowerCase();
    return { ext: ext.toUpperCase(), order: 0 };
  };

  const getNameCategory = (name: string): { category: string; order: number } => {
    const firstChar = name.charAt(0).toUpperCase();
    if (firstChar >= 'A' && firstChar <= 'Z') {
      return { category: firstChar, order: firstChar.charCodeAt(0) - 65 };
    }
    if (firstChar >= '0' && firstChar <= '9') {
      return { category: '0-9', order: -1 };
    }
    return { category: 'Other', order: 100 };
  };

  const groupFiles = (files: FileItem[]): FileGroup[] => {
    if (groupBy.value === 'none') {
      return [{
        id: 'all',
        name: 'All Items',
        icon: '📋',
        color: '#6B7280',
        items: files,
        sortOrder: 0,
      }];
    }

    const groups = new Map<string, FileGroup>();

    files.forEach(file => {
      let groupId: string;
      let groupName: string;
      let groupIcon: string;
      let groupColor: string;
      let sortOrder: number;

      switch (groupBy.value) {
        case 'type': {
          const typeInfo = getTypeInfo(file.type);
          groupId = file.type;
          groupName = typeInfo.name;
          groupIcon = typeInfo.icon;
          groupColor = typeInfo.color;
          sortOrder = typeInfo.order;
          break;
        }

        case 'date': {
          const dateCategory = getDateCategory(extractDateFrom(file));
          groupId = dateCategory.name;
          groupName = dateCategory.name;
          groupIcon = '📅';
          groupColor = '#3B82F6';
          sortOrder = dateCategory.order;
          break;
        }

        case 'size': {
          const sizeCategory = getSizeCategory(file.size || 0);
          groupId = sizeCategory.name;
          groupName = sizeCategory.name;
          groupIcon = '📏';
          groupColor = '#10B981';
          sortOrder = sizeCategory.order;
          break;
        }

        case 'extension': {
          const extInfo = getExtensionInfo(file.name);
          groupId = extInfo.ext;
          groupName = extInfo.ext;
          groupIcon = '🏷️';
          groupColor = '#8B5CF6';
          sortOrder = extInfo.order + (extInfo.ext === 'No Extension' ? 0 : file.name.charCodeAt(0));
          break;
        }

        case 'name': {
          const nameCategory = getNameCategory(file.name);
          groupId = nameCategory.category;
          groupName = nameCategory.category;
          groupIcon = '🔤';
          groupColor = '#EC4899';
          sortOrder = nameCategory.order;
          break;
        }

        default:
          groupId = 'all';
          groupName = 'All Items';
          groupIcon = '📋';
          groupColor = '#6B7280';
          sortOrder = 0;
      }

      if (!groups.has(groupId)) {
        groups.set(groupId, {
          id: groupId,
          name: groupName,
          icon: groupIcon,
          color: groupColor,
          items: [],
          sortOrder,
        });
      }

      groups.get(groupId)!.items.push(file);
    });

    // Sort groups by sortOrder
    return Array.from(groups.values()).sort((a, b) => a.sortOrder - b.sortOrder);
  };

  return {
    groupBy,
    groupByOptions,
    groupFiles,
  };
}
