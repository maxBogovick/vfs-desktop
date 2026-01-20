export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory: number;
  user: string | null;
  status: string;
  exe_path: string | null;
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
      // Debug log to verify data flow
      if (history.value.length % 10 === 0) {
          console.log('System Monitor Update:', snap.cpu_global_usage, 'History size:', history.value.length);
      }
      
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
    if (history.value.length < 1) return '';

    const stepX = width / (HISTORY_SIZE - 1);
    const startIndex = HISTORY_SIZE - history.value.length;

    const effectiveHeight = height - 4; // Buffer for stroke width
    const offsetY = 2;

    // Start from the first data point, not from 0,height
    const firstSnap = history.value[0];
    const firstX = startIndex * stepX;
    const firstVal = dataExtractor(firstSnap);
    const firstNormalized = Math.min(Math.max(firstVal / maxVal, 0), 1);
    const firstY = (height - offsetY) - (firstNormalized * effectiveHeight);

    let path = `M ${firstX} ${height} `; // Start at bottom of first point
    path += `L ${firstX} ${firstY} `; // Go up to first data point

    // Draw the rest of the line
    history.value.forEach((snap, i) => {
      if (i === 0) return; // Skip first point as we already handled it

      const x = (startIndex + i) * stepX;
      const val = dataExtractor(snap);
      const normalized = Math.min(Math.max(val / maxVal, 0), 1);
      const y = (height - offsetY) - (normalized * effectiveHeight);
      path += `L ${x} ${y} `;
    });

    // Close the path to create filled area
    const lastX = (startIndex + history.value.length - 1) * stepX;
    path += `L ${lastX} ${height} Z`;

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
          console.error("Error killing process:", e);
          throw e;
      }
  };

  const suspendProcess = async (pid: number) => {
      try {
          await invoke('suspend_process', { pid });
      } catch (e) {
          console.error("Error suspending process:", e);
          throw e;
      }
  };

  const resumeProcess = async (pid: number) => {
      try {
          await invoke('resume_process', { pid });
      } catch (e) {
          console.error("Error resuming process:", e);
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
    killProcess,
    suspendProcess,
    resumeProcess
  };
}
