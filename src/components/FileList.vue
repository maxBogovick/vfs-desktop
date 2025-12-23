<script setup lang="ts">
import type { FileItem, ViewMode } from '../types';

interface Props {
  items: FileItem[];
  viewMode: ViewMode;
  selectedIds: Set<string>;
  focusedId?: string | null;
  isLoading?: boolean;
  isDragging?: boolean;
  dragTargetId?: string | null;
}

interface Emits {
  (e: 'itemClick', item: FileItem, event: MouseEvent): void;
  (e: 'itemDoubleClick', item: FileItem): void;
  (e: 'itemContextMenu', item: FileItem, event: MouseEvent): void;
  (e: 'dragStart', item: FileItem, event: DragEvent): void;
  (e: 'dragOver', item: FileItem, event: DragEvent): void;
  (e: 'dragLeave', item: FileItem): void;
  (e: 'drop', item: FileItem, event: DragEvent): void;
  (e: 'toggleSelection', item: FileItem): void;
  (e: 'copyItem', item: FileItem): void;
  (e: 'cutItem', item: FileItem): void;
  (e: 'deleteItem', item: FileItem): void;
  (e: 'renameItem', item: FileItem): void;
}

const props = withDefaults(defineProps<Props>(), {
  focusedId: null,
  isLoading: false,
  isDragging: false,
  dragTargetId: null,
});

const emit = defineEmits<Emits>();

const getFileIcon = (item: FileItem) => {
  const icons: Record<string, string> = {
    drive: '💾',
    folder: '📁', // Yellow folder icon
    image: '🖼️',
    pdf: '📄',
    code: '📜',
    file: '📄',
    system: '⚙️',
    video: '🎬',
    audio: '🎵',
    archive: '📦',
  };
  return icons[item.type] || '📄';
};

const isFolder = (item: FileItem) => {
  return item.type === 'folder' || item.type === 'drive' || item.type === 'system';
};

const getTagColor = (tag: string) => {
  const colors: Record<string, string> = {
    work: 'bg-blue-400',
    urgent: 'bg-red-400',
    finance: 'bg-green-500',
    dev: 'bg-purple-400',
    design: 'bg-pink-400',
  };
  return colors[tag] || 'bg-gray-400';
};

const isDragTarget = (itemId: string) => {
  return props.dragTargetId === itemId;
};

const isBeingDragged = (itemId: string) => {
  return props.isDragging && props.selectedIds.has(itemId);
};

const isFocused = (itemId: string) => {
  return props.focusedId === itemId;
};
</script>

<template>
  <div class="flex-1 p-4 overflow-y-auto bg-white">
    <!-- Loading State -->
    <div v-if="isLoading" class="flex items-center justify-center h-full">
      <div class="text-center">
        <div class="text-4xl mb-2">⏳</div>
        <div class="text-sm text-gray-500">Loading...</div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else-if="items.length === 0" class="flex items-center justify-center h-full">
      <div class="text-center">
        <div class="text-4xl mb-2">📂</div>
        <div class="text-sm text-gray-500">This folder is empty</div>
      </div>
    </div>

    <!-- Grid View -->
    <div v-else-if="viewMode === 'grid'" class="grid grid-cols-4 gap-4">
      <div
        v-for="item in items"
        :key="item.id"
        :draggable="true"
        @dragstart="emit('dragStart', item, $event)"
        @dragover="emit('dragOver', item, $event)"
        @dragleave="emit('dragLeave', item)"
        @drop="emit('drop', item, $event)"
        @click="emit('itemClick', item, $event)"
        @dblclick="emit('itemDoubleClick', item)"
        @contextmenu="emit('itemContextMenu', item, $event)"
        :class="[
          'flex flex-col items-center justify-center p-3 rounded cursor-pointer transition-all relative',
          selectedIds.has(item.id) ? 'bg-[#C1D2EE] border border-[#0A246A]' : 'hover:bg-[#E8F2FD] border border-transparent hover:border-[#C1D2EE]',
          isFocused(item.id) && !selectedIds.has(item.id) && 'ring-2 ring-[#0054E3] ring-inset',
          isDragTarget(item.id) && 'ring-2 ring-blue-400 bg-blue-50',
          isBeingDragged(item.id) && 'opacity-50',
        ]"
      >
        <!-- Checkbox (top-left corner) -->
        <input
          type="checkbox"
          :checked="selectedIds.has(item.id)"
          @click.stop="emit('toggleSelection', item)"
          class="absolute top-1 left-1 w-4 h-4 cursor-pointer accent-[#0054E3]"
        />

        <!-- Action Icons (top-right corner, visible on selection or focus) -->
        <div
          v-if="selectedIds.has(item.id) || isFocused(item.id)"
          class="absolute top-1 right-1 flex gap-0.5 bg-white/90 rounded p-0.5"
          @click.stop
        >
          <button
            @click="emit('copyItem', item)"
            class="p-0.5 hover:bg-gray-200 rounded transition-colors"
            title="Copy (Ctrl+C)"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2" stroke-width="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" stroke-width="2"/>
            </svg>
          </button>
          <button
            @click="emit('cutItem', item)"
            class="p-0.5 hover:bg-gray-200 rounded transition-colors"
            title="Cut (Ctrl+X)"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <circle cx="6" cy="6" r="3" stroke-width="2"/>
              <circle cx="6" cy="18" r="3" stroke-width="2"/>
              <line x1="20" y1="4" x2="8.12" y2="15.88" stroke-width="2"/>
              <line x1="14.47" y1="14.48" x2="20" y2="20" stroke-width="2"/>
            </svg>
          </button>
          <button
            @click="emit('renameItem', item)"
            class="p-0.5 hover:bg-gray-200 rounded transition-colors"
            title="Rename (F2)"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" stroke-width="2"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" stroke-width="2"/>
            </svg>
          </button>
          <button
            @click="emit('deleteItem', item)"
            class="p-0.5 hover:bg-red-100 rounded transition-colors text-red-600"
            title="Delete (Del)"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <polyline points="3 6 5 6 21 6" stroke-width="2"/>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" stroke-width="2"/>
            </svg>
          </button>
        </div>

        <div class="w-12 h-12 text-4xl mb-2 flex items-center justify-center">
          {{ getFileIcon(item) }}
        </div>
        <span :class="[
          'text-[11px] text-center break-words w-full px-1',
          isFolder(item) && 'font-bold'
        ]">{{ item.name }}</span>
        <span v-if="item.sizeFormatted && !isFolder(item)" class="text-[9px] text-gray-500 mt-0.5">{{ item.sizeFormatted }}</span>
        <div v-if="item.tags && item.tags.length > 0" class="flex gap-0.5 mt-1">
          <span
            v-for="tag in item.tags.slice(0, 3)"
            :key="tag"
            :class="`w-1.5 h-1.5 rounded-full ${getTagColor(tag)}`"
            :title="tag"
          ></span>
        </div>
      </div>
    </div>

    <!-- List View -->
    <div v-else-if="viewMode === 'list'" class="space-y-0.5">
      <div
        v-for="item in items"
        :key="item.id"
        :draggable="true"
        @dragstart="emit('dragStart', item, $event)"
        @dragover="emit('dragOver', item, $event)"
        @dragleave="emit('dragLeave', item)"
        @drop="emit('drop', item, $event)"
        @click="emit('itemClick', item, $event)"
        @dblclick="emit('itemDoubleClick', item)"
        @contextmenu="emit('itemContextMenu', item, $event)"
        :class="[
          'flex items-center gap-3 px-3 py-2 rounded cursor-pointer transition-all group',
          selectedIds.has(item.id) ? 'bg-[#C1D2EE] border border-[#0A246A]' : 'hover:bg-[#E8F2FD]',
          isFocused(item.id) && !selectedIds.has(item.id) && 'ring-2 ring-[#0054E3] ring-inset',
          isDragTarget(item.id) && 'ring-2 ring-blue-400 bg-blue-50',
          isBeingDragged(item.id) && 'opacity-50',
        ]"
      >
        <!-- Checkbox -->
        <input
          type="checkbox"
          :checked="selectedIds.has(item.id)"
          @click.stop="emit('toggleSelection', item)"
          class="w-4 h-4 flex-shrink-0 cursor-pointer accent-[#0054E3]"
        />

        <!-- Icon -->
        <div class="w-6 h-6 text-2xl flex-shrink-0 flex items-center justify-center">
          {{ getFileIcon(item) }}
        </div>

        <!-- Name -->
        <span :class="[
          'text-[11px] flex-1 truncate',
          isFolder(item) && 'font-bold'
        ]">{{ item.name }}</span>

        <!-- Action Icons (visible on hover, selection or focus) -->
        <div
          v-if="selectedIds.has(item.id) || isFocused(item.id)"
          class="flex gap-1 flex-shrink-0"
          @click.stop
        >
          <button
            @click="emit('copyItem', item)"
            class="p-1 hover:bg-white/50 rounded transition-colors"
            title="Copy (Ctrl+C)"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2" stroke-width="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" stroke-width="2"/>
            </svg>
          </button>
          <button
            @click="emit('cutItem', item)"
            class="p-1 hover:bg-white/50 rounded transition-colors"
            title="Cut (Ctrl+X)"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <circle cx="6" cy="6" r="3" stroke-width="2"/>
              <circle cx="6" cy="18" r="3" stroke-width="2"/>
              <line x1="20" y1="4" x2="8.12" y2="15.88" stroke-width="2"/>
              <line x1="14.47" y1="14.48" x2="20" y2="20" stroke-width="2"/>
            </svg>
          </button>
          <button
            @click="emit('renameItem', item)"
            class="p-1 hover:bg-white/50 rounded transition-colors"
            title="Rename (F2)"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" stroke-width="2"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" stroke-width="2"/>
            </svg>
          </button>
          <button
            @click="emit('deleteItem', item)"
            class="p-1 hover:bg-red-100 rounded transition-colors text-red-600"
            title="Delete (Del)"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <polyline points="3 6 5 6 21 6" stroke-width="2"/>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" stroke-width="2"/>
            </svg>
          </button>
        </div>

        <!-- Modified Date -->
        <span class="text-[10px] text-gray-500 w-24 truncate flex-shrink-0">{{ item.modified }}</span>

        <!-- Size -->
        <span v-if="!isFolder(item)" class="text-[10px] text-gray-500 w-20 text-right flex-shrink-0">{{ item.sizeFormatted }}</span>
        <span v-else class="text-[10px] text-gray-500 w-20 text-right italic flex-shrink-0">Folder</span>

        <!-- Tags -->
        <div class="flex gap-1 flex-shrink-0">
          <span
            v-for="tag in item.tags"
            :key="tag"
            :class="`w-2 h-2 rounded-full ${getTagColor(tag)}`"
            :title="tag"
          ></span>
        </div>
      </div>
    </div>

    <!-- Details View (Advanced table view) -->
    <div v-else-if="viewMode === 'details'" class="w-full">
      <table class="w-full text-[11px]">
        <thead class="bg-[#F1EFE2] border-b border-[#919B9C] sticky top-0">
          <tr>
            <th class="text-left px-3 py-2 font-normal">Name</th>
            <th class="text-left px-3 py-2 font-normal w-24">Modified</th>
            <th class="text-left px-3 py-2 font-normal w-20">Type</th>
            <th class="text-right px-3 py-2 font-normal w-20">Size</th>
            <th class="text-left px-3 py-2 font-normal w-24">Tags</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item in items"
            :key="item.id"
            :draggable="true"
            @dragstart="emit('dragStart', item, $event)"
            @dragover="emit('dragOver', item, $event)"
            @dragleave="emit('dragLeave', item)"
            @drop="emit('drop', item, $event)"
            @click="emit('itemClick', item, $event)"
            @dblclick="emit('itemDoubleClick', item)"
            @contextmenu="emit('itemContextMenu', item, $event)"
            :class="[
              'cursor-pointer transition-all',
              selectedIds.has(item.id) ? 'bg-[#C1D2EE]' : 'hover:bg-[#E8F2FD]',
              isDragTarget(item.id) && 'ring-2 ring-blue-400 bg-blue-50',
              isBeingDragged(item.id) && 'opacity-50',
            ]"
          >
            <td class="px-3 py-1.5">
              <div class="flex items-center gap-2">
                <span class="text-xl">{{ getFileIcon(item) }}</span>
                <span :class="[
                  'truncate',
                  isFolder(item) && 'font-bold'
                ]">{{ item.name }}</span>
              </div>
            </td>
            <td class="px-3 py-1.5 text-gray-600">{{ item.modified }}</td>
            <td class="px-3 py-1.5 text-gray-600 capitalize">{{ item.type }}</td>
            <td class="px-3 py-1.5 text-right text-gray-600">
              <span v-if="!isFolder(item)">{{ item.sizeFormatted }}</span>
              <span v-else class="italic">--</span>
            </td>
            <td class="px-3 py-1.5">
              <div class="flex gap-1">
                <span
                  v-for="tag in item.tags"
                  :key="tag"
                  :class="`${getTagColor(tag)} text-white px-1.5 py-0.5 rounded text-[9px]`"
                >
                  {{ tag }}
                </span>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
