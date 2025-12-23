<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import type { FileItem } from '../types';
import { useFileSystem } from '../composables/useFileSystem';
import { useDirectoryTree, type TreeNode } from '../composables/useDirectoryTree';
import TreeNodeComponent from './TreeNode.vue';

interface Props {
  currentPath?: string;
}

interface Emits {
  (e: 'navigate', path: string): void;
  (e: 'drop', targetPath: string, event: DragEvent): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();
const { getSystemFolders } = useFileSystem();

const sidebarExpanded = ref({
  quickAccess: true,
  folderTree: true,
  favorites: false,
});

const systemFolders = ref<FileItem[]>([]);
const dragOverNodePath = ref<string | null>(null);

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

const toggleSection = (section: keyof typeof sidebarExpanded.value) => {
  sidebarExpanded.value[section] = !sidebarExpanded.value[section];
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

  // Expand to current path if provided
  if (props.currentPath) {
    await expandToPath(props.currentPath);
  }
});
</script>

<template>
  <div class="w-[240px] bg-white border-r border-[#919B9C] overflow-y-auto flex-shrink-0">
    <div class="p-2">
      <!-- Quick Access -->
      <div class="mb-2">
        <div
          @click="toggleSection('quickAccess')"
          class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[#C1D2EE] cursor-pointer rounded"
        >
          <span class="text-xs">{{ sidebarExpanded.quickAccess ? '▼' : '▶' }}</span>
          <span class="text-base">⭐</span>
          <span class="text-[11px] font-bold">Quick Access</span>
        </div>

        <transition name="expand">
          <div v-if="sidebarExpanded.quickAccess" class="pl-6 mt-1 space-y-0.5">
            <div
              v-for="folder in systemFolders.slice(0, 4)"
              :key="folder.id"
              @click="navigateTo(folder.path)"
              class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[#C1D2EE] cursor-pointer rounded"
            >
              <span class="text-sm">{{ getFileIcon(folder) }}</span>
              <span class="text-[11px] truncate">{{ folder.name }}</span>
            </div>
          </div>
        </transition>
      </div>

      <div class="border-t border-[#D0D0BF] my-2"></div>

      <!-- Folder Tree -->
      <div class="mb-2">
        <div
          @click="toggleSection('folderTree')"
          class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[#C1D2EE] cursor-pointer rounded"
        >
          <span class="text-xs">{{ sidebarExpanded.folderTree ? '▼' : '▶' }}</span>
          <span class="text-base">💾</span>
          <span class="text-[11px] font-bold">Folders</span>
        </div>

        <transition name="expand">
          <div v-if="sidebarExpanded.folderTree" class="mt-1">
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

      <div class="border-t border-[#D0D0BF] my-2"></div>

      <!-- Favorites -->
      <div>
        <div
          @click="toggleSection('favorites')"
          class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[#C1D2EE] cursor-pointer rounded"
        >
          <span class="text-xs">{{ sidebarExpanded.favorites ? '▼' : '▶' }}</span>
          <span class="text-base">❤️</span>
          <span class="text-[11px] font-bold">Favorites</span>
        </div>

        <transition name="expand">
          <div v-if="sidebarExpanded.favorites" class="pl-6 mt-1">
            <div class="text-[10px] text-gray-400 py-2 px-1 italic">No favorites yet</div>
          </div>
        </transition>
      </div>
    </div>
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
