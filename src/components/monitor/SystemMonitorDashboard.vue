<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue';
import { useSystemMetrics } from '../../composables/useSystemMetrics';
import LiveChart from './LiveChart.vue';
import CoreHeatmap from './CoreHeatmap.vue';
import AnomalyLog from './AnomalyLog.vue';
import ProcessTable from './ProcessTable.vue';

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits(['close']);

const { 
    start, stop, 
    currentSnapshot, 
    generatePath, 
    generateNetworkPath,
    anomalies 
} = useSystemMetrics();

const activeTab = ref<'anomalies' | 'processes'>('processes');

// Start monitoring when mounted/opened
onMounted(() => {
    start();
});

onUnmounted(() => {
    stop();
});

const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

const cpuPath = computed(() => {
    return generatePath(s => s.cpu_global_usage, 300, 100, 100);
});

const ramPath = computed(() => {
    if (!currentSnapshot.value) return '';
    return generatePath(s => s.ram_used, 300, 100, currentSnapshot.value.ram_total);
});

const netRx = computed(() => generateNetworkPath(true, 300, 100));
const netTx = computed(() => generateNetworkPath(false, 300, 100));

</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[3000] bg-black text-white font-mono flex flex-col overflow-hidden">
    <!-- Header -->
    <div class="h-10 border-b border-[#333] bg-[#0a0a0a] flex items-center justify-between px-4">
        <div class="flex items-center gap-4">
            <span class="text-green-500 font-bold tracking-widest text-lg">SYSTEM//MONITOR</span>
            <span class="text-[10px] text-gray-600 bg-[#111] px-2 py-0.5 rounded border border-[#222]">
                STATUS: {{ anomalies.length > 0 ? 'WARNING' : 'NOMINAL' }}
            </span>
        </div>
        <button @click="emit('close')" class="hover:text-red-500 transition-colors text-xl">×</button>
    </div>

    <!-- Dashboard Grid -->
    <div class="flex-1 p-2 grid grid-cols-12 grid-rows-12 gap-2 min-h-0">
        
        <!-- CPU Section -->
        <div class="col-span-8 row-span-4 relative">
            <LiveChart 
                :path="cpuPath" 
                color="green" 
                label="CPU Usage" 
                :value="currentSnapshot ? `${currentSnapshot.cpu_global_usage.toFixed(1)}%` : '--'"
            />
        </div>

        <!-- Cores -->
        <div class="col-span-4 row-span-4">
            <CoreHeatmap :cores="currentSnapshot?.cpu_cores_usage || []" />
        </div>

        <!-- RAM Section -->
        <div class="col-span-6 row-span-4">
            <LiveChart 
                :path="ramPath" 
                color="yellow" 
                label="Memory Usage" 
                :value="currentSnapshot ? `${formatBytes(currentSnapshot.ram_used)} / ${formatBytes(currentSnapshot.ram_total)}` : '--'"
            />
        </div>

        <!-- Network Section -->
        <div class="col-span-6 row-span-4 flex gap-2">
             <div class="flex-1">
                <LiveChart 
                    :path="netRx.path" 
                    color="blue" 
                    label="Network RX" 
                    :value="currentSnapshot ? `${formatBytes(currentSnapshot.network_rx_bytes)}/s` : '--'"
                    :sub-value="`Peak: ${formatBytes(netRx.max)}/s`"
                />
             </div>
             <div class="flex-1">
                <LiveChart 
                    :path="netTx.path" 
                    color="red" 
                    label="Network TX" 
                    :value="currentSnapshot ? `${formatBytes(currentSnapshot.network_tx_bytes)}/s` : '--'"
                    :sub-value="`Peak: ${formatBytes(netTx.max)}/s`"
                />
             </div>
        </div>

        <!-- Bottom: Tabbed Section (Processes / Anomalies) -->
        <div class="col-span-12 row-span-4 flex flex-col bg-[#111] border border-[#333] rounded-sm overflow-hidden">
            <div class="flex border-b border-[#333]">
                <button 
                    @click="activeTab = 'processes'" 
                    class="px-4 py-1 text-[10px] uppercase tracking-wider font-mono hover:bg-[#222] transition-colors"
                    :class="activeTab === 'processes' ? 'bg-[#222] text-white border-b-2 border-green-500' : 'text-gray-500'"
                >
                    Processes
                </button>
                <button 
                    @click="activeTab = 'anomalies'" 
                    class="px-4 py-1 text-[10px] uppercase tracking-wider font-mono hover:bg-[#222] transition-colors"
                    :class="activeTab === 'anomalies' ? 'bg-[#222] text-white border-b-2 border-yellow-500' : 'text-gray-500'"
                >
                    Anomalies <span v-if="anomalies.length" class="ml-1 text-yellow-500">({{anomalies.length}})</span>
                </button>
            </div>
            
            <div class="flex-1 overflow-hidden relative">
                <div v-show="activeTab === 'processes'" class="absolute inset-0">
                    <ProcessTable :processes="currentSnapshot?.top_processes || []" />
                </div>
                <div v-show="activeTab === 'anomalies'" class="absolute inset-0">
                    <AnomalyLog :anomalies="anomalies" />
                </div>
            </div>
        </div>

    </div>
  </div>
</template>