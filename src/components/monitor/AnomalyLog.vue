<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import type { AnomalyEvent } from '../../composables/useSystemMetrics';
import { useSystemMetrics } from '../../composables/useSystemMetrics';

interface Props {
  anomalies: AnomalyEvent[];
}

const props = defineProps<Props>();
const listRef = ref<HTMLElement | null>(null);
const expandedIndex = ref<number | null>(null);
const { killProcess } = useSystemMetrics();

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
        case 'CpuSpike': return 'High CPU Load';
        case 'MemoryLeak': return 'Memory Leak Risk';
        case 'NetworkSurge': return 'Network Surge';
        case 'SystemSaturation': return 'System Saturation';
        default: return anomaly.type;
    }
};

const getSummary = (anomaly: AnomalyEvent) => {
    if (!anomaly.metadata) return '';
    const meta = anomaly.metadata as any;
    
    switch(anomaly.type) {
        case 'CpuSpike': 
            return meta.value ? `${meta.value.toFixed(1)}%` : '';
        case 'MemoryLeak':
            return meta.rate_mb_per_sec ? `+${meta.rate_mb_per_sec.toFixed(1)} MB/s` : '';
        case 'NetworkSurge':
            return meta.current ? `${(meta.current / 1024 / 1024).toFixed(1)} MB/s` : '';
        case 'SystemSaturation':
            return meta.resource ? `${meta.resource} ${meta.usage_percent.toFixed(0)}%` : '';
        default:
            return '';
    }
};

const getRecommendation = (anomaly: AnomalyEvent) => {
    const meta = anomaly.metadata as any;
    switch(anomaly.type) {
        case 'MemoryLeak':
            return `The process "${meta.process_name}" is consuming memory at an abnormal rate (+${meta.rate_mb_per_sec.toFixed(2)} MB/s). If this continues, it may destabilize your system. Recommend saving work and restarting this application.`;
        case 'CpuSpike':
            return meta.culprit 
                ? `Process "${meta.culprit}" is dominating CPU resources. This may cause system lag. If this is not an expected heavy task (like rendering), consider terminating it.`
                : "System CPU is critically high. Unable to pinpoint a single cause. Check for background updates or virus scans.";
        case 'NetworkSurge':
            return "Unusual network activity detected. If you are not downloading large files, check for background updates or potential unauthorized data exfiltration.";
        case 'SystemSaturation':
            return `Critical shortage of ${meta.resource}. System may become unresponsive. Close unused applications immediately to free up resources.`;
        default:
            return "Monitor the situation. If performance degrades, check the Process table for outliers.";
    }
};

const handleAction = async (action: string, anomaly: AnomalyEvent) => {
    const meta = anomaly.metadata as any;
    if (action === 'kill' && meta.pid) {
        if (confirm(`Force kill process "${meta.process_name || meta.culprit}" (PID ${meta.pid})?`)) {
            try {
                await killProcess(meta.pid);
            } catch (e) {
                alert(e);
            }
        }
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
            <div v-if="expandedIndex === i" class="px-2 pb-2 pt-0.5 text-[10px] text-gray-300 border-t border-[#222] bg-[#141414] cursor-default" @click.stop>
                <div class="grid grid-cols-[max-content_1fr] gap-x-2 gap-y-1 mt-1">
                    <template v-if="event.type === 'CpuSpike'">
                        <span class="text-gray-500">Global Load:</span>
                        <span class="text-red-300">{{ event.metadata.value.toFixed(1) }}%</span>
                        <span class="text-gray-500">Culprit:</span>
                        <span v-if="event.metadata.culprit" class="text-yellow-300 font-bold">{{ event.metadata.culprit }} (PID: {{ event.metadata.culprit_pid }})</span>
                        <span v-else class="text-gray-600 italic">Unknown</span>
                        <span class="text-gray-500">Confidence:</span>
                        <span>{{ (event.metadata.confidence * 100).toFixed(0) }}%</span>
                    </template>

                    <template v-else-if="event.type === 'MemoryLeak'">
                        <span class="text-gray-500">Process:</span>
                        <span class="text-yellow-300 font-bold">{{ event.metadata.process_name }} ({{ event.metadata.pid }})</span>
                        <span class="text-gray-500">Leak Rate:</span>
                        <span class="text-red-300 font-bold">+{{ event.metadata.rate_mb_per_sec.toFixed(2) }} MB/s</span>
                        <span class="text-gray-500">Impact:</span>
                        <span class="text-gray-400">~{{ (event.metadata.rate_mb_per_sec * 60).toFixed(0) }} MB/min</span>
                        <span class="text-gray-500">Confidence:</span>
                        <span>{{ (event.metadata.confidence * 100).toFixed(0) }}%</span>
                    </template>

                    <template v-else-if="event.type === 'NetworkSurge'">
                        <span class="text-gray-500">Speed:</span>
                        <span class="text-yellow-300">{{ formatBytes(event.metadata.current) }}/s</span>
                        <span class="text-gray-500">Baseline:</span>
                        <span>{{ formatBytes(event.metadata.avg) }}/s</span>
                        <span class="text-gray-500">Deviation:</span>
                        <span class="text-red-300">{{ (event.metadata.current / Math.max(1, event.metadata.avg)).toFixed(1) }}x normal</span>
                    </template>

                    <template v-else-if="event.type === 'SystemSaturation'">
                        <span class="text-gray-500">Resource:</span>
                        <span class="text-red-400 font-bold">{{ event.metadata.resource }}</span>
                        <span class="text-gray-500">Usage:</span>
                        <span class="text-red-300">{{ event.metadata.usage_percent.toFixed(1) }}%</span>
                    </template>

                    <template v-else>
                        <div class="col-span-2 text-gray-500 break-all">
                            {{ JSON.stringify(event.metadata) }}
                        </div>
                    </template>
                </div>

                <!-- Analysis & Recommendation Block -->
                <div class="mt-2 bg-[#1a1a1a] border border-[#333] p-2 rounded">
                    <div class="text-gray-500 mb-1 flex items-center gap-1">
                        <span class="text-blue-400 font-bold">ⓘ ANALYSIS:</span>
                    </div>
                    <p class="text-gray-300 leading-relaxed">
                        {{ getRecommendation(event) }}
                    </p>
                </div>

                <!-- Action Buttons -->
                <div v-if="(event.type === 'MemoryLeak' || (event.type === 'CpuSpike' && event.metadata.culprit_pid))" class="mt-2 flex justify-end gap-2">
                    <button 
                        @click="handleAction('kill', event)"
                        class="px-3 py-1 bg-red-900/20 text-red-400 hover:bg-red-900/40 border border-red-900/50 rounded text-xs transition-colors flex items-center gap-1"
                    >
                        <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
                        Force Kill Process
                    </button>
                </div>
            </div>
        </div>
    </div>
  </div>
</template>