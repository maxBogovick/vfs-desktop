<script setup lang="ts">
import { onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  isOpen: boolean;
  shareInfo: { url: string; qr_svg: string; filename: string } | null;
}>();

const emit = defineEmits(['close']);

const handleClose = async () => {
  try {
    await invoke('stop_share');
  } catch (err) {
    console.error('Failed to stop share:', err);
  }
  emit('close');
};

</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm" @click.self="handleClose">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 max-w-sm w-full mx-4 transform transition-all animate-fade-in">
      <div class="text-center">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-1">Magic Share 🪄</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          Scan to download (Same Wi-Fi required)
        </p>
        
        <div v-if="shareInfo" class="flex flex-col items-center">
          <!-- QR Code Container -->
          <div class="bg-white p-2 rounded-xl shadow-inner border border-gray-100 mb-4">
             <div class="w-48 h-48" v-html="shareInfo.qr_svg"></div>
          </div>
          
          <!-- URL -->
          <div class="bg-gray-50 dark:bg-gray-900/50 rounded px-3 py-2 mb-4 w-full">
            <p class="font-mono text-xs text-center text-blue-600 dark:text-blue-400 break-all select-all">
              {{ shareInfo.url }}
            </p>
          </div>

          <div class="flex items-center justify-center space-x-2 mb-6">
             <span class="text-2xl">📄</span>
             <span class="text-sm font-medium text-gray-700 dark:text-gray-200 truncate max-w-[200px]">
                {{ shareInfo.filename }}
             </span>
          </div>
        </div>

        <button
          @click="handleClose"
          class="w-full inline-flex justify-center items-center rounded-lg border border-transparent shadow-sm px-4 py-2 bg-red-50 text-red-700 hover:bg-red-100 dark:bg-red-900/20 dark:text-red-400 dark:hover:bg-red-900/30 transition-colors font-medium sm:text-sm"
        >
          Stop Sharing
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes fade-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
.animate-fade-in {
  animation: fade-in 0.2s ease-out;
}
</style>
