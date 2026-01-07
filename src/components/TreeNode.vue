<script setup lang="ts">
import { ref, computed } from 'vue';
import type { TreeNode } from '../composables/useDirectoryTree';
import { useFileColoring } from '../composables/useFileColoring';

interface Props {
  node: TreeNode;
  currentPath?: string;
  dragOverNodePath?: string | null;
}

interface Emits {
  (e: 'toggle', node: TreeNode): void;
  (e: 'navigate', path: string): void;
  (e: 'dragOver', node: TreeNode, event: DragEvent): void;
  (e: 'dragLeave', node: TreeNode): void;
  (e: 'drop', node: TreeNode, event: DragEvent): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const { getFileStyle } = useFileColoring();

const getFileIcon = (node: TreeNode) => {
  const item = node.item;
  const name = item.name.toLowerCase();

  // Special icons for known folders
  if (name.includes('desktop')) return '💻';
  if (name.includes('documents')) return '📄';
  if (name.includes('downloads')) return '📥';
  if (name.includes('pictures') || name.includes('photos')) return '🖼️';
  if (name.includes('music') || name.includes('audio')) return '🎵';
  if (name.includes('videos') || name.includes('movies')) return '🎬';
  if (name.includes('home') || name.includes('users')) return '🏠';

  // Default icons
  const icons: Record<string, string> = {
    folder: '📁',
    drive: '💾',
    system: '⚙️',
  };

  return icons[item.type] || '📁';
};

const isActive = computed(() => {
  return props.currentPath === props.node.item.path;
});

const isDragOver = computed(() => {
  return props.dragOverNodePath === props.node.item.path;
});

const paddingLeft = computed(() => {
  return `${props.node.level * 12 + 4}px`;
});

const handleToggle = (event: MouseEvent) => {
  event.stopPropagation();
  emit('toggle', props.node);
};

const handleNavigate = () => {
  emit('navigate', props.node.item.path);
};

const handleDragOver = (event: DragEvent) => {
  event.preventDefault();
  emit('dragOver', props.node, event);
};

const handleDragLeave = () => {
  emit('dragLeave', props.node);
};

const handleDrop = (event: DragEvent) => {
  event.preventDefault();
  emit('drop', props.node, event);
};
</script>

<template>
  <div>
    <!-- Node itself -->
    <div
      @click="handleNavigate"
      @dragover="handleDragOver"
      @dragleave="handleDragLeave"
      @drop="handleDrop"
      :class="[
        'flex items-center gap-1 py-1 px-1 hover:bg-[#C1D2EE] cursor-pointer rounded transition-colors',
        isActive && 'bg-[#C1D2EE] font-semibold',
        isDragOver && 'ring-2 ring-blue-400 bg-blue-50',
      ]"
      :style="{ paddingLeft }"
    >
      <!-- Expand/Collapse arrow -->
      <span
        v-if="node.isFolder"
        @click="handleToggle"
        class="text-[10px] w-3 flex-shrink-0 hover:bg-gray-200 rounded cursor-pointer"
      >
        {{ node.isExpanded ? '▼' : '▶' }}
      </span>
      <span v-else class="w-3 flex-shrink-0"></span>

      <!-- Icon -->
      <span class="text-sm flex-shrink-0">{{ getFileIcon(node) }}</span>

      <!-- Name -->
      <span class="text-[11px] truncate flex-1" :style="getFileStyle(node.item)">{{ node.item.name }}</span>
    </div>

    <!-- Children (recursive) -->
    <transition name="expand">
      <div v-if="node.isExpanded && node.children.length > 0">
        <TreeNode
          v-for="child in node.children"
          :key="child.item.id"
          :node="child"
          :current-path="currentPath"
          :drag-over-node-path="dragOverNodePath"
          @toggle="(node) => $emit('toggle', node)"
          @navigate="(path) => $emit('navigate', path)"
          @drag-over="(node, event) => $emit('dragOver', node, event)"
          @drag-leave="(node) => $emit('dragLeave', node)"
          @drop="(node, event) => $emit('drop', node, event)"
        />
      </div>
    </transition>
  </div>
</template>

<style scoped>
.expand-enter-active,
.expand-leave-active {
  transition: all 0.15s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
}

.expand-enter-to,
.expand-leave-from {
  max-height: 1000px;
  opacity: 1;
}
</style>
