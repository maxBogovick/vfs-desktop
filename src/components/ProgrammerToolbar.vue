<script setup lang="ts">
interface Props {
  hasMultipleSelected: boolean;
  isTerminalVisible: boolean;
}

interface Emits {
  (e: 'toggle-terminal'): void;
  (e: 'batch-rename'): void;
  (e: 'open-ftp'): void;
  (e: 'open-file-colors'): void;
  (e: 'toggle-resource-monitor'): void;
  (e: 'open-widgets'): void;
  (e: 'open-stego'): void;
}

defineProps<Props>();
const emit = defineEmits<Emits>();
</script>

<template>
  <div class="flex items-center h-[32px] bg-[#F1EFE2] border-b border-[#919B9C] px-2 gap-1">
    <!-- Terminal Button -->
    <button
      @click="emit('toggle-terminal')"
      :class="[
        'flex items-center gap-1 px-2 py-1 text-[11px] border rounded',
        isTerminalVisible
          ? 'bg-[#C1D2EE] border-[#0A246A]'
          : 'bg-[#ECE9D8] border-[#919B9C] hover:bg-[#C1D2EE] hover:border-[#0A246A]'
      ]"
      title="Toggle Terminal (Ctrl+`)"
    >
      <span class="text-[14px]">$</span>
      <span>Terminal</span>
    </button>

    <!-- Batch Rename Button -->
    <button
      @click="emit('batch-rename')"
      :disabled="!hasMultipleSelected"
      :class="[
        'flex items-center gap-1 px-2 py-1 text-[11px] border rounded',
        hasMultipleSelected
          ? 'bg-[#ECE9D8] border-[#919B9C] hover:bg-[#C1D2EE] hover:border-[#0A246A] cursor-pointer'
          : 'bg-[#D4D0C8] border-[#808080] text-gray-400 cursor-not-allowed'
      ]"
      title="Batch Rename (requires multiple files selected)"
    >
      <span class="text-[14px]">📝</span>
      <span>Batch Rename</span>
    </button>

    <!-- Steganography Open Button -->
    <button
      @click="emit('open-stego')"
      class="flex items-center gap-1 px-2 py-1 text-[11px] border rounded bg-[#ECE9D8] border-[#919B9C] hover:bg-[#C1D2EE] hover:border-[#0A246A]"
      title="Open Hidden Vault from File"
    >
      <span class="text-[14px]">🕵️</span>
      <span>Open from...</span>
    </button>

    <!-- FTP Button -->
    <button
      @click="emit('open-ftp')"
      class="flex items-center gap-1 px-2 py-1 text-[11px] border rounded bg-[#ECE9D8] border-[#919B9C] hover:bg-[#C1D2EE] hover:border-[#0A246A]"
      title="Open FTP Connection"
    >
      <span class="text-[14px]">🌐</span>
      <span>FTP</span>
    </button>

    <!-- Resource Monitor Button -->
    <button
      @click="emit('toggle-resource-monitor')"
      class="flex items-center gap-1 px-2 py-1 text-[11px] border rounded bg-[#ECE9D8] border-[#919B9C] hover:bg-[#C1D2EE] hover:border-[#0A246A]"
      title="Toggle Resource Monitor"
    >
      <span class="text-[14px]">📊</span>
      <span>Monitor</span>
    </button>

    <!-- File Colors Button -->
    <button
      @click="emit('open-file-colors')"
      class="flex items-center gap-1 px-2 py-1 text-[11px] border rounded bg-[#ECE9D8] border-[#919B9C] hover:bg-[#C1D2EE] hover:border-[#0A246A]"
      title="Configure File Colors"
    >
      <span class="text-[14px]">🎨</span>
      <span>Colors</span>
    </button>

    <!-- Widgets Button -->
    <button
      @click="emit('open-widgets')"
      class="flex items-center gap-1 px-2 py-1 text-[11px] border rounded bg-[#ECE9D8] border-[#919B9C] hover:bg-[#C1D2EE] hover:border-[#0A246A]"
      title="Open Widgets"
    >
      <span class="text-[14px]">🧩</span>
      <span>Widgets</span>
    </button>
  </div>
</template>
