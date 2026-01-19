<script setup lang="ts">
import { computed, ref } from 'vue';
import type { ProcessInfo } from '../../composables/useSystemMetrics';
import { useSystemMetrics } from '../../composables/useSystemMetrics';

interface Props {
  processes: ProcessInfo[];
}

const props = defineProps<Props>();
const { killProcess } = useSystemMetrics();

// State
const searchQuery = ref('');
const sortBy = ref<'cpu' | 'memory' | 'name' | 'pid'>('cpu');
const sortDesc = ref(true);
const expandedGroups = ref<Set<string>>(new Set());
const watchedName = ref<string | null>(null); // For "Watch" mode
const detailProcess = ref<ProcessInfo | null>(null); // For "Info" modal

// Helpers
const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

interface ProcessGroup {
    isGroup: boolean;
    name: string;
    pid: number; // Main PID or representative
    cpu_usage: number; // Sum
    memory: number; // Sum
    user: string;
    count: number;
    children: ProcessInfo[];
    expanded: boolean;
}

// Grouping & Filtering Logic
const groupedProcesses = computed(() => {
    // 1. Filter raw list first
    let raw = props.processes;
    
    if (watchedName.value) {
        raw = raw.filter(p => p.name === watchedName.value);
    }
    
    if (searchQuery.value) {
        const q = searchQuery.value.toLowerCase();
        raw = raw.filter(p => 
            p.name.toLowerCase().includes(q) || 
            p.pid.toString().includes(q) ||
            (p.user && p.user.toLowerCase().includes(q))
        );
    }

    // 2. Group by Name
    const groups: Record<string, ProcessGroup> = {};
    const singles: ProcessGroup[] = [];

    for (const p of raw) {
        if (!groups[p.name]) {
            groups[p.name] = {
                isGroup: true,
                name: p.name,
                pid: p.pid, // Use first found PID as ID
                cpu_usage: 0,
                memory: 0,
                user: p.user || '',
                count: 0,
                children: [],
                expanded: expandedGroups.value.has(p.name)
            };
        }
        groups[p.name].cpu_usage += p.cpu_usage;
        groups[p.name].memory += p.memory;
        groups[p.name].count++;
        groups[p.name].children.push(p);
    }

    // Convert map to array, flattening singles if count is 1
    const result: ProcessGroup[] = [];
    Object.values(groups).forEach(g => {
        if (g.count === 1) {
            // Treat as single process, essentially a group of 1 but we render differently
            const child = g.children[0];
            result.push({
                isGroup: false,
                name: child.name,
                pid: child.pid,
                cpu_usage: child.cpu_usage,
                memory: child.memory,
                user: child.user || '',
                count: 1,
                children: [],
                expanded: false
            });
        } else {
            result.push(g);
        }
    });

    // 3. Sort
    return result.sort((a, b) => {
        let valA: any = a.cpu_usage;
        let valB: any = b.cpu_usage;

        if (sortBy.value === 'memory') {
            valA = a.memory;
            valB = b.memory;
        } else if (sortBy.value === 'name') {
            valA = a.name.toLowerCase();
            valB = b.name.toLowerCase();
        } else if (sortBy.value === 'pid') {
            valA = a.pid;
            valB = b.pid;
        }

        if (valA < valB) return sortDesc.value ? 1 : -1;
        if (valA > valB) return sortDesc.value ? -1 : 1;
        return 0;
    });
});

// Actions
const toggleGroup = (groupName: string) => {
    if (expandedGroups.value.has(groupName)) {
        expandedGroups.value.delete(groupName);
    } else {
        expandedGroups.value.add(groupName);
    }
};

const handleHeaderClick = (key: 'cpu' | 'memory' | 'name' | 'pid') => {
    if (sortBy.value === key) {
        sortDesc.value = !sortDesc.value;
    } else {
        sortBy.value = key;
        sortDesc.value = true;
    }
};

const handleKill = async (pid: number, name: string, isGroup: boolean) => {
    const msg = isGroup 
        ? `Kill representative process of group "${name}" (PID ${pid})?` 
        : `Force kill process "${name}" (PID ${pid})?`;
        
    if (!confirm(msg)) return;
    
    try {
        await killProcess(pid);
    } catch (e) {
        alert('Failed to kill process');
    }
};

const handleWatch = (name: string) => {
    if (watchedName.value === name) {
        watchedName.value = null; // Toggle off
    } else {
        watchedName.value = name;
    }
};

const showDetails = (proc: ProcessInfo) => {
    detailProcess.value = proc;
};

// --- Icons (SVG) ---
const IconKill = 'M3 6v18h18v-18h-18zm5 14c0 .552-.448 1-1 1s-1-.448-1-1v-10c0-.552.448-1 1-1s1 .448 1 1v10zm5 0c0 .552-.448 1-1 1s-1-.448-1-1v-10c0-.552.448-1 1-1s1 .448 1 1v10zm5 0c0 .552-.448 1-1 1s-1-.448-1-1v-10c0-.552.448-1 1-1s1 .448 1 1v10zm4-18v2h-20v-2h5.711c.9 0 1.631-1.099 1.631-2h5.316c0 .901.73 2 1.631 2h5.711z';
const IconInfo = 'M12 2c5.514 0 10 4.486 10 10s-4.486 10-10 10-10-4.486-10-10 4.486-10 10-10zm0-2c-6.627 0-12 5.373-12 12s5.373 12 12 12 12-5.373 12-12-5.373-12-12-12zm-.001 5.75c.69 0 1.251.56 1.251 1.25s-.561 1.25-1.251 1.25-1.249-.56-1.249-1.25.559-1.25 1.249-1.25zm2.001 12.25h-4v-1c.484-.179 1-.201 1-.735v-4.467c0-.534-.516-.618-1-.797v-1h3v6.265c0 .535.517.558 1 1v1z';
const IconEye = 'M15 12c0 1.654-1.346 3-3 3s-3-1.346-3-3 1.346-3 3-3 3 1.346 3 3zm9-.449s-4.252 8.449-11.985 8.449c-7.18 0-12.015-8.449-12.015-8.449s4.446-7.551 12.015-7.551c7.694 0 11.985 7.551 11.985 7.551zm-7 .449c0-2.757-2.243-5-5-5s-5 2.243-5 5 2.243 5 5 5 5-2.243 5-5z';
const IconChevron = 'M10.707 17.707 16.414 12l-5.707-5.707-1.414 1.414 4.293 4.293-4.293 4.293z';

</script>

<template>
  <div class="h-full flex flex-col bg-[#111] border border-[#333] rounded-sm overflow-hidden text-xs relative">
    
    <!-- Toolbar -->
    <div class="px-2 py-1 bg-[#1a1a1a] border-b border-[#333] flex justify-between items-center gap-2 h-[30px]">
        <div class="flex items-center gap-2">
            <span class="text-[10px] uppercase tracking-wider text-gray-500 font-mono">Processes</span>
            <span v-if="watchedName" class="text-[10px] bg-blue-900/30 text-blue-400 px-2 py-0.5 rounded border border-blue-800/50 flex items-center gap-1">
                Watching: {{ watchedName }}
                <button @click="watchedName = null" class="hover:text-white ml-1">×</button>
            </span>
        </div>
        
        <div class="flex-1 max-w-[200px] relative">
            <input 
                v-model="searchQuery" 
                type="text" 
                placeholder="Filter processes..." 
                class="w-full bg-[#0a0a0a] border border-[#333] rounded px-2 py-0.5 text-gray-300 focus:border-blue-500 outline-none text-[10px]"
            />
        </div>
    </div>

    <!-- Table Header -->
    <div class="grid grid-cols-[80px_50px_1fr_60px_70px_80px] bg-[#161616] border-b border-[#333] text-[10px] font-mono text-gray-500 sticky top-0 z-10 select-none">
        <div class="px-2 py-1">Actions</div>
        <div @click="handleHeaderClick('pid')" class="px-2 py-1 cursor-pointer hover:text-gray-300 border-l border-[#222]">PID</div>
        <div @click="handleHeaderClick('name')" class="px-2 py-1 cursor-pointer hover:text-gray-300 border-l border-[#222]">Name</div>
        <div @click="handleHeaderClick('cpu')" class="px-2 py-1 cursor-pointer hover:text-gray-300 text-right border-l border-[#222]">CPU%</div>
        <div @click="handleHeaderClick('memory')" class="px-2 py-1 cursor-pointer hover:text-gray-300 text-right border-l border-[#222]">Mem</div>
        <div class="px-2 py-1 text-right border-l border-[#222]">User</div>
    </div>

    <!-- Table Body -->
    <div class="flex-1 overflow-y-auto font-mono scrollbar-thin">
        <template v-for="group in groupedProcesses" :key="group.name">
            
            <!-- Group/Main Row -->
            <div 
                class="grid grid-cols-[80px_50px_1fr_60px_70px_80px] group hover:bg-[#1c1c1c] border-b border-[#222]/50 text-[10px] items-center transition-colors"
                :class="watchedName === group.name ? 'bg-blue-900/10' : ''"
            >
                <!-- Actions -->
                <div class="px-2 flex items-center gap-1 opacity-60 group-hover:opacity-100 transition-opacity">
                    <button @click="handleKill(group.pid, group.name, group.isGroup)" class="p-1 hover:text-red-500 text-gray-500" title="Kill">
                        <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path :d="IconKill"/></svg>
                    </button>
                    <!-- Info only for single child or representative -->
                    <button v-if="!group.isGroup || group.children.length > 0" @click="showDetails(group.children[0])" class="p-1 hover:text-blue-400 text-gray-500" title="Details">
                        <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path :d="IconInfo"/></svg>
                    </button>
                    <button @click="handleWatch(group.name)" class="p-1 hover:text-yellow-400 text-gray-500" :class="watchedName === group.name ? 'text-yellow-400' : ''" title="Watch/Filter">
                        <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path :d="IconEye"/></svg>
                    </button>
                </div>

                <!-- PID -->
                <div class="px-2 py-1 text-gray-500 truncate border-l border-[#222]/0 group-hover:border-[#333]">
                    <span v-if="!group.isGroup">{{ group.pid }}</span>
                    <span v-else class="text-[9px] bg-[#222] px-1 rounded">{{ group.count }}</span>
                </div>

                <!-- Name (Expandable) -->
                <div class="px-2 py-1 text-gray-300 truncate font-bold flex items-center gap-1 border-l border-[#222]/0 group-hover:border-[#333]">
                    <button 
                        v-if="group.isGroup" 
                        @click="toggleGroup(group.name)" 
                        class="w-4 h-4 flex items-center justify-center hover:bg-[#333] rounded mr-1"
                    >
                        <svg class="w-2.5 h-2.5 fill-current text-gray-500 transition-transform" :class="group.expanded ? 'rotate-90' : ''" viewBox="0 0 24 24"><path :d="IconChevron"/></svg>
                    </button>
                    {{ group.name }}
                </div>

                <!-- CPU -->
                <div class="px-2 py-1 text-right border-l border-[#222]/0 group-hover:border-[#333]" :class="group.cpu_usage > 50 ? 'text-red-400' : (group.cpu_usage > 10 ? 'text-yellow-400' : 'text-gray-400')">
                    {{ group.cpu_usage.toFixed(1) }}
                </div>

                <!-- Mem -->
                <div class="px-2 py-1 text-right text-gray-400 border-l border-[#222]/0 group-hover:border-[#333]">
                    {{ formatBytes(group.memory) }}
                </div>

                <!-- User -->
                <div class="px-2 py-1 text-right text-gray-600 truncate border-l border-[#222]/0 group-hover:border-[#333]">
                    {{ group.user }}
                </div>
            </div>

            <!-- Children Rows (if expanded) -->
            <template v-if="group.isGroup && group.expanded">
                <div 
                    v-for="child in group.children" 
                    :key="child.pid"
                    class="grid grid-cols-[80px_50px_1fr_60px_70px_80px] bg-[#141414] hover:bg-[#1a1a1a] border-b border-[#222]/30 text-[9px] items-center"
                >
                    <!-- Child Actions -->
                    <div class="px-2 flex items-center gap-1 opacity-30 hover:opacity-100 justify-end pr-4">
                        <button @click="handleKill(child.pid, child.name, false)" class="p-0.5 hover:text-red-500 text-gray-500" title="Kill Instance">
                            <svg class="w-2.5 h-2.5 fill-current" viewBox="0 0 24 24"><path :d="IconKill"/></svg>
                        </button>
                        <button @click="showDetails(child)" class="p-0.5 hover:text-blue-400 text-gray-500" title="Details">
                            <svg class="w-2.5 h-2.5 fill-current" viewBox="0 0 24 24"><path :d="IconInfo"/></svg>
                        </button>
                    </div>

                    <div class="px-2 py-0.5 text-gray-600 pl-4 border-l border-[#222]/0">{{ child.pid }}</div>
                    <div class="px-2 py-0.5 text-gray-500 pl-8 border-l border-[#222]/0 flex items-center">
                        <div class="w-2 h-[1px] bg-gray-700 mr-2"></div> {{ child.name }}
                    </div>
                    <div class="px-2 py-0.5 text-right text-gray-600 border-l border-[#222]/0">{{ child.cpu_usage.toFixed(1) }}</div>
                    <div class="px-2 py-0.5 text-right text-gray-600 border-l border-[#222]/0">{{ formatBytes(child.memory) }}</div>
                    <div class="px-2 py-0.5 text-right text-gray-700 truncate border-l border-[#222]/0">{{ child.user }}</div>
                </div>
            </template>

        </template>
        
        <div v-if="groupedProcesses.length === 0" class="p-4 text-center text-gray-600 italic">
            No processes found
        </div>
    </div>

    <!-- Details Modal -->
    <div v-if="detailProcess" class="absolute inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
        <div class="bg-[#1a1a1a] border border-[#333] rounded-sm shadow-2xl w-full max-w-sm flex flex-col">
            <div class="px-3 py-2 border-b border-[#333] bg-[#222] flex justify-between items-center">
                <span class="font-bold text-gray-200">Process Details</span>
                <button @click="detailProcess = null" class="text-gray-500 hover:text-white">✕</button>
            </div>
            <div class="p-4 space-y-2 text-sm font-mono">
                <div class="grid grid-cols-[80px_1fr] gap-2">
                    <span class="text-gray-500">Name:</span> <span class="text-green-400 font-bold">{{ detailProcess.name }}</span>
                    <span class="text-gray-500">PID:</span> <span class="text-gray-300">{{ detailProcess.pid }}</span>
                    <span class="text-gray-500">User:</span> <span class="text-gray-300">{{ detailProcess.user || 'System' }}</span>
                    <span class="text-gray-500">Status:</span> <span class="text-gray-300">{{ detailProcess.status }}</span>
                    <span class="text-gray-500">CPU:</span> <span class="text-yellow-400">{{ detailProcess.cpu_usage.toFixed(2) }}%</span>
                    <span class="text-gray-500">Memory:</span> <span class="text-blue-400">{{ formatBytes(detailProcess.memory) }}</span>
                </div>
            </div>
            <div class="px-3 py-2 border-t border-[#333] bg-[#161616] flex justify-end">
                <button 
                    @click="handleKill(detailProcess.pid, detailProcess.name, false); detailProcess = null;"
                    class="px-3 py-1 bg-red-900/20 text-red-400 hover:bg-red-900/40 border border-red-900/50 rounded text-xs"
                >
                    Force Kill
                </button>
            </div>
        </div>
    </div>

  </div>
</template>

<style scoped>
.scrollbar-thin::-webkit-scrollbar {
  width: 6px;
}
.scrollbar-thin::-webkit-scrollbar-track {
  background: #111;
}
.scrollbar-thin::-webkit-scrollbar-thumb {
  background: #333;
  border-radius: 3px;
}
.scrollbar-thin::-webkit-scrollbar-thumb:hover {
  background: #444;
}
</style>