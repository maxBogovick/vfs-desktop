<script setup lang="ts">
import { ref, computed } from 'vue';

interface Command {
  id: string;
  icon: string;
  name: string;
  shortcut?: string;
  action: () => void;
}

interface Props {
  isOpen: boolean;
}

interface Emits {
  (e: 'close'): void;
  (e: 'execute', command: Command): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const searchQuery = ref('');
const selectedIndex = ref(0);

const commands: Command[] = [
  { id: 'new-folder', icon: '📁', name: 'New Folder', shortcut: 'Ctrl+Shift+N', action: () => {} },
  { id: 'new-file', icon: '📝', name: 'New File', shortcut: 'Ctrl+N', action: () => {} },
  { id: 'search', icon: '🔍', name: 'Search in Files', shortcut: 'Ctrl+Shift+F', action: () => {} },
  { id: 'goto', icon: '➡️', name: 'Go to Path', shortcut: 'Ctrl+G', action: () => {} },
  { id: 'refresh', icon: '🔄', name: 'Refresh', shortcut: 'F5', action: () => {} },
  { id: 'copy-path', icon: '📋', name: 'Copy Path', shortcut: 'Ctrl+Shift+C', action: () => {} },
  { id: 'select-all', icon: '☑️', name: 'Select All', shortcut: 'Ctrl+A', action: () => {} },
  { id: 'new-tab', icon: '➕', name: 'New Tab', shortcut: 'Ctrl+T', action: () => {} },
  { id: 'close-tab', icon: '✖️', name: 'Close Tab', shortcut: 'Ctrl+W', action: () => {} },
  { id: 'settings', icon: '⚙️', name: 'Settings', shortcut: 'Ctrl+,', action: () => {} },
];

const filteredCommands = computed(() => {
  if (!searchQuery.value) return commands;
  const query = searchQuery.value.toLowerCase();
  return commands.filter(cmd =>
    cmd.name.toLowerCase().includes(query) ||
    cmd.shortcut?.toLowerCase().includes(query)
  );
});

const executeCommand = (command: Command) => {
  emit('execute', command);
  command.action();
  emit('close');
  searchQuery.value = '';
  selectedIndex.value = 0;
};

const handleKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, filteredCommands.value.length - 1);
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
  } else if (event.key === 'Enter' && filteredCommands.value.length > 0) {
    event.preventDefault();
    executeCommand(filteredCommands.value[selectedIndex.value]);
  }
};

const handleBackdropClick = () => {
  emit('close');
  searchQuery.value = '';
  selectedIndex.value = 0;
};
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen"
      @click="handleBackdropClick"
      class="fixed inset-0 bg-black/20 backdrop-blur-[1px] z-50 flex items-start justify-center pt-[15vh]"
    >
      <div
        @click.stop
        class="w-[550px] bg-white rounded-xl shadow-2xl border border-[#919B9C] overflow-hidden flex flex-col animate-pop-in"
      >
        <!-- Search Input -->
        <div class="flex items-center px-4 py-3 border-b border-gray-100">
          <span class="text-xl mr-3">🚀</span>
          <input
            v-model="searchQuery"
            @keydown="handleKeyDown"
            type="text"
            placeholder="Type a command or search..."
            class="flex-1 text-base outline-none text-gray-700 placeholder:text-gray-300"
            autofocus
          />
          <div class="text-[10px] bg-gray-100 text-gray-400 px-1.5 py-0.5 rounded">ESC</div>
        </div>

        <!-- Commands List -->
        <div class="max-h-[400px] overflow-y-auto">
          <div v-if="filteredCommands.length === 0" class="p-8 text-center text-gray-400">
            <div class="text-3xl mb-2">🔍</div>
            <div class="text-sm">No commands found</div>
          </div>

          <div v-else class="p-2">
            <div class="text-[10px] font-bold text-gray-400 px-2 py-1 mb-1 uppercase tracking-wide">Commands</div>
            <div
              v-for="(cmd, index) in filteredCommands"
              :key="cmd.id"
              @click="executeCommand(cmd)"
              @mouseenter="selectedIndex = index"
              :class="[
                'flex items-center justify-between px-3 py-2.5 rounded-lg cursor-pointer group transition-all',
                selectedIndex === index
                  ? 'bg-[#0054E3] text-white shadow-md'
                  : 'hover:bg-gray-50'
              ]"
            >
              <div class="flex items-center gap-3">
                <span class="text-lg">{{ cmd.icon }}</span>
                <span :class="['font-medium text-sm', selectedIndex === index ? 'text-white' : 'text-gray-700']">
                  {{ cmd.name }}
                </span>
              </div>
              <span
                v-if="cmd.shortcut"
                :class="[
                  'text-[10px] px-2 py-0.5 rounded font-mono',
                  selectedIndex === index
                    ? 'bg-white/20 text-white'
                    : 'bg-gray-100 text-gray-500'
                ]"
              >
                {{ cmd.shortcut }}
              </span>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="bg-[#ECE9D8] px-4 py-2 text-[10px] text-gray-500 flex justify-between border-t border-[#D0D0BF]">
          <div class="flex gap-3">
            <span>↑↓ Navigate</span>
            <span>↵ Execute</span>
            <span>ESC Close</span>
          </div>
          <span class="text-gray-400">Windows XP Remastered</span>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.animate-pop-in {
  animation: popIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes popIn {
  0% {
    transform: scale(0.95) translateY(10px);
    opacity: 0;
  }
  100% {
    transform: scale(1) translateY(0);
    opacity: 1;
  }
}
</style>
