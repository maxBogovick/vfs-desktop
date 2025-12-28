<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { FileItem } from '../types';

interface Props {
  files: FileItem[];
  width?: number;
}

interface Emits {
  (e: 'close'): void;
  (e: 'resize', width: number): void;
}

const props = withDefaults(defineProps<Props>(), {
  width: 400,
});

const emit = defineEmits<Emits>();

// Resizer state
const isResizing = ref(false);
const startX = ref(0);
const startWidth = ref(0);

// Recursive size calculation
const showFullSize = ref(false);
const recursiveSizes = ref<Map<string, { total_bytes: number; total_items: number }>>(new Map());
const isCalculatingRecursive = ref(false);

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

  const delta = startX.value - event.clientX;
  const newWidth = Math.max(300, Math.min(800, startWidth.value + delta));

  emit('resize', newWidth);
};

const handleResizeEnd = () => {
  isResizing.value = false;

  document.removeEventListener('mousemove', handleResizeMove);
  document.removeEventListener('mouseup', handleResizeEnd);
};

// Format bytes to human-readable
const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// Calculate recursive sizes when showFullSize changes
watch(showFullSize, async (newVal) => {
  if (newVal && !isCalculatingRecursive.value) {
    await calculateAllRecursiveSizes();
  }
});

watch(() => props.files, () => {
  // Reset recursive sizes when files change
  recursiveSizes.value.clear();
  if (showFullSize.value) {
    calculateAllRecursiveSizes();
  }
});

const calculateAllRecursiveSizes = async () => {
  isCalculatingRecursive.value = true;

  const folders = props.files.filter(f => f.type === 'folder' || f.type === 'drive');

  for (const folder of folders) {
    try {
      const result = await invoke<{ total_bytes: number; total_items: number }>(
        'calculate_directory_size',
        { path: folder.path }
      );
      recursiveSizes.value.set(folder.id, result);
    } catch (err) {
      console.error(`Failed to calculate size for ${folder.path}:`, err);
    }
  }

  isCalculatingRecursive.value = false;
};

const getItemSize = (item: FileItem): number => {
  if (showFullSize.value && (item.type === 'folder' || item.type === 'drive')) {
    const recursive = recursiveSizes.value.get(item.id);
    return recursive ? recursive.total_bytes : (item.size || 0);
  }
  return item.size || 0;
};

// Statistics
const stats = computed(() => {
  const folders = props.files.filter(f => f.type === 'folder' || f.type === 'drive');
  const files = props.files.filter(f => f.type !== 'folder' && f.type !== 'drive');

  const totalSize = props.files.reduce((sum, f) => sum + getItemSize(f), 0);

  // Type distribution
  const typeDistribution: Record<string, { count: number; size: number }> = {};
  props.files.forEach(file => {
    if (!typeDistribution[file.type]) {
      typeDistribution[file.type] = { count: 0, size: 0 };
    }
    typeDistribution[file.type].count++;
    typeDistribution[file.type].size += getItemSize(file);
  });

  // Largest files (top 10) - include folders if showFullSize is enabled
  const largestFiles = [...props.files]
    .filter(f => showFullSize.value ? true : (f.type !== 'folder' && f.type !== 'drive'))
    .sort((a, b) => getItemSize(b) - getItemSize(a))
    .slice(0, 10);

  return {
    totalItems: props.files.length,
    totalFolders: folders.length,
    totalFiles: files.length,
    totalSize,
    typeDistribution: Object.entries(typeDistribution)
      .map(([type, data]) => ({ type, ...data }))
      .sort((a, b) => b.size - a.size),
    largestFiles,
  };
});

// Icon for file types
const getTypeIcon = (type: string): string => {
  const icons: Record<string, string> = {
    folder: '📁',
    drive: '💾',
    image: '🖼️',
    pdf: '📄',
    code: '📜',
    video: '🎬',
    audio: '🎵',
    archive: '📦',
    file: '📄',
    system: '⚙️',
  };
  return icons[type] || '📄';
};

// Color for each type (for pie chart)
const getTypeColor = (type: string, index: number): string => {
  const colors = [
    '#3B82F6', '#10B981', '#F59E0B', '#EF4444', '#8B5CF6',
    '#EC4899', '#06B6D4', '#84CC16', '#F97316', '#14B8A6'
  ];
  return colors[index % colors.length];
};

// Calculate pie chart slices
const pieSlices = computed(() => {
  const total = stats.value.totalSize;
  if (total === 0) return [];

  let currentAngle = 0;
  return stats.value.typeDistribution.map((item, index) => {
    const percentage = (item.size / total) * 100;
    const angle = (item.size / total) * 360;
    const slice = {
      type: item.type,
      percentage,
      startAngle: currentAngle,
      endAngle: currentAngle + angle,
      color: getTypeColor(item.type, index),
      size: item.size,
      count: item.count,
    };
    currentAngle += angle;
    return slice;
  });
});

// Create SVG path for pie slice
const createPieSlicePath = (startAngle: number, endAngle: number): string => {
  const radius = 80;
  const centerX = 100;
  const centerY = 100;

  const startRad = (startAngle - 90) * Math.PI / 180;
  const endRad = (endAngle - 90) * Math.PI / 180;

  const x1 = centerX + radius * Math.cos(startRad);
  const y1 = centerY + radius * Math.sin(startRad);
  const x2 = centerX + radius * Math.cos(endRad);
  const y2 = centerY + radius * Math.sin(endRad);

  const largeArcFlag = endAngle - startAngle > 180 ? 1 : 0;

  return `M ${centerX} ${centerY} L ${x1} ${y1} A ${radius} ${radius} 0 ${largeArcFlag} 1 ${x2} ${y2} Z`;
};
</script>

<template>
  <transition name="slide-right">
    <div
      class="bg-gradient-to-b from-[#C1D9F4] to-[#A5C8E1] border-l border-[#919B9C] flex flex-col overflow-hidden relative"
      :style="{ width: `${width}px` }"
    >
      <!-- Resizer Handle -->
      <div
        @mousedown="handleResizeStart"
        class="absolute top-0 left-0 w-1 h-full cursor-col-resize hover:bg-blue-400 transition-colors z-10"
        :class="{ 'bg-blue-500': isResizing }"
      ></div>

      <!-- Header -->
      <div class="flex justify-between items-center p-3 border-b border-[#8BA5C7]">
        <div class="text-[12px] font-bold text-[#003D7A]">📊 Folder Statistics</div>
        <button
          @click="emit('close')"
          class="text-[#003D7A] hover:text-[#0054E3] text-lg leading-none w-5 h-5 flex items-center justify-center hover:bg-white/30 rounded"
        >
          ✕
        </button>
      </div>

      <!-- Options Bar -->
      <div class="px-3 py-2 border-b border-[#8BA5C7] bg-white/30">
        <label class="flex items-center gap-2 cursor-pointer text-[10px] text-[#003D7A]">
          <input
            type="checkbox"
            v-model="showFullSize"
            :disabled="isCalculatingRecursive"
            class="w-3 h-3 cursor-pointer disabled:opacity-50"
          />
          <span :class="{ 'opacity-50': isCalculatingRecursive }">
            {{ isCalculatingRecursive ? 'Calculating folder sizes...' : 'Include subfolder sizes (recursive)' }}
          </span>
        </label>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-3 space-y-3">
        <!-- Overview Cards -->
        <div class="grid grid-cols-2 gap-2">
          <div class="bg-white/90 rounded-lg p-3 shadow-sm">
            <div class="text-[9px] text-gray-500 font-bold uppercase mb-1">Total Items</div>
            <div class="text-[20px] font-bold text-[#003D7A]">{{ stats.totalItems }}</div>
          </div>
          <div class="bg-white/90 rounded-lg p-3 shadow-sm">
            <div class="text-[9px] text-gray-500 font-bold uppercase mb-1">Total Size</div>
            <div class="text-[16px] font-bold text-[#10B981]">{{ formatBytes(stats.totalSize) }}</div>
          </div>
          <div class="bg-white/90 rounded-lg p-3 shadow-sm">
            <div class="text-[9px] text-gray-500 font-bold uppercase mb-1">Folders</div>
            <div class="text-[20px] font-bold text-[#F59E0B]">{{ stats.totalFolders }}</div>
          </div>
          <div class="bg-white/90 rounded-lg p-3 shadow-sm">
            <div class="text-[9px] text-gray-500 font-bold uppercase mb-1">Files</div>
            <div class="text-[20px] font-bold text-[#3B82F6]">{{ stats.totalFiles }}</div>
          </div>
        </div>

        <!-- Pie Chart -->
        <div v-if="stats.totalSize > 0" class="bg-white/90 rounded-lg p-3 shadow-sm">
          <div class="text-[10px] font-bold text-[#003D7A] mb-3 uppercase tracking-wide">Distribution by Type</div>
          <div class="flex items-center justify-center">
            <svg width="200" height="200" viewBox="0 0 200 200">
              <g v-for="slice in pieSlices" :key="slice.type">
                <path
                  :d="createPieSlicePath(slice.startAngle, slice.endAngle)"
                  :fill="slice.color"
                  stroke="white"
                  stroke-width="2"
                  class="hover:opacity-80 transition-opacity cursor-pointer"
                >
                  <title>{{ slice.type }}: {{ formatBytes(slice.size) }} ({{ slice.percentage.toFixed(1) }}%)</title>
                </path>
              </g>
            </svg>
          </div>
        </div>

        <!-- Type Breakdown -->
        <div class="bg-white/90 rounded-lg p-3 shadow-sm">
          <div class="text-[10px] font-bold text-[#003D7A] mb-2 uppercase tracking-wide">Breakdown by Type</div>
          <div class="space-y-2">
            <div
              v-for="(item, index) in stats.typeDistribution"
              :key="item.type"
              class="flex items-center gap-2"
            >
              <div
                class="w-3 h-3 rounded-sm flex-shrink-0"
                :style="{ backgroundColor: getTypeColor(item.type, index) }"
              ></div>
              <span class="text-[11px] flex-shrink-0 w-4">{{ getTypeIcon(item.type) }}</span>
              <div class="flex-1 min-w-0">
                <div class="text-[10px] font-medium text-gray-700 truncate">{{ item.type }}</div>
                <div class="text-[9px] text-gray-500">{{ item.count }} items • {{ formatBytes(item.size) }}</div>
              </div>
              <div class="text-[10px] text-gray-600 font-medium">
                {{ ((item.size / stats.totalSize) * 100).toFixed(1) }}%
              </div>
            </div>
          </div>
        </div>

        <!-- Largest Files -->
        <div v-if="stats.largestFiles.length > 0" class="bg-white/90 rounded-lg p-3 shadow-sm">
          <div class="text-[10px] font-bold text-[#003D7A] mb-2 uppercase tracking-wide">Largest Files</div>
          <div class="space-y-1.5">
            <div
              v-for="(file, index) in stats.largestFiles"
              :key="file.id"
              class="flex items-center gap-2 p-1.5 hover:bg-blue-50 rounded transition-colors"
            >
              <div class="text-[9px] text-gray-400 font-mono w-4">{{ index + 1 }}</div>
              <span class="text-[11px]">{{ getTypeIcon(file.type) }}</span>
              <div class="flex-1 min-w-0">
                <div class="text-[10px] font-medium text-gray-700 truncate">{{ file.name }}</div>
              </div>
              <div class="text-[10px] text-gray-600 font-medium flex-shrink-0">
                {{ formatBytes(getItemSize(file)) }}
              </div>
            </div>
          </div>
        </div>

        <!-- Empty State -->
        <div v-if="stats.totalItems === 0" class="bg-white/90 rounded-lg p-6 shadow-sm text-center">
          <div class="text-4xl mb-2">📂</div>
          <div class="text-[11px] text-gray-500">This folder is empty</div>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.slide-right-enter-active,
.slide-right-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-right-enter-from,
.slide-right-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
