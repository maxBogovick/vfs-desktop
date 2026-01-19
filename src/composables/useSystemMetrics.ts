export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory: number;
  user: string | null;
  status: string;
}

export interface SystemSnapshot {
  timestamp: number;
  cpu_global_usage: number;
  cpu_cores_usage: number[];
  ram_used: number;
  ram_total: number;
  swap_used: number;
  swap_total: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
  process_count: number;
  top_processes: ProcessInfo[];
}

import { ref, computed, onUnmounted, shallowRef } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface AnomalyEvent {
  timestamp: number;
  type: string; 
  severity: string; 
  metadata: any;
}

const HISTORY_SIZE = 120;

const history = shallowRef<SystemSnapshot[]>([]);
const anomalies = ref<AnomalyEvent[]>([]);
const isMonitoring = ref(false);
let unlistenUpdate: UnlistenFn | null = null;
let unlistenAnomaly: UnlistenFn | null = null;

export function useSystemMetrics() {
  
  const start = async () => {
    if (isMonitoring.value) return;
    
    await invoke('start_monitoring');
    isMonitoring.value = true;

    unlistenUpdate = await listen<SystemSnapshot>('sys_mon_update', (event) => {
      const snap = event.payload;
      const newHistory = [...history.value, snap];
      if (newHistory.length > HISTORY_SIZE) {
        newHistory.shift();
      }
      history.value = newHistory;
    });

    unlistenAnomaly = await listen<AnomalyEvent[]>('sys_mon_anomaly', (event) => {
      anomalies.value.push(...event.payload);
      if (anomalies.value.length > 50) {
        anomalies.value = anomalies.value.slice(-50);
      }
    });
  };

  const stop = async () => {
    if (!isMonitoring.value) return;
    
    await invoke('stop_monitoring');
    if (unlistenUpdate) {
      unlistenUpdate();
      unlistenUpdate = null;
    }
    if (unlistenAnomaly) {
      unlistenAnomaly();
      unlistenAnomaly = null;
    }
    isMonitoring.value = false;
  };

  const currentSnapshot = computed(() => {
    return history.value.length > 0 ? history.value[history.value.length - 1] : null;
  });

  const generatePath = (
    dataExtractor: (snap: SystemSnapshot) => number, 
    width: number, 
    height: number, 
    maxVal: number
  ) => {
    if (history.value.length < 2) return '';

    const stepX = width / (HISTORY_SIZE - 1);
    const startIndex = HISTORY_SIZE - history.value.length;
    
    let path = `M 0 ${height} `;
    
    history.value.forEach((snap, i) => {
      const x = (startIndex + i) * stepX;
      const val = dataExtractor(snap);
      const normalized = Math.min(Math.max(val / maxVal, 0), 1);
      const y = height - (normalized * height);
      path += `L ${x} ${y} `;
    });

    path += `L ${width} ${height} Z`;
    return path;
  };

  const generateNetworkPath = (
    isRx: boolean,
    width: number,
    height: number
  ) => {
    if (history.value.length < 2) return { path: '', max: 0 };
    
    const extractor = (s: SystemSnapshot) => isRx ? s.network_rx_bytes : s.network_tx_bytes;
    let max = 1024 * 10;
    for (const s of history.value) {
        max = Math.max(max, extractor(s));
    }
    
    return {
        path: generatePath(extractor, width, height, max),
        max
    };
  };

  const killProcess = async (pid: number) => {
      try {
          await invoke('kill_process', { pid });
      } catch (e) {
          console.error(e);
          throw e;
      }
  };

  return {
    start,
    stop,
    isMonitoring,
    history,
    anomalies,
    currentSnapshot,
    generatePath,
    generateNetworkPath,
    killProcess
  };
}