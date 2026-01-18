<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useTorrent } from '../composables/useTorrent';
import FilePickerDialog from './FilePickerDialog.vue';

// Fallback formatBytes if not found (I'll define it here to be safe)
function formatBytesLocal(bytes: number, decimals = 2) {
  if (!+bytes) return '0 Bytes';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

const { torrents, showManager, addTorrent: addTorrentBackend, toggleManager, startPolling, stopPolling } = useTorrent();
const showFilePicker = ref(false);

const handleAddClick = () => {
    // Instead of calling addTorrent (which uses OS dialog), open our FilePickerDialog
    showFilePicker.value = true;
};

const handleFileSelect = async (path: string) => {
    // Manually invoke backend command or use a exposed method from composable that takes path
    // useTorrent's addTorrent likely uses open() then invoke().
    // We need to just invoke().
    // Let's modify useTorrent or just import invoke here?
    // useTorrent exports addTorrent which handles UI. 
    // I should create a separate method in useTorrent or just call invoke directly here.
    // Importing invoke is cleaner if useTorrent doesn't expose a "addByPath" method.
    
    // Actually, looking at useTorrent.ts, it imports invoke.
    // Let's check if I can import invoke here too.
    try {
        await import('@tauri-apps/api/core').then(async ({ invoke }) => {
            await invoke('add_torrent_file', { path });
            // trigger refresh
            // useTorrent exposes fetchTorrents locally but not exported?
            // "startPolling" calls it.
            // But we don't have access to fetchTorrents directly.
            // It will update on next poll (1s).
        });
    } catch (e) {
        console.error("Failed to add torrent:", e);
    }
    
    showFilePicker.value = false;
};

onMounted(() => {
  startPolling();
});

onUnmounted(() => {
  stopPolling();
});
</script>

<template>
  <!-- Main Torrent Manager Modal -->
  <div v-if="showManager" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div class="bg-white dark:bg-[#1e1e1e] w-[800px] h-[600px] rounded-lg shadow-xl flex flex-col border border-gray-200 dark:border-[#333]">
      
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-[#333] bg-gray-50 dark:bg-[#252526]">
        <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">Torrent Download Manager</h2>
        <button 
          @click="toggleManager" 
          class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 text-xl leading-none"
        >
          &times;
        </button>
      </div>

      <!-- Toolbar -->
      <div class="p-3 bg-white dark:bg-[#1e1e1e] border-b border-gray-200 dark:border-[#333]">
        <button 
          @click="handleAddClick" 
          class="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm transition-colors"
        >
          <span>+</span> Add Torrent
        </button>
      </div>

      <!-- List -->
      <div class="flex-1 overflow-y-auto p-4 bg-gray-50 dark:bg-[#1e1e1e]">
        <div v-if="torrents.length === 0" class="flex flex-col items-center justify-center h-full text-gray-400">
          <span class="text-4xl mb-2">📥</span>
          <p>No downloads yet</p>
        </div>

        <div v-else class="space-y-3">
          <div 
            v-for="torrent in torrents" 
            :key="torrent.id" 
            class="bg-white dark:bg-[#2d2d2d] rounded border border-gray-200 dark:border-[#333] p-3 shadow-sm"
          >
            <div class="flex justify-between items-start mb-2">
              <div class="font-medium text-gray-800 dark:text-gray-200 truncate pr-4" :title="torrent.name || 'Unknown'">
                {{ torrent.name || 'Unknown Torrent' }}
              </div>
              <div class="text-xs text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-[#333] px-2 py-0.5 rounded">
                {{ torrent.state }}
              </div>
            </div>

            <div class="w-full bg-gray-200 dark:bg-[#404040] rounded-full h-2 mb-2">
              <div 
                class="bg-blue-600 h-2 rounded-full transition-all duration-500" 
                :style="{ width: `${torrent.progress}%` }"
              ></div>
            </div>

            <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400">
              <div>
                <span>{{ formatBytesLocal(torrent.finished_bytes) }}</span> / 
                <span>{{ formatBytesLocal(torrent.total_bytes) }}</span>
              </div>
              <div>
                <span v-if="torrent.download_speed > 0" class="text-green-600 dark:text-green-400">
                  ⬇ {{ formatBytesLocal(torrent.download_speed) }}/s
                </span>
                <span v-else>
                  --
                </span>
              </div>
              <div>
                {{ torrent.progress.toFixed(1) }}%
              </div>
            </div>
          </div>
        </div>
      </div>
      
    </div>
  </div>

  <!-- Internal File Picker Dialog -->
  <FilePickerDialog 
    :is-open="showFilePicker" 
    title="Select .torrent file"
    :allowed-extensions="['torrent']"
    @close="showFilePicker = false"
    @select="handleFileSelect"
  />
</template>