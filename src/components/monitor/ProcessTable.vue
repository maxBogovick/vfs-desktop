<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { ProcessInfo } from '../../composables/useSystemMetrics';
import { useSystemMetrics } from '../../composables/useSystemMetrics';

interface Props {
  processes: ProcessInfo[];
}

const props = defineProps<Props>();
const { killProcess, suspendProcess: suspendProcessBackend, resumeProcess: resumeProcessBackend } = useSystemMetrics();

// Wrappers to handle UI feedback
const suspendProcess = async (pid: number) => {
    try {
        await suspendProcessBackend(pid);
    } catch(e) {
        alert(e); // Show backend error message
    }
}

const resumeProcess = async (pid: number) => {
    try {
        await resumeProcessBackend(pid);
    } catch(e) {
        alert(e);
    }
}

// State
const searchQuery = ref('');
const sortBy = ref<'cpu' | 'memory' | 'name' | 'pid' | 'disk'>('cpu');
const sortDesc = ref(true);
const expandedGroups = ref<Set<string>>(new Set());
const watchedName = ref<string | null>(null); 
const detailProcess = ref<ProcessInfo | null>(null);

// Context Menu State
const contextMenuVisible = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextProcess = ref<ProcessInfo | null>(null);

const handleContextMenu = (e: MouseEvent, proc: ProcessInfo) => {
    e.preventDefault();
    contextMenuX.value = e.clientX;
    contextMenuY.value = e.clientY;
    contextProcess.value = proc;
    contextMenuVisible.value = true;
};

const closeContextMenu = () => {
    contextMenuVisible.value = false;
    contextProcess.value = null;
};

// Global click listener to close context menu
const onGlobalClick = () => {
    if (contextMenuVisible.value) closeContextMenu();
};

onMounted(() => {
    window.addEventListener('click', onGlobalClick);
});

onUnmounted(() => {
    window.removeEventListener('click', onGlobalClick);
});

// Actions
const googleProcess = async (name: string) => {
    const query = encodeURIComponent(`what is ${name} process macos`);
    await openUrl(`https://www.google.com/search?q=${query}`);
};

const searchOnline = async () => {
    if (!contextProcess.value) return;
    await googleProcess(contextProcess.value.name);
    closeContextMenu();
};

const revealInFinder = async () => {
    if (!contextProcess.value || !contextProcess.value.exe_path) {
        alert('Executable path not available for this process.');
        return;
    }
    try {
        await invoke('reveal_in_finder', { path: contextProcess.value.exe_path });
    } catch (e) {
        alert('Failed to reveal in Finder: ' + e);
    }
    closeContextMenu();
};

const copyDetails = async () => {
    if (!contextProcess.value) return;
    const details = {
        pid: contextProcess.value.pid,
        name: contextProcess.value.name,
        user: contextProcess.value.user,
        path: contextProcess.value.exe_path || 'N/A',
        cpu: contextProcess.value.cpu_usage.toFixed(1) + '%',
        memory: formatBytes(contextProcess.value.memory)
    };
    try {
        await navigator.clipboard.writeText(JSON.stringify(details, null, 2));
    } catch (e) {
        console.error('Copy failed', e);
    }
    closeContextMenu();
};

// Helpers
const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

const formatDiskSpeed = (read: number, write: number) => {
    if (read === 0 && write === 0) return '0';
    const r = read > 0 ? `R:${formatBytes(read)}/s` : '';
    const w = write > 0 ? `W:${formatBytes(write)}/s` : '';
    return [r, w].filter(x => x).join(' ');
};

interface ProcessGroup {
    isGroup: boolean;
    name: string;
    pid: number;
    cpu_usage: number;
    memory: number;
    disk_read: number;
    disk_write: number;
    user: string;
    count: number;
    children: ProcessInfo[];
    expanded: boolean;
    exe_path: string | null;
}

const groupedProcesses = computed(() => {
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

    const groups: Record<string, ProcessGroup> = {};

    for (const p of raw) {
        if (!groups[p.name]) {
            groups[p.name] = {
                isGroup: true,
                name: p.name,
                pid: p.pid, 
                cpu_usage: 0,
                memory: 0,
                disk_read: 0,
                disk_write: 0,
                user: p.user || '',
                count: 0,
                children: [],
                expanded: expandedGroups.value.has(p.name),
                exe_path: p.exe_path
            };
        }
        groups[p.name].cpu_usage += p.cpu_usage;
        groups[p.name].memory += p.memory;
        groups[p.name].disk_read += p.disk_read_speed;
        groups[p.name].disk_write += p.disk_write_speed;
        groups[p.name].count++;
        groups[p.name].children.push(p);
    }

    const result: ProcessGroup[] = [];
    Object.values(groups).forEach(g => {
        if (g.count === 1) {
            const child = g.children[0];
            result.push({
                isGroup: false,
                name: child.name,
                pid: child.pid,
                cpu_usage: child.cpu_usage,
                memory: child.memory,
                disk_read: child.disk_read_speed,
                disk_write: child.disk_write_speed,
                user: child.user || '',
                count: 1,
                children: [child],
                expanded: false,
                exe_path: child.exe_path
            });
        } else {
            result.push(g);
        }
    });

    return result.sort((a, b) => {
        let valA: number | string = 0;
        let valB: number | string = 0;

        switch (sortBy.value) {
            case 'cpu':
                valA = a.cpu_usage; valB = b.cpu_usage; break;
            case 'memory':
                valA = a.memory; valB = b.memory; break;
            case 'disk':
                valA = a.disk_read + a.disk_write; valB = b.disk_read + b.disk_write; break;
            case 'name':
                valA = a.name.toLowerCase(); valB = b.name.toLowerCase(); break;
            case 'pid':
                valA = a.pid; valB = b.pid; break;
        }

        if (valA < valB) return sortDesc.value ? 1 : -1;
        if (valA > valB) return sortDesc.value ? -1 : 1;
        return 0;
    });
});

const toggleGroup = (groupName: string) => {
    if (expandedGroups.value.has(groupName)) {
        expandedGroups.value.delete(groupName);
    } else {
        expandedGroups.value.add(groupName);
    }
};

const handleHeaderClick = (key: any) => {
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
        alert(e); 
    }
};

const handleWatch = (name: string) => {
    if (watchedName.value === name) {
        watchedName.value = null; 
    } else {
        watchedName.value = name;
    }
};

const showDetails = (proc: ProcessInfo) => {
    detailProcess.value = proc;
};

const isSuspended = (status: string) => {
    return status.includes('Stop') || status.includes('T');
};

const IconKill = 'M3 6v18h18v-18h-18zm5 14c0 .552-.448 1-1 1s-1-.448-1-1v-10c0-.552.448-1 1-1s1 .448 1 1v10zm5 0c0 .552-.448 1-1 1s-1-.448-1-1v-10c0-.552.448-1 1-1s1 .448 1 1v10zm5 0c0 .552-.448 1-1 1s-1-.448-1-1v-10c0-.552.448-1 1-1s1 .448 1 1v10zm4-18v2h-20v-2h5.711c.9 0 1.631-1.099 1.631-2h5.316c0 .901.73 2 1.631 2h5.711z';
const IconInfo = 'M12 2c5.514 0 10 4.486 10 10s-4.486 10-10 10-10-4.486-10-10 4.486-10 10-10zm0-2c-6.627 0-12 5.373-12 12s5.373 12 12 12 12-5.373 12-12-5.373-12-12-12zm-.001 5.75c.69 0 1.251.56 1.251 1.25s-.561 1.25-1.251 1.25-1.249-.56-1.249-1.25.559-1.25 1.249-1.25zm2.001 12.25h-4v-1c.484-.179 1-.201 1-.735v-4.467c0-.534-.516-.618-1-.797v-1h3v6.265c0 .535.517.558 1 1v1z';
const IconEye = 'M15 12c0 1.654-1.346 3-3 3s-3-1.346-3-3 1.346-3 3-3 3 1.346 3 3zm9-.449s-4.252 8.449-11.985 8.449c-7.18 0-12.015-8.449-12.015-8.449s4.446-7.551 12.015-7.551c7.694 0 11.985 7.551 11.985 7.551zm-7 .449c0-2.757-2.243-5-5-5s-5 2.243-5 5 2.243 5 5 5 5-2.243 5-5z';
const IconChevron = 'M10.707 17.707 16.414 12l-5.707-5.707-1.414 1.414 4.293 4.293-4.293 4.293z';
const IconPause = 'M8 7h3v10H8zm5 0h3v10h-3z';
const IconPlay = 'M7 6v12l10-6z';
const IconSearch = 'M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z';
const IconFolder = 'M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z';
const IconCopy = 'M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z';

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
    <div class="grid grid-cols-[100px_50px_1fr_60px_70px_90px_80px] bg-[#161616] border-b border-[#333] text-[10px] font-mono text-gray-500 sticky top-0 z-10 select-none">
        <div class="px-2 py-1">Actions</div>
        <div @click="handleHeaderClick('pid')" class="px-2 py-1 cursor-pointer hover:text-gray-300 border-l border-[#222]">PID</div>
        <div @click="handleHeaderClick('name')" class="px-2 py-1 cursor-pointer hover:text-gray-300 border-l border-[#222]">Name</div>
        <div @click="handleHeaderClick('cpu')" class="px-2 py-1 cursor-pointer hover:text-gray-300 text-right border-l border-[#222]">CPU%</div>
        <div @click="handleHeaderClick('memory')" class="px-2 py-1 cursor-pointer hover:text-gray-300 text-right border-l border-[#222]">Mem</div>
        <div @click="handleHeaderClick('disk')" class="px-2 py-1 cursor-pointer hover:text-gray-300 text-right border-l border-[#222]">Disk I/O</div>
        <div class="px-2 py-1 text-right border-l border-[#222]">User</div>
    </div>

    <!-- Table Body -->
    <div class="flex-1 overflow-y-auto font-mono scrollbar-thin">
        <template v-for="group in groupedProcesses" :key="group.name">
            
            <!-- Group/Main Row -->
            <div 
                class="grid grid-cols-[100px_50px_1fr_60px_70px_90px_80px] group hover:bg-[#1c1c1c] border-b border-[#222]/50 text-[10px] items-center transition-colors cursor-context-menu"
                :class="watchedName === group.name ? 'bg-blue-900/10' : ''"
                @contextmenu.prevent="(e) => group.isGroup ? null : handleContextMenu(e, group.children[0])"
            >
                <!-- Actions -->
                <div class="px-2 flex items-center gap-1 opacity-60 group-hover:opacity-100 transition-opacity">
                    <button @click.stop="handleKill(group.pid, group.name, group.isGroup)" class="p-1 hover:text-red-500 text-gray-500" title="Kill">
                        <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path :d="IconKill"/></svg>
                    </button>
                    <!-- Pause/Resume (only if not group or group has 1 child) -->
                    <template v-if="!group.isGroup">
                        <button v-if="!isSuspended(group.children.length > 0 ? group.children[0].status : '')" @click.stop="suspendProcess(group.pid)" class="p-1 hover:text-yellow-400 text-gray-500" title="Suspend (Freeze)">
                            <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path :d="IconPause"/></svg>
                        </button>
                        <button v-else @click.stop="resumeProcess(group.pid)" class="p-1 hover:text-green-400 text-yellow-500" title="Resume">
                            <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path :d="IconPlay"/></svg>
                        </button>
                    </template>
                    
                    <button v-if="!group.isGroup || group.children.length > 0" @click.stop="showDetails(group.children[0])" class="p-1 hover:text-blue-400 text-gray-500" title="Details">
                        <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path :d="IconInfo"/></svg>
                    </button>
                    <button @click.stop="handleWatch(group.name)" class="p-1 hover:text-yellow-400 text-gray-500" :class="watchedName === group.name ? 'text-yellow-400' : ''" title="Watch/Filter">
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
                        @click.stop="toggleGroup(group.name)" 
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

                <!-- Disk -->
                <div class="px-2 py-1 text-right text-blue-300 border-l border-[#222]/0 group-hover:border-[#333] text-[9px] tracking-tighter">
                    {{ formatDiskSpeed(group.disk_read, group.disk_write) }}
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
                    class="grid grid-cols-[100px_50px_1fr_60px_70px_90px_80px] bg-[#141414] hover:bg-[#1a1a1a] border-b border-[#222]/30 text-[9px] items-center cursor-context-menu"
                    @contextmenu.prevent="(e) => handleContextMenu(e, child)"
                >
                    <!-- Child Actions -->
                    <div class="px-2 flex items-center gap-1 opacity-30 hover:opacity-100 justify-end pr-4">
                        <button @click.stop="handleKill(child.pid, child.name, false)" class="p-0.5 hover:text-red-500 text-gray-500" title="Kill Instance">
                            <svg class="w-2.5 h-2.5 fill-current" viewBox="0 0 24 24"><path :d="IconKill"/></svg>
                        </button>
                        <button v-if="!isSuspended(child.status)" @click.stop="suspendProcess(child.pid)" class="p-0.5 hover:text-yellow-400 text-gray-500" title="Suspend">
                            <svg class="w-2.5 h-2.5 fill-current" viewBox="0 0 24 24"><path :d="IconPause"/></svg>
                        </button>
                        <button v-else @click.stop="resumeProcess(child.pid)" class="p-0.5 hover:text-green-400 text-yellow-500" title="Resume">
                            <svg class="w-2.5 h-2.5 fill-current" viewBox="0 0 24 24"><path :d="IconPlay"/></svg>
                        </button>
                        <button @click.stop="showDetails(child)" class="p-0.5 hover:text-blue-400 text-gray-500" title="Details">
                            <svg class="w-2.5 h-2.5 fill-current" viewBox="0 0 24 24"><path :d="IconInfo"/></svg>
                        </button>
                    </div>

                    <div class="px-2 py-0.5 text-gray-600 pl-4 border-l border-[#222]/0">{{ child.pid }}</div>
                    <div class="px-2 py-0.5 text-gray-500 pl-8 border-l border-[#222]/0 flex items-center">
                        <div class="w-2 h-[1px] bg-gray-700 mr-2"></div> {{ child.name }}
                    </div>
                    <div class="px-2 py-0.5 text-right text-gray-600 border-l border-[#222]/0">{{ child.cpu_usage.toFixed(1) }}</div>
                    <div class="px-2 py-0.5 text-right text-gray-600 border-l border-[#222]/0">{{ formatBytes(child.memory) }}</div>
                    <div class="px-2 py-0.5 text-right text-blue-900 border-l border-[#222]/0">{{ formatDiskSpeed(child.disk_read_speed, child.disk_write_speed) }}</div>
                    <div class="px-2 py-0.5 text-right text-gray-700 truncate border-l border-[#222]/0">{{ child.user }}</div>
                </div>
            </template>

        </template>
        
        <div v-if="groupedProcesses.length === 0" class="p-4 text-center text-gray-600 italic">
            No processes found
        </div>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
        <div 
            v-if="contextMenuVisible && contextProcess" 
            class="fixed z-[99999] bg-[#1a1a1a] border border-[#333] shadow-xl rounded py-1 w-48 text-gray-300 text-xs font-mono"
            :style="{ top: `${contextMenuY}px`, left: `${contextMenuX}px` }"
        >
            <div class="px-3 py-1 text-[10px] text-gray-500 border-b border-[#222] mb-1 truncate">
                {{ contextProcess.name }} ({{ contextProcess.pid }})
            </div>
            <button @click="searchOnline" class="w-full text-left px-3 py-1.5 hover:bg-[#2c2c2c] flex items-center gap-2">
                <svg class="w-3 h-3 fill-current text-blue-400" viewBox="0 0 24 24"><path :d="IconSearch"/></svg>
                What is this?
            </button>
            <button @click="revealInFinder" class="w-full text-left px-3 py-1.5 hover:bg-[#2c2c2c] flex items-center gap-2">
                <svg class="w-3 h-3 fill-current text-yellow-500" viewBox="0 0 24 24"><path :d="IconFolder"/></svg>
                Reveal Executable
            </button>
            <button @click="copyDetails" class="w-full text-left px-3 py-1.5 hover:bg-[#2c2c2c] flex items-center gap-2">
                <svg class="w-3 h-3 fill-current text-gray-400" viewBox="0 0 24 24"><path :d="IconCopy"/></svg>
                Copy Details
            </button>
            <div class="h-[1px] bg-[#222] my-1"></div>
            <button @click="handleKill(contextProcess.pid, contextProcess.name, false); closeContextMenu();" class="w-full text-left px-3 py-1.5 hover:bg-red-900/30 text-red-400 flex items-center gap-2">
                <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path :d="IconKill"/></svg>
                Force Kill
            </button>
        </div>
    </Teleport>

    <!-- Details Modal -->
    <Teleport to="body">
        <div v-if="detailProcess" class="fixed inset-0 z-[9999] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4" @click.self="detailProcess = null">
            <div class="bg-[#1a1a1a] border border-[#333] rounded-sm shadow-2xl w-full max-w-sm flex flex-col">
                <div class="px-3 py-2 border-b border-[#333] bg-[#222] flex justify-between items-center">
                    <span class="font-bold text-gray-200">Process Details</span>
                    <button @click="detailProcess = null" class="text-gray-500 hover:text-white">✕</button>
                </div>
                <div class="p-4 space-y-2 text-sm font-mono text-gray-300">
                    <div class="grid grid-cols-[80px_1fr] gap-2">
                        <span class="text-gray-500">Name:</span> <span class="text-green-400 font-bold select-text">{{ detailProcess.name }}</span>
                        <span class="text-gray-500">PID:</span> <span class="text-gray-300 select-text">{{ detailProcess.pid }}</span>
                        <span class="text-gray-500">User:</span> <span class="text-gray-300 select-text">{{ detailProcess.user || 'System' }}</span>
                        <span class="text-gray-500">Path:</span> <span class="text-gray-400 text-[10px] break-all select-text">{{ detailProcess.exe_path || 'N/A' }}</span>
                        <span class="text-gray-500">Status:</span> <span class="text-gray-300">{{ detailProcess.status }}</span>
                        <span class="text-gray-500">CPU:</span> <span class="text-yellow-400">{{ detailProcess.cpu_usage.toFixed(2) }}%</span>
                        <span class="text-gray-500">Memory:</span> <span class="text-blue-400">{{ formatBytes(detailProcess.memory) }}</span>
                        <span class="text-gray-500">Disk R/W:</span> <span class="text-purple-400">{{ formatDiskSpeed(detailProcess.disk_read_speed, detailProcess.disk_write_speed) }}</span>
                    </div>
                </div>
                <div class="px-3 py-2 border-t border-[#333] bg-[#161616] flex justify-between gap-2">
                    <button 
                         @click="googleProcess(detailProcess.name)"
                         class="px-3 py-1 bg-blue-900/20 text-blue-400 hover:bg-blue-900/40 border border-blue-900/50 rounded text-xs"
                    >
                        ? Google It
                    </button>
                    <div class="flex gap-2">
                        <button 
                            v-if="!isSuspended(detailProcess.status)"
                            @click="suspendProcess(detailProcess.pid)"
                            class="px-3 py-1 bg-yellow-900/20 text-yellow-400 hover:bg-yellow-900/40 border border-yellow-900/50 rounded text-xs"
                        >
                            Suspend
                        </button>
                        <button 
                            v-else
                            @click="resumeProcess(detailProcess.pid)"
                            class="px-3 py-1 bg-green-900/20 text-green-400 hover:bg-green-900/40 border border-green-900/50 rounded text-xs"
                        >
                            Resume
                        </button>
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
    </Teleport>

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