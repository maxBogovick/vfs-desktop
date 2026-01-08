import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { FileItem, FileSystemEntry, FileType } from '../types';

export function useFileSystem() {
  const files = ref<FileItem[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Convert file size to human readable format
  const formatFileSize = (bytes?: number): string => {
    if (!bytes) return '--';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
  };

  // Determine file type from extension
  const getFileType = (name: string, isDir: boolean): FileType => {
    if (isDir) return 'folder';

    const ext = name.split('.').pop()?.toLowerCase() || '';
    const typeMap: Record<string, FileType> = {
      // Images
      'jpg': 'image', 'jpeg': 'image', 'png': 'image', 'gif': 'image', 'webp': 'image', 'svg': 'image', 'bmp': 'image',
      // Code
      'js': 'code', 'ts': 'code', 'vue': 'code', 'jsx': 'code', 'tsx': 'code', 'py': 'code', 'rs': 'code',
      'go': 'code', 'java': 'code', 'cpp': 'code', 'c': 'code', 'h': 'code', 'css': 'code', 'scss': 'code',
      'html': 'code', 'json': 'code', 'xml': 'code', 'yaml': 'code', 'toml': 'code', 'md': 'code',
      // Documents
      'pdf': 'pdf', 'doc': 'file', 'docx': 'file', 'xls': 'file', 'xlsx': 'file', 'ppt': 'file', 'pptx': 'file',
      'txt': 'file', 'rtf': 'file',
      // Video
      'mp4': 'video', 'avi': 'video', 'mkv': 'video', 'mov': 'video', 'wmv': 'video', 'flv': 'video',
      // Audio
      'mp3': 'audio', 'wav': 'audio', 'flac': 'audio', 'm4a': 'audio', 'aac': 'audio', 'ogg': 'audio',
      // Archives
      'zip': 'archive', 'rar': 'archive', '7z': 'archive', 'tar': 'archive', 'gz': 'archive', 'bz2': 'archive',
    };

    return typeMap[ext] || 'file';
  };

  // Helper function to convert entries to FileItems
  const entriesToFileItems = (entries: FileSystemEntry[]): FileItem[] => {
    return entries.map((entry) => ({
      id: entry.path,
      name: entry.name,
      path: entry.path,
      type: getFileType(entry.name, entry.isDir),
      size: entry.size,
      sizeFormatted: formatFileSize(entry.size),
      modified: entry.modified ? new Date(entry.modified * 1000).toLocaleDateString() : undefined,
      modifiedTimestamp: entry.modified ? entry.modified * 1000 : undefined,
      created: entry.created ? new Date(entry.created * 1000).toLocaleDateString() : undefined,
      accessed: entry.accessed ? new Date(entry.accessed * 1000).toLocaleDateString() : undefined,
      tags: [],
      permissions: {
        readable: true,
        writable: true,
        executable: entry.isDir,
      },
    }));
  };

  // Load directory contents from Tauri backend
  const loadDirectory = async (path: string, panelFs?: string): Promise<void> => {
    isLoading.value = true;
    error.value = null;

    try {
      // Check if path is an archive (ends with archive extension)
      // This is a simple check for MVP. Robust check would require backend helper.
      if (/\.(zip|tar|gz|tgz)$/i.test(path)) {
         const entries: FileSystemEntry[] = await invoke('list_archive_contents', { archivePath: path });
         files.value = entriesToFileItems(entries);
      } else {
         const entries: FileSystemEntry[] = await invoke('read_directory', { path, panelFs: panelFs || null });
         files.value = entriesToFileItems(entries);
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load directory';
      files.value = [];
      console.error('Error loading directory:', e);
    } finally {
      isLoading.value = false;
    }
  };

  // Get directory contents (for tree view) - returns array instead of updating state
  const getDirectoryContents = async (path: string, panelFs?: string): Promise<FileItem[]> => {
    try {
      const entries: FileSystemEntry[] = await invoke('read_directory', { path, panelFs: panelFs || null });
      return entriesToFileItems(entries);
    } catch (e) {
      console.error('Error getting directory contents:', e);
      return [];
    }
  };

  // Get home directory
  const getHomeDirectory = async (panelFs?: string): Promise<string> => {
    try {
      return await invoke('get_home_directory', { panelFs: panelFs || null });
    } catch (e) {
      console.error('Error getting home directory:', e);
      return '/';
    }
  };

  // Get file info
  const getFileInfo = async (path: string, panelFs?: string): Promise<FileItem | null> => {
    try {
      const entry: FileSystemEntry = await invoke('get_file_info', { path, panelFs: panelFs || null });
      return {
        id: entry.path,
        name: entry.name,
        path: entry.path,
        type: getFileType(entry.name, entry.isDir),
        size: entry.size,
        sizeFormatted: formatFileSize(entry.size),
        modified: entry.modified ? new Date(entry.modified * 1000).toLocaleDateString() : undefined,
        modifiedTimestamp: entry.modified ? entry.modified * 1000 : undefined,
        created: entry.created ? new Date(entry.created * 1000).toLocaleDateString() : undefined,
        accessed: entry.accessed ? new Date(entry.accessed * 1000).toLocaleDateString() : undefined,
        tags: [],
        permissions: {
          readable: true,
          writable: true,
          executable: entry.isDir,
        },
      };
    } catch (e) {
      console.error('Error getting file info:', e);
      return null;
    }
  };

  // Delete file or folder
  const deleteItem = async (path: string, panelFs?: string): Promise<void> => {
    try {
      await invoke('delete_item', { path, panelFs: panelFs || null });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to delete item');
    }
  };

  // Rename file or folder
  const renameItem = async (oldPath: string, newName: string, panelFs?: string): Promise<void> => {
    try {
      await invoke('rename_item', { oldPath, newName, panelFs: panelFs || null });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to rename item');
    }
  };

  // Create new folder
  const createFolder = async (path: string, name: string, panelFs?: string): Promise<void> => {
    try {
      await invoke('create_folder', { path, name, panelFs: panelFs || null });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to create folder');
    }
  };

  // Create new file
  const createFile = async (path: string, name: string, content?: string, panelFs?: string): Promise<void> => {
    try {
      await invoke('create_file', { path, name, content: content || null, panelFs: panelFs || null });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to create file');
    }
  };

  // Batch create files
  const createFilesBatch = async (path: string, files: Array<{ name: string; content?: string }>, panelFs?: string): Promise<any> => {
    try {
      const result = await invoke('create_files_batch', { path, files, panelFs: panelFs || null });
      return result;
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to create files');
    }
  };

  // Copy items
  const copyItems = async (sources: string[], destination: string, sourceFileSystem?: string, destinationFileSystem?: string): Promise<void> => {
    try {
      await invoke('copy_items', { 
        sources, 
        destination, 
        sourceFileSystem: sourceFileSystem || null, 
        destinationFileSystem: destinationFileSystem || null 
      });
    } catch (e) {
      console.error(e);
      throw new Error(e instanceof Error ? e.message : 'Failed to copy items');
    }
  };

  // Move items
  const moveItems = async (sources: string[], destination: string, sourceFileSystem?: string, destinationFileSystem?: string): Promise<void> => {
    try {
      await invoke('move_items', { 
        sources, 
        destination, 
        sourceFileSystem: sourceFileSystem || null, 
        destinationFileSystem: destinationFileSystem || null 
      });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to move items');
    }
  };

  // Open file in default application
  const openFile = async (path: string, panelFs?: string): Promise<void> => {
    try {
      await invoke('open_file', { path, panelFs: panelFs || null });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to open file');
    }
  };

  // Reveal file in Finder/Explorer
  const revealInFinder = async (path: string, panelFs?: string): Promise<void> => {
    try {
      await invoke('reveal_in_finder', { path, panelFs: panelFs || null });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to reveal in Finder');
    }
  };

  // Get system folders
  const getSystemFolders = async (panelFs?: string): Promise<FileItem[]> => {
    try {
      const entries: FileSystemEntry[] = await invoke('get_system_folders', { panelFs: panelFs || null });
      return entries.map((entry) => ({
        id: entry.path,
        name: entry.name,
        path: entry.path,
        type: getFileType(entry.name, entry.isDir),
        size: entry.size,
        sizeFormatted: formatFileSize(entry.size),
        modified: entry.modified ? new Date(entry.modified * 1000).toLocaleDateString() : undefined,
        created: entry.created ? new Date(entry.created * 1000).toLocaleDateString() : undefined,
        tags: [],
        permissions: {
          readable: true,
          writable: true,
          executable: entry.isDir,
        },
      }));
    } catch (e) {
      console.error('Error getting system folders:', e);
      return [];
    }
  };

  // Read file content (returns base64 for images, text for text files)
  const readFileContent = async (path: string, maxSize?: number, panelFs?: string): Promise<string> => {
    try {
      return await invoke('read_file_content', { path, maxSize, panelFs: panelFs || null });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to read file content');
    }
  };

  // Write file content
  const writeFileContent = async (path: string, content: string, panelFs?: string): Promise<void> => {
    try {
      await invoke('write_file_content', { path, content, panelFs: panelFs || null });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to write file content');
    }
  };

  // Normalize path (expand ~, resolve to absolute path)
  const normalizePath = async (path: string, panelFs?: string): Promise<string> => {
    try {
      return await invoke('normalize_path', { path, panelFs: panelFs || null });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to normalize path');
    }
  };

  // Open terminal in directory
  const openTerminal = async (path: string): Promise<void> => {
    try {
      await invoke('open_terminal', { path });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to open terminal');
    }
  };

  // Extract archive
  const extractArchive = async (archivePath: string, destinationPath: string): Promise<void> => {
    try {
      await invoke('extract_archive', { archivePath, destinationPath });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to extract archive');
    }
  };

  // List archive contents
  const listArchiveContents = async (archivePath: string): Promise<FileItem[]> => {
    try {
      const entries: FileSystemEntry[] = await invoke('list_archive_contents', { archivePath });
      return entriesToFileItems(entries);
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to list archive contents');
    }
  };

  // Create archive
  const createArchive = async (sourcePaths: string[], destinationPath: string): Promise<void> => {
    try {
      await invoke('create_archive', { sourcePaths, destinationPath });
    } catch (e) {
      throw new Error(e instanceof Error ? e.message : 'Failed to create archive');
    }
  };

  return {
    files,
    isLoading,
    error,
    loadDirectory,
    getDirectoryContents,
    getHomeDirectory,
    getFileInfo,
    deleteItem,
    renameItem,
    createFolder,
    createFile,
    createFilesBatch,
    copyItems,
    moveItems,
    openFile,
    revealInFinder,
    getSystemFolders,
    readFileContent,
    writeFileContent,
    formatFileSize,
    normalizePath,
    openTerminal,
    extractArchive,
    listArchiveContents,
    createArchive,
  };
}
