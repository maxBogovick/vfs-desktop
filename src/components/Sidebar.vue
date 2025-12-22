<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { FileItem } from '../types';
import { useFileSystem } from '../composables/useFileSystem';

interface Emits {
  (e: 'navigate', path: string): void;
}

const emit = defineEmits<Emits>();
const { getSystemFolders } = useFileSystem();

const sidebarExpanded = ref({
  quickAccess: true,
  myComputer: false,
  favorites: false,
});

const systemFolders = ref<FileItem[]>([]);

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

onMounted(async () => {
  systemFolders.value = await getSystemFolders();
});
</script>

<template>
  <div class="w-[200px] bg-white border-r border-[#919B9C] overflow-y-auto flex-shrink-0">
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

      <!-- My Computer / This PC -->
      <div class="mb-2">
        <div
          @click="toggleSection('myComputer')"
          class="flex items-center gap-1.5 py-1.5 px-1 hover:bg-[#C1D2EE] cursor-pointer rounded"
        >
          <span class="text-xs">{{ sidebarExpanded.myComputer ? '▼' : '▶' }}</span>
          <span class="text-base">💾</span>
          <span class="text-[11px] font-bold">This Mac</span>
        </div>

        <transition name="expand">
          <div v-if="sidebarExpanded.myComputer" class="pl-6 mt-1 space-y-0.5">
            <div
              v-for="folder in systemFolders"
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
