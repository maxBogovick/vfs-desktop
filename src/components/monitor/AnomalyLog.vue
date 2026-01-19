<script setup lang="ts">
import { ref, watch } from 'vue';
import type { AnomalyEvent } from '../../composables/useSystemMetrics';

interface Props {
  anomalies: AnomalyEvent[];
}

const props = defineProps<Props>();
const listRef = ref<HTMLElement | null>(null);
const expandedIndex = ref<number | null>(null);

// Auto-scroll to bottom only if not interacting (simple heuristic: if at bottom)
watch(() => props.anomalies.length, () => {
  if (listRef.value) {
    // Force scroll for now as it's a log
    setTimeout(() => {
        if (listRef.value) {
            listRef.value.scrollTop = listRef.value.scrollHeight;
        }
    }, 50);
  }
});

const formatTime = (ts: number) => {
  return new Date(ts * 1000).toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
};

const getTitle = (anomaly: AnomalyEvent) => {
    switch(anomaly.type) {
        case 'CpuSpike': return 'CPU Violation';
        case 'MemoryLeak': return 'Memory Leak';
        case 'NetworkSurge': return 'Network Surge';
        default: return anomaly.type;
    }
};

const getSummary = (anomaly: AnomalyEvent) => {
    if (!anomaly.metadata) return '';
    switch(anomaly.type) {
        case 'CpuSpike': 
            return `${(anomaly.metadata.value || 0).toFixed(1)}%`;
        case 'MemoryLeak':
            // Show growth
            const diff = (anomaly.metadata.current_val - anomaly.metadata.start_val) / 1024 / 1024;
            return `+${diff.toFixed(0)} MB`;
        case 'NetworkSurge':
            const speed = (anomaly.metadata.current / 1024 / 1024).toFixed(1);
            return `${speed} MB/s`;
        default:
            return '';
    }
};

const toggleExpand = (index: number) => {
    if (expandedIndex.value === index) {
        expandedIndex.value = null;
    } else {
        expandedIndex.value = index;
    }
};

const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};
</script>

<template>
  <div class="h-full flex flex-col bg-[#111] border border-[#333] rounded-sm overflow-hidden text-xs">
    <div class="px-2 py-1 bg-[#1a1a1a] border-b border-[#333] text-[10px] uppercase tracking-wider text-gray-500 font-mono flex justify-between items-center">
        <span>Anomaly Log</span>
        <span class="text-[9px] bg-[#222] px-1 rounded">{{ anomalies.length }}</span>
    </div>
    
    <div ref="listRef" class="flex-1 overflow-y-auto p-1 space-y-1 font-mono">
        <div v-if="anomalies.length === 0" class="text-gray-600 italic px-2 py-1 text-[10px]">
            System Normal. No anomalies detected.
        </div>
        <div 
            v-for="(event, i) in anomalies" 
            :key="i"
            class="flex flex-col rounded bg-[#161616] border border-[#222] overflow-hidden"
            :class="[
                event.severity === 'CRITICAL' ? 'border-l-2 border-l-red-500' : 'border-l-2 border-l-yellow-500',
                expandedIndex === i ? 'bg-[#1a1a1a]' : 'hover:bg-[#1c1c1c] cursor-pointer'
            ]"
            @click="toggleExpand(i)"
        >
            <!-- Header Row -->
            <div class="flex items-center gap-2 px-2 py-1.5">
                <span class="text-gray-500 text-[10px] whitespace-nowrap">[{{ formatTime(event.timestamp) }}]</span>
                <span class="font-bold whitespace-nowrap" :class="event.severity === 'CRITICAL' ? 'text-red-400' : 'text-yellow-400'">
                    {{ getTitle(event) }}
                </span>
                <span class="ml-auto text-gray-400 text-[10px] truncate">
                    {{ getSummary(event) }}
                </span>
                <span class="text-gray-600 text-[9px] transform transition-transform" :class="expandedIndex === i ? 'rotate-180' : ''">
                    ▼
                </span>
            </div>

            <!-- Details Section (Expanded) -->
            <div v-if="expandedIndex === i" class="px-2 pb-2 pt-0.5 text-[10px] text-gray-300 border-t border-[#222] bg-[#141414]">
                <div class="grid grid-cols-[max-content_1fr] gap-x-2 gap-y-1 mt-1">
                    <template v-if="event.type === 'CpuSpike'">
                        <span class="text-gray-500">Usage:</span>
                        <span class="text-red-300">{{ event.metadata.value.toFixed(1) }}%</span>
                        <span class="text-gray-500">Threshold:</span>
                        <span>>85% for 30s</span>
                    </template>

                    <template v-else-if="event.type === 'MemoryLeak'">
                        <span class="text-gray-500">Started:</span>
                        <span>{{ formatBytes(event.metadata.start_val) }}</span>
                        <span class="text-gray-500">Current:</span>
                        <span class="text-yellow-300">{{ formatBytes(event.metadata.current_val) }}</span>
                        <span class="text-gray-500">Growth:</span>
                        <span class="text-red-300">+{{ formatBytes(event.metadata.current_val - event.metadata.start_val) }}</span>
                    </template>

                    <template v-else-if="event.type === 'NetworkSurge'">
                        <span class="text-gray-500">Current:</span>
                        <span class="text-yellow-300">{{ formatBytes(event.metadata.current) }}/s</span>
                        <span class="text-gray-500">Average:</span>
                        <span>{{ formatBytes(event.metadata.avg) }}/s</span>
                        <span class="text-gray-500">Factor:</span>
                        <span class="text-red-300">{{ (event.metadata.current / Math.max(1, event.metadata.avg)).toFixed(1) }}x</span>
                    </template>

                    <template v-else>
                        <div class="col-span-2 text-gray-500 break-all">
                            {{ JSON.stringify(event.metadata) }}
                        </div>
                    </template>
                </div>
            </div>
        </div>
    </div>
  </div>
</template>