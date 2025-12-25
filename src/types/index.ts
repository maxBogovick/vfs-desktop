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

export interface Bookmark {
  id: string;
  name: string;
  path: string;
  created_at: number;
}

export interface WindowState {
  width?: number;
  height?: number;
  x?: number;
  y?: number;
  maximized: boolean;
}

export interface SidebarState {
  expanded_folders: string[];
  quick_access_expanded: boolean;
  folder_tree_expanded: boolean;
  favorites_expanded: boolean;
}

export interface TabState {
  id: number;
  path: string[];
  name: string;
}

export interface UIState {
  sidebar_width: number;
  preview_width: number;
  tabs: TabState[];
  active_tab_id?: number;
  last_path?: string[];
  window: WindowState;
  sidebar: SidebarState;
}

export interface AppConfig {
  filesystem_backend: FileSystemBackend;
  show_hidden_files: boolean;
  default_view_mode: ViewMode;
  theme: string;
  bookmarks: Bookmark[];
  ui_state: UIState;
}

export type OperationType = 'copy' | 'move' | 'delete';

export type OperationStatus = 'running' | 'paused' | 'completed' | 'cancelled' | 'failed';

export interface ProgressEvent {
  operationId: string;
  operationType: OperationType;
  status: OperationStatus;
  currentBytes: number;
  totalBytes: number;
  currentItems: number;
  totalItems: number;
  currentFile: string | null;
  speedBytesPerSec: number;
  etaSeconds: number | null;
  errorMessage: string | null;
}

export interface FileOperation {
  id: string;
  type: OperationType;
  status: OperationStatus;
  progress: ProgressEvent;
  startTime: number;
}
