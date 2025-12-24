export type FileType = 'folder' | 'drive' | 'system' | 'file' | 'image' | 'code' | 'pdf' | 'video' | 'audio' | 'archive';

export type ViewMode = 'grid' | 'list' | 'details';

export type SortBy = 'name' | 'date' | 'size' | 'type';

export type SortOrder = 'asc' | 'desc';

export interface FileItem {
  id: string;
  name: string;
  type: FileType;
  size?: number;
  sizeFormatted?: string;
  modified?: string;
  created?: string;
  accessed?: string;
  free?: string;
  tags?: string[];
  icon?: string;
  path: string;
  isHidden?: boolean;
  permissions?: {
    readable: boolean;
    writable: boolean;
    executable: boolean;
  };
}

export interface Tab {
  id: number;
  path: string[];
  name: string;
  history: string[][];
  historyIndex: number;
}

export interface NavigationState {
  currentPath: string[];
  tabs: Tab[];
  activeTabId: number;
}

export interface SelectionState {
  selectedIds: Set<string>;
  lastSelectedId: string | null;
  rangeAnchorId: string | null;
}

export interface SearchFilter {
  query: string;
  fileTypes: FileType[];
  dateRange?: {
    from: Date;
    to: Date;
  };
  sizeRange?: {
    min: number;
    max: number;
  };
  tags?: string[];
}

export interface ContextMenuPosition {
  x: number;
  y: number;
  item: FileItem | null;
}

export interface PreviewState {
  file: FileItem | null;
  isLoading: boolean;
  content?: string;
  error?: string;
}

export interface DragState {
  isDragging: boolean;
  draggedItems: FileItem[];
  dropTarget: FileItem | null;
}

export interface AppSettings {
  viewMode: ViewMode;
  sortBy: SortBy;
  sortOrder: SortOrder;
  showHidden: boolean;
  theme: 'classic' | 'luna' | 'royale';
  sidebarWidth: number;
  previewWidth: number;
}

export interface FileSystemEntry {
  path: string;
  name: string;
  isDir: boolean;
  isFile: boolean;
  size?: number;
  modified?: number;
  created?: number;
  accessed?: number;
}

export type FileSystemBackend = 'real' | 'virtual';

export interface AppConfig {
  filesystem_backend: FileSystemBackend;
  show_hidden_files: boolean;
  default_view_mode: ViewMode;
  theme: string;
}
