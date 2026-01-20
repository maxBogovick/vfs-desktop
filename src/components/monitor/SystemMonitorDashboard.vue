<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue';
import { useSystemMetrics } from '../../composables/useSystemMetrics';
import LiveChart from './LiveChart.vue';
import CoreHeatmap from './CoreHeatmap.vue';
import AnomalyLog from './AnomalyLog.vue';
import ProcessTable from './ProcessTable.vue';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits(['close']);

const {
  start, stop,
  history,
  currentSnapshot,
  anomalies
} = useSystemMetrics();

const activeTab = ref<'anomalies' | 'processes'>('processes');
const showCharts = ref(true);
const chartsHeight = ref(400);
const isResizing = ref(false);
const currentTime = ref(new Date().toLocaleTimeString());

// Individual chart visibility
const showCpu = ref(true);
const showCores = ref(true);
const showRam = ref(true);
const showNetwork = ref(true);

// Update time every second
let timeInterval: number;

onMounted(() => {
  start();
  window.addEventListener('mousemove', handleResize);
  window.addEventListener('mouseup', stopResize);

  timeInterval = setInterval(() => {
    currentTime.value = new Date().toLocaleTimeString();
  }, 1000);
});

onUnmounted(() => {
  stop();
  window.removeEventListener('mousemove', handleResize);
  window.removeEventListener('mouseup', stopResize);
  clearInterval(timeInterval);
});

const startResize = (e: MouseEvent) => {
  isResizing.value = true;
  e.preventDefault();
};

const handleResize = (e: MouseEvent) => {
  if (!isResizing.value) return;
  const maxHeight = window.innerHeight - 300;
  const newHeight = Math.max(200, Math.min(maxHeight, e.clientY - 40));
  chartsHeight.value = newHeight;
};

const stopResize = () => {
  isResizing.value = false;
};

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

const formatPercent = (val: number) => val.toFixed(0) + '%';

const systemStatus = computed(() => {
  if (!currentSnapshot.value) return { text: 'INITIALIZING', color: 'text-blue-400', bg: 'bg-blue-900/20', border: 'border-blue-900/50' };
  if (anomalies.value.length > 0) return { text: 'WARNING', color: 'text-yellow-400', bg: 'bg-yellow-900/20', border: 'border-yellow-900/50' };
  if (currentSnapshot.value.cpu_global_usage > 80) return { text: 'HIGH LOAD', color: 'text-orange-400', bg: 'bg-orange-900/20', border: 'border-orange-900/50' };
  return { text: 'NOMINAL', color: 'text-green-400', bg: 'bg-green-900/20', border: 'border-green-900/50' };
});

const cpuData = computed(() => history.value.map(s => s.cpu_global_usage));
const ramData = computed(() => history.value.map(s => s.ram_used));
const netRxData = computed(() => history.value.map(s => s.network_rx_bytes));
const netTxData = computed(() => history.value.map(s => s.network_tx_bytes));

const ramPercentage = computed(() => {
  if (!currentSnapshot.value) return 0;
  return ((currentSnapshot.value.ram_used / currentSnapshot.value.ram_total) * 100).toFixed(1);
});

const visibleChartsCount = computed(() => {
  let count = 0;
  if (showCpu.value) count++;
  if (showCores.value) count++;
  if (showRam.value) count++;
  if (showNetwork.value) count++;
  return count;
});
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[3000] bg-gradient-to-br from-black via-gray-950 to-black text-white font-mono flex flex-col">
    <!-- Professional Header with Gradient -->
    <div class="h-14 border-b border-[#333] bg-gradient-to-r from-[#0a0a0a] via-[#0f0f0f] to-[#0a0a0a] flex items-center justify-between px-6 flex-shrink-0 shadow-lg backdrop-blur-sm">
      <div class="flex items-center gap-6">
        <!-- Logo/Title -->
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded bg-gradient-to-br from-green-500 to-green-700 flex items-center justify-center shadow-lg shadow-green-500/20">
            <svg class="w-5 h-5 text-black" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
          </div>
          <div>
            <span class="text-green-400 font-bold tracking-[0.2em] text-lg block leading-none">SYSTEM MONITOR</span>
            <span class="text-[9px] text-gray-500 tracking-wider">REAL-TIME DIAGNOSTICS</span>
          </div>
        </div>

        <!-- Status Badge -->
        <div :class="['text-[10px] px-3 py-1.5 rounded-md border font-semibold tracking-wider flex items-center gap-2', systemStatus.bg, systemStatus.border, systemStatus.color]">
          <div class="w-1.5 h-1.5 rounded-full animate-pulse" :class="systemStatus.color.replace('text-', 'bg-')"></div>
          {{ systemStatus.text }}
        </div>

        <!-- Quick Stats -->
        <div v-if="currentSnapshot" class="flex items-center gap-4 ml-4 text-[11px]">
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">CPU:</span>
            <span class="font-bold text-green-400">{{ currentSnapshot.cpu_global_usage.toFixed(1) }}%</span>
          </div>
          <div class="w-px h-4 bg-gray-700"></div>
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">RAM:</span>
            <span class="font-bold text-yellow-400">{{ ramPercentage }}%</span>
          </div>
          <div class="w-px h-4 bg-gray-700"></div>
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">PROC:</span>
            <span class="font-bold text-blue-400">{{ currentSnapshot.process_count }}</span>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <!-- Time Display -->
        <div class="text-xs text-gray-400 font-mono bg-black/30 px-3 py-1.5 rounded border border-[#333]">
          {{ currentTime }}
        </div>

        <!-- Controls -->
        <button
            @click="showCharts = !showCharts"
            class="text-xs px-3 py-1.5 border border-[#444] rounded-md hover:bg-[#222] transition-all duration-200 flex items-center gap-2"
            :class="showCharts ? 'text-green-400 border-green-900/50 bg-green-900/10 shadow-sm shadow-green-500/10' : 'text-gray-500 hover:text-gray-300'"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z" />
          </svg>
          {{ showCharts ? 'Hide Charts' : 'Show Charts' }}
          <span v-if="!showCharts && visibleChartsCount < 4" class="text-[9px] px-1.5 py-0.5 rounded bg-gray-800 text-gray-400">
                    {{ visibleChartsCount }}/4
                </span>
        </button>

        <button
            @click="emit('close')"
            class="w-9 h-9 flex items-center justify-center rounded-md border border-red-900/30 hover:border-red-500 hover:bg-red-500/10 text-gray-400 hover:text-red-400 transition-all duration-200"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Content Area with proper flex layout -->
    <div class="flex-1 flex flex-col overflow-hidden bg-gradient-to-b from-[#050505] to-black">

      <!-- Charts Section (Resizable with proper bounds) -->
      <div
          v-if="showCharts"
          class="w-full flex-shrink-0 border-b border-[#333] bg-gradient-to-b from-[#0a0a0a] to-[#050505] overflow-hidden shadow-2xl"
          :style="{ height: `${chartsHeight}px` }"
      >
        <div class="h-full w-full p-4 flex flex-col gap-3 overflow-hidden">
          <!-- Row 1: CPU (2/3) + Cores (1/3) -->
          <div class="flex gap-3 overflow-hidden" style="height: calc(50% - 6px); min-height: 0;">
            <!-- CPU Chart with collapse -->
            <div v-if="showCpu" class="w-2/3 h-full overflow-hidden rounded-lg shadow-xl relative group" style="min-height: 0;">
              <button
                  @click="showCpu = false"
                  class="absolute top-2 right-2 z-[50] w-6 h-6 rounded-md bg-black/60 hover:bg-red-500/20 border border-[#333] hover:border-red-500/50 text-gray-400 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-all duration-200 flex items-center justify-center"
                  title="Hide CPU chart"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
              <LiveChart
                  :data="cpuData"
                  :capacity="120"
                  :max="100"
                  color="green"
                  label="CPU Usage"
                  unit="%"
                  :format-tick="formatPercent"
                  :value="currentSnapshot ? `${currentSnapshot.cpu_global_usage.toFixed(1)}%` : '--'"
              />
            </div>
            <!-- CPU Collapsed State -->
            <div v-else class="w-2/3 h-full rounded-lg border border-dashed border-[#333] bg-[#0a0a0a]/50 backdrop-blur-sm flex items-center justify-center hover:border-green-500/50 hover:bg-green-500/5 transition-all duration-200 cursor-pointer group" @click="showCpu = true">
              <div class="text-center">
                <svg class="w-8 h-8 mx-auto text-gray-600 group-hover:text-green-400 transition-colors mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z" />
                </svg>
                <div class="text-xs text-gray-500 group-hover:text-green-400 transition-colors font-semibold">CPU USAGE</div>
                <div v-if="currentSnapshot" class="text-lg font-bold text-green-400 mt-1">{{ currentSnapshot.cpu_global_usage.toFixed(1) }}%</div>
              </div>
            </div>

            <!-- Cores Heatmap with collapse -->
            <div v-if="showCores" class="w-1/3 h-full overflow-hidden rounded-lg shadow-xl relative group" style="min-height: 0;">
              <button
                  @click="showCores = false"
                  class="absolute top-2 right-2 z-[50] w-6 h-6 rounded-md bg-black/60 hover:bg-red-500/20 border border-[#333] hover:border-red-500/50 text-gray-400 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-all duration-200 flex items-center justify-center"
                  title="Hide cores heatmap"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
              <CoreHeatmap :cores="currentSnapshot?.cpu_cores_usage || []" />
            </div>
            <!-- Cores Collapsed State -->
            <div v-else class="w-1/3 h-full rounded-lg border border-dashed border-[#333] bg-[#0a0a0a]/50 backdrop-blur-sm flex items-center justify-center hover:border-blue-500/50 hover:bg-blue-500/5 transition-all duration-200 cursor-pointer group" @click="showCores = true">
              <div class="text-center">
                <svg class="w-8 h-8 mx-auto text-gray-600 group-hover:text-blue-400 transition-colors mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                </svg>
                <div class="text-xs text-gray-500 group-hover:text-blue-400 transition-colors font-semibold">CPU CORES</div>
                <div v-if="currentSnapshot" class="text-sm text-blue-400 mt-1">{{ currentSnapshot.cpu_cores_usage?.length || 0 }} cores</div>
              </div>
            </div>
          </div>

          <!-- Row 2: RAM (1/2) + Network (1/2) -->
          <div class="flex gap-3 overflow-hidden" style="height: calc(50% - 6px); min-height: 0;">
            <!-- RAM Chart with collapse -->
            <div v-if="showRam" class="w-1/2 h-full overflow-hidden rounded-lg shadow-xl relative group" style="min-height: 0;">
              <button
                  @click="showRam = false"
                  class="absolute top-2 right-2 z-[50] w-6 h-6 rounded-md bg-black/60 hover:bg-red-500/20 border border-[#333] hover:border-red-500/50 text-gray-400 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-all duration-200 flex items-center justify-center"
                  title="Hide RAM chart"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
              <LiveChart
                  :data="ramData"
                  :capacity="120"
                  :max="currentSnapshot?.ram_total"
                  color="yellow"
                  label="Memory Usage"
                  :format-tick="formatBytes"
                  :value="currentSnapshot ? `${formatBytes(currentSnapshot.ram_used)} / ${formatBytes(currentSnapshot.ram_total)}` : '--'"
              />
            </div>
            <!-- RAM Collapsed State -->
            <div v-else class="w-1/2 h-full rounded-lg border border-dashed border-[#333] bg-[#0a0a0a]/50 backdrop-blur-sm flex items-center justify-center hover:border-yellow-500/50 hover:bg-yellow-500/5 transition-all duration-200 cursor-pointer group" @click="showRam = true">
              <div class="text-center">
                <svg class="w-8 h-8 mx-auto text-gray-600 group-hover:text-yellow-400 transition-colors mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                </svg>
                <div class="text-xs text-gray-500 group-hover:text-yellow-400 transition-colors font-semibold">MEMORY</div>
                <div v-if="currentSnapshot" class="text-lg font-bold text-yellow-400 mt-1">{{ ramPercentage }}%</div>
              </div>
            </div>

            <!-- Network Charts with collapse -->
            <div v-if="showNetwork" class="w-1/2 h-full flex gap-3 overflow-hidden" style="min-height: 0;">
              <div class="flex-1 h-full overflow-hidden rounded-lg shadow-xl relative group" style="min-height: 0;">
                <button
                    @click="showNetwork = false"
                    class="absolute top-2 right-2 z-[50] w-6 h-6 rounded-md bg-black/60 hover:bg-red-500/20 border border-[#333] hover:border-red-500/50 text-gray-400 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-all duration-200 flex items-center justify-center"
                    title="Hide network charts"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
                <LiveChart
                    :data="netRxData"
                    :capacity="120"
                    color="blue"
                    label="Network RX"
                    :format-tick="formatBytes"
                    :value="currentSnapshot ? `${formatBytes(currentSnapshot.network_rx_bytes)}/s` : '--'"
                />
              </div>
              <div class="flex-1 h-full overflow-hidden rounded-lg shadow-xl" style="min-height: 0;">
                <LiveChart
                    :data="netTxData"
                    :capacity="120"
                    color="red"
                    label="Network TX"
                    :format-tick="formatBytes"
                    :value="currentSnapshot ? `${formatBytes(currentSnapshot.network_tx_bytes)}/s` : '--'"
                />
              </div>
            </div>
            <!-- Network Collapsed State -->
            <div v-else class="w-1/2 h-full rounded-lg border border-dashed border-[#333] bg-[#0a0a0a]/50 backdrop-blur-sm flex items-center justify-center hover:border-purple-500/50 hover:bg-purple-500/5 transition-all duration-200 cursor-pointer group" @click="showNetwork = true">
              <div class="text-center">
                <svg class="w-8 h-8 mx-auto text-gray-600 group-hover:text-purple-400 transition-colors mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
                <div class="text-xs text-gray-500 group-hover:text-purple-400 transition-colors font-semibold">NETWORK</div>
                <div v-if="currentSnapshot" class="text-xs text-blue-400 mt-1">RX: {{ formatBytes(currentSnapshot.network_rx_bytes) }}/s</div>
                <div v-if="currentSnapshot" class="text-xs text-red-400">TX: {{ formatBytes(currentSnapshot.network_tx_bytes) }}/s</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Resizer Handle - Enhanced -->
      <div
          v-if="showCharts"
          class="h-2 bg-gradient-to-b from-[#1a1a1a] to-[#0a0a0a] hover:from-green-900/30 hover:to-green-900/10 cursor-row-resize flex-shrink-0 flex items-center justify-center transition-all duration-200 group select-none border-y border-[#222]"
          @mousedown="startResize"
      >
        <div class="flex gap-1 opacity-40 group-hover:opacity-100 transition-opacity">
          <div class="w-8 h-0.5 bg-gray-600 group-hover:bg-green-400 rounded-full transition-colors"></div>
        </div>
      </div>

      <!-- Bottom: Tabbed Section (Processes / Anomalies) - Enhanced -->
      <div class="flex-1 flex flex-col overflow-hidden bg-[#0a0a0a] p-4">
        <!-- Professional Tabs -->
        <div class="flex border-b border-[#333] mb-3 flex-shrink-0 gap-1">
          <button
              @click="activeTab = 'processes'"
              class="relative px-6 py-2.5 text-[11px] uppercase tracking-[0.15em] font-semibold hover:bg-[#1a1a1a] transition-all duration-200 rounded-t-md"
              :class="activeTab === 'processes' ? 'bg-[#1a1a1a] text-green-400' : 'text-gray-500 hover:text-gray-300'"
          >
            <div class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              Processes
              <span v-if="currentSnapshot" class="text-[9px] px-1.5 py-0.5 rounded bg-gray-800 text-gray-400">
                            {{ currentSnapshot.process_count }}
                        </span>
            </div>
            <div v-if="activeTab === 'processes'" class="absolute bottom-0 left-0 right-0 h-0.5 bg-gradient-to-r from-transparent via-green-400 to-transparent"></div>
          </button>

          <button
              @click="activeTab = 'anomalies'"
              class="relative px-6 py-2.5 text-[11px] uppercase tracking-[0.15em] font-semibold hover:bg-[#1a1a1a] transition-all duration-200 rounded-t-md"
              :class="activeTab === 'anomalies' ? 'bg-[#1a1a1a] text-yellow-400' : 'text-gray-500 hover:text-gray-300'"
          >
            <div class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5" :class="anomalies.length > 0 ? 'animate-pulse' : ''" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              Anomalies
              <span v-if="anomalies.length" class="text-[9px] px-1.5 py-0.5 rounded bg-yellow-900/50 text-yellow-400 animate-pulse">
                            {{ anomalies.length }}
                        </span>
            </div>
            <div v-if="activeTab === 'anomalies'" class="absolute bottom-0 left-0 right-0 h-0.5 bg-gradient-to-r from-transparent via-yellow-400 to-transparent"></div>
          </button>
        </div>

        <!-- Content Area with smooth transitions -->
        <div class="flex-1 overflow-hidden border border-[#333] rounded-lg shadow-2xl bg-black/40 backdrop-blur-sm">
          <div v-show="activeTab === 'processes'" class="h-full overflow-auto">
            <ProcessTable :processes="currentSnapshot?.top_processes || []" />
          </div>
          <div v-show="activeTab === 'anomalies'" class="h-full overflow-auto">
            <AnomalyLog :anomalies="anomalies" />
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
/* Smooth animations */
@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Custom scrollbar */
:deep(*::-webkit-scrollbar) {
  width: 8px;
  height: 8px;
}

:deep(*::-webkit-scrollbar-track) {
  background: #0a0a0a;
}

:deep(*::-webkit-scrollbar-thumb) {
  background: #333;
  border-radius: 4px;
}

:deep(*::-webkit-scrollbar-thumb:hover) {
  background: #444;
}
</style>