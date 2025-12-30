# VFDir Architecture

Modern, professional architecture for Windows XP-style file explorer built with Vue 3, TypeScript, and Tailwind CSS.

## Project Structure

```
src/
├── types/
│   └── index.ts              # TypeScript types and interfaces
├── composables/
│   ├── useFileSystem.ts      # File system operations (Tauri integration)
│   ├── useNavigation.ts      # Navigation, tabs, history
│   ├── useSelection.ts       # File selection logic
│   ├── useSearch.ts          # Search and filtering
│   ├── useDragDrop.ts        # Drag & drop functionality
│   └── useKeyboard.ts        # Keyboard shortcuts
├── components/               # (To be created)
│   ├── Toolbar.vue
│   ├── Sidebar.vue
│   ├── FileList.vue
│   ├── FileGrid.vue
│   ├── Preview.vue
│   ├── CommandPalette.vue
│   ├── ContextMenu.vue
│   └── StatusBar.vue
├── App.vue                   # Original implementation
├── App-refactored.vue        # New architecture demo
└── main.ts
```

## Architecture Principles

### 1. **Composables-First Approach**
Business logic is extracted into reusable composables following Vue 3 Composition API best practices.

### 2. **Type Safety**
Strong TypeScript typing throughout the application with centralized type definitions.

### 3. **Separation of Concerns**
- **Composables**: Business logic and state management
- **Components**: UI presentation and user interaction
- **Types**: Type definitions and interfaces

### 4. **Tauri Integration Ready**
File system composable is structured for easy integration with Tauri backend.

## Composables

### `useFileSystem()`
Manages file system operations with Tauri backend integration.

**Features:**
- Load directory contents
- File operations (copy, move, delete, rename)
- Create folders
- File type detection
- File size formatting
- Mock data for development

**Usage:**
```ts
const { files, isLoading, loadDirectory, deleteItem, renameItem, createFolder } = useFileSystem();
```

### `useNavigation()`
Handles navigation, tabs, and browsing history.

**Features:**
- Multi-tab support
- Browser-like history (back/forward)
- Breadcrumb navigation
- Path management

**Usage:**
```ts
const { currentPath, goBack, goForward, goUp, goHome, navigateInto, addTab, closeTab } = useNavigation();
```

### `useSelection()`
Manages file selection with keyboard modifiers support.

**Features:**
- Single selection
- Multi-selection (Ctrl/Cmd + click)
- Range selection (Shift + click)
- Select all / Clear selection
- Invert selection

**Usage:**
```ts
const { selectedIds, selectedCount, handleItemClick, selectAll, clearSelection } = useSelection();
```

### `useSearch()`
Advanced search and filtering with multiple criteria.

**Features:**
- Full-text search
- Filter by file type
- Filter by date range
- Filter by size range
- Filter by tags
- Sorting (name, date, size, type)
- Sort order (ascending/descending)

**Usage:**
```ts
const { searchQuery, processFiles, setSorting, addFileTypeFilter, setDateRangeFilter } = useSearch();
```

### `useDragDrop()`
Drag and drop functionality for file operations.

**Features:**
- Drag single or multiple files
- Visual feedback during drag
- Drop on folders
- Copy on Ctrl/Cmd + drop
- Move on normal drop

**Usage:**
```ts
const { startDrag, handleDragOver, handleDrop, isDragTarget } = useDragDrop();
```

### `useKeyboard()`
Global keyboard shortcuts management.

**Features:**
- Declarative shortcut definitions
- Modifier keys support (Ctrl, Shift, Alt, Meta)
- Auto cleanup on unmount

**Usage:**
```ts
useKeyboard([
  { key: 'k', ctrl: true, description: 'Command palette', callback: openCommandPalette },
  { key: 'a', ctrl: true, description: 'Select all', callback: selectAll },
]);
```

## Type System

All types are defined in `src/types/index.ts`:

- `FileItem` - File/folder metadata
- `Tab` - Tab state with history
- `ViewMode` - Display mode (grid/list/details)
- `SearchFilter` - Search criteria
- `DragState` - Drag & drop state
- `AppSettings` - User preferences

## Features

### ✅ Implemented
- Multi-tab navigation
- Browser-like history (back/forward)
- Grid and list view modes
- File selection (single, multi, range)
- Keyboard shortcuts
- Command palette (Ctrl+K)
- Context menu
- Preview panel
- Drag & drop support
- Advanced search and filtering
- Sorting options
- Windows XP authentic styling

### 🚧 To Be Implemented
- Real Tauri file system integration
- File operations (copy, cut, paste, delete, rename)
- Advanced filters UI
- Settings persistence
- Multiple theme support (Classic, Luna, Royale)
- Custom file tags
- Bookmarks/favorites
- File preview for images, PDFs, text

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Open command palette |
| `Ctrl+A` | Select all files |
| `Ctrl+T` | New tab |
| `Backspace` | Go up one directory |
| `Escape` | Close dialogs / Clear selection |
| `Ctrl+Click` | Multi-select |
| `Shift+Click` | Range select |

## Tauri Backend Integration

The `useFileSystem` composable is ready for Tauri integration. Uncomment the real implementation and add these Tauri commands:

```rust
#[tauri::command]
fn read_directory(path: String) -> Result<Vec<FileSystemEntry>, String> {
    // Implementation
}

#[tauri::command]
fn delete_item(path: String) -> Result<(), String> {
    // Implementation
}

#[tauri::command]
fn rename_item(old_path: String, new_name: String) -> Result<(), String> {
    // Implementation
}

#[tauri::command]
fn create_folder(path: String, name: String) -> Result<(), String> {
    // Implementation
}

#[tauri::command]
fn copy_items(sources: Vec<String>, destination: String) -> Result<(), String> {
    // Implementation
}

#[tauri::command]
fn move_items(sources: Vec<String>, destination: String) -> Result<(), String> {
    // Implementation
}
```

## Development

1. **Using the refactored version:**
   ```bash
   # Rename App-refactored.vue to App.vue
   mv src/App-refactored.vue src/App.vue
   ```

2. **Run development server:**
   ```bash
   npm run dev
   ```

3. **Build for production:**
   ```bash
   npm run build
   ```

## Best Practices

1. **Keep composables focused** - Each composable should handle one concern
2. **Use TypeScript strictly** - No `any` types, leverage type inference
3. **Prefer composition over inheritance** - Combine composables for complex features
4. **Keep components dumb** - Logic in composables, presentation in components
5. **Test composables independently** - They're pure functions, easy to unit test

## Next Steps

1. Create individual Vue components from App-refactored.vue
2. Implement Tauri backend commands
3. Add unit tests for composables
4. Implement advanced filter UI
5. Add settings panel with theme switching
6. Implement file preview for common formats
