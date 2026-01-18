<script setup lang="ts">
import type { FileItem } from '../types';

interface Props {
  x: number;
  y: number;
  item: FileItem | null;
  selectedCount?: number;
  hasClipboardContent?: boolean;
  showProgrammerMode?: boolean;
}

interface Emits {
  (e: 'open'): void;
  (e: 'edit'): void;
  (e: 'copy'): void;
  (e: 'cut'): void;
  (e: 'paste'): void;
  (e: 'rename'): void;
  (e: 'delete'): void;
  (e: 'addToFavorites'): void;
  (e: 'openTerminal'): void;
  (e: 'properties'): void;
  (e: 'extractHere'): void;
  (e: 'extractToFolder'): void;
  (e: 'compressToZip'): void;
  (e: 'compressToTar'): void;
  (e: 'compressToTarGz'): void;
  (e: 'batchRename'): void;
  (e: 'batchAttributes'): void;
  (e: 'refresh'): void;
  (e: 'newFolder'): void;
  (e: 'newFile'): void;
  (e: 'selectAll'): void;
  (e: 'queueCopy'): void;
  (e: 'queueMove'): void;
  (e: 'queueDelete'): void;
  (e: 'queueArchive'): void;
  (e: 'queueExtract'): void;
  (e: 'share'): void;
  (e: 'hideTo'): void;
  (e: 'extractHidden'): void;
  (e: 'createSecureFolder'): void;
  (e: 'protectSelection'): void;
  (e: 'close'): void;
}

const props = withDefaults(defineProps<Props>(), {
  selectedCount: 0,
  hasClipboardContent: false,
  showProgrammerMode: false,
});
const emit = defineEmits<Emits>();

const handleAction = (action: keyof Emits) => {
  emit(action);
  emit('close');
};
</script>

<template>
  <div
    :style="{ top: `${y}px`, left: `${x}px` }"
    class="fixed bg-white border border-[#919B9C] shadow-lg rounded text-[11px] py-1 z-50 min-w-[180px]"
  >
    <!-- FILE CONTEXT MENU -->
    <template v-if="item">
      <!-- Open -->
      <div
        @click="handleAction('open')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">📂</span>
        <span class="flex-1">Open</span>
        <span class="text-[9px] text-gray-400">Enter</span>
      </div>

      <!-- Hide to... (Programmer Mode) -->
      <div
        v-if="showProgrammerMode"
        @click="handleAction('hideTo')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2 text-purple-700 font-medium"
      >
        <span class="w-4">🕵️</span>
        <span class="flex-1">Hide to...</span>
      </div>

      <!-- Extract Hidden (Programmer Mode) -->
      <div
        v-if="showProgrammerMode && item.type !== 'folder'"
        @click="handleAction('extractHidden')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2 text-purple-700 font-medium"
      >
        <span class="w-4">🔓</span>
        <span class="flex-1">Extract Hidden Data</span>
      </div>

      <!-- Edit (only for text/code files) -->
      <div
        v-if="(item.type === 'file' || item.type === 'code') && selectedCount <= 1"
        @click="handleAction('edit')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">✏️</span>
        <span class="flex-1">Edit</span>
        <span class="text-[9px] text-gray-400">F4</span>
      </div>

      <div class="border-t border-[#D0D0BF] my-1"></div>

      <!-- Copy -->
      <div
        @click="handleAction('copy')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">📋</span>
        <span class="flex-1">Copy</span>
        <span class="text-[9px] text-gray-400">Ctrl+C</span>
      </div>

      <!-- Cut -->
      <div
        @click="handleAction('cut')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">✂️</span>
        <span class="flex-1">Cut</span>
        <span class="text-[9px] text-gray-400">Ctrl+X</span>
      </div>

      <!-- Share via QR -->
      <div
        v-if="item.type !== 'folder' && item.type !== 'drive' && item.type !== 'system'"
        @click="handleAction('share')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">📱</span>
        <span class="flex-1">Share via QR</span>
      </div>

      <div class="border-t border-[#D0D0BF] my-1"></div>

      <!-- Add to Queue (with operation submenu) -->
      <div
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2 group relative"
      >
        <span class="w-4">⏱️</span>
        <span class="flex-1">Add to Queue</span>
        <span class="text-[9px] text-gray-400">►</span>

        <!-- Operations Submenu -->
        <div class="hidden group-hover:block absolute left-full top-0 bg-white border border-[#919B9C] shadow-lg rounded text-[11px] py-1 min-w-[180px]">
          <!-- Copy to... -->
          <div
            @click.stop="handleAction('queueCopy')"
            class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
          >
            <span>📋 Copy to...</span>
          </div>

          <!-- Move to... -->
          <div
            @click.stop="handleAction('queueMove')"
            class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
          >
            <span>➡️ Move to...</span>
          </div>

          <div class="border-t border-[#D0D0BF] my-1"></div>

          <!-- Delete -->
          <div
            @click.stop="handleAction('queueDelete')"
            class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
          >
            <span>🗑️ Delete</span>
          </div>

          <div class="border-t border-[#D0D0BF] my-1"></div>

          <!-- Archive -->
          <div
            @click.stop="handleAction('queueArchive')"
            class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
          >
            <span>📦 Create Archive...</span>
          </div>

          <!-- Extract (only for archives) -->
          <div
            v-if="item.type === 'archive'"
            @click.stop="handleAction('queueExtract')"
            class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
          >
            <span>📂 Extract Here</span>
          </div>
        </div>
      </div>

      <!-- Paste -->
      <div
        v-if="hasClipboardContent"
        @click="handleAction('paste')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">📄</span>
        <span class="flex-1">Paste</span>
        <span class="text-[9px] text-gray-400">Ctrl+V</span>
      </div>

      <div class="border-t border-[#D0D0BF] my-1"></div>

      <!-- Rename -->
      <div
        v-if="selectedCount <= 1"
        @click="handleAction('rename')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">✏️</span>
        <span class="flex-1">Rename</span>
        <span class="text-[9px] text-gray-400">F2</span>
      </div>

      <!-- Batch Operations (when multiple items selected) -->
      <div v-if="selectedCount > 1" class="border-t border-[#D0D0BF] my-1"></div>

      <!-- Archive Operations (Submenu) -->
      <div
        v-if="selectedCount > 0"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2 group relative"
      >
        <span class="w-4">📦</span>
        <span class="flex-1">Archive...</span>
        <span class="text-[9px] text-gray-400">►</span>
        
        <!-- Submenu -->
        <div class="hidden group-hover:block absolute left-full top-0 bg-white border border-[#919B9C] shadow-lg rounded text-[11px] py-1 min-w-[150px]">
            <div
                @click.stop="handleAction('compressToZip')"
                class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
            >
                <span>Add to .zip</span>
            </div>
            <div
                @click.stop="handleAction('compressToTar')"
                class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
            >
                <span>Add to .tar</span>
            </div>
            <div
                @click.stop="handleAction('compressToTarGz')"
                class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
            >
                <span>Add to .tar.gz</span>
            </div>
        </div>
      </div>

      <!-- Protect with Password -->
      <div
        v-if="selectedCount > 0"
        @click="handleAction('protectSelection')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2 text-blue-700 font-medium"
      >
        <span class="w-4">🛡️</span>
        <span class="flex-1">Protect with Password</span>
      </div>

      <div
        v-if="selectedCount > 1"
        @click="handleAction('batchRename')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">📝</span>
        <span class="flex-1">Batch Rename</span>
        <span class="text-[9px] text-gray-400">{{ selectedCount }} items</span>
      </div>

      <div
        v-if="selectedCount > 1"
        @click="handleAction('batchAttributes')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">🔧</span>
        <span class="flex-1">Batch Attributes</span>
        <span class="text-[9px] text-gray-400">{{ selectedCount }} items</span>
      </div>

      <!-- Delete -->
      <div
        @click="handleAction('delete')"
        class="px-3 py-1.5 hover:bg-red-50 hover:text-red-600 cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">🗑️</span>
        <span class="flex-1">Delete</span>
        <span class="text-[9px] text-gray-400">Del</span>
      </div>

      <div class="border-t border-[#D0D0BF] my-1"></div>

      <!-- Add to Favorites (only for folders) -->
      <div
        v-if="item.type === 'folder'"
        @click="handleAction('addToFavorites')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">⭐</span>
        <span class="flex-1">Add to Favorites</span>
        <span class="text-[9px] text-gray-400">Ctrl+D</span>
      </div>

      <!-- Open in Terminal -->
      <div
        @click="handleAction('openTerminal')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">💻</span>
        <span class="flex-1">Open in Terminal</span>
      </div>

      <!-- Extract (for archives) -->
      <div
        v-if="item.type === 'archive'"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2 group relative"
      >
        <span class="w-4">📦</span>
        <span class="flex-1">Extract...</span>
        <span class="text-[9px] text-gray-400">►</span>

        <!-- Submenu -->
        <div class="hidden group-hover:block absolute left-full top-0 bg-white border border-[#919B9C] shadow-lg rounded text-[11px] py-1 min-w-[150px]">
            <div
                @click.stop="handleAction('extractHere')"
                class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
            >
                <span>Extract Here</span>
            </div>
            <div
                @click.stop="handleAction('extractToFolder')"
                class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
            >
                <span>Extract to Folder</span>
            </div>
        </div>
      </div>

      <!-- Properties -->
      <div
        @click="handleAction('properties')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">ℹ️</span>
        <span class="flex-1">Properties</span>
        <span class="text-[9px] text-gray-400">Alt+Enter</span>
      </div>

      <!-- File Info Footer -->
      <div class="border-t border-[#D0D0BF] mt-1 px-3 py-2 bg-[#F5F5F5]">
        <div class="text-[9px] text-gray-500 mb-0.5">{{ item.name }}</div>
        <div class="text-[9px] text-gray-400">{{ item.type }} • {{ item.sizeFormatted }}</div>
      </div>
    </template>

    <!-- BACKGROUND CONTEXT MENU -->
    <template v-else>
       <!-- Paste -->
       <div
        v-if="hasClipboardContent"
        @click="handleAction('paste')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">📄</span>
        <span class="flex-1">Paste</span>
        <span class="text-[9px] text-gray-400">Ctrl+V</span>
      </div>

      <div v-if="hasClipboardContent" class="border-t border-[#D0D0BF] my-1"></div>

      <!-- Refresh -->
      <div
        @click="handleAction('refresh')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">🔄</span>
        <span class="flex-1">Refresh</span>
        <span class="text-[9px] text-gray-400">F5</span>
      </div>

      <div class="border-t border-[#D0D0BF] my-1"></div>

      <!-- New Folder -->
      <div
        @click="handleAction('newFolder')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">📁</span>
        <span class="flex-1">New Folder</span>
        <span class="text-[9px] text-gray-400">F7</span>
      </div>

      <!-- New Secure Folder -->
      <div
        @click="handleAction('createSecureFolder')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2 text-blue-700 font-medium"
      >
        <span class="w-4">🔒</span>
        <span class="flex-1">New Secure Folder</span>
      </div>

       <!-- New File -->
       <div
        @click="handleAction('newFile')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">📄</span>
        <span class="flex-1">New File</span>
        <span class="text-[9px] text-gray-400">Shift+F4</span>
      </div>

      <div class="border-t border-[#D0D0BF] my-1"></div>

      <!-- Select All -->
      <div
        @click="handleAction('selectAll')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">✅</span>
        <span class="flex-1">Select All</span>
        <span class="text-[9px] text-gray-400">Ctrl+A</span>
      </div>

      <div class="border-t border-[#D0D0BF] my-1"></div>

      <!-- Open in Terminal -->
      <div
        @click="handleAction('openTerminal')"
        class="px-3 py-1.5 hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
      >
        <span class="w-4">💻</span>
        <span class="flex-1">Open Terminal Here</span>
      </div>
    </template>
  </div>
</template>