import { ref, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export interface TorrentInfo {
  id: number;
  name: string | null;
  progress: number;
  state: string;
  total_bytes: number;
  finished_bytes: number;
  download_speed: number;
}

const torrents = ref<TorrentInfo[]>([]);
const isPolling = ref(false);
const showManager = ref(false);
let pollInterval: number | null = null;

export function useTorrent() {
  const fetchTorrents = async () => {
    try {
      torrents.value = await invoke<TorrentInfo[]>('get_torrents');
    } catch (e) {
      console.error('Failed to fetch torrents:', e);
    }
  };

  const startPolling = () => {
    if (isPolling.value) return;
    isPolling.value = true;
    fetchTorrents();
    pollInterval = window.setInterval(fetchTorrents, 1000);
  };

  const stopPolling = () => {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
    isPolling.value = false;
  };

  const addTorrent = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Torrent Files', extensions: ['torrent'] }]
      });

      if (selected) {
        await invoke('add_torrent_file', { path: selected });
        await fetchTorrents();
      }
    } catch (e) {
      console.error('Failed to add torrent:', e);
    }
  };

  const toggleManager = () => {
    showManager.value = !showManager.value;
    if (showManager.value) {
      startPolling();
    } else {
      stopPolling();
    }
  };

  return {
    torrents,
    showManager,
    addTorrent,
    toggleManager,
    startPolling,
    stopPolling
  };
}
