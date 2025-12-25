import { ref, reactive, Ref, computed } from 'vue';
import type { FileItem } from '../types';
import { useFileSystem } from './useFileSystem';
import { useUIState } from './useUIState';

export interface TreeNode {
  item: FileItem;
  children: TreeNode[];
  isExpanded: boolean;
  isLoaded: boolean;
  level: number;
  isFolder: boolean; // Флаг, что это папка которая может иметь детей
}

export function useDirectoryTree() {
  const { getDirectoryContents } = useFileSystem();
  const { expandedFolders, toggleFolderExpansion } = useUIState();
  const rootNodes = ref<TreeNode[]>([]);

  // Convert expandedFolders array to Set for faster lookup
  const expandedPaths = computed(() => new Set(expandedFolders.value));

  // Check if item is a folder that can have children
  const canHaveChildren = (item: FileItem): boolean => {
    return item.type === 'folder' || item.type === 'drive' || item.type === 'system';
  };

  // Create a tree node from a FileItem
  const createNode = (item: FileItem, level: number = 0): TreeNode => {
    return reactive({
      item,
      children: [],
      isExpanded: expandedPaths.value.has(item.path),
      isLoaded: false,
      level,
      isFolder: canHaveChildren(item),
    });
  };

  // Load children for a node
  const loadChildren = async (node: TreeNode): Promise<void> => {
    if (node.isLoaded) return;

    console.log('[loadChildren] Loading children for:', node.item.path);

    try {
      const items = await getDirectoryContents(node.item.path);

      // Проверяем что items не undefined и является массивом
      if (!items || !Array.isArray(items)) {
        console.error('[loadChildren] Invalid result from getDirectoryContents:', items);
        node.children = [];
        node.isLoaded = true;
        return;
      }

      console.log('[loadChildren] Loaded', items.length, 'items');

      // Only include folders in the tree
      const folders = items.filter(
        item => item.type === 'folder' || item.type === 'drive' || item.type === 'system'
      );

      console.log('[loadChildren] Found', folders.length, 'folders');

      node.children = folders.map(item => createNode(item, node.level + 1));
      node.isLoaded = true;
    } catch (err) {
      console.error('[loadChildren] Failed to load children:', err);
      node.children = [];
      node.isLoaded = true;
    }
  };

  // Toggle node expansion
  const toggleNode = async (node: TreeNode): Promise<void> => {
    console.log('[toggleNode] Toggling node:', node.item.name, 'current expanded:', node.isExpanded);

    if (!node.isExpanded && !node.isLoaded) {
      await loadChildren(node);
    }

    node.isExpanded = !node.isExpanded;

    // Update global state
    toggleFolderExpansion(node.item.path);

    console.log('[toggleNode] Node now expanded:', node.isExpanded, 'children:', node.children.length);
  };

  // Initialize tree with root folders
  const initializeTree = async (rootFolders: FileItem[]): Promise<void> => {
    console.log('[initializeTree] Initializing tree with', rootFolders.length, 'root folders');
    rootNodes.value = rootFolders.map(item => createNode(item, 0));
    console.log('[initializeTree] Created', rootNodes.value.length, 'root nodes');

    // Optionally auto-expand first level
    for (const node of rootNodes.value) {
      if (expandedPaths.value.has(node.item.path)) {
        console.log('[initializeTree] Auto-expanding:', node.item.name);
        await loadChildren(node);
        node.isExpanded = true;
      }
    }
  };

  // Find a node by path
  const findNodeByPath = (path: string, nodes: TreeNode[] = rootNodes.value): TreeNode | null => {
    // Нормализуем пути для сравнения (убираем trailing slash)
    const normalizedSearchPath = path.endsWith('/') && path !== '/' ? path.slice(0, -1) : path;

    for (const node of nodes) {
      const normalizedNodePath = node.item.path.endsWith('/') && node.item.path !== '/'
        ? node.item.path.slice(0, -1)
        : node.item.path;

      if (normalizedNodePath === normalizedSearchPath) {
        return node;
      }
      if (node.children.length > 0) {
        const found = findNodeByPath(normalizedSearchPath, node.children);
        if (found) return found;
      }
    }
    return null;
  };

  // Expand to a specific path (reveal in tree)
  const expandToPath = async (path: string): Promise<void> => {
    if (!path || path === '/') return;

    // Нормализуем путь - убираем trailing slash и убеждаемся что начинается с /
    let normalizedPath = path.endsWith('/') ? path.slice(0, -1) : path;
    if (!normalizedPath.startsWith('/')) {
      normalizedPath = '/' + normalizedPath;
    }

    console.log('[expandToPath] Expanding to:', normalizedPath);

    const pathParts = normalizedPath.split('/').filter(p => p);
    let currentPath = '';

    for (let i = 0; i < pathParts.length; i++) {
      currentPath += '/' + pathParts[i];
      console.log('[expandToPath] Looking for node:', currentPath);

      const node = findNodeByPath(currentPath);

      if (node) {
        console.log('[expandToPath] Found node:', node.item.name, 'isFolder:', node.isFolder, 'isLoaded:', node.isLoaded);

        if (node.isFolder) {
          // Загружаем детей если еще не загружены
          if (!node.isLoaded) {
            console.log('[expandToPath] Loading children for:', node.item.name);
            await loadChildren(node);
          }
          // Раскрываем узел только если это не последний элемент пути
          // (чтобы показать текущую папку, но не раскрывать её автоматически)
          if (i < pathParts.length - 1) {
            console.log('[expandToPath] Expanding node:', node.item.name);
            node.isExpanded = true;
            expandedPaths.value.add(node.item.path);
          }
        }
      } else {
        console.log('[expandToPath] Node not found:', currentPath);
      }
    }
  };

  // Refresh a specific node
  const refreshNode = async (node: TreeNode): Promise<void> => {
    const wasExpanded = node.isExpanded;
    node.isLoaded = false;
    node.children = [];

    if (wasExpanded) {
      await loadChildren(node);
      node.isExpanded = true;
    }
  };

  // Check if a node is a folder that can contain children
  const isFolder = (node: TreeNode): boolean => {
    return node.item.type === 'folder' ||
           node.item.type === 'drive' ||
           node.item.type === 'system';
  };

  return {
    rootNodes,
    expandedPaths,
    createNode,
    loadChildren,
    toggleNode,
    initializeTree,
    findNodeByPath,
    expandToPath,
    refreshNode,
    isFolder,
  };
}
