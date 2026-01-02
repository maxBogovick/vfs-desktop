<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import type { FileItem, Bookmark } from '../types';
import { useFileSystem } from '../composables/useFileSystem';
import { useDirectoryTree, type TreeNode } from '../composables/useDirectoryTree';
import { useBookmarks } from '../composables/useBookmarks';
import { useUIState } from '../composables/useUIState';
import TreeNodeComponent from './TreeNode.vue';

interface Props {
  currentPath?: string;
  width?: number;
}

interface Emits {
  (e: 'navigate', path: string): void;
  (e: 'drop', targetPath: string, event: DragEvent): void;
  (e: 'resize', width: number): void;
}

const props = withDefaults(defineProps<Props>(), {
  width: 240,
});

const emit = defineEmits<Emits>();
const { getSystemFolders } = useFileSystem();
const { bookmarks, loadBookmarks, addBookmark, removeBookmark, renameBookmark } = useBookmarks();
const { expandedFolders, sidebarSectionsExpanded } = useUIState();

const systemFolders = ref<FileItem[]>([]);
const dragOverNodePath = ref<string | null>(null);
const editingBookmarkId = ref<string | null>(null);
const editingBookmarkName = ref('');

// Resizer state
const isResizing = ref(false);
const startX = ref(0);
const startWidth = ref(0);

const handleResizeStart = (event: MouseEvent) => {
  isResizing.value = true;
  startX.value = event.clientX;
  startWidth.value = props.width;

  document.addEventListener('mousemove', handleResizeMove);
  document.addEventListener('mouseup', handleResizeEnd);

  event.preventDefault();
};

const handleResizeMove = (event: MouseEvent) => {
  if (!isResizing.value) return;

  const delta = event.clientX - startX.value;
  const newWidth = Math.max(150, Math.min(500, startWidth.value + delta));

  emit('resize', newWidth);
};

const handleResizeEnd = () => {
  isResizing.value = false;

  document.removeEventListener('mousemove', handleResizeMove);
  document.removeEventListener('mouseup', handleResizeEnd);
};

// Directory tree
const {
  rootNodes,
  toggleNode,
  initializeTree,
  expandToPath,
} = useDirectoryTree();

const getFileIcon = (item: FileItem) => {
  const icons: Record<string, string> = {
    folder: '📁',
    drive: '💾',
  };

  // Special icons for known folders
  const name = item.name.toLowerCase();
  if (name.includes('desktop')) return '💻';
  if (name.includes('documents')) return '📄';
  if (name.includes('downloads')) return '📥';
  if (name.includes('pictures') || name.includes('photos')) return '🖼️';
  if (name.includes('music') || name.includes('audio')) return '🎵';
  if (name.includes('videos') || name.includes('movies')) return '🎬';
  if (name.includes('home') || name.includes('users')) return '🏠';

  return icons[item.type] || '📁';
};

const toggleSection = (section: keyof typeof sidebarSectionsExpanded.value) => {
  const oldValue = sidebarSectionsExpanded.value[section];
  sidebarSectionsExpanded.value[section] = !oldValue;
  const newValue = sidebarSectionsExpanded.value[section];
  console.log(`[Sidebar] 🔄 Toggled ${section}: ${oldValue} -> ${newValue}`);
  console.log('[Sidebar] Current state:', sidebarSectionsExpanded.value);
};

const navigateTo = (path: string) => {
  emit('navigate', path);
};

const handleTreeToggle = async (node: TreeNode) => {
  await toggleNode(node);
};

const handleTreeNavigate = (path: string) => {
  emit('navigate', path);
};

const handleTreeDragOver = (node: TreeNode, event: DragEvent) => {
  event.preventDefault();
  dragOverNodePath.value = node.item.path;
};

const handleTreeDragLeave = (node: TreeNode) => {
  dragOverNodePath.value = null;
};

const handleTreeDrop = (node: TreeNode, event: DragEvent) => {
  event.preventDefault();
  dragOverNodePath.value = null;
  emit('drop', node.item.path, event);
};

// No drag & drop - we use simpler methods now!

// Bookmark actions
const handleBookmarkClick = (bookmark: Bookmark) => {
  if (editingBookmarkId.value === bookmark.id) return;
  navigateTo(bookmark.path);
};

const startEditingBookmark = (bookmark: Bookmark, event: Event) => {
  event.stopPropagation();
  editingBookmarkId.value = bookmark.id;
  editingBookmarkName.value = bookmark.name;
};

const finishEditingBookmark = async () => {
  if (editingBookmarkId.value && editingBookmarkName.value.trim()) {
    await renameBookmark(editingBookmarkId.value, editingBookmarkName.value.trim());
  }
  editingBookmarkId.value = null;
  editingBookmarkName.value = '';
};

const cancelEditingBookmark = () => {
  editingBookmarkId.value = null;
  editingBookmarkName.value = '';
};

const handleRemoveBookmark = async (bookmark: Bookmark, event: Event) => {
  event.stopPropagation();
  await removeBookmark(bookmark.id);
};

// Watch current path and expand tree to show it
watch(() => props.currentPath, async (newPath, oldPath) => {
  if (newPath && newPath !== oldPath) {
    console.log('Expanding tree to path:', newPath);
    await expandToPath(newPath);
  }
}, { immediate: false });

onMounted(async () => {
  systemFolders.value = await getSystemFolders();

  // Initialize tree with system folders
  await initializeTree(systemFolders.value);

  // Load bookmarks
  await loadBookmarks();

  // Expand to current path if provided
  if (props.currentPath) {
    await expandToPath(props.currentPath);
  }
});
</script>

<template>
  <div class="bg-[var(--vf-surface-default)] border-r border-[var(--vf-border-default)] overflow-y-auto flex-shrink-0 relative" :style="{ width: `${width}px` }">
    <div class="p-2">
      <!-- Quick Access -->
      <div class="mb-2">
        <div
          @click="toggleSection('quickAccess')"
          class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[var(--vf-surface-hover)] cursor-pointer rounded"
        >
          <span class="text-xs">{{ sidebarSectionsExpanded.quickAccess ? '▼' : '▶' }}</span>
          <span class="text-base">⭐</span>
          <span class="text-[11px] font-bold">Quick Access</span>
        </div>

        <transition name="expand">
          <div v-if="sidebarSectionsExpanded.quickAccess" class="pl-6 mt-1 space-y-0.5">
            <div
              v-for="folder in systemFolders.slice(0, 4)"
              :key="folder.id"
              @click="navigateTo(folder.path)"
              class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[var(--vf-surface-hover)] cursor-pointer rounded"
            >
              <span class="text-sm">{{ getFileIcon(folder) }}</span>
              <span class="text-[11px] truncate">{{ folder.name }}</span>
            </div>
          </div>
        </transition>
      </div>

      <div class="border-t border-[var(--vf-border-subtle)] my-2"></div>

      <!-- Folder Tree -->
      <div class="mb-2">
        <div
          @click="toggleSection('folderTree')"
          class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[var(--vf-surface-hover)] cursor-pointer rounded"
        >
          <span class="text-xs">{{ sidebarSectionsExpanded.folderTree ? '▼' : '▶' }}</span>
          <span class="text-base">💾</span>
          <span class="text-[11px] font-bold">Folders</span>
        </div>

        <transition name="expand">
          <div v-if="sidebarSectionsExpanded.folderTree" class="mt-1">
            <TreeNodeComponent
              v-for="node in rootNodes"
              :key="node.item.id"
              :node="node"
              :current-path="currentPath"
              :drag-over-node-path="dragOverNodePath"
              @toggle="handleTreeToggle"
              @navigate="handleTreeNavigate"
              @drag-over="handleTreeDragOver"
              @drag-leave="handleTreeDragLeave"
              @drop="handleTreeDrop"
            />
          </div>
        </transition>
      </div>

      <div class="border-t border-[var(--vf-border-subtle)] my-2"></div>

      <!-- Favorites -->
      <div>
        <div
          @click="toggleSection('favorites')"
          class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[var(--vf-surface-hover)] cursor-pointer rounded"
        >
          <span class="text-xs">{{ sidebarSectionsExpanded.favorites ? '▼' : '▶' }}</span>
          <span class="text-base">❤️</span>
          <span class="text-[11px] font-bold">Favorites</span>
        </div>

        <transition name="expand">
          <div v-if="sidebarSectionsExpanded.favorites" class="pl-6 mt-1">
            <div
              v-if="bookmarks.length === 0"
              class="text-[10px] py-2 px-1 text-gray-400 italic"
            >
              No favorites yet. Right-click on a folder to add.
            </div>

            <div v-else class="space-y-0.5">
              <div
                v-for="bookmark in bookmarks"
                :key="bookmark.id"
                class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[var(--vf-surface-hover)] cursor-pointer rounded group"
                @click="handleBookmarkClick(bookmark)"
              >
                <span class="text-sm">📁</span>

                <input
                  v-if="editingBookmarkId === bookmark.id"
                  v-model="editingBookmarkName"
                  @click.stop
                  @blur="finishEditingBookmark"
                  @keydown.enter="finishEditingBookmark"
                  @keydown.esc="cancelEditingBookmark"
                  class="flex-1 text-[11px] px-1 py-0.5 border border-[var(--vf-border-accent)] rounded outline-none"
                  autofocus
                />

                <span
                  v-else
                  class="flex-1 text-[11px] truncate"
                  @dblclick="startEditingBookmark(bookmark, $event)"
                  :title="bookmark.path"
                >
                  {{ bookmark.name }}
                </span>

                <button
                  v-if="editingBookmarkId !== bookmark.id"
                  @click="handleRemoveBookmark(bookmark, $event)"
                  class="opacity-0 group-hover:opacity-100 text-[10px] text-gray-500 hover:text-red-600 transition-opacity"
                  title="Remove bookmark"
                >
                  ✕
                </button>
              </div>
            </div>
          </div>
        </transition>
      </div>
    </div>

    <!-- Resizer Handle -->
    <div
      @mousedown="handleResizeStart"
      class="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-blue-400 transition-colors"
      :class="{ 'bg-blue-500': isResizing }"
    ></div>
  </div>
</template>

<style scoped>
.expand-enter-active,
.expand-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
}

.expand-enter-to,
.expand-leave-from {
  max-height: 500px;
  opacity: 1;
}
</style>
