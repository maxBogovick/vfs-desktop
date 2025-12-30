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

export type PanelMode = 'single' | 'dual';
export type ActivePanel = 'left' | 'right';

export interface PanelState {
  tabs: TabState[];
  active_tab_id?: number;
}

export interface DualPanelConfig {
  left_panel_width_percent: number;
  left_panel: PanelState;
  right_panel: PanelState;
  active_panel: ActivePanel;
}

export interface UIState {
  sidebar_width: number;
  preview_width: number;
  tabs: TabState[];
  active_tab_id?: number;
  last_path?: string[];
  panel_mode: PanelMode;
  dual_panel_config: DualPanelConfig;
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

// Conflict Resolution Types
export type ConflictAction = 'skip' | 'replace' | 'rename' | 'compare';

export interface FileConflict {
  sourcePath: string;
  destinationPath: string;
  sourceFile: {
    name: string;
    size: number;
    modified: number;
  };
  destinationFile: {
    name: string;
    size: number;
    modified: number;
  };
}

export interface ConflictResolution {
  action: ConflictAction;
  newName?: string; // For 'rename' action
  applyToAll: boolean;
}

// ===== Batch Operations Types =====

// Batch Rename Pattern Types
export type RenamePatternType = 'prefix' | 'suffix' | 'replace' | 'regex' | 'numbering' | 'case';
export type CaseChangeType = 'uppercase' | 'lowercase' | 'titlecase' | 'camelcase' | 'snakecase' | 'kebabcase';

export interface RenamePattern {
  type: RenamePatternType;
  enabled: boolean;
}

export interface PrefixPattern extends RenamePattern {
  type: 'prefix';
  text: string;
}

export interface SuffixPattern extends RenamePattern {
  type: 'suffix';
  text: string;
  beforeExtension: boolean; // If true, add before file extension
}

export interface ReplacePattern extends RenamePattern {
  type: 'replace';
  searchText: string;
  replaceText: string;
  caseSensitive: boolean;
  wholeWord: boolean;
}

export interface RegexPattern extends RenamePattern {
  type: 'regex';
  pattern: string;
  replacement: string;
  flags: string; // 'g', 'i', 'gi', etc.
}

export interface NumberingPattern extends RenamePattern {
  type: 'numbering';
  startNumber: number;
  increment: number;
  padding: number; // Zero padding (e.g., 3 -> 001, 002, 003)
  position: 'prefix' | 'suffix' | 'replace';
  separator: string; // e.g., '-', '_', ' '
}

export interface CasePattern extends RenamePattern {
  type: 'case';
  caseType: CaseChangeType;
}

export type BatchRenamePattern =
  | PrefixPattern
  | SuffixPattern
  | ReplacePattern
  | RegexPattern
  | NumberingPattern
  | CasePattern;

export interface RenamePreviewItem {
  originalPath: string;
  originalName: string;
  newName: string;
  hasError: boolean;
  errorMessage?: string;
  fileItem: FileItem;
}

export interface BatchRenameConfig {
  patterns: BatchRenamePattern[];
  applyToFolders: boolean;
  applyToFiles: boolean;
  preserveExtension: boolean; // Don't modify file extensions in patterns
}

// Batch Attribute Change Types
export interface PermissionsChange {
  readable?: boolean;
  writable?: boolean;
  executable?: boolean;
  recursive: boolean; // Apply to subdirectories
}

export interface DateChange {
  modified?: number; // Unix timestamp
  created?: number;
  accessed?: number;
}

export interface TagsChange {
  operation: 'add' | 'remove' | 'replace';
  tags: string[];
}

export interface BatchAttributeChange {
  permissions?: PermissionsChange;
  dates?: DateChange;
  tags?: TagsChange;
}

export interface AttributePreviewItem {
  path: string;
  name: string;
  currentAttributes: {
    permissions?: FileItem['permissions'];
    modified?: string;
    created?: string;
    tags?: string[];
  };
  newAttributes: {
    permissions?: FileItem['permissions'];
    modified?: string;
    created?: string;
    tags?: string[];
  };
  fileItem: FileItem;
}

// Batch Operations Queue Types
export type BatchOperationType = 'rename' | 'attribute_change';
export type QueuedOperationStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface QueuedBatchOperation {
  id: string;
  type: BatchOperationType;
  status: QueuedOperationStatus;
  itemsCount: number;
  processedCount: number;
  failedCount: number;
  createdAt: number;
  startedAt?: number;
  completedAt?: number;
  errorMessage?: string;
  config: BatchRenameConfig | BatchAttributeChange;
  items: string[]; // File paths
  results?: BatchOperationResult[];
}

export interface BatchOperationResult {
  path: string;
  success: boolean;
  errorMessage?: string;
  originalName?: string; // For rename operations
  newName?: string;
}

export interface BatchOperationProgress {
  operationId: string;
  status: QueuedOperationStatus;
  currentItem: number;
  totalItems: number;
  currentFile: string | null;
  failedItems: string[];
}

// Validation Types
export interface ValidationError {
  path: string;
  name: string;
  error: string;
}

export interface BatchValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationError[];
}
